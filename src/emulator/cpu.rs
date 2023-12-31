pub mod instruction;

use instruction::*;
use crate::emulator::address_bus::AddressBus;
use crate::emulator::dbg::*;
use self::interrupts::handle_interrupts;

pub mod interrupts;



const Z_FLAG: u8 = 0x80;
const N_FLAG: u8 = 0x40;
const H_FLAG: u8 = 0x20;
const C_FLAG: u8 = 0x10;

struct Registers {
    /* 8-bit Registers */
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    /* Program counter */
    pc: u16,
    /* Stack pointer */
    sp: u16,
}

/**
 * A struct that defines the CPU context
 * https://www.youtube.com/watch?v=17cdj-HYpb0&list=PLVxiWMqQvhg_yk4qy2cSC3457wZJga_e5&index=3
 */
pub struct CPU {
    pub ticks: u32,
    // In trace mode
    trace: bool,
    halted: bool,
    // In stepping mode
    stepping: bool,
    // Interrupt
    interrupt_master_enabled: bool,
    enabling_ime: bool,
    int_flags: u8,
    // Current fetch
    // Current opcode
    opcode: u8,
    fetched_data: u16,
    mem_dest: u16,
    dest_is_mem: bool,
    // Current instruction
    instr: *const Instruction,
    /* Interrupt enable register */
    ie_register: u8,
    registers: Registers,
}

impl CPU {
    /**
     * Creates a new CPU instance
     */
    pub fn new(trace: bool) -> CPU {
        log::info!("Initializing CPU...");

        let cpu = CPU {
            ticks: 0,
            trace: trace,
            halted: false,
            stepping: false,
            interrupt_master_enabled: false,
            enabling_ime: false,
            int_flags: 0,
            opcode: 0,
            fetched_data: 0,
            mem_dest: 0,
            dest_is_mem: false,
            // Initialize the current instruction to null
            instr: std::ptr::null::<Instruction>(),
            ie_register: 0,
            // Initializes PC to the entry point
            registers: Registers {
                a: 0, f: 0, b: 0, c: 0,
                d: 0, e: 0, h: 0, l: 0,
                pc: 0x100, sp: 0
            },            
        };
        log::info!(target: "stdout", "Initializing CPU: SUCCESS");
        return cpu;
    }


    /*****************************************
     * Functions that process instructions
     *****************************************/

    fn exec_none(&mut self) -> () {
        return;
    }
    
    /**
     * Executes the LD instruction
     */
    fn exec_ld(&mut self, bus: &mut AddressBus) -> () {

        if self.dest_is_mem {
            // E.g., LD (HL), A
            // If the destination is memory, write the fetched data
            // to the memory location specified by mem_dest
            if unsafe { (*self.instr).reg2.is_16_bit()  } {
                // Writes 16-bit value to memory
                bus.write_16(self.mem_dest, self.fetched_data);
            } else {
                // Writes 8-bit value to memory
                bus.write(self.mem_dest, self.fetched_data as u8);
            }
            self.tick(1);
            return;
        }

        if unsafe { (*self.instr).addr_mode == AddrMode::AM_HL_SPR } {
            // Special case: LD HL, SP + r8
            unsafe {
                assert! ((*self.instr).reg1 == RegType::RT_HL && 
                         (*self.instr).reg2 == RegType::RT_SP);
            }
            // Half Carry Flag (H) is set if there is a carry from bit 3
            // to bit 4
            let h_flag = ((self.read_reg(&RegType::RT_SP) & 0x0F) +
                (self.fetched_data & 0x0F)) > 0x10;
            // Carry Flag (C) is set if there is a carry from bit 7
            // to bit 8
            let c_flag = ((self.read_reg(&RegType::RT_SP) & 0xFF) +
                (self.fetched_data & 0xFF)) > 0x100;
            
            self.set_flags(0, 0, h_flag as i8, c_flag as i8);
            let res: u16 = 
                self.read_reg(&RegType::RT_SP).checked_add_signed(self.fetched_data as i16).unwrap();
            
            self.set_register(&RegType::RT_HL, res);
            return;
        }

        // The most common case: setting the value of a register
        // to the fetched data
        unsafe {
            self.set_register(&(*self.instr).reg1, self.fetched_data);
        }
    }


    /**
     * Executes the LDH instruction, i.e., Load into HRAM
     */
    fn exec_ldh(&mut self, bus: &mut AddressBus) -> () {
        if unsafe { (*self.instr).reg1 == RegType::RT_A } {
            // LDH A, (a8)
            let addr = bus.read(self.fetched_data | 0xFF00) as u16;
            self.set_register(&RegType::RT_A, addr);
        } else {
            // LDH (a8), A
            bus.write(self.mem_dest, self.read_reg(&RegType::RT_A) as u8);
        }
        self.tick(1);
    }

    /**
     * A helper function that executes instructions that
     * perform some type of jump operation. If `push_pc`
     * is true, the current PC value is pushed onto the stack.
     */
    fn goto_addr(&mut self, bus: &mut AddressBus,
            address: u16, push_pc: bool) -> () {

        if self.check_cond() {
            if push_pc {
                self.stack_push16(bus, self.read_reg(&RegType::RT_PC));
                self.tick(2);
            }
            self.set_register(&RegType::RT_PC, address);
            self.tick(1);
        }
    }

    /**
     * Executes the JP instruction. A wrapper function for goto_addr
     */
    fn exec_jp(&mut self, bus: &mut AddressBus) -> () {
        self.goto_addr(bus, self.fetched_data, false);
    }
    
    /**
     * Executes the JP instruction. A wrapper function for goto_addr
     */
    fn exec_jr(&mut self, bus: &mut AddressBus) -> () {
        let rel: i8 = (self.fetched_data & 0xFF) as i8;
        let pc = self.read_reg(&RegType::RT_PC);
        let addr = pc.checked_add_signed(rel as i16).unwrap();
        self.goto_addr(bus, addr, false);
    }

