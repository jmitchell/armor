// TODO: Ideally, don't expose any non-camel case types in the public
// interface.
#![allow(non_camel_case_types)]
#![allow(dead_code)]            // TODO: remove

use registers::{RegisterBank, RegisterFile};


pub struct Processor {
    pub register_file: RegisterFile, // TODO: add other parts as needed
}

impl Processor {
    pub fn new() -> Processor {
        Processor { register_file: Default::default() }
    }

    pub fn decode_instruction(&self, data: u32) -> Option<Instruction> {
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
pub enum Condition {
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

#[cfg_attr(rustfmt, rustfmt_skip)]
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
                return i as u32;
            }
        }
        unreachable!()
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
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

impl RegisterBank {
    fn decode(code: u32) -> RegisterBank {
        let index = code as usize;
        assert!(index < REGISTER_BANK_TABLE.len());
        REGISTER_BANK_TABLE[index].clone()
    }
}

impl Encodable for RegisterBank {
    fn encode(register_bank: RegisterBank) -> u32 {
        for i in 0..REGISTER_BANK_TABLE.len() {
            if REGISTER_BANK_TABLE[i] == register_bank {
                return i as u32;
            }
        }
        unreachable!()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ShiftSize {
    Imm(u32),
    Reg(RegisterBank),
}

#[derive(Clone, PartialEq, Eq, Debug)]
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
    fn decode(src_reg: RegisterBank, op_code: u32, shift_size: ShiftSize) -> Option<BarrelShiftOp> {
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
                    }
                    ShiftSize::Reg(_) => Some(BarrelShiftOp::LSR(src_reg, shift_size)),
                }
            }
            0b10 => {
                match shift_size {
                    ShiftSize::Imm(n) => {
                        let amount = if n == 0 {
                            32u32
                        } else {
                            n
                        };
                        Some(BarrelShiftOp::ASR(src_reg, ShiftSize::Imm(amount)))
                    }
                    ShiftSize::Reg(_) => Some(BarrelShiftOp::ASR(src_reg, shift_size)),
                }
            }
            0b11 => {
                match shift_size {
                    ShiftSize::Imm(n) => {
                        if n == 0 {
                            Some(BarrelShiftOp::RRX(src_reg))
                        } else {
                            Some(BarrelShiftOp::ROR(src_reg, ShiftSize::Imm(n)))
                        }
                    }
                    ShiftSize::Reg(_) => Some(BarrelShiftOp::ROR(src_reg, shift_size)),
                }
            }
            _ => {
                // 'The shift value is implicit: for PKHBT it is 00. For
                // PKHTB it is 10. For SAT it is 2*sh.'
                panic!("TODO: see last row in Table B.4");
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CondInstr {
    // TODO: use BarrelShiftOp
    AND { s: bool, rd: RegisterBank, rn: RegisterBank, rotate: u32, immed: u32 },

    B(i32),
    MRS { rd: RegisterBank, psr: RegisterBank },
    //MSR { psr: RegisterBank, f: bool, s: bool, x: bool, c: bool, rotate: u32, immed: u32 },
    TEQ { rn: RegisterBank, rotate: u32, immed: u32 },

    // TODO: Remove when done. Helps avert unreachable pattern errors
    // during development.
    DUMMY,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum UncondInstr {
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Instruction {
    Cond(CondInstr, Condition),
    Uncond(UncondInstr),
}

fn bits(n: u32, hi: u16, lo: u16) -> u32 {
    debug_assert!(lo <= hi && hi < 32);
    let mask = (1 << (hi - lo + 1)) - 1;
    (n >> lo) & mask
}

impl Instruction {
    fn decode(code: u32) -> Option<Instruction> {
        match Condition::decode(bits(code, 31, 28)) {
            Some(condition) =>
                if let Some(instr) = Self::decode_conditional(code) {
                    Some(Instruction::Cond(instr, condition))
                } else {
                    None
                },
            None =>
                if let Some(instr) = Self::decode_unconditional(code) {
                    Some(Instruction::Uncond(instr))
                } else {
                    None
                },
        }
    }

    fn decode_conditional(code: u32) -> Option<CondInstr> {
        match bits(code, 27, 24) {
            0b0001 => {
                if bits(code, 21, 16) == 0b001111 && bits(code, 11, 0) == 0 {
                    let rd = RegisterBank::decode(bits(code, 15, 12));
                    debug_assert!(rd != RegisterBank::R15);
                    Some(CondInstr::MRS {
                        rd: rd,
                        psr: if bits(code, 22, 22) == 0 {
                            RegisterBank::CPSR
                        } else {
                            RegisterBank::SPSR
                        },
                    })
                } else {
                    // TODO
                    None
                }
            },
            0b0010 => {
                let s = bits(code, 20, 20) == 1;
                let rn = RegisterBank::decode(bits(code, 19, 16));
                let rd = RegisterBank::decode(bits(code, 15, 12));
                let rotate = bits(code, 11, 8);
                let immed = bits(code, 7, 0);
                match bits(code, 23, 21) {
                    0b000 =>
                        Some(CondInstr::AND {
                            s: s,
                            rd: rd,
                            rn: rn,
                            rotate: rotate,
                            immed: immed,
                        }),
                    _ => None
                }
            },
            0b0011 => {
                if bits(code, 23, 23) == 0 {
                    if bits(code, 20, 20) == 0 {
                        None
                    } else {
                        debug_assert!(bits(code, 15, 12) == 0);
                        let rn = RegisterBank::decode(bits(code, 19, 16));
                        let rotate = bits(code, 11, 8);
                        let immed = bits(code, 7, 0);
                        match bits(code, 22, 21) {
                            0b01 => {
                                Some(CondInstr::TEQ {
                                    rn: rn,
                                    rotate: rotate,
                                    immed: immed
                                })
                            },
                            _ => None,
                        }
                    }
                } else {
                    None
                }
            },
            0b1010 => {
                let offset_bits = bits(code, 23, 0);
                let signed_num = {
                    if offset_bits & (1 << 23) == 0 {
                        offset_bits as i32
                    } else {
                        let hi_mask: u32 = 0b11111111 << 24;
                        (offset_bits | hi_mask) as i32
                    }
                };
                let rel_offset = 8 + (signed_num << 2);
                Some(CondInstr::B(rel_offset))
            },
            x => {
                None
            }
        }
    }

    fn decode_unconditional(_code: u32) -> Option<UncondInstr> {
        None
    }
}



#[cfg(test)]
mod test {
    use super::{Condition, Instruction, CondInstr};
    use registers::RegisterBank;

    #[test]
    fn decode_instructions() {
        let decodings = vec![
            (0b1110_1010_000000000000000010111110,
             Instruction::Cond(
                 CondInstr::B(768),
                 Condition::AL)),

            (0b1110_0001_000011110000000000000000,
             Instruction::Cond(
                 CondInstr::MRS {
                     rd: RegisterBank::R0,
                     psr: RegisterBank::CPSR,
                 },
                 Condition::AL)),

            (0b1110_0010_0000_0000_0001_0000_00011111,
             Instruction::Cond(
                 CondInstr::AND {
                     s: false,
                     rn: RegisterBank::R0,
                     rd: RegisterBank::R1,
                     rotate: 0,
                     immed: 0b00011111,
                 },
                 Condition::AL)),

            (0b1110_0011_0011_000100000_0000_0011010,
             Instruction::Cond(
                 CondInstr::TEQ {
                     rn: RegisterBank::R1,
                     rotate: 0,
                     immed: 0b0011010,
                 },
                 Condition::AL)),
        ];

        for (code, expected_instr) in decodings {
            assert_eq!(Instruction::decode(code).unwrap(), expected_instr);
        }
    }

}
