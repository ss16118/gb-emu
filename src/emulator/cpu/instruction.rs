use phf::{phf_map, Map};

/* Addressing mode */
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code, non_camel_case_types)]
pub enum AddrMode {
    AM_IMP,
    AM_R_D16,
    AM_R_R,
    AM_MR_R,
    AM_R,
    AM_R_D8,
    AM_R_MR,
    AM_R_HLI,
    AM_R_HLD,
    AM_HLI_R,
    AM_HLD_R,
    AM_R_A8,
    AM_A8_R,
    AM_HL_SPR,
    AM_D16,
    AM_D8,
    AM_D16_R,
    AM_MR_D8,
    AM_MR,
    AM_A16_R,
    AM_R_A16,
}


/* Register type */
#[derive(strum_macros::Display, Debug, PartialEq, Eq, PartialOrd)]
#[allow(dead_code, non_camel_case_types)]
pub enum RegType {
    RT_NONE,
    RT_A,
    RT_F,
    RT_B,
    RT_C,
    RT_D,
    RT_E,
    RT_H,
    RT_L,
    RT_AF,
    RT_BC,
    RT_DE,
    RT_HL,
    RT_SP,
    RT_PC
}

impl RegType {
    /**
     * Returns true if the register type is 16-bit.
     */
    pub fn is_16_bit(&self) -> bool {
        return self >= &RegType::RT_AF;
    }
    /**
     * Returns a string representation of the register type.
     */
    fn str(&self) -> String {
        return self.to_string()[3..].to_string();
    }
}

/**
 * An enum that defines the type of conditions
 */
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code, non_camel_case_types)]
pub enum CondType {
    CT_NONE,
    CT_NZ,
    CT_Z,
    CT_NC,
    CT_C
}

/* Instruction type */
#[derive(strum_macros::Display, Eq, PartialEq, Hash, Debug)]
#[allow(non_camel_case_types)]
pub enum InstrType {
    IN_NONE,
    IN_NOP,
    IN_LD,
    IN_INC,
    IN_DEC,
    IN_RLCA,
    IN_ADD,
    IN_RRCA,
    IN_STOP,
    IN_RLA,
    IN_JR,
    IN_RRA,
    IN_DAA,
    IN_CPL,
    IN_SCF,
    IN_CCF,
    IN_HALT,
    IN_ADC,
    IN_SUB,
    IN_SBC,
    IN_AND,
    IN_XOR,
    IN_OR,
    IN_CP,
    IN_POP,
    IN_JP,
    IN_PUSH,
    IN_RET,
    IN_CB,
    IN_CALL,
    IN_RETI,
    IN_LDH,
    IN_JPHL,
    IN_DI,
    IN_EI,
    IN_RST,
    IN_ERR,
    //CB instructions...
    IN_RLC, 
    IN_RRC,
    IN_RL, 
    IN_RR,
    IN_SLA, 
    IN_SRA,
    IN_SWAP, 
    IN_SRL,
    IN_BIT, 
    IN_RES, 
    IN_SET
}

impl InstrType {
    /**
     * Returns a string representation of the instruction type.
     */
    fn str(&self) -> String {
        return self.to_string()[3..].to_string();
    }
}


/**
 * A struct that represents the instructions
 * https://gbdev.io/pandocs/CPU_Instruction_Set.html
 */
pub struct Instruction {
    pub param: u8,
    pub instr_type: InstrType,
    pub addr_mode: AddrMode,
    pub reg1: RegType,
    pub reg2: RegType,
    pub cond_type: CondType,
}

#[allow(dead_code)]
impl Instruction {
    /* ============== Constructors ============== */
    const fn default(instr_type: InstrType, addr_mode: AddrMode) 
        -> Instruction {
        return Instruction {
            param: 0,
            instr_type: instr_type,
            addr_mode: addr_mode,
            reg1: RegType::RT_NONE,
            reg2: RegType::RT_NONE,
            cond_type: CondType::CT_NONE
        };
        
    }

    const fn with_one_reg(instr_type: InstrType, addr_mode: AddrMode,
            reg: RegType) -> Instruction {        
        return Instruction {
            param: 0,
            instr_type: instr_type,
            addr_mode: addr_mode,
            reg1: reg,
            reg2: RegType::RT_NONE,
            cond_type: CondType::CT_NONE
        };
    
    }