    /**
     * Executes the CALL instruction. A wrapper function for goto_addr
     */
    fn exec_call(&mut self, bus: &mut AddressBus) -> () {
        self.goto_addr(bus, self.fetched_data, true);
    }

    /**
     * Executes the RET instruction.
     */
    fn exec_ret(&mut self, bus: &mut AddressBus) -> () {
        if unsafe { (*self.instr).cond_type != CondType::CT_NONE } {
            self.tick(1);
        }
        if self.check_cond() {
            let addr = self.stack_pop16(bus);
            self.tick(2);
            self.set_register(&RegType::RT_PC, addr);
            self.tick(1);
        }
    }
    
    /**
     * Executes the RETI instruction. A wrapper for exec_ret
     */
    fn exec_reti(&mut self, bus: &mut AddressBus) -> () {
        // Re-enables interrupts
        self.interrupt_master_enabled = true;
        self.exec_ret(bus);
    }

    /**
     * Executes the RST instruction. A wrapper for goto_addr
     */
    fn exec_rst(&mut self, bus: &mut AddressBus) -> () {
        unsafe { self.goto_addr(bus, (*self.instr).param as u16, true); }
    }

    /**
     * Executes the DI instruction. Disables interrupts.
     */
    fn exec_di(&mut self) -> () {
        self.interrupt_master_enabled = false;
    }

    /**
     * Executes the XOR instruction
     */
    fn exec_xor(&mut self) -> () {
        unsafe {
            let val = self.read_reg(&(*self.instr).reg1) ^ self.fetched_data;
            self.set_register(&(*self.instr).reg1, val);
            self.set_flags((val == 0) as i8, 0, 0, 0);
        }
    }

    /**
     * Executes the AND instruction
     */
    fn exec_and(&mut self) -> () {
        unsafe {
            let val = self.read_reg(&(*self.instr).reg1) & self.fetched_data;
            self.set_register(&(*self.instr).reg1, val);
            self.set_flags((val == 0) as i8, 0, 1, 0)
        }
    }

    /**
     * Executes the OR instruction
     */
    fn exec_or(&mut self) -> () {
        unsafe {
            let val = self.read_reg(&(*self.instr).reg1) | self.fetched_data;
            self.set_register(&(*self.instr).reg1, val);
            self.set_flags((val == 0) as i8, 0, 0, 0);
        }
    }

    /**
     * Executes the CP instruction
     */
    fn exec_cp(&mut self) -> () {
        let op1 = unsafe { self.read_reg(&(*self.instr).reg1) }; 
        let val = op1 as i32  - self.fetched_data as i32;
        
        let h_flag = ((op1 as i32) & 0x0F) - ((self.fetched_data as i32) & 0x0F) < 0;

        self.set_flags((val == 0) as i8, 1, h_flag as i8, (val < 0) as i8);
    }

    /**
     * Executes the INC instruction
     */
    fn exec_inc(&mut self, bus: &mut AddressBus) -> () {
        let mut val = self.fetched_data.wrapping_add(1);

        if unsafe { (*self.instr).reg1.is_16_bit() } {
            self.tick(1);
        }

        if unsafe { (*self.instr).reg1 == RegType::RT_HL && self.dest_is_mem } {
            // Special case: INC (HL)
            val &= 0xFF;
            bus.write(self.mem_dest, val as u8);
        } else {
            // Normal case
            unsafe {
                self.set_register(&(*self.instr).reg1, val);
                val = self.read_reg(&(*self.instr).reg1);
            }
        }
        if (self.opcode & 0x03) == 0x03 {
            // Do not set flags for INC BC, INC DE, INC HL, INC SP
            return;
        }
        self.set_flags((val == 0) as i8, 0, ((val & 0x0F) == 0) as i8, -1);
    }


    /**
     * Executes the DEC instruction
     */
    fn exec_dec(&mut self, bus: &mut AddressBus) -> () {
        let mut val = self.fetched_data.wrapping_sub(1);

        if unsafe { (*self.instr).reg1.is_16_bit() } {
            self.tick(1);
        }

        if unsafe { (*self.instr).reg1 == RegType::RT_HL && self.dest_is_mem } {
            // Special case: DEC (HL)
            bus.write(self.mem_dest, val as u8);
        } else {
            // Normal case
            unsafe {
                self.set_register(&(*self.instr).reg1, val);
                val = self.read_reg(&(*self.instr).reg1);
            }
        }
        
        if (self.opcode & 0x0B) == 0x0B {
            // Do not set flags for DEC BC, DEC DE, DEC HL, DEC SP
            return;
        }

        self.set_flags((val == 0) as i8, 1, ((val & 0x0F) == 0x0F) as i8, -1);
    }


    /**
     * Executes the ADD instruction
     */
    fn exec_add(&mut self) -> () {        
        let mut val: u32 = 
            (unsafe { self.read_reg(&(*self.instr).reg1) } + self.fetched_data) as u32;

        let is_16_bit = unsafe { (*self.instr).reg1.is_16_bit() };
        if is_16_bit {
            self.tick(1);
        }

        if unsafe { (*self.instr).reg1 == RegType::RT_SP } {
            // Dealing with the special case of ADD SP, r8
            // Converts `fetched_data` to signed 8-bit integer
            let rel: i8 = self.fetched_data as i8;
            val = self.read_reg(&RegType::RT_SP).checked_add_signed(rel as i16).unwrap() as u32;
        }

        // Flags
        unsafe {
            let mut z_flag: i8;
            let mut h_flag: i8;
            let mut c_flag: i8;
            
            // FIXME: The control flow here can probably be improved
            if is_16_bit {
                z_flag = -1;
                h_flag = (((self.read_reg(&(*self.instr).reg1) & 0x0FFF) +
                    (self.fetched_data & 0x0FFF)) >= 0x1000) as i8;
                let tmp = self.read_reg(&(*self.instr).reg1) as u32 +
                    self.fetched_data as u32;
                c_flag = (tmp >= 0x10000) as i8;
            } else {
                z_flag = (val & 0xFF == 0) as i8;
                h_flag = (((self.read_reg(&(*self.instr).reg1) & 0x0F) +
                    (self.fetched_data & 0x0F)) >= 0x10) as i8;
                c_flag = (((self.read_reg(&(*self.instr).reg1) & 0xFF) +
                    (self.fetched_data & 0xFF)) >= 0x100) as i8;
            }

            if (*self.instr).reg1 == RegType::RT_SP {
                z_flag = 0;
                h_flag = (((self.read_reg(&RegType::RT_SP) & 0x0F) +
                    (self.fetched_data & 0x0F)) >= 0x10) as i8;
                c_flag = (((self.read_reg(&RegType::RT_SP) & 0xFF) +
                    (self.fetched_data & 0xFF)) >= 0x100) as i8;
            }


            self.set_register(&(*self.instr).reg1, (val & 0xFFFF) as u16);
            self.set_flags(z_flag, 0, h_flag, c_flag);

        }
    }


