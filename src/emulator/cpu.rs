use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;
pub mod instruction;
use instruction::*;
use crate::emulator::address_bus::AddressBus;
use crate::emulator::Emulator;

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
    sp: u16
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
    interrupt_master_enable: bool,
    // Current fetch
    // Current opcode
    opcode: u8,
    fetched_data: u16,
    mem_dest: u16,
    dest_is_mem: bool,
    // Current instruction
    instr: *const Instruction,

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
            interrupt_master_enable: false,
            opcode: 0,
            fetched_data: 0,
            mem_dest: 0,
            dest_is_mem: false,
            // Initialize the current instruction to null
            instr: std::ptr::null::<Instruction>(),
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
    fn exec_ld(&mut self) -> () {

    }

    /**
     * Executes the JP instruction
     */
    fn exec_jp(&mut self) -> () {
        if self.check_cond() {
            self.registers.pc = self.fetched_data;
            self.tick(1);
        }
    }
    
    /**
     * Executes the DI instruction. Disables interrupts.
     */
    fn exec_di(&mut self) -> () {
        self.interrupt_master_enable = false;
    }

    /**
     * Executes the XOR instruction
     */
    fn exec_xor(&mut self) -> () {
        unsafe {
            let result = self.read_reg(&(*self.instr).reg1) ^ self.fetched_data;
            self.set_register(&(*self.instr).reg1, result);
            self.set_flags(result == 0, false, false, false);
        }
    }

    /*****************************************
     * End of functions that process instructions
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
    fn tick(&mut self, ticks: u32) -> () {
        self.ticks += ticks;
    }

    /**
     * A private function that reads a byte from the
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
     * A private function that sets the value of a register
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
     * A private function that sets the value of all flags
     */
    #[inline(always)]
    fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) -> () {
        self.set_flag(0x80, z);
        self.set_flag(0x40, n);
        self.set_flag(0x20, h);
        self.set_flag(0x10, c);
    }

    /**
     * A private function that checks if the condition of the
     * current instruction is true.
     */
    fn check_cond(&mut self) -> bool {
        let z_flag = self.get_flag(0x80);
        let c_flag = self.get_flag(0x10);

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
    fn fetch_data(&mut self, bus: &AddressBus) -> () {
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
    fn execute(&mut self) -> () {
        unsafe {
            // FIXME There is no better way to do it in Rust?
            match (*self.instr).instr_type {
                InstrType::IN_NOP => {
                    self.exec_none();
                },
                InstrType::IN_LD => {
                    self.exec_ld();
                },
                InstrType::IN_JP => {
                    self.exec_jp();
                },
                InstrType::IN_DI => {
                    self.exec_di();
                },
                InstrType::IN_XOR => {
                    self.exec_xor();
                },
                _ => {
                    log::error!(target: "stdout", "Instruction {:?} not implemented",
                        (*self.instr).instr_type);
                    std::process::exit(-1);
                }
            }
            
        }
    }
    
    pub fn step(&mut self, bus: &AddressBus) -> bool {
        if !self.halted {
            let pc = self.read_reg(&RegType::RT_PC);
            // Fetch and Decode
            self.fetch_instruction(bus);

            if self.trace {
                unsafe {
                    let instr_str = (*self.instr).str();
                    log::trace!(target: "trace_file", "0x{:04X}: {:<10} ({:02X} {:02X} {:02X})", 
                                pc, instr_str, self.opcode, bus.read(pc + 1), bus.read(pc + 2));
                }
            }

            // Execute            
            self.fetch_data(bus);

            self.execute();
        }
        return true;
    }

    /**
     * Prints all the flags in register f.
     */
    pub fn print_flags(self) -> () {
        log::info!(target: "stdout", "Z: {}, N: {}, H: {}, C: {}",
            self.get_flag(0x80), self.get_flag(0x40),
            self.get_flag(0x20), self.get_flag(0x10));
    }
}