#![allow(dead_code)]

use address;
use address::Region;
use processor;
use processor::{
    Condition,
    CondInstr,
    Instruction,
    UncondInstr,
};
use registers::{
    Register32,
    RegisterBank,
};

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
        let pc_addr = self.program_counter().bits;
        match self.instruction_at(pc_addr as address::Address) {
            Err(s) => panic!(s),
            Ok(instr) => self.execute(instr),
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

    fn program_counter(&mut self) -> &mut Register32 {
        self.cpu.register_file.lookup_mut(RegisterBank::R15).unwrap()
    }

    fn execute(&mut self, instr: Instruction) {
        match instr {
            Instruction::Cond(instr, cond) =>
                if self.condition_satisfied(cond) {
                    self.execute_conditional(&instr)
                },
            Instruction::Uncond(instr) => (),
        }
    }

    fn condition_satisfied(&self, cond: Condition) -> bool {
        true                    // TODO: implement properly
    }

    fn execute_conditional(&mut self, instr: &CondInstr) {
        match *instr {
            CondInstr::B(rel_offset) => {
                let mut pc = self.program_counter();
                pc.bits = ((pc.bits as i32) + rel_offset) as u32;
            },
            _ => panic!("Failed to execute instruction {:?}", *instr),
        }
    }
}