    /**
     * Executes the ADC instruction, i.e., Add with Carry
     */
    fn exec_adc(&mut self) -> () {
        unsafe {
            let op1 = self.read_reg(&(*self.instr).reg1);
            let op2 = self.fetched_data;
            let c_flag = self.get_flag(C_FLAG) as u16;
            let val: u16 = ((op1 + op2 + c_flag) & 0xFF) as u16;
            self.set_register(&(*self.instr).reg1, val);


            let h_flag = (op1 & 0x0F + op2 & 0x0F + c_flag) > 0x0F;
            self.set_flags((val == 0) as i8, 0, h_flag as i8, (val > 0xFF) as i8);
        }
    }

    /**
     * Executes the SUB instruction
     */
    fn exec_sub(&mut self) -> () {
        let op1 = unsafe { self.read_reg(&(*self.instr).reg1) };
        let val = op1.wrapping_sub(self.fetched_data);
        
        let z_flag = (val == 0) as i8;
        let h_flag = (((op1 as i32 & 0x0F) - (self.fetched_data as i32 & 0x0F)) < 0) as i8;
        let c_flag = (((op1 as i32) - (self.fetched_data as i32)) < 0) as i8;

        unsafe { self.set_register(&(*self.instr).reg1, val) };
        self.set_flags(z_flag, 1, h_flag, c_flag);
    }

    /**
     * Executes the SBC instruction
     */
    fn exec_sbc(&mut self) -> () {
        let c_val = self.get_flag(C_FLAG) as u8;
        let op1 = unsafe { self.read_reg(&(*self.instr).reg1) };
        let val = self.fetched_data - (c_val as u16);
        
        let z_flag = ((op1 - val) == 0) as i8;
        let h_flag = (((op1 as i32 & 0x0F) - (self.fetched_data as i32 & 0x0F) -
                (c_val as i32)) < 0) as i8;
        let c_flag = (((op1 as i32) - (self.fetched_data as i32) -
                (c_val as i32)) < 0) as i8;
        
        unsafe { self.set_register(&(*self.instr).reg1, op1 - val) };
        self.set_flags(z_flag, 1, h_flag, c_flag);
    }

    /**
     * Executes the POP instruction
     */
    fn exec_pop(&mut self, bus: &mut AddressBus) -> () {
        let value = self.stack_pop16(bus);
        self.tick(2);

        unsafe {
            assert! ((*self.instr).reg1.is_16_bit());
            if (*self.instr).reg1 == RegType::RT_AF {
                // Special case: AF register
                // The lower 4 bits of F are always 0
                self.set_register(&RegType::RT_AF, value & 0xFFF0);
            } else {
                self.set_register(&(*self.instr).reg1, value);
            }
        }
    }

    /**
     * Executes the PUSH instruction
     */
    fn exec_push(&mut self, bus: &mut AddressBus) -> () {
        let hi = ((self.fetched_data & 0xFF00) >> 8) as u8;
        let lo = (self.fetched_data & 0x00FF) as u8;

        self.stack_push(bus, hi);
        self.tick(1);
        self.stack_push(bus, lo);
        self.tick(1);

        self.tick(1);
    }

