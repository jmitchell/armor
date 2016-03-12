// TODO: Ideally, don't expose any non-camel case types in the public
// interface.
#![allow(non_camel_case_types)]
#![allow(dead_code)]            // TODO: remove

use address::Address;
use registers::{
    Register32,
    RegisterBank,
    RegisterFile,
};


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
    RotateImmed { immed: u32, rotate: u32 },
    RRX(RegisterBank),
}

impl BarrelShiftOp {
    pub fn decode(src_reg: RegisterBank, op_code: u32, shift_size: ShiftSize) -> Option<BarrelShiftOp> {
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
pub enum AddressingOffset12 {
    Immed { base_addr: RegisterBank, offset12: u16 },
    Register { base_addr: RegisterBank, offset: RegisterBank },
    // ScaledRegister { base_addr: RegisterBank,
}

impl AddressingOffset12 {
    fn get_base(&self) -> &RegisterBank {
        match self {
            &AddressingOffset12::Immed { ref base_addr, offset12 } => base_addr,
            &AddressingOffset12::Register { ref base_addr, offset } => base_addr,
        }
    }

    fn get_offset(&self, positive: bool) -> i32 {
        match self {
            &AddressingOffset12::Immed { base_addr: _, offset12: offset } => {
                if positive {
                    offset as i32
                } else {
                    -(offset as i32)
                }
            },
            &AddressingOffset12::Register { base_addr: _, offset: offset_reg } => {
                panic!("TODO: need access to offset register");
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WordOrUnsignedByte {
    PreIndex { offset: AddressingOffset12, positive: bool, writeback: bool },
    PostIndex { offset: AddressingOffset12, positive: bool },
}

impl WordOrUnsignedByte {
    pub fn get_addr(&self, rn: &Register32) -> Address {
        ((rn.bits as i32) + self.get_offset_amount()) as Address
    }

    pub fn get_base(&self) -> &RegisterBank {
        &self.get_offset().get_base()
    }

    fn get_offset_amount(&self) -> i32 {
        self.get_offset().get_offset(self.is_positive_offset())
    }

    fn get_offset(&self) -> &AddressingOffset12 {
        match self {
            &WordOrUnsignedByte::PreIndex { ref offset, positive, writeback } => offset,
            &WordOrUnsignedByte::PostIndex { ref offset, positive } => offset,
        }
    }

    fn is_positive_offset(&self) -> bool {
        match self {
            &WordOrUnsignedByte::PreIndex { ref offset, positive, writeback } => positive,
            &WordOrUnsignedByte::PostIndex { ref offset, positive } => positive,
        }
    }

    fn is_writeback(&self) -> bool {
        match self {
            &WordOrUnsignedByte::PreIndex { ref offset, positive, writeback } => writeback,
            _ => false,
        }
    }

    pub fn handle_writeback(&self, rn: &mut Register32) {
        if self.is_writeback() {
            panic!("TODO: implement writeback handler");
            rn.bits = {
                // FIXME: some value in terms of rn.bits and self.
                rn.bits         // placeholder
            };
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AddressingOffset8 {
    Immed { base_addr: RegisterBank, offset8: u8 },
    Register { base_addr: RegisterBank, offset: RegisterBank },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum HalfwordOrSigned {
    PreIndex { offset: AddressingOffset8, positive: bool, writeback: bool },
    PostIndex { offset: AddressingOffset8, positive: bool },
}



#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CondInstr {
    // TODO: use BarrelShiftOp
    AND { s: bool, rd: RegisterBank, rn: RegisterBank, rotate: u32, immed: u32 },

    B(i32),
    BL(i32),
    BX(RegisterBank),
    BIC { s: bool, rd: RegisterBank, rn: RegisterBank, rotate: u32, immed: u32 },
    // LDR { u: bool, w: bool, rd: RegisterBank, rn: RegisterBank, immed12: u32 },
    LDR { rd: RegisterBank, addr_ref: WordOrUnsignedByte },
    LDRB { rd: RegisterBank, addr_ref: WordOrUnsignedByte },
    LDRH { rd: RegisterBank, addr_ref: HalfwordOrSigned },
    LDRSB { rd: RegisterBank, addr_ref: HalfwordOrSigned },
    LDRSH { rd: RegisterBank, addr_ref: HalfwordOrSigned },
    MCR { op1: u32, cn: u32, rd: RegisterBank, copro: u32, op2: u32, cm: u32 },
    MOV { s: bool, rd: RegisterBank, shift_op: BarrelShiftOp },
    MRC { op1: u32, cn: u32, rd: RegisterBank, copro: u32, op2: u32, cm: u32 },
    MRS { rd: RegisterBank, psr: RegisterBank },
    MSR { psr: RegisterBank, rm: RegisterBank, f: bool, s: bool, x: bool, c: bool },
    ORR { s: bool, rd: RegisterBank, rn: RegisterBank, rotate: u32, immed: u32 },
    STMDB { carrot: bool, w: bool, rn: RegisterBank, reg_list: Vec<RegisterBank> },
    STR { rd: RegisterBank, addr_ref: HalfwordOrSigned },
    STRB { rd: RegisterBank, addr_ref: HalfwordOrSigned },
    STRH { rd: RegisterBank, addr_ref: HalfwordOrSigned },
    SUB { s: bool, rd: RegisterBank, rn: RegisterBank, rotate: u32, immed: u32 },
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

fn cond_as_str(instr: &CondInstr, cond: &Condition) -> String {
    let cond_str = if *cond == Condition::AL {
        "".to_owned()
    } else {
        format!("{:?}", *cond)
    };

    match *instr {
        // TODO: compute effective address after rel_offset
        // CondInstr::B(rel_offset) => format!("B{} 0x{:0x}", cond_str, rel_offset),
        // CondInstr::BL(rel_offset) => format!("BL{} 0x{:0x}", cond_str, rel_offset),

        // TODO: add remaining instructions
        _ => format!("{:?} {:?}", cond, instr)
    }
}

fn uncond_as_str(instr: &UncondInstr) -> String {
    format!("{:?}", instr)
}

impl Instruction {
    pub fn as_str(&self) -> String {
        return match self {
            &Instruction::Cond(ref instr, ref cond) => cond_as_str(instr, cond),
            &Instruction::Uncond(ref instr) => uncond_as_str(instr),
        }
    }

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

    fn rel_offset(offset_bits: u32) -> i32 {
        let signed_num = {
            if offset_bits & (1 << 23) == 0 {
                offset_bits as i32
            } else {
                let hi_mask: u32 = 0b11111111 << 24;
                (offset_bits | hi_mask) as i32
            }
        };
        8 + (signed_num << 2)
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
                } else if bits(code, 21, 20) == 2 && bits(code, 15, 4) == 0b111100000000 {
                    Some(CondInstr::MSR {
                        psr: if bits(code, 22, 22) == 0 {
                            RegisterBank::CPSR
                        } else {
                            RegisterBank::SPSR
                        },
                        rm: RegisterBank::decode(bits(code, 3, 0)),
                        f: bits(code, 19, 19) == 1,
                        s: bits(code, 18, 18) == 1,
                        x: bits(code, 17, 17) == 1,
                        c: bits(code, 16, 16) == 1,
                    })
                } else if bits(code, 23, 23) == 1 && bits(code, 21, 21) == 1 && bits(code, 19, 16) == 0 && bits(code, 7, 7) == 0 && bits(code, 4, 4) == 0 {
                    let s = bits(code, 20, 20) == 1;
                    let rd = RegisterBank::decode(bits(code, 15, 12));
                    let shift_size = ShiftSize::Imm(bits(code, 11, 7));
                    let shift_code = bits(code, 6, 5);
                    let rm = RegisterBank::decode(bits(code, 3, 0));
                    if bits(code, 22, 22) == 0 {
                        Some(CondInstr::MOV {
                            s: s,
                            rd: rd,
                            shift_op: BarrelShiftOp::decode(rm, shift_code, shift_size).unwrap(),
                        })
                    } else {
                        None
                    }
                } else if bits(code, 23, 6) == 0b0010_1111_1111_1111_00 && bits(code, 4, 4) == 1 {
                    if bits(code, 5, 5) == 0 {
                        let rm = RegisterBank::decode(bits(code, 3, 0));
                        Some(CondInstr::BX(rm))
                    } else {
                        None // BLX
                    }
                } else {
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
                    0b010 =>
                        Some(CondInstr::SUB {
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
                    if bits(code, 21, 21) == 0 {
                        let s = bits(code, 20, 20) == 1;
                        let rn = RegisterBank::decode(bits(code, 19, 16));
                        let rd = RegisterBank::decode(bits(code, 15, 12));
                        let rotate = bits(code, 11, 8);
                        let immed = bits(code, 7, 0);
                        if bits(code, 22, 22) == 0 {
                            Some(CondInstr::ORR {
                                s: s,
                                rd: rd,
                                rn: rn,
                                rotate: rotate,
                                immed: immed,
                            })
                        } else {
                            Some(CondInstr::BIC {
                                s: s,
                                rd: rd,
                                rn: rn,
                                rotate: rotate,
                                immed: immed,
                            })
                        }
                    } else {
                        assert_eq!(bits(code, 19, 16), 0);
                        let s = bits(code, 20, 20) == 1;
                        let rd = RegisterBank::decode(bits(code, 15, 12));
                        let rotate = bits(code, 11, 8);
                        let immed = bits(code, 7, 0);
                        if bits(code, 22, 22) == 0 {
                            Some(CondInstr::MOV {
                                s: s,
                                rd: rd,
                                shift_op: BarrelShiftOp::RotateImmed {
                                    immed: immed,
                                    rotate: 2 * rotate,
                                },
                            })
                        } else {
                            None // MVN
                        }
                    }
                }
            },
            0b0101 => {
                let u = bits(code, 23, 23) == 1;
                let w = bits(code, 21, 21) == 1;
                let rn = RegisterBank::decode(bits(code, 19, 16));
                let rd = RegisterBank::decode(bits(code, 15, 12));
                let immed12 = bits(code, 11, 0);
                match (bits(code, 22, 22) << 1) | bits(code, 20, 20) {
                    0b01 => {
                        // Some(CondInstr::LDR {
                        //     u: u,
                        //     w: w,
                        //     rd: rd,
                        //     rn: rn,
                        //     immed12: immed12,
                        // })

                        // pre
                        // W=writeback
                        // U=up (positive offset, else neg)
                        Some(CondInstr::LDR {
                            rd: rd,
                            addr_ref: WordOrUnsignedByte::PreIndex {
                                offset: AddressingOffset12::Immed {
                                    base_addr: rn,
                                    offset12: immed12 as u16,
                                },
                                positive: u,
                                writeback: w,
                            },
                        })
                    },
                    _ => None
                }
            },
            0b1001 => {
                let carrot = bits(code, 22, 22) == 1;
                let w = bits(code, 21, 21) == 1;
                let rn = RegisterBank::decode(bits(code, 19, 16));
                let reg_list = {
                    let mut regs = vec![];
                    let mut n = bits(code, 15, 0);
                    let mut i = 0;
                    loop {
                        if n % 2 == 1 {
                            regs.push(RegisterBank::decode(i));
                        }
                        n >>= 1;
                        i += 1;

                        if n == 0 {
                            break;
                        }
                    }
                    regs
                };
                Some(CondInstr::STMDB {
                    carrot: carrot,
                    w: w,
                    rn: rn,
                    reg_list: reg_list,
                })
            },
            0b1010 => {
                Some(CondInstr::B(Self::rel_offset(bits(code, 23, 0))))
            },
            0b1011 => {
                Some(CondInstr::BL(Self::rel_offset(bits(code, 23, 0))))
            },
            0b1110 => {
                let cn = bits(code, 19, 16);
                let copro = bits(code, 11, 8);
                let op2 = bits(code, 7, 5);
                let cm = bits(code, 3, 0);
                if bits(code, 4, 4) == 0 {
                    None        // TODO: CDP
                } else {
                    let op1 = bits(code, 23, 21);
                    let rd = RegisterBank::decode(bits(code, 15, 12));
                    if bits(code, 20, 20) == 0 {
                        Some(CondInstr::MCR {
                            op1: op1,
                            cn: cn,
                            rd: rd,
                            copro: copro,
                            op2: op2,
                            cm: cm,
                        })
                    } else {
                        Some(CondInstr::MRC {
                            op1: op1,
                            cn: cn,
                            rd: rd,
                            copro: copro,
                            op2: op2,
                            cm: cm,
                        })
                    }
                }
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
    use super::{Condition, Instruction, CondInstr, WordOrUnsignedByte, AddressingOffset12, ShiftSize, BarrelShiftOp};
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

            (0b0001_0011_110000000000000000011111,
             Instruction::Cond(
                 CondInstr::BIC {
                     s: false,
                     rd: RegisterBank::R0,
                     rn: RegisterBank::R0,
                     rotate: 0,
                     immed: 0b00011111,
                 },
                 Condition::NE)),

            (0b0001_0011_1000_00000000000000010011,
             Instruction::Cond(
                 CondInstr::ORR {
                     s: false,
                     rd: RegisterBank::R0,
                     rn: RegisterBank::R0,
                     rotate: 0,
                     immed: 0b00010011,
                 },
                 Condition::NE)),

            (0b1110_0001_0010_1001_1111000000000000,
             Instruction::Cond(
                 CondInstr::MSR {
                     psr: RegisterBank::CPSR,
                     rm: RegisterBank::R0,
                     f: true,
                     s: false,
                     x: false,
                     c: true,
                 },
                 Condition::AL)),

            (0b1110_1110_0001_0001_0000_1111_0001_0000,
             Instruction::Cond(
                 CondInstr::MRC {
                     op1: 0,
                     cn: 1,
                     rd: RegisterBank::R0,
                     copro: 0b1111,
                     op2: 0,
                     cm: 0,
                 },
                 Condition::AL)),

            (0b1110_1110_0000_0001_0000_1111_0001_0000,
             Instruction::Cond(
                 CondInstr::MCR {
                     op1: 0,
                     cn: 1,
                     rd: RegisterBank::R0,
                     copro: 0b1111,
                     op2: 0,
                     cm: 0,
                 },
                 Condition::AL)),

            (0b1110_0101_1001_1111_0000_0000_0110_1100,
             Instruction::Cond(
                 CondInstr::LDR {
                     rd: RegisterBank::R0,
                     addr_ref: WordOrUnsignedByte::PreIndex {
                         offset: AddressingOffset12::Immed {
                             base_addr: RegisterBank::R15,
                             offset12: 0b0000_0110_1100,
                         },
                         positive: true,
                         writeback: false,
                     }
                 },
                 Condition::AL)),

            (0b1110_1011_0000_0000_0000_0000_0011_1001,
             Instruction::Cond(
                 CondInstr::BL(236),
                 Condition::AL)),

            (0b1110_0001_1010_0000_0000_0000_0000_1101,
             Instruction::Cond(
                 CondInstr::MOV {
                     s: false,
                     rd: RegisterBank::R0,
                     shift_op: BarrelShiftOp::LSL(RegisterBank::R13, ShiftSize::Imm(0)),
                 },
                 Condition::AL)),

            (0b1110_0010_0100_0000_0000_1101_0001_0011,
             Instruction::Cond(
                 CondInstr::SUB {
                     s: false,
                     rd: RegisterBank::R0,
                     rn: RegisterBank::R0,
                     rotate: 0b1101,
                     immed: 0b00010011,
                 },
                 Condition::AL)),

            (0b1110_0001_0010_1111_1111_1111_0001_1110,
             Instruction::Cond(
                 CondInstr::BX(RegisterBank::R14),
                 Condition::AL)),

            (0b1110_1001_0010_1101_0100_0000_0001_0000,
             Instruction::Cond(
                 CondInstr::STMDB {
                     carrot: false,
                     w: true,
                     rn: RegisterBank::R13,
                     reg_list: vec![ RegisterBank::R4, RegisterBank::R14 ],
                 },
                 Condition::AL)),

            (0b1110_0011_1010_0000_0001_0000_0000_0000,
             Instruction::Cond(
                 // TODO: support this other MOV instruction too
                 CondInstr::MOV {
                     s: false,
                     rd: RegisterBank::R1,
                     shift_op: BarrelShiftOp::RotateImmed {
                         immed: 0,
                         rotate: 0,
                     },
                 },
                 Condition::AL)),
        ];

        for (code, expected_instr) in decodings {
            assert_eq!(Instruction::decode(code).unwrap(), expected_instr);
        }
    }

    #[test]
    fn roundtrip_instructions() {
        // TODO: For several instructions, verify that
        //
        //     decode(encode(instruction)) == instruction
        //
        // Requires implementing an encoder.
    }
}
