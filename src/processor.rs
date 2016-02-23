#![allow(dead_code)]

use registers::{
    RegisterBank,
    RegisterFile,
};
use std::fmt;

pub struct Processor {
    pub register_file: RegisterFile,
    // TODO: add other parts as needed
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            register_file: Default::default(),
        }
    }

    pub fn decode_instruction(&mut self, data: u32) -> Option<Instruction>
    {
        Instruction::decode(data)
    }
}

impl Default for Processor {
    fn default() -> Processor {
        Processor::new()
    }
}




#[derive(Clone, PartialEq, Eq, Debug)]
enum Condition {
    EQ,
    NE,
    CS_HS,
    CC_LO,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    AL,
}

const CONDITION_TABLE: [Condition; 15] = [
    Condition::EQ,
    Condition::NE,
    Condition::CS_HS,
    Condition::CC_LO,
    Condition::MI,
    Condition::PL,
    Condition::VS,
    Condition::VC,
    Condition::HI,
    Condition::LS,
    Condition::GE,
    Condition::LT,
    Condition::GT,
    Condition::LE,
    Condition::AL,
];

fn decode_condition(code: u32) -> Option<Condition> {
    let index = code as usize;
    if index < CONDITION_TABLE.len() {
        Some(CONDITION_TABLE[index as usize].clone())
    } else {
        None
    }
}

fn encode_condition(cond: Condition) -> u32 {
    for i in 0..CONDITION_TABLE.len() {
        if CONDITION_TABLE[i] == cond {
            return i as u32
        }
    }
    unreachable!()
}


const REGISTER_BANK_TABLE: [RegisterBank; 16] = [
    RegisterBank::R0,
    RegisterBank::R1,
    RegisterBank::R2,
    RegisterBank::R3,
    RegisterBank::R4,
    RegisterBank::R5,
    RegisterBank::R6,
    RegisterBank::R7,
    RegisterBank::R8,
    RegisterBank::R9,
    RegisterBank::R10,
    RegisterBank::R11,
    RegisterBank::R12,
    RegisterBank::R13,
    RegisterBank::R14,
    RegisterBank::R15,
];

fn decode_register(code: u32) -> Option<RegisterBank> {
    let index = code as usize;
    if index < REGISTER_BANK_TABLE.len() {
        Some(REGISTER_BANK_TABLE[index as usize].clone())
    } else {
        None
    }
}

fn encode_register(register_bank: RegisterBank) -> u32 {
    for i in 0..REGISTER_BANK_TABLE.len() {
        if REGISTER_BANK_TABLE[i] == register_bank {
            return i as u32
        }
    }
    unreachable!()
}

#[derive(PartialEq, Eq, Debug)]
pub enum Mnemonic {
    ADC,
    ADD,
    AND,
    B,
    BIC,
    BKPT,
    BL,
    BLX,
    BX,
    CDP,
    CDP2,
    CLZ,
    CMN,
    CMP,
    EOR,
    LDC,
    LDC2,
    LDM,
    LDR,
    MCR,
    MCR2,
    MCRR,
    MLA,
    MOV,
    MRC,
    MRC2,
    MRRC,
    MRS,
    MSR,
    MUL,
    MVN,
    ORR,
    PLD,
    QADD,
    QDADD,
    QDSUB,
    QSUB,
    RSB,
    RSC,
    SBC,
    SMLAxy,
    SMLAL,
    SMLALxy,
    SMLAWy,
    SMULL,
    SMULxy,
    SMULWy,
    STC,
    STC2,
    STM,
    STR,
    SUB,
    SWI,
    SWP,
    TEQ,
    TST,
    UMLAL,
    UMULL,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Either<X, Y> {
    Left(X),
    Right(Y),
}

#[derive(PartialEq, Eq, Debug)]
pub enum BarrelShiftTemplate {
    Imm(u32),
    Reg(RegisterBank),
    LSL(RegisterBank, Either<u32, RegisterBank>),
    LSR(RegisterBank, Either<u32, RegisterBank>),
    ASR(RegisterBank, Either<u32, RegisterBank>),
    ROR(RegisterBank, Either<u32, RegisterBank>),
    RRX(RegisterBank),
}

#[derive(PartialEq, Eq, Debug)]
enum InstructionTemplate {
    // Comparisons: CMN, CMP, TEQ, TST
    // NB: comparisons implicitly set the condition flags
    Cond_Rd_N {
        cond: Condition,
        rd: RegisterBank,
        n: BarrelShiftTemplate,
    },

