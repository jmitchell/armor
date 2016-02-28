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
    ConditionFlag,
    ProgramStatusRegister,
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
                Some(instr) => Ok(instr),
            },
        }
    }

    fn register(&mut self, reg_bank: RegisterBank) -> Option<&mut Register32> {
        self.cpu.register_file.lookup_mut(reg_bank)
    }

    fn program_counter(&mut self) -> &mut Register32 {
        self.register(RegisterBank::R15).unwrap()
    }

    fn execute(&mut self, instr: Instruction) {
        match instr {
            Instruction::Cond(instr, cond) =>
                if self.condition_satisfied(cond) {
                    self.execute_conditional(&instr)
                },
            Instruction::Uncond(instr) => (),
        }
        self.program_counter().bits += 4;
    }

    fn condition_satisfied(&self, cond: Condition) -> bool {
        true                    // TODO: implement properly
    }

    // TODO: delegate to the BarrelShiftOp
    fn ror(x: u32, y: u32) -> u32 {
        if y == 0 {
            x
        } else {
            (x >> y) | (x << (32 - y))
        }
    }

    fn execute_conditional(&mut self, instr: &CondInstr) {
        match *instr {
            CondInstr::AND { s, rd, rn, rotate, immed } => {
                let bits = self.register(rn).unwrap().bits & Self::ror(immed, 2 * rotate);
                self.register(rd).unwrap().bits = bits;
                if s {
                    // TODO: update CPSR's condition flags
                }
            },
            CondInstr::B(rel_offset) => {
                let mut pc = self.program_counter();
                pc.bits = ((pc.bits as i32) + rel_offset) as u32;

                // hack to invert effect of PC increment behavior
                pc.bits -= 4;
            },
            CondInstr::MRS { rd, psr } => {
                self.copy_register(rd, psr);
            },
            CondInstr::TEQ { rn, rotate, immed } => {
                let shift = Self::ror(immed, 2 * rotate);
                let val = self.register(rn).unwrap().bits ^ shift;

                let cpsr = self.register(RegisterBank::CPSR).unwrap();
                // TODO: update C according to shifter carry
                // cpsr.set_condition_flag(ConditionFlag::Carry, false);

                cpsr.set_condition_flag(ConditionFlag::Zero, val == 0);
                cpsr.set_condition_flag(ConditionFlag::Negative, (val as i32) < 0); // TODO: test
            },
            _ => panic!("Unhandled instruction {:?}", instr),
        }
    }

    fn copy_register(&mut self, dest: RegisterBank, src: RegisterBank) {
        let bits = {
            let s = self.register(src).unwrap();
            s.bits
        };
        let mut d = self.register(dest).unwrap();
        d.bits = bits;
    }
}
