#![allow(dead_code)]

use address;
use address::Region;
use processor;
use registers;

// TODO: Support serializing and deserializing to a human-readable file.

// TODO: GUI that shows state of address space, CPU registers, and
// derivative information like the current mode. Support basic
// debugging features, like breakpoints, run, step, etc.

pub struct Computer {
    pub cpu: processor::Processor,
    pub mem: address::MemMap32,
    pub big_endian: bool,
}

impl Computer {
    pub fn new(boot_code: Vec<address::Cell>) -> Computer {
        Computer {
            cpu: Default::default(),
            mem: address::MemMap32::new(boot_code),
            big_endian: false,  // TODO: look up endianness in the CPSR instead.
        }
    }

    pub fn execute_next_instruction(&mut self) {
        let pc_addr = self.cpu.register_file
            .lookup(registers::RegisterBank::R15).unwrap().bits;
        match self.instruction_at(pc_addr as address::Address) {
            Err(s) => panic!(s),
            Ok(instr) => if self.condition_satisfied(&instr) {
                match instr.mnemonic {
                    processor::Mnemonic::B => {
                        match self.cpu.register_file.lookup_mut(registers::RegisterBank::R15) {
                            None => panic!(),
                            Some(mut pc) => if let processor::InstructionTemplate::Cond_Offset {
                                cond: _,
                                offset: offset,
                            } = instr.args {
                                // interpret `offset` as a signed
                                // 24-bit twos complement integer
                                let signed_offset = {
                                    if offset & (1 << 23) == 0 {
                                        offset as i32
                                    } else {
                                        let hi_mask: u32 = 0b11111111 << 24;
                                        let n: i32 = (offset | hi_mask) as i32;
                                        debug_assert!(n < 0);
                                        n
                                    }
                                };
                                // TODO: address underflow/overflow
                                pc.bits = (pc.bits as i32 + 8 + (signed_offset << 2)) as u32;
                            }

                        }
                    },
                    _ => panic!("TODO: handle instruction {:?}", instr),
                }
            }
        }

    }

    pub fn instruction_at(&self, addr: address::Address) -> Result<processor::Instruction, String> {
        debug_assert!(addr % 4 == 0);
        debug_assert!(addr <= self.mem.address_space.end());
        match self.mem.get32(addr, self.big_endian) {
            None => Err("[ uninitialized memory ]".to_owned()),
            Some(word) => match self.cpu.decode_instruction(word) {
                None => Err(format!("[ ??? '{:#032b}' ]", word)),
                Some(instr) => {
                    print!("[ '{:#032b}' ] ", word);
                    Ok(instr)
                },
            },
        }
    }

    fn condition_satisfied(&self, cond: &processor::Instruction) -> bool {
        // TODO: read CPSR's condition flags and compare against
        // instruction's cond (if it exists; otherwise, default to
        // true).
        true
    }
}