    fn exec_cb(&mut self, bus: &mut AddressBus) -> () {
        let cb_opcode = self.fetched_data as u8;
        // On which register to perform the operation
        let reg = cb_decode_reg(cb_opcode & 0b111);
        // On which bit to perform the operation
        let bit = (cb_opcode >> 3) & 0b111;
        // The operation to perform
        let bit_op = (cb_opcode >> 6) & 0b11;
        let reg_val = self.read_cb_reg(reg, bus);

        self.tick(1);


        if *reg == RegType::RT_HL {
            self.tick(2);
        }

        match bit_op {
            1 => {
                // BIT
                // Copies the complement of the specified bit to the Z flag
                let z_flag = ((reg_val & (1 << bit)) == 0) as i8;
                self.set_flags(z_flag, 0, 1, -1);
                return;
            },
            2 => {
                // RES
                // Resets the specified bit
                let new_val = reg_val & !(1 << bit);
                self.set_cb_reg(reg, new_val, bus);
                return;
            },
            3 => {
                // SET
                let new_val = reg_val | (1 << bit);
                self.set_cb_reg(reg, new_val, bus);
                return;
            },
            _ => {
                // Handle all other cases
                let c_flag = self.get_flag(C_FLAG) as u8;
                match bit {
                    0 => {
                        // RLC
                        // Rotates the register left
                        let mut set_c = false;
                        let mut new_val = (reg_val << 1) & 0xFF;
                        // If bit 7 is not set
                        if (reg_val & (1 << 7)) != 0 {
                            new_val |= 1;
                            set_c = true;
                        }
                        self.set_cb_reg(reg, new_val, bus);
                        self.set_flags((new_val == 0) as i8, 0, 0, set_c as i8);
                    },
                    1 => {
                        // RRC
                        // Rotates the register right
                        let mut new_val = reg_val >> 1;
                        new_val |= reg_val << 7;
                        self.set_cb_reg(reg, new_val, bus);
                        self.set_flags((new_val == 0) as i8, 0, 0, (reg_val & 1) as i8);
                    },
                    2 => {
                        // RL
                        // Rotates the register left through the carry flag
                        let mut new_val = reg_val << 1;
                        new_val |= c_flag;

                        self.set_cb_reg(reg, new_val, bus);
                        self.set_flags((new_val == 0) as i8, 0, 0, (reg_val & 0x80) as i8);
                    },
                    3 => {
                        // RR
                        // Rotates the register right through the carry flag
                        let mut new_val = reg_val >> 1;
                        new_val |= c_flag << 7;

                        self.set_cb_reg(reg, new_val, bus);
                        self.set_flags((new_val == 0) as i8, 0, 0, (reg_val & 1) as i8);
                    },
                    4 => {
                        // SLA
                        // Shifts the register left into the carry flag
                        let new_val = reg_val << 1;

                        self.set_cb_reg(reg, new_val, bus);
                        self.set_flags((new_val == 0) as i8, 0, 0, (reg_val & 0x80) as i8);
                    },
                    5 => {
                        // SRA
                        // Shifts the register right into the carry flag
                        let new_val = (reg_val as i8 >> 1) as u8;
                        self.set_cb_reg(reg, new_val, bus);
                        self.set_flags((new_val == 0) as i8, 0, 0, (reg_val & 1) as i8);
                    },
                    6 => {
                        // SWAP
                        // Swaps the upper and lower nibbles of the register
                        let new_val = ((reg_val & 0x0F) << 4) | ((reg_val & 0xF0) >> 4);
                        self.set_cb_reg(reg, new_val, bus);
                        self.set_flags((new_val == 0) as i8, 0, 0, 0);
                    },
                    7 => {
                        // SRL
                        // Shifts the register right into the carry flag
                        let new_val = reg_val >> 1;
                        self.set_cb_reg(reg, new_val, bus);
                        self.set_flags((new_val == 0) as i8, 0, 0, (reg_val & 1) as i8);
                    },
                    _ => {
                        log::error!(target: "stdout",
                            "Invalid CB instruction: {:02X}", cb_opcode);
                        std::process::exit(-1);
                    }
                }
            }
        }

    }

    /**
     * Executes the CPL instruction.
     * Complements the contents of register A.
     */
    fn exec_cpl(&mut self) -> () {
        let val = self.read_reg(&RegType::RT_A);
        self.set_register(&RegType::RT_A, !val);
        self.set_flags(-1, 1, 1, -1);
    }

    /**
     * Executes the CCF instruction.
     * Complements the carry flag.
     */
    fn exec_ccf(&mut self) -> () {
        let c_flag = self.get_flag(C_FLAG);
        self.set_flags(-1, 0, 0, c_flag as i8 ^ 1);
    }

    /**
     * Executes the SCF instruction.
     * Sets the carry flag.
     */
    fn exec_scf(&mut self) -> () {
        self.set_flags(-1, 0, 0, 1);
    }

    /**
     * Executes the DAA instruction.
     * Adjusts register A to contain a binary coded decimal.
     */
    fn exec_daa(&mut self) -> () {
        let c_flag = self.get_flag(C_FLAG);
        let h_flag = self.get_flag(H_FLAG);
        let n_flag = self.get_flag(N_FLAG);

        let a_val = self.read_reg(&RegType::RT_A);

        let mut adjust = if c_flag { 0x60 } else { 0 };
        if h_flag {
            adjust |= 0x6;
        }
        let new_val: u16;
        if !n_flag {
            if (a_val & 0x0F) > 0x09 {
                adjust |= 0x06;
            }
            if a_val > 0x99 {
                adjust |= 0x60;
            }
            new_val = a_val.wrapping_add(adjust);
        } else {
            new_val = a_val.wrapping_sub(adjust);
        }

        self.set_register(&RegType::RT_A, new_val);
        self.set_flags((new_val == 0) as i8, -1, 0, (adjust >= 0x60) as i8);
    }

    /**
     * Executes the RLCA instruction.
     * Rotates the contents of register A left by 1 bit.
     */
    fn exec_rlca(&mut self) -> () {
        let mut val = self.read_reg(&RegType::RT_A);
        let c_flag = (val >> 7) & 1;
        val = (val << 1) | c_flag;
        self.set_register(&RegType::RT_A, val);
        self.set_flags((val == 0) as i8, 0, 0, c_flag as i8);
    }

    /**
     * Executes the RRCA instruction.
     * Rotates the contents of register A right by 1 bit.
     */
    fn exec_rrca(&mut self) -> () {
        let mut val = self.read_reg(&RegType::RT_A);
        let c_flag = val & 1;
        val = (val >> 1) | (c_flag << 7);
        self.set_register(&RegType::RT_A, val);
        self.set_flags(0, 0, 0, c_flag as i8);
    }

    /**
     * Executes the RLA instruction.
     * Rotates the contents of register A left through the carry flag.
     */
    fn exec_rla(&mut self) -> () {
        let mut val = self.read_reg(&RegType::RT_A);
        let c_flag = self.get_flag(C_FLAG) as u16;
        val = (val << 1) | c_flag;
        self.set_register(&RegType::RT_A, val);
        self.set_flags(0, 0, 0, (val >> 7) as i8);
    }

    /**
     * Executes the RRA instruction.
     * Rotates the contents of register A right through the carry flag.
     */
    fn exec_rra(&mut self) -> () {
        let c_flag = self.get_flag(C_FLAG) as u16;
        let mut val = self.read_reg(&RegType::RT_A);
        let new_c_flag = val & 1;
        val = (val >> 1) | (c_flag << 7);
        self.set_register(&RegType::RT_A, val);
        self.set_flags(0, 0, 0, new_c_flag as i8);
    }