    const fn with_two_regs(instr_type: InstrType, addr_mode: AddrMode,
            reg1: RegType, reg2: RegType) -> Instruction {
        return Instruction {
            param: 0,
            instr_type: instr_type,
            addr_mode: addr_mode,
            reg1: reg1,
            reg2: reg2,
            cond_type: CondType::CT_NONE
        };
    }

    const fn new(instr_type: InstrType, addr_mode: AddrMode, reg1: RegType,
            reg2: RegType, cond_type: CondType, param: u8) -> Instruction {
        return Instruction {
            param: param,
            instr_type: instr_type,
            addr_mode: addr_mode,
            reg1: reg1,
            reg2: reg2,
            cond_type: cond_type
        };
    }
    /* ============== End of constructors ============== */

    /**
     * Returns a string representation of the instruction.
     */
    pub fn str(&self) -> String {
        let mut result = String::new();
        result.push_str(&self.instr_type.str());
        if self.reg1 != RegType::RT_NONE {
            result.push_str(&format!(" {}", self.reg1.str()));
        }
        if self.reg2 != RegType::RT_NONE {
            result.push_str(&format!(", {}", self.reg2.str()));
        }
        if self.param != 0 {
            result.push_str(&format!(" {:#04X}", self.param));
        }
        return result;
    }

    pub fn get_instruction(opcode: u8) -> &'static Instruction {
        if INSTRUCTIONS.contains_key(&opcode) {
            return &INSTRUCTIONS[&opcode];
        } else {
            log::error!(target: "stdout", "Opcode: 0x{:02X} not implemented", opcode);
            std::process::exit(-1);
        }
    }
}

/**************************************************
 * https://meganesu.github.io/generate-gb-opcodes/
 *************************************************/
