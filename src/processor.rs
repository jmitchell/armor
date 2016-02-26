// TODO: Ideally, don't expose any non-camel case types in the public
// interface.
#![allow(non_camel_case_types)]
#![allow(dead_code)]            // TODO: remove

use registers::{
    RegisterBank,
    RegisterFile,
};


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


trait Decodable where Self : Sized {
    fn decode(code: u32) -> Option<Self>;
}

trait Encodable where Self : Sized {
    fn encode(val: Self) -> u32;
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

impl Decodable for Condition {
    fn decode(code: u32) -> Option<Condition> {
        let index = code as usize;
        if index < CONDITION_TABLE.len() {
            Some(CONDITION_TABLE[index as usize].clone())
        } else {
            None
        }
    }
}

impl Encodable for Condition {
    fn encode(cond: Condition) -> u32 {
        for i in 0..CONDITION_TABLE.len() {
            if CONDITION_TABLE[i] == cond {
                return i as u32
            }
        }
        unreachable!()
    }
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

impl Decodable for RegisterBank {
    fn decode(code: u32) -> Option<RegisterBank> {
        let index = code as usize;
        if index < REGISTER_BANK_TABLE.len() {
            Some(REGISTER_BANK_TABLE[index as usize].clone())
        } else {
            None
        }
    }
}

impl Encodable for RegisterBank {
    fn encode(register_bank: RegisterBank) -> u32 {
        for i in 0..REGISTER_BANK_TABLE.len() {
            if REGISTER_BANK_TABLE[i] == register_bank {
                return i as u32
            }
        }
        unreachable!()
    }
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
pub enum ShiftSize {
    Imm(u32),
    Reg(RegisterBank),
}

#[derive(PartialEq, Eq, Debug)]
pub enum BarrelShiftOp {
    // TODO: Are Imm and Reg needed here? They're included in Table
    // 3.3, but not in Table B.4.
    Imm(u32),
    Reg(RegisterBank),
    LSL(RegisterBank, ShiftSize),
    LSR(RegisterBank, ShiftSize),
    ASR(RegisterBank, ShiftSize),
    ROR(RegisterBank, ShiftSize),
    RRX(RegisterBank),
}

impl BarrelShiftOp {
    fn decode(src_reg: RegisterBank, op_code: u32, shift_size: ShiftSize) ->
        Option<BarrelShiftOp>
    {
        debug_assert!(match shift_size {
            ShiftSize::Imm(n) => n <= 32,
            ShiftSize::Reg(_) => true,
        });

        match op_code {
            0b00 => Some(BarrelShiftOp::LSL(src_reg, shift_size)),
            0b01 => {
                match shift_size {
                    ShiftSize::Imm(n) => {
                        let amount = if n == 0 {
                            32u32
                        } else {
                            n
                        };
                        Some(BarrelShiftOp::LSR(src_reg, ShiftSize::Imm(amount)))
                    },
                    ShiftSize::Reg(_) => Some(BarrelShiftOp::LSR(src_reg, shift_size)),
                }
            },
            0b10 => {
                match shift_size {
                    ShiftSize::Imm(n) => {
                        let amount = if n == 0 {
                            32u32
                        } else {
                            n
                        };
                        Some(BarrelShiftOp::ASR(src_reg, ShiftSize::Imm(amount)))
                    },
                    ShiftSize::Reg(_) => Some(BarrelShiftOp::ASR(src_reg, shift_size)),
                }
            },
            0b11 => {
                match shift_size {
                    ShiftSize::Imm(n) => {
                        if n == 0 {
                            Some(BarrelShiftOp::RRX(src_reg))
                        } else {
                            Some(BarrelShiftOp::ROR(src_reg, ShiftSize::Imm(n)))
                        }
                    },
                    ShiftSize::Reg(_) => Some(BarrelShiftOp::ROR(src_reg, shift_size)),
                }
            },
            _ => {
                // 'The shift value is implicit: for PKHBT it is 00. For
                // PKHTB it is 10. For SAT it is 2*sh.'
                panic!("TODO: see last row in Table B.4");
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum InstructionTemplate {
    // Comparisons: CMN, CMP, TEQ, TST
    // NB: comparisons implicitly set the condition flags
    Cond_Rd_N {
        cond: Condition,
        rd: RegisterBank,
        n: BarrelShiftOp,
    },

    // MOV, MVN, ...
    Cond_S_Rd_N {
        cond: Condition,
        s_flag: bool,
        rd: RegisterBank,
        n: BarrelShiftOp,
    },

    // ADC, ADD, RSB, RSC, SBC, SUB, AND, ORR, EOR, BIC, ...
    Cond_S_Rd_Rn_N {
        cond: Condition,
        s_flag: bool,
        rd: RegisterBank,
        rn: RegisterBank,
        n: BarrelShiftOp,
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

        match Condition::decode(code >> 28) {
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
                                    // TODO: BarrelShiftOp from shift, shift_size, and Rm

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
