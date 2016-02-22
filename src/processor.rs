#![allow(dead_code)]

use registers;
use std::fmt;

pub struct Processor {
    pub register_file: registers::RegisterFile,
    // TODO: add other parts as needed
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            register_file: Default::default(),
        }
    }

    pub fn decode_instruction(&mut self, data: u32) -> Instruction
    {
        Instruction::new(data)
    }
}

impl Default for Processor {
    fn default() -> Processor {
        Processor::new()
    }
}

pub struct Instruction {
    // TODO: decoded representation of instructions
    pub data: u32,
}

impl Instruction {
    fn new(data: u32) -> Instruction {
        Instruction {
            data: data,
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Reverse the bits because by convention bit 31 is displayed
        // on the left.
        // let rev_bits = {
        //     let mut ans: u32 = 0;
        //     let mut orig = self.data;
        //     for i in 0..32 {
        //         if (orig >> i) % 2 == 1 {
        //             ans |= 1 << (31 - i);
        //         }
        //     }
        //     ans
        // };
        // write!(f, "{:0>32b}", rev_bits)
        write!(f, "{:0>32b}", self.data)
    }
}