    /**
     * Executes the HALT instruction.
     */
    fn exec_halt(&mut self) -> () {
        self.halted = true;
    }

    /**
     * Executes the STOP instruction.
     */
    fn exec_stop(&mut self) -> () {
        log::info!("STOP instruction executed");
        std::process::exit(0);
    }
    

    /**
     * Executes the EI instruction.
     */
    fn exec_ei(&mut self) -> () {
        self.enabling_ime = true;
    }

    /*****************************************
     * End of functions that process instructions
     *****************************************/


    /*****************************************
     * Stack operations
     *****************************************/

    /**
     * A private function that first decrements the stack
     * pointer then  pushes an 8-bit value onto the memory
     * address specified by the stack pointer.
     */
    fn stack_push(&mut self, bus: &mut AddressBus, data: u8) -> () {
        let mut sp_val = self.read_reg(&RegType::RT_SP);
        self.set_register(&RegType::RT_SP, sp_val - 1);
        sp_val = self.read_reg(&RegType::RT_SP);
        bus.write(sp_val, data);
    }

    /**
     * Pushes a 16-bit value onto the stack
     */
    fn stack_push16(&mut self, bus: &mut AddressBus, data: u16) -> () {
        self.stack_push(bus, ((data & 0xFF00) >> 8) as u8);
        self.stack_push(bus, (data & 0x00FF) as u8);
    }

    /**
     * A private function that first pops an 8-bit value from
     * the memory address specified by the stack pointer then
     * increments the stack pointer.
     */
    fn stack_pop(&mut self, bus: &mut AddressBus) -> u8 {
        let sp_val = self.read_reg(&RegType::RT_SP);
        let data = bus.read(sp_val);
        self.set_register(&RegType::RT_SP, sp_val + 1);
        return data;
    }

    /**
     * Pops a 16-bit value from the stack and returns it.
     */
    fn stack_pop16(&mut self, bus: &mut AddressBus) -> u16 {
        let lo = self.stack_pop(bus) as u16;
        let hi = self.stack_pop(bus) as u16;
        return (hi << 8) | lo;
    }

    /*****************************************
     * End of stack operations
     *****************************************/


    /**
     * Increments the program counter
     */
    fn increment_pc(&mut self) -> () {
        self.registers.pc += 1;
    }

    /**
     * Increments the CPU timer.
     */
    pub fn tick(&mut self, ticks: u32) -> () {
        self.ticks += ticks;
    }

    /**
     * A private function that reads a byte from the given register
     * (except for IE)
     */
    #[inline(always)]
    fn read_reg(&self, reg: &RegType) -> u16 {
        match reg {
            RegType::RT_A => { return self.registers.a as u16; },
            RegType::RT_B => { return self.registers.b as u16; },
            RegType::RT_C => { return self.registers.c as u16; },
            RegType::RT_D => { return self.registers.d as u16; },
            RegType::RT_E => { return self.registers.e as u16; },
            RegType::RT_H => { return self.registers.h as u16; },
            RegType::RT_L => { return self.registers.l as u16; },
            RegType::RT_SP => { return self.registers.sp; },
            RegType::RT_PC => { return self.registers.pc; },
            // FIXME: Repetition
            RegType::RT_AF => {
                // Accumulator and flags
                let hi = self.registers.a;
                let lo = self.registers.f;
                return ((hi as u16) << 8) | (lo as u16);
            }
            RegType::RT_BC => {
                let hi = self.registers.b;
                let lo = self.registers.c;
                return ((hi as u16) << 8) | (lo as u16);
            }
            RegType::RT_DE => {
                let hi = self.registers.d;
                let lo = self.registers.e;
                return ((hi as u16) << 8) | (lo as u16);
            }
            RegType::RT_HL => {
                let hi = self.registers.h;
                let lo = self.registers.l;
                return ((hi as u16) << 8) | (lo as u16);
            }
            _ => {
                log::error!(target: "stdout", "Register {:?} not implemented", reg);
                std::process::exit(-1);
            }
        }

    }

    /**
     * A private function that sets the value of a register (except for IE)
     */
    #[inline(always)]
    fn set_register(&mut self, reg: &RegType, value: u16) -> () {
        match reg {
            RegType::RT_A => { self.registers.a = value as u8; },
            RegType::RT_B => { self.registers.b = value as u8; },
            RegType::RT_C => { self.registers.c = value as u8; },
            RegType::RT_D => { self.registers.d = value as u8; },
            RegType::RT_E => { self.registers.e = value as u8; },
            RegType::RT_H => { self.registers.h = value as u8; },
            RegType::RT_L => { self.registers.l = value as u8; },
            RegType::RT_SP => { self.registers.sp = value; },
            RegType::RT_PC => { self.registers.pc = value; },
            RegType::RT_AF => {
                self.registers.a = ((value & 0xFF00) >> 8) as u8;
                self.registers.f = (value & 0x00FF) as u8;
            },
            RegType::RT_BC => {
                self.registers.b = ((value & 0xFF00) >> 8) as u8;
                self.registers.c = (value & 0x00FF) as u8;
            },
            RegType::RT_DE => {
                self.registers.d = ((value & 0xFF00) >> 8) as u8;
                self.registers.e = (value & 0x00FF) as u8;
            },
            RegType::RT_HL => {
                self.registers.h = ((value & 0xFF00) >> 8) as u8;
                self.registers.l = (value & 0x00FF) as u8;
            },
            _ => {
                log::error!(target: "stdout", "Register {:?} not implemented", reg);
                std::process::exit(-1);
            }
        };
    }


    /**
     * A private function that reads the value of the register
     * used by a CB instruction. If HL is used, the value of
     * the memory location specified by HL is returned.
     */
    #[inline(always)]
    fn read_cb_reg(&mut self, reg: &RegType, bus: &mut AddressBus) -> u8 {
        if *reg == RegType::RT_HL {
            return bus.read(self.read_reg(&RegType::RT_HL));
        } else {
            if reg.is_16_bit() {
                log::error!(target: "stdout", 
                    "16-bit register {:?} not supported for CB instructions", reg);
            }
            return self.read_reg(reg) as u8;
        }
    }


