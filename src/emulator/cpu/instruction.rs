use phf::{phf_map, Map};

/* Addressing mode */
#[derive(Debug, PartialEq, Eq)]
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
pub enum CondType {
    CT_NONE,
    CT_NZ,
    CT_Z,
    CT_NC,
    CT_C
}

/* Instruction type */
#[derive(strum_macros::Display, Eq, PartialEq, Hash, Debug)]
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
    0x05_u8 => Instruction::with_one_reg(InstrType::IN_DEC, AddrMode::AM_R, RegType::RT_B),
    0x06_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_B),
    0x08_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_A16_R,
        RegType::RT_NONE, RegType::RT_HL),
    0x0A_u8 => Instruction::with_two_regs(InstrType::IN_LD, AddrMode::AM_R_MR,
        RegType::RT_A, RegType::RT_BC),
    0x0E_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D8, RegType::RT_C),

    // 0x10 - 0x1F
    0x10_u8 => Instruction::default(InstrType::IN_STOP, AddrMode::AM_D8),
    0x11_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D16, RegType::RT_DE),

    // 0x20 - 0x2F
    // 0x30 - 0x3F
    0x31_u8 => Instruction::with_one_reg(InstrType::IN_LD, AddrMode::AM_R_D16, RegType::RT_DE),

    // 0x40 - 0x4F
    0x40_u8 => Instruction::new(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_B, RegType::RT_B, CondType::CT_NONE, 0),
    0x41_u8 => Instruction::new(InstrType::IN_LD, AddrMode::AM_R_R,
        RegType::RT_B, RegType::RT_C, CondType::CT_NONE, 0),
    
    // 0xA0 - 0xAF
    0xAF_u8 => Instruction::with_two_regs(InstrType::IN_XOR, AddrMode::AM_R_R,
        RegType::RT_A, RegType::RT_A),

    // 0xC0 - 0xCF
    0xC3_u8 => Instruction::default(InstrType::IN_JP, AddrMode::AM_D16),

    // 0xD0 - 0xDF
    // 0xE0 - 0xEF

    // 0xF0 - 0xFF
    0xF3_u8 => Instruction::default(InstrType::IN_DI, AddrMode::AM_IMP),
    0xFE_u8 => Instruction::with_one_reg(InstrType::IN_CP, AddrMode::AM_R_D8, RegType::RT_A),
};


