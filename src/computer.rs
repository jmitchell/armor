#![allow(dead_code)]

use address;
use address::Region;
use processor;
use processor::{
    Condition,
    CondInstr,
    Instruction,
    UncondInstr,
    MOVInstr,
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
                None => Err(format!("[ ??? '0b{:0>32b}' ]", word)),
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
            CondInstr::BL(rel_offset) => {
                let ret = self.program_counter().bits + 4;
                {
                    let mut lr = self.register(RegisterBank::R14).unwrap();
                    lr.bits = ret;
                }

                let mut pc = self.program_counter();
                pc.bits = ((pc.bits as i32) + rel_offset) as u32;

                // hack to invert effect of PC increment behavior
                pc.bits -= 4;
            },
            CondInstr::BIC { s, rd, rn, rotate, immed } => {
                let val = self.register(rn).unwrap().bits & !Self::ror(immed, 2 * rotate);
                let cpsr = self.register(RegisterBank::CPSR).unwrap();
                if s {
                    // TODO: update C according to shifter carry
                    // cpsr.set_condition_flag(ConditionFlag::Carry, false);

                    cpsr.set_condition_flag(ConditionFlag::Zero, val == 0);
                    cpsr.set_condition_flag(ConditionFlag::Negative, (val as i32) < 0);
                }
            },
            CondInstr::BX(rm) => {
                let val = self.register(rm).unwrap().bits & 0xfffffffe;
                let mut pc = self.program_counter();
                pc.bits = val;

                // hack to invert effect of PC increment behavior
                pc.bits -= 4;

                // TODO: Set T-flag to 'Rm & 1' (may enable Thumb
                // mode)
            },
            CondInstr::LDR { rd, ref addr_ref } => {
                let addr = {
                    let mut rn = self.register(*addr_ref.get_base()).unwrap();
                    let addr = addr_ref.get_addr(rn);
                    addr_ref.handle_writeback(&mut rn);
                    addr
                };

                if let Some(word) = self.mem.get32(addr, self.big_endian) {
                    let mut rd = self.register(rd).unwrap();
                    rd.bits = word;
                } else {
                    panic!("Failed to read memory at address {:#x}", addr);
                }
            },
            CondInstr::MCR { op1, cn, rd, copro, op2, cm } => {
                // TODO: implement coprocessors and interpret this instruction
                println!("Skipping coprocessor logic for now!");
            },
            CondInstr::MOV(ref mov) => {
                match mov {
                    &MOVInstr::Shift { s, rd, shift_size, shift, rm } => {
                        // TODO
                        println!("Skipping MOV logic for now!");
                    },
                    &MOVInstr::Rotate { s, rd, rotate, immed } => {
                        // TODO
                        println!("Skipping MOV logic for now!");
                    },
                }
            },
            CondInstr::MRC { op1, cn, rd, copro, op2, cm } => {
                // TODO: implement coprocessors and interpret this instruction
                println!("Skipping coprocessor logic for now!");
            },
            CondInstr::MRS { rd, psr } => {
                self.copy_register(rd, psr);
            },
            CondInstr::MSR { psr, rm, f, s, x, c } => {
                // TODO: observe note that bits[23:0] of the cpsr are
                // unaffected in User mode.
                let mask: u32 = {
                    let c_mask = if c { 0x000000ff } else { 0 };
                    let x_mask = if x { 0x0000ff00 } else { 0 };
                    let s_mask = if s { 0x00ff0000 } else { 0 };
                    let f_mask = if f { 0xff000000 } else { 0 };
                    c_mask | x_mask | s_mask | f_mask
                };
                let mut psr_bits = self.register(psr).unwrap().bits;
                let masked_psr = psr_bits & !mask;
                let masked_reg = self.register(rm).unwrap().bits & mask;
                psr_bits = masked_psr | masked_reg;
            },
            CondInstr::ORR { s, rd, rn, rotate, immed } => {
                // TODO: address Notes section of ORR in A.3.

                let val = self.register(rn).unwrap().bits | Self::ror(immed, 2 * rotate);
                let cpsr = self.register(RegisterBank::CPSR).unwrap();
                if s {
                    // TODO: update C according to shifter carry
                    // cpsr.set_condition_flag(ConditionFlag::Carry, false);

                    cpsr.set_condition_flag(ConditionFlag::Zero, val == 0);
                    cpsr.set_condition_flag(ConditionFlag::Negative, (val as i32) < 0);
                }
            },
            CondInstr::STMDB { carrot, w, rn, ref reg_list } => {
                // TODO
                println!("Skipping STMDB logic for now!");
            },
            CondInstr::SUB { s, rd, rn, rotate, immed } => {
                // TODO
                println!("Skipping SUB logic for now!");
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



#[cfg(test)]
mod test {
    use super::Computer;
    use processor::{
        Condition,
        CondInstr,
        Instruction,
    };


    #[test]
    fn branch() {
        let rel_offset = 48;
        let branch = Instruction::Cond(
            CondInstr::B(rel_offset),
            Condition::AL);

        // TODO: Create a Computer with a random PC addr, and the
        // encoded form of `branch` at that PC addr. Execute one
        // instruction and assert post-conditions hold.
    }

    #[test]
    fn conditional_instructions() {
        // TODO: Test combinations of `Condition` and CPSR values.
        // Test both when condition is satisfied and not.
    }
}