    /**
     * A private function that sets the value of the register used
     * by a CB instruction. If HL is used, the value of the memory
     * location specified by HL is set.
     */
    #[inline(always)]
    fn set_cb_reg(&mut self, reg: &RegType, value: u8, bus: &mut AddressBus) -> () {
        if *reg == RegType::RT_HL {
            bus.write(self.read_reg(&RegType::RT_HL), value);
        } else {
            if reg.is_16_bit() {
                log::error!(target: "stdout",
                    "16-bit register {:?} not supported for CB instructions", reg);
            }
            self.set_register(reg, value as u16);
        }
    }

    /**
     * A private function that retrieves the value of the interrupt
     * enable register
     */
    #[inline(always)]
    pub fn get_ie_register(&self) -> u8 {
        return self.ie_register;
    }

    /**
     * A private function that sets the value of the interrupt
     * enable register
     */
    #[inline(always)]
    pub fn set_ie_register(&mut self, value: u8) -> () {
        self.ie_register = value;
    }

    /**
     * A private function that retrieves the value of the interrupt
     * flags register
     */
    #[inline(always)]
    pub fn get_int_flags(&self) -> u8 {
        return self.int_flags;
    }

    /**
     * A private function that sets the value of the interrupt
     * flags register
     */
    #[inline(always)]
    pub fn set_int_flags(&mut self, value: u8) -> () {
        self.int_flags = value;
    }

    /**
     * A private function retrieves the value of a flag
     */
    #[inline(always)]
    fn get_flag(&self, flag: u8) -> bool {
        return (self.registers.f & flag) != 0;
    }

    /**
     * A private function that sets the value of a single flag
     */
    #[inline(always)]
    fn set_flag(&mut self, flag: u8, value: bool) -> () {
        if value {
            self.registers.f |= flag;
        } else {
            self.registers.f &= !flag;
        }
    }

    /**
     * A private function that sets the value of all flags.
     * If the given value is positive, the flag is set.
     * Otherwise, the flag is unmodified.
     */
    #[inline(always)]
    fn set_flags(&mut self, z: i8, n: i8, h: i8, c: i8) -> () {
        if z >= 0 { self.set_flag(Z_FLAG, z > 0); }
        if n >= 0 { self.set_flag(N_FLAG, n > 0); }
        if h >= 0 { self.set_flag(H_FLAG, h > 0); }
        if c >= 0 { self.set_flag(C_FLAG, c > 0); }
    }

    /**
     * A private function that checks if the condition of the
     * current instruction is true.
     */
    fn check_cond(&mut self) -> bool {
        let z_flag = self.get_flag(Z_FLAG);
        let c_flag = self.get_flag(C_FLAG);

        match &unsafe { &(*self.instr) }.cond_type {
            CondType::CT_NONE => { return true; },
            CondType::CT_NZ => {
                // Z flag is not set
                return !z_flag;
            },
            CondType::CT_Z => {
                // Z flag is set
                return z_flag;
            },
            CondType::CT_NC => {
                // C flag is not set
                return !c_flag;
            },
            CondType::CT_C => {
                // C flag is set
                return c_flag;
            }
        }   
    }

    /**
     * Fetches the next instruction
     */
    fn fetch_instruction(&mut self, bus: &AddressBus) -> () {
        let pc = self.read_reg(&RegType::RT_PC);
        self.opcode = bus.read(pc);
        self.instr = Instruction::get_instruction(self.opcode);
        self.increment_pc();
    }

