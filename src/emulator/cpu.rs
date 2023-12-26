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
    // Current fetch
    // Current opcode
    opcode: u8,
    fetched_data: u16,
    mem_dest: u16,
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
            opcode: 0,
            fetched_data: 0,
            mem_dest: 0,
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
     * Fetches the next instruction
     */
    fn fetch_instruction(&mut self, bus: &AddressBus) -> () {
        let pc = self.read_reg(&RegType::RT_PC);
        self.opcode = bus.read(pc);
        self.instr = Instruction::get_instruction(self.opcode);
        self.increment_pc();
    }

    /**
     * Fetches data depending on the address mode of
     * the current instruction.
     */
    fn fetch_data(&mut self, bus: &AddressBus) -> () {
        unsafe {
            match (*self.instr).addr_mode {
                AddrMode::AM_IMP => { return; },
                AddrMode::AM_R => {
                    self.fetched_data = self.read_reg(&(*self.instr).reg1);
                    return;
                },
                AddrMode::AM_R_D8 => {
                    // TODO Incorrect implementation
                    let pc = self.read_reg(&RegType::RT_PC);
                    self.fetched_data = bus.read(pc) as u16;
                    self.increment_pc();
                    self.tick(1);
                    return;
                },
                AddrMode::AM_D16 => {
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
                }
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
        
    }
    
    pub fn step(&mut self, bus: &AddressBus) -> bool {
        if !self.halted {
            let pc = self.read_reg(&RegType::RT_PC);
            // Fetch
            self.fetch_instruction(bus);
            
            self.fetch_data(bus);
            // Decode

            // Execute
            self.execute();

            if self.trace {
                unsafe {
                    let instr_str = (*self.instr).str();
                    log::trace!("0x{:04X} {}", pc, instr_str);
                }
            }
        }
        return true;
    }
}