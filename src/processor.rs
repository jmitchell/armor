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

fn decode_shift(src_reg: RegisterBank, shift: u32, shift_size: u32, reg: Option<RegisterBank>) ->
    Option<BarrelShiftTemplate>
{
    debug_assert!(shift_size <= 32 || reg.is_some());
    match shift {
        0b00 => {
            match reg {
                None => Some(BarrelShiftTemplate::LSL(src_reg, Either::Left(shift_size))),
                Some(rs) => Some(BarrelShiftTemplate::LSL(src_reg, Either::Right(rs))),
            }
        },
        0b01 => {
            match reg {
                None => {
                    let n = if shift_size == 0 {
                        32u32
                    } else {
                        shift_size
                    };
                    Some(BarrelShiftTemplate::LSR(src_reg, Either::Left(n)))
                },
                Some(rs) => Some(BarrelShiftTemplate::LSR(src_reg, Either::Right(rs))),
            }
        },
        0b10 => {
            match reg {
                None => {
                    let n = if shift_size == 0 {
                        32u32
                    } else {
                        shift_size
                    };
                    Some(BarrelShiftTemplate::ASR(src_reg, Either::Left(n)))
                },
                Some(rs) => Some(BarrelShiftTemplate::ASR(src_reg, Either::Right(rs))),
            }
        },
        0b11 => {
            match reg {
                None => {
                    if shift_size == 0 {
                        Some(BarrelShiftTemplate::RRX(src_reg))
                    } else {
                        Some(BarrelShiftTemplate::ROR(src_reg, Either::Left(shift_size)))
                    }
                },
                Some(rs) => Some(BarrelShiftTemplate::ROR(src_reg, Either::Right(rs))),
            }
        },
        _ => {
            // 'The shift value is implicit: for PKHBT it is 00. For
            // PKHTB it is 10. For SAT it is 2*sh.'
            panic!("TODO: see last row in Table B.4");
        },
    }
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
                    0b0001 => {
                        // Cond_S_Rd_N
                        if code & (1 << 23) == 0 {
                            println!("TODO");
                        } else {
                            if code & (1 << 4) == 0 {
                                if code & (1 << 21) == 0 {
                                    println!("TODO: ORR, BIC")
                                } else {
                                    // TODO: BarrelShiftTemplate from shift, shift_size, and Rm

                                    // mnemonic = Some(if code & (1 << 22) == 0 {
                                    //     Mnemonic::MOV
                                    // } else {
                                    //     Mnemonic::MVN
                                    // });
                                    // template = Some(InstructionTemplate::Cond_S_Rd_N {
                                    //     cond: cond,
                                    //     s_flag: code & (1 << 20) == 1,
                                    //     rd: decode_register((code >> 12) & 0b1111).unwrap(),
                                    //     n: ,
                                    // });
                                }
                            } else {
                                println!("TODO");
                            }
                        }
                    },
                    0b0100 => {
                        println!("TODO: decode STR, LDR, STRB, and LDRB(post) with U, T and Imm12");
                    },
                    0b0101 => {
                        println!("TODO: decode STR, LDR, STRB, and LDRB(pre) with U, W, and Imm12");
                    },
                    0b0110 => {
                        println!("TODO: decode STR, LDR, STRB, and LDRB(pre) with U, T, and shift op");
                    },
                    0b1010 => {
                        mnemonic = Some(Mnemonic::B);
                        template = Some(InstructionTemplate::Cond_Offset {
                            cond: cond,
                            offset: code & ((1 << 25) - 1),
                        });
                    },
                    0b1011 => {
                        mnemonic = Some(Mnemonic::BL);
                        template = Some(InstructionTemplate::Cond_Offset {
                            cond: cond,
                            offset: code & ((1 << 25) - 1),
                        });
                    },
                    x => println!("Unrecognized bits [27:24]: {:04b}", x),
                }
            },
            None => {
                println!("TODO: decode unconditional instructions");
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