    /*********************************************************
     * Fetches data depending on the address mode of
     * the current instruction.
     * @param bus: The address bus
     * @return (): Nothing
     *********************************************************/
    fn fetch_data(&mut self, bus: &mut AddressBus) -> () {
        self.mem_dest = 0;
        self.dest_is_mem = false;
        unsafe {
            match (*self.instr).addr_mode {
                AddrMode::AM_IMP => { return; },
                AddrMode::AM_R => {
                    // Load register
                    self.fetched_data = self.read_reg(&(*self.instr).reg1);
                    return;
                },
                AddrMode::AM_R_R => {
                    // Load register into register
                    self.fetched_data = self.read_reg(&(*self.instr).reg2);
                    return;
                },
                AddrMode::AM_R_D8 => {
                    // Load 8-bit immediate value
                    let pc = self.read_reg(&RegType::RT_PC);
                    self.fetched_data = bus.read(pc) as u16;
                    self.increment_pc();
                    self.tick(1);
                    return;
                },
                AddrMode::AM_D16 | AddrMode::AM_R_D16 => {
                    // Load 16-bit immediate value
                    let mut pc = self.read_reg(&RegType::RT_PC);
                    // Lower byte
                    let lo = bus.read(pc);
                    self.increment_pc();
                    self.tick(1);

                    // Upper byte
                    pc = self.read_reg(&RegType::RT_PC);
                    let hi = bus.read(pc);
                    self.increment_pc();
                    self.fetched_data = ((hi as u16) << 8) | (lo as u16);
                    self.tick(1);
                    return;
                },
                AddrMode::AM_MR_R => {
                    // Store value of register into memory
                    self.fetched_data = self.read_reg(&(*self.instr).reg2);
                    self.mem_dest = self.read_reg(&(*self.instr).reg1);
                    self.dest_is_mem = true;
                    // Special case LD (C), A
                    if (*self.instr).reg1 == RegType::RT_C {
                        self.mem_dest |= 0xFF00;
                    }
                    return;
                },
                AddrMode::AM_R_MR => {
                    // Load value from memory into register
                    let mut addr = self.read_reg(&(*self.instr).reg2);
                    if (*self.instr).reg2 == RegType::RT_C {
                        addr |= 0xFF00;
                    }
                    self.fetched_data = bus.read(addr) as u16;
                    self.tick(1);
                },
                AddrMode::AM_R_HLI => {
                    // Load value from the memory location specified by HL
                    // into register and increment HL
                    assert! ((*self.instr).reg2 == RegType::RT_HL);
                    let hl_val = self.read_reg(&RegType::RT_HL);
                    self.fetched_data = bus.read(hl_val) as u16;
                    self.tick(1);
                    
                    // Sets the value of HL to HL + 1
                    self.set_register(&RegType::RT_HL, hl_val + 1);
                    return;
                },
                AddrMode::AM_R_HLD => {
                    // Load value from the memory location specified by HL 
                    // into register and decrement HL
                    assert! ((*self.instr).reg2 == RegType::RT_HL);
                    let hl_val = self.read_reg(&RegType::RT_HL);
                    self.fetched_data = bus.read(hl_val) as u16;
                    self.tick(1);
                    
                    // Sets the value of HL to HL - 1
                    self.set_register(&RegType::RT_HL, hl_val - 1);
                    return;
                },
                AddrMode::AM_HLI_R => {
                    // Store value from register into the memory location
                    // specified by register HL and increment HL
                    assert! ((*self.instr).reg1 == RegType::RT_HL);
                    self.fetched_data = self.read_reg(&(*self.instr).reg2);

                    let hl_val = self.read_reg(&RegType::RT_HL);
                    self.mem_dest = hl_val;
                    self.dest_is_mem = true;
                    // Sets the value of HL to HL + 1
                    self.set_register(&RegType::RT_HL, hl_val + 1);
                },
                AddrMode::AM_HLD_R => {
                    // Store value from register into the memory location
                    // specified by register HL and decrement HL
                    assert! ((*self.instr).reg1 == RegType::RT_HL);
                    self.fetched_data = self.read_reg(&(*self.instr).reg2);

                    let hl_val = self.read_reg(&RegType::RT_HL);
                    self.mem_dest = hl_val;
                    self.dest_is_mem = true;
                    // Sets the value of HL to HL - 1
                    self.set_register(&RegType::RT_HL, hl_val - 1);
                },
                AddrMode::AM_R_A8 => {
                    // Load value from memory location specified by 8-bit
                    // immediate value into register
                    let pc = self.read_reg(&RegType::RT_PC);
                    self.fetched_data = bus.read(pc) as u16;
                    self.tick(1);
                    self.increment_pc();
                    return;
                },
                AddrMode::AM_A8_R => {
                    // Store value from register into memory location
                    // specified by 8-bit immediate value
                    let pc = self.read_reg(&RegType::RT_PC);
                    self.mem_dest = bus.read(pc) as u16 | 0xFF00;
                    self.dest_is_mem = true;
                    self.tick(1);
                    self.increment_pc();
                    return;
                },
                AddrMode::AM_HL_SPR => {
                    // Load value from memory location specified by SP +
                    // signed 8-bit immediate value into register
                    let pc = self.read_reg(&RegType::RT_PC);
                    self.fetched_data = bus.read(pc) as u16;
                    self.tick(1);
                    self.increment_pc();
                    return;
                },
                AddrMode::AM_D8 => {
                    // Load 8-bit immediate value
                    let pc = self.read_reg(&RegType::RT_PC);
                    self.fetched_data = bus.read(pc) as u16;
                    self.tick(1);
                    self.increment_pc();
                    return;
                },
                AddrMode::AM_D16_R | AddrMode::AM_A16_R => {
                    // ============ UNUSED ============
                    // Store the value of register into memory location
                    // specified by 16-bit immediate value
                    let mut pc = self.read_reg(&RegType::RT_PC);
                    // Lower byte
                    let lo = bus.read(pc);
                    self.increment_pc();
                    self.tick(1);
                    // Upper byte
                    pc = self.read_reg(&RegType::RT_PC);
                    let hi = bus.read(pc);
                    self.increment_pc();
                    self.mem_dest = ((hi as u16) << 8) | (lo as u16);
                    self.dest_is_mem = true;
                    self.tick(1);

                    self.fetched_data = self.read_reg(&(*self.instr).reg2);
                    return;
                },
                AddrMode::AM_MR_D8 => {
                    // Store 8-bit immediate value into memory location
                    // specified by register
                    let pc = self.read_reg(&RegType::RT_PC);
                    self.fetched_data = bus.read(pc) as u16;
                    self.tick(1);
                    self.increment_pc();

                    self.mem_dest = self.read_reg(&(*self.instr).reg1);
                    self.dest_is_mem = true;
                    return;
                },
                AddrMode::AM_MR => {
                    // Load value from memory location specified by register
                    self.mem_dest = self.read_reg(&(*self.instr).reg1);
                    self.dest_is_mem = true;
                    self.fetched_data = bus.read(self.mem_dest) as u16;
                    self.tick(1);
                    return;
                },
                AddrMode::AM_R_A16 => {
                    // Load value from memory location specified by 16-bit
                    // immediate value into register
                    let mut pc = self.read_reg(&RegType::RT_PC);
                    // Lower byte
                    let lo = bus.read(pc);
                    self.increment_pc();
                    self.tick(1);
                    // Upper byte
                    pc = self.read_reg(&RegType::RT_PC);
                    let hi = bus.read(pc);
                    self.increment_pc();
                    self.tick(1);
                    
                    let addr = ((hi as u16) << 8) | (lo as u16);
                    self.fetched_data = bus.read(addr) as u16;
                    self.tick(1);
                    return;
                },
                _ => {
                    log::error!(target: "stdout", "Address mode {:?} not implemented",
                        (*self.instr).addr_mode);
                    std::process::exit(-1);
                }
            }
        }
    }