    // MOV, MVN, ...
    Cond_S_Rd_N {
        cond: Condition,
        s_flag: bool,
        rd: RegisterBank,
        n: BarrelShiftTemplate,
    },

    // ADC, ADD, RSB, RSC, SBC, SUB, AND, ORR, EOR, BIC, ...
    Cond_S_Rd_Rn_N {
        cond: Condition,
        s_flag: bool,
        rd: RegisterBank,
        rn: RegisterBank,
        n: BarrelShiftTemplate,
    },

    // MUL
    Cond_S_Rd_Rm_Rs {
        cond: Condition,
        rd: RegisterBank,
        rm: RegisterBank,
        rs: RegisterBank,
    },

    // MLA
    Cond_S_Rd_Rm_Rs_Rn {
        cond: Condition,
        rd: RegisterBank,
        rm: RegisterBank,
        rs: RegisterBank,
        rn: RegisterBank,
    },

    // SMLAL, SMULL, UMLAL, UMULL
    Cond_S_RdLo_RdHi_Rm_Rs {
        cond: Condition,
        rd_lo: RegisterBank,
        rd_hi: RegisterBank,
        rm: RegisterBank,
        rs: RegisterBank,
    },

    // B, BL
    Cond_Offset {
        cond: Condition,
        offset: u32,
    },

    // BX
    Cond_Rm {
        cond: Condition,
        rm: RegisterBank,
    },

    // BLX
    Cond_A_Offset {
        cond: Condition,
        a: bool,
        offset: u32,
    },

    // TODO: Continue from ASDG:3.3
}

#[derive(PartialEq, Eq, Debug)]
pub struct Instruction {
    mnemonic: Mnemonic,
    args: InstructionTemplate,
}

impl Instruction {
    fn new(mnemonic: Mnemonic, args: InstructionTemplate) -> Instruction {
        Instruction {
            mnemonic: mnemonic,
            args: args,
        }
    }
}

impl Instruction {

    fn decode(code: u32) -> Option<Instruction> {
        let mut mnemonic: Option<Mnemonic> = None;
        let mut template: Option<InstructionTemplate> = None;

        match decode_condition(code >> 28) {
            Some(cond) => {
                match (code >> 24) & 0b1111 {
                    0b0100 => {
                        panic!("TODO: decode STR, LDR, STRB, and LDRB(post) with U, T and Imm12");
                    },
                    0b0101 => {
                        panic!("TODO: decode STR, LDR, STRB, and LDRB(pre) with U, W, and Imm12");
                    },
                    0b0110 => {
                        panic!("TODO: decode STR, LDR, STRB, and LDRB(pre) with U, T, and shift op");
                    },
                    0b1010 => {
                        mnemonic = Some(Mnemonic::B);
                        template = Some(InstructionTemplate::Cond_Offset {
                            cond: cond,
                            offset: code & 0b111111111111111111111111,
                        });
                    },
                    x => panic!("Unrecognized bits [27:24]: {:04b}", x),
                }
            },
            None => {
                panic!("TODO: decode unconditional instructions");
            },
        }

        if let (Some(mnem), Some(args)) = (mnemonic, template) {
            Some(Instruction::new(mnem, args))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        Condition,
        Instruction,
        InstructionTemplate,
        Mnemonic,
    };

    #[test]
    fn decode_branch_instruction() {
        let code: u32 = 0b1110_1010_111111111111111111111111;
        assert_eq!(Instruction::decode(code).unwrap(),
                   Instruction::new(
                       Mnemonic::B,
                       InstructionTemplate::Cond_Offset {
                           cond: Condition::AL,
                           offset: 0b111111111111111111111111,
                       }));
    }
}