/* A map that maps each opcode to an instruction struct */
pub static INSTRUCTIONS: Map<u8, Instruction> = phf_map! {
    // 0x00 - 0x0F
    0x00_u8 => Instruction::default(InstrType::IN_NOP, AddrMode::AM_IMP),
    0x01_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D16, RegType::RT_BC),
    0x02_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_MR_R,
        RegType::RT_BC, RegType::RT_A),
    0x03_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_BC),
    0x04_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_B),
    0x05_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_B),
    0x06_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_B),
    0x08_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_A16_R,
        RegType::RT_NONE, RegType::RT_HL),
    0x09_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_HL, RegType::RT_BC),
    0x0A_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_A, RegType::RT_BC),
    0x0B_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_BC),
    0x0C_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_C),
    0x0D_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_C),
    0x0E_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_C),
    0x0F_u8 => Instruction::with_one_reg(InstrType::IN_RRCA, AddrMode::AM_IMP, RegType::RT_NONE),

    // 0x10 - 0x1F
    0x10_u8 => Instruction::default(InstrType::IN_STOP, AddrMode::AM_D8),
    0x11_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D16, RegType::RT_DE),
    0x13_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_DE),
    0x14_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_D),
    0x15_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_D),
    0x16_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_D),
    0x18_u8 => Instruction::default(InstrType::IN_JR, AddrMode::AM_D8),
    0x19_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_HL, RegType::RT_DE),
    0x1A_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_A, RegType::RT_DE),
    0x1B_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_DE),
    0x1C_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_E),
    0x1D_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_E),
    0x1E_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_E),

    // 0x20 - 0x2F
    0x20_u8 => Instruction::new(InstrType::IN_JR, AddrMode::AM_D8,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NZ, 0),
    0x21_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D16, RegType::RT_HL),
    0x22_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_HLI_R,
        RegType::RT_HL, RegType::RT_A),
    0x23_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_HL),
    0x24_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_H),
    0x25_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_H),
    0x26_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_H),
    0x28_u8 => Instruction::new(InstrType::IN_JR, AddrMode::AM_D8,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_Z, 0),
    0x29_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_HL, RegType::RT_HL),
    0x2A_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_HLI,
        RegType::RT_A, RegType::RT_HL),
    0x2B_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_HL),
    0x2C_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_L),
    0x2D_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_L),
    0x2E_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_L),
    0x2F_u8 => Instruction::with_one_reg(InstrType::IN_CPL, AddrMode::AM_IMP, RegType::RT_NONE),

    // 0x30 - 0x3F
    0x30_u8 => Instruction::new(InstrType::IN_JR, AddrMode::AM_D8,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NC, 0),
    0x31_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D16, RegType::RT_SP),
    0x32_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_HLD_R,
        RegType::RT_HL, RegType::RT_A),
    0x33_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_SP),
    0x34_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_MR, RegType::RT_HL),
    0x35_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_MR, RegType::RT_HL),
    0x36_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_HL),
    0x38_u8 => Instruction::new(InstrType::IN_JR, AddrMode::AM_D8,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_C, 0),
    0x39_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_HL, RegType::RT_SP),
    0x3A_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_HLD,
        RegType::RT_A, RegType::RT_HL),
    0x3B_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_SP),
    0x3C_u8 => Instruction::with_one_reg(InstrType::IN_INC, AddrMode::AM_R, RegType::RT_A),
    0x3D_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_A),
    0x3E_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_A),
    0x3F_u8 => Instruction::with_one_reg(InstrType::IN_CCF, AddrMode::AM_IMP, RegType::RT_NONE),

    // 0x40 - 0x4F
    0x40_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_B, RegType::RT_B),
    0x41_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_B, RegType::RT_C),
    0x42_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_B, RegType::RT_D),
    0x43_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_B, RegType::RT_E),
    0x44_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_B, RegType::RT_H),
    0x45_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_B, RegType::RT_L),
    0x46_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_B, RegType::RT_HL),
    0x47_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_B, RegType::RT_A),
    0x48_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_C, RegType::RT_B),
    0x49_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_C, RegType::RT_C),
    0x4A_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_C, RegType::RT_D),
    0x4B_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_C, RegType::RT_E),
    0x4C_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_C, RegType::RT_H),
    0x4D_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_C, RegType::RT_L),
    0x4E_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_C, RegType::RT_HL),
    0x4F_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_C, RegType::RT_A),
    
    // 0x50 - 0x5F
    0x50_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_D, RegType::RT_B),
    0x51_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_D, RegType::RT_C),
    0x52_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_D, RegType::RT_D),
    0x53_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_D, RegType::RT_E),
    0x54_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_D, RegType::RT_H),
    0x55_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_D, RegType::RT_L),
    0x56_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_D, RegType::RT_HL),
    0x57_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_D, RegType::RT_A),
    0x58_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_E, RegType::RT_B),
    0x59_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_E, RegType::RT_C),
    0x5A_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_E, RegType::RT_D),
    0x5B_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_E, RegType::RT_E),
    0x5C_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_E, RegType::RT_H),
    0x5D_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_E, RegType::RT_L),
    0x5E_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_E, RegType::RT_HL),
    0x5F_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_E, RegType::RT_A),

    // 0x60 - 0x6F
    0x60_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_H, RegType::RT_B),
    0x61_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_H, RegType::RT_C),
    0x62_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_H, RegType::RT_D),
    0x63_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_H, RegType::RT_E),
    0x64_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_H, RegType::RT_H),
    0x65_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_H, RegType::RT_L),
    0x66_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_H, RegType::RT_HL),
    0x67_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_H, RegType::RT_A),
    0x68_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_L, RegType::RT_B),
    0x69_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_L, RegType::RT_C),
    0x6A_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_L, RegType::RT_D),
    0x6B_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_L, RegType::RT_E),
    0x6C_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_L, RegType::RT_H),
    0x6D_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_L, RegType::RT_L),
    0x6E_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_L, RegType::RT_HL),
    0x6F_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_L, RegType::RT_A),
    
    // 0x70 - 0x7F
    0x70_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_MR_R,
        RegType::RT_HL, RegType::RT_B),
    0x71_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_MR_R,
        RegType::RT_HL, RegType::RT_C),
    0x72_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_MR_R,
        RegType::RT_HL, RegType::RT_D),
    0x73_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_MR_R,
        RegType::RT_HL, RegType::RT_E),
    0x74_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_MR_R,
        RegType::RT_HL, RegType::RT_H),
    0x75_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_MR_R,
        RegType::RT_HL, RegType::RT_L),
    0x76_u8 => Instruction::default(InstrType::IN_HALT, AddrMode::AM_IMP),
    0x77_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_MR_R,
        RegType::RT_HL, RegType::RT_A),
    0x78_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_B),
    0x79_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_C),
    0x7A_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_D),
    0x7B_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_E),
    0x7C_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_H),
    0x7D_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_L),
    0x7E_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_A, RegType::RT_HL),
    0x7F_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_A),

    // 0x80 - 0x8F
    0x80_u8 => Instruction::with_two_regs(InstrType::IN_SUB, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_B),
    0x81_u8 => Instruction::with_two_regs(InstrType::IN_SUB, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_C),
    0x82_u8 => Instruction::with_two_regs(InstrType::IN_SUB, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_D),
    0x83_u8 => Instruction::with_two_regs(InstrType::IN_SUB, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_E),
    0x84_u8 => Instruction::with_two_regs(InstrType::IN_SUB, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_H),
    0x85_u8 => Instruction::with_two_regs(InstrType::IN_SUB, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_L),
    0x86_u8 => Instruction::with_two_regs(InstrType::IN_SUB, AddrMode::AM_R_MR,
        RegType::RT_A, RegType::RT_HL),
    0x87_u8 => Instruction::with_two_regs(InstrType::IN_SUB, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_A),
    0x88_u8 => Instruction::with_two_regs(InstrType::IN_SBC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_B),
    0x89_u8 => Instruction::with_two_regs(InstrType::IN_SBC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_C),
    0x8A_u8 => Instruction::with_two_regs(InstrType::IN_SBC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_D),
    0x8B_u8 => Instruction::with_two_regs(InstrType::IN_SBC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_E),
    0x8C_u8 => Instruction::with_two_regs(InstrType::IN_SBC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_H),
    0x8D_u8 => Instruction::with_two_regs(InstrType::IN_SBC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_L),
    0x8E_u8 => Instruction::with_two_regs(InstrType::IN_SBC, AddrMode::AM_R_MR,
        RegType::RT_A, RegType::RT_HL),
    0x8F_u8 => Instruction::with_two_regs(InstrType::IN_SBC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_A),

    // 0x90 - 0x9F
    0x90_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_B),
    0x91_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_C),
    0x92_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_D),
    0x93_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_E),
    0x94_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_H),
    0x95_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_L),
    0x96_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_MR,
        RegType::RT_A, RegType::RT_HL),
    0x97_u8 => Instruction::with_two_regs(InstrType::IN_ADD, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_A),
    0x98_u8 => Instruction::with_two_regs(InstrType::IN_ADC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_B),
    0x99_u8 => Instruction::with_two_regs(InstrType::IN_ADC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_C),
    0x9A_u8 => Instruction::with_two_regs(InstrType::IN_ADC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_D),
    0x9B_u8 => Instruction::with_two_regs(InstrType::IN_ADC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_E),
    0x9C_u8 => Instruction::with_two_regs(InstrType::IN_ADC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_H),
    0x9D_u8 => Instruction::with_two_regs(InstrType::IN_ADC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_L),
    0x9E_u8 => Instruction::with_two_regs(InstrType::IN_ADC, AddrMode::AM_R_MR,
        RegType::RT_A, RegType::RT_HL),
    0x9F_u8 => Instruction::with_two_regs(InstrType::IN_ADC, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_A),
    
    // 0xA0 - 0xAF
    0xAF_u8 => Instruction::with_two_regs(InstrType::IN_XOR, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_A),

    // 0xC0 - 0xCF
    0xC0_u8 => Instruction::new(InstrType::IN_RET, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NZ, 0),
    0xC1_u8 => Instruction::with_one_reg(InstrType::IN_POP, AddrMode::AM_R, RegType::RT_BC),
    0xC2_u8 => Instruction::new(InstrType::IN_JP, AddrMode::AM_D16,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NZ, 0),
    0xC3_u8 => Instruction::default(InstrType::IN_JP, AddrMode::AM_D16),
    0xC4_u8 => Instruction::new(InstrType::IN_CALL, AddrMode::AM_D16_R,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NZ, 0),
    0xC5_u8 => Instruction::with_one_reg(InstrType::IN_PUSH, AddrMode::AM_R, RegType::RT_BC),
    0xC6_u8 => Instruction::with_one_reg(InstrType::IN_ADD, AddrMode::AM_R_A8, RegType::RT_A),
    0xC7_u8 => Instruction::new(InstrType::IN_RST, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NONE, 0x00),
    0xC8_u8 => Instruction::new(InstrType::IN_RET, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_Z, 0),
    0xC9_u8 => Instruction::default(InstrType::IN_RET, AddrMode::AM_IMP),
    0xCA_u8 => Instruction::new(InstrType::IN_JP, AddrMode::AM_D16,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_Z, 0),
    0xCC_u8 => Instruction::new(InstrType::IN_CALL, AddrMode::AM_D16_R,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_Z, 0),
    0xCD_u8 => Instruction::default(InstrType::IN_CALL, AddrMode::AM_D16),
    0xCE_u8 => Instruction::with_one_reg(InstrType::IN_ADC, AddrMode::AM_R_D8, RegType::RT_A),
    0xCF_u8 => Instruction::new(InstrType::IN_RST, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NONE, 0x08),

    // 0xD0 - 0xDF
    0xD0_u8 => Instruction::new(InstrType::IN_RET, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NC, 0),
    0xD1_u8 => Instruction::with_one_reg(InstrType::IN_POP, AddrMode::AM_R, RegType::RT_DE),
    0xD2_u8 => Instruction::new(InstrType::IN_JP, AddrMode::AM_D16,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NC, 0),
    0xD4_u8 => Instruction::new(InstrType::IN_CALL, AddrMode::AM_D16_R,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NC, 0),
    0xD5_u8 => Instruction::with_one_reg(InstrType::IN_PUSH, AddrMode::AM_R, RegType::RT_DE),
    0xD7_u8 => Instruction::new(InstrType::IN_RST, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NONE, 0x10),
    0xD8_u8 => Instruction::new(InstrType::IN_RET, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_C, 0),
    0xD9_u8 => Instruction::default(InstrType::IN_RETI, AddrMode::AM_IMP),
    0xDA_u8 => Instruction::new(InstrType::IN_JP, AddrMode::AM_D16,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_C, 0),
    0xDC_u8 => Instruction::new(InstrType::IN_CALL, AddrMode::AM_D16_R,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_C, 0),
    0xDF_u8 => Instruction::new(InstrType::IN_RST, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NONE, 0x18),

    // 0xE0 - 0xEF
    0xE0_u8 => Instruction::with_two_regs(InstrType::IN_LDH, AddrMode::AM_A8_R,
        RegType::RT_NONE, RegType::RT_A),
    0xE1_u8 => Instruction::with_one_reg(InstrType::IN_POP, AddrMode::AM_R, RegType::RT_HL),
    0xE2_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_MR_R,
        RegType::RT_C, RegType::RT_A),
    0xE5_u8 => Instruction::with_one_reg(InstrType::IN_PUSH, AddrMode::AM_R, RegType::RT_HL),
    0xE7_u8 => Instruction::new(InstrType::IN_RST, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NONE, 0x20),
    0xE8_u8 => Instruction::with_one_reg(InstrType::IN_ADD, AddrMode::AM_R_D8, RegType::RT_SP),
    0xE9_u8 => Instruction::with_one_reg(InstrType::IN_JP, AddrMode::AM_R, RegType::RT_HL),
    0xEA_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_A16_R,
        RegType::RT_NONE, RegType::RT_A),
    0xEF_u8 => Instruction::new(InstrType::IN_RST, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NONE, 0x28),

    // 0xF0 - 0xFF
    0xF0_u8 => Instruction::with_two_regs(InstrType::IN_LDH, AddrMode::AM_R_A8,
        RegType::RT_A, RegType::RT_NONE),
    0xF1_u8 => Instruction::with_one_reg(InstrType::IN_POP, AddrMode::AM_R, RegType::RT_AF),
    0xF2_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_A, RegType::RT_C),
    0xF3_u8 => Instruction::default(InstrType::IN_DI, AddrMode::AM_IMP),
    0xF5_u8 => Instruction::with_one_reg(InstrType::IN_PUSH, AddrMode::AM_R, RegType::RT_AF),
    0xF7_u8 => Instruction::new(InstrType::IN_RST, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NONE, 0x30),
    0xFA_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_A16,
        RegType::RT_A, RegType::RT_NONE),
    0xFE_u8 => Instruction::with_one_reg(InstrType::IN_CP, AddrMode::AM_R_D8, RegType::RT_A),
    0xFF_u8 => Instruction::new(InstrType::IN_RST, AddrMode::AM_IMP,
        RegType::RT_NONE, RegType::RT_NONE, CondType::CT_NONE, 0x38),
};