    /**
     * Executes the current instruction
     */
    fn execute(&mut self, bus: &mut AddressBus) -> () {
        unsafe {
            // FIXME There is no better way to do it in Rust?
            match (*self.instr).instr_type {
                InstrType::IN_NOP   => { self.exec_none(); },
                // Load instructions
                InstrType::IN_LD    => { self.exec_ld(bus); },
                InstrType::IN_LDH   => { self.exec_ldh(bus); },

                // Arithmetic instructions
                InstrType::IN_INC   => { self.exec_inc(bus); },
                InstrType::IN_DEC   => { self.exec_dec(bus); },
                InstrType::IN_ADD   => { self.exec_add(); },
                InstrType::IN_ADC   => { self.exec_adc(); },
                InstrType::IN_SUB   => { self.exec_sub(); },
                InstrType::IN_SBC   => { self.exec_sbc(); },

                // Bitwise instructions
                InstrType::IN_XOR   => { self.exec_xor(); },
                InstrType::IN_AND   => { self.exec_and(); },
                InstrType::IN_OR    => { self.exec_or(); },
                InstrType::IN_CP    => { self.exec_cp(); },

                // Jump instructions
                InstrType::IN_JP    => { self.exec_jp(bus); },
                InstrType::IN_JR    => { self.exec_jr(bus); },
                InstrType::IN_CALL  => { self.exec_call(bus); },
                InstrType::IN_RET   => { self.exec_ret(bus); },
                InstrType::IN_RETI  => { self.exec_reti(bus); },
                InstrType::IN_RST   => { self.exec_rst(bus); },

                // Misc instructions
                InstrType::IN_DI    => { self.exec_di(); },
                InstrType::IN_CB    => { self.exec_cb(bus); }
                InstrType::IN_RLCA  => { self.exec_rlca(); },
                InstrType::IN_RLA   => { self.exec_rla(); },
                InstrType::IN_RRCA  => { self.exec_rrca(); },
                InstrType::IN_RRA   => { self.exec_rra(); },
                InstrType::IN_CPL   => { self.exec_cpl(); },
                InstrType::IN_CCF   => { self.exec_ccf(); },
                InstrType::IN_SCF   => { self.exec_scf(); },
                InstrType::IN_DAA   => { self.exec_daa(); },
                InstrType::IN_HALT  => { self.exec_halt(); },
                InstrType::IN_STOP  => { self.exec_stop(); },
                InstrType::IN_EI    => { self.exec_ei(); },

                // Stack-related instructions
                InstrType::IN_PUSH  => { self.exec_push(bus); },
                InstrType::IN_POP   => { self.exec_pop(bus); },
                _ => {
                    log::error!(target: "stdout", "Instruction {:?} not implemented",
                        (*self.instr).instr_type);
                    std::process::exit(-1);
                }
            }
            
        }
    }
    
    /*****************************************
     * Executes a single instruction
     *****************************************/
    pub fn step(&mut self, bus: &mut AddressBus) -> bool {
        if !self.halted {
            let pc = self.read_reg(&RegType::RT_PC);

            // Fetch and Decode
            self.fetch_instruction(bus);
            // Execute
            self.fetch_data(bus);
            if self.trace {
                let instr_str = unsafe { (*self.instr).disass(self) };
                // log::trace!(target: "trace_file", "{:<6} - 0x{:04X}: {:<12} ({:02X} {:02X} {:02X}) A:{:02X} F: {}{}{}{} BC: {:02X}{:02X} DE:{:02X}{:02X} HL: {:02X}{:02X}",
                println!("{:<6} - 0x{:04X}: {:<12} ({:02X} {:02X} {:02X}) A:{:02X} F: {}{}{}{} BC: {:02X}{:02X} DE:{:02X}{:02X} HL: {:02X}{:02X}",
                            self.ticks, pc, instr_str, 
                            self.opcode, bus.read(pc + 1), bus.read(pc + 2),
                            self.registers.a,
                            if self.get_flag(Z_FLAG) { 'Z' } else { '-' },
                            if self.get_flag(N_FLAG) { 'N' } else { '-' },
                            if self.get_flag(H_FLAG) { 'H' } else { '-' },
                            if self.get_flag(C_FLAG) { 'C' } else { '-' },
                            self.registers.b, self.registers.c,
                            self.registers.d, self.registers.e,
                            self.registers.h, self.registers.l
                        );
            }

            dbg_update(bus);
            dbg_print();

            self.execute(bus);
        } else {
            self.tick(1);
            // If the CPU is halted
            if self.int_flags > 0 {
                self.halted = false;
            }
        }

        if self.interrupt_master_enabled {
             handle_interrupts(self, bus);
             self.enabling_ime = false;
        }

        if self.enabling_ime {
            self.interrupt_master_enabled = true;
        }

        return true;
    }

    /**
     * Dumps the CPU state
     */
    pub fn print_state(&self, logger: &str) -> () {
        let mut state = String::new();
        state.push_str(&format!("======= CPU state =======\n"));
        state.push_str(&format!("A : 0x{:02X}\t", self.registers.a));
        state.push_str(&format!("BC: 0x{:02X}{:02X}\t", self.registers.b, self.registers.c));
        state.push_str(&format!("DE: 0x{:02X}{:02X}\n", self.registers.d, self.registers.e));
        state.push_str(&format!("HL: 0x{:02X}{:02X}\t", self.registers.h, self.registers.l));
        state.push_str(&format!("PC: 0x{:04X}\t", self.registers.pc));
        state.push_str(&format!("SP: 0x{:04X}", self.registers.sp));
        log::debug!(target: logger, "{}", state);
        self.print_flags(logger);
    }

    /**
     * Prints all the flags in register f.
     */
    pub fn print_flags(&self, logger: &str) -> () {
        log::debug!(target: logger, "Flags: {}{}{}{}",
            if self.get_flag(Z_FLAG) { 'Z' } else { '-' },
            if self.get_flag(N_FLAG) { 'N' } else { '-' },
            if self.get_flag(H_FLAG) { 'H' } else { '-' },
            if self.get_flag(C_FLAG) { 'C' } else { '-' });
    }
}