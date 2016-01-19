#![allow(dead_code)]

use std::collections::HashMap;

pub trait Register {
    fn _bit_width(&self) -> u8;

    fn valid_index(&self, i: u8) -> bool {
        i < self._bit_width()
    }

    fn _read_bit(&self, u8) -> bool;

    fn permit_read(&self, _i: u8) -> bool {
        true
    }

    fn read_bit(&self, index: u8) -> bool {
        assert!(self.valid_index(index));
        assert!(self.permit_read(index));
        self._read_bit(index)
    }

    fn _write_bit(&mut self, u8, bool);

    fn permit_write(&self, _i: u8) -> bool {
        true
    }

    fn write_bit(&mut self, index: u8, v: bool) {
        assert!(self.valid_index(index));
        assert!(self.permit_write(index));
        self._write_bit(index, v)
    }
}

struct Register32 {
    bits: u32,
}

impl Register for Register32 {
    fn _bit_width(&self) -> u8 {
        32
    }

    fn _read_bit(&self, i: u8) -> bool {
        self.bits & (1 << i) != 0
    }

    fn _write_bit(&mut self, i: u8, v: bool) {
        let n = 1 << i;
        if v {
            self.bits |= n;
        } else {
            self.bits &= !n;
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
enum ProcessorMode {
    Abort,                      // failed attempt to access memory
    FastInterruptRequest,       // fast interrupt system
    InterruptRequest,           // interrupt system
    Supervisor,                 // after reset
    System,                     // User, except with RW access to CPSR
    Undefined,                  // undefined or unsupported instruction
    User,                       // used for programs and applications
}

enum ConditionFlag {
    Negative,                   // N
    Zero,                       // Z
    Carry,                      // C
    Overflow,                   // V
    // Saturation,                 // Q
}

enum InterruptMask {
    IRQ,
    FIQ,
}

enum InstructionSet {
    ARM,
    Thumb,
    // Jazelle,                    // Java-optimized IS
}

trait ProgramStatusRegister : Register {
    fn mode(&self) -> Option<ProcessorMode>;

    fn set_mode(&mut self, ProcessorMode);

    fn is_privileged_mode(&self) -> bool {
        self.mode().map_or(false, |v| v != ProcessorMode::User)
    }

    fn is_control_field(&self, u8) -> bool;

    fn is_condition_flag(&self, u8) -> bool;

    fn permit_read(&self, i: u8) -> bool {
        self.is_privileged_mode() ||
            self.is_control_field(i) ||
            self.is_condition_flag(i)
    }

    fn permit_write(&self, i: u8) -> bool {
        self.is_privileged_mode() ||
            self.is_condition_flag(i)
    }

    fn condition_flag_index(&self, ConditionFlag) -> u8;

    fn set_condition_flag(&mut self, cf: ConditionFlag, setting: bool) {
        let index = self.condition_flag_index(cf);
        self.write_bit(index, setting);
    }

    fn is_condition_flag_on(&self, cf: ConditionFlag) -> bool {
        self.read_bit(self.condition_flag_index(cf))
    }

    fn interrupt_mask_index(&self, InterruptMask) -> u8;

    fn set_interrupt_mask(&mut self, im: InterruptMask, disable: bool) {
        let index = self.interrupt_mask_index(im);
        self.write_bit(index, disable);
    }

    fn permit_interrupt(&self, im: InterruptMask) -> bool {
        !self.read_bit(self.interrupt_mask_index(im))
    }

    fn thumb_state_index(&self) -> u8;

    fn set_instruction_set(&mut self, instr_set: InstructionSet) {
        let index = self.thumb_state_index();
        match instr_set {
            InstructionSet::Thumb => self.write_bit(index, true),
            InstructionSet::ARM => self.write_bit(index, false),
        }
    }

    fn active_instruction_set(&self) -> InstructionSet {
        if self.read_bit(self.thumb_state_index()) {
            InstructionSet::Thumb
        } else {
            InstructionSet::ARM
        }
    }
}

impl ProgramStatusRegister for Register32 {
    fn mode(&self) -> Option<ProcessorMode> {
        let code: u8 = {
            let mut n = 0;
            for i in 0..5 {
                if self.read_bit(i) {
                    n |= 1 << i;
                }
            }
            n
        };

        match code {
            0x10 => Some(ProcessorMode::User),
            0x11 => Some(ProcessorMode::FastInterruptRequest),
            0x12 => Some(ProcessorMode::InterruptRequest),
            0x13 => Some(ProcessorMode::Supervisor),
            0x17 => Some(ProcessorMode::Abort),
            0x1b => Some(ProcessorMode::Undefined),
            0x1f => Some(ProcessorMode::System),
            _ => None
        }
    }

    fn set_mode(&mut self, mode: ProcessorMode) {
        // TODO: Eliminate redundancy between this and the mode()
        // method by storing the mapping in one place, and
        // inverting/searching as needed. Benefit: less error-prone
        // and more maintainable.

        let code = match mode {
            ProcessorMode::User => 0x10,
            ProcessorMode::FastInterruptRequest => 0x11,
            ProcessorMode::InterruptRequest => 0x12,
            ProcessorMode::Supervisor => 0x13,
            ProcessorMode::Abort => 0x17,
            ProcessorMode::Undefined => 0x1b,
            ProcessorMode::System => 0x1f,
        };

        for i in 0..5 {
            self.write_bit(i, (code >> i) % 2 != 0);
        }
    }

    fn is_control_field(&self, i: u8) -> bool {
        i < 8
    }

    fn is_condition_flag(&self, i: u8) -> bool {
        i > 27 && i < 32
    }

    fn condition_flag_index(&self, cf: ConditionFlag) -> u8 {
        match cf {
            ConditionFlag::Negative => 31,
            ConditionFlag::Zero => 30,
            ConditionFlag::Carry => 29,
            ConditionFlag::Overflow => 28,
        }
    }

    fn interrupt_mask_index(&self, im: InterruptMask) -> u8 {
        match im {
            InterruptMask::IRQ => 7,
            InterruptMask::FIQ => 6,
        }
    }

    fn thumb_state_index(&self) -> u8 {
        5
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum RegisterBank {
    R0, R1, R2, R3,
    R4, R5, R6, R7,
    R8, R9, R10, R11,
    R12, R13, R14, R15,
    CPSR, SPSR,
}

#[derive(PartialEq, Eq, Hash)]
struct RegisterID(RegisterBank, ProcessorMode);

struct RegisterFile {
    table: HashMap<RegisterID, Register32>,
}

impl RegisterFile {
    pub fn new() -> RegisterFile {
        let mut rf = RegisterFile { table: HashMap::new() };
        rf.reset();
        rf
    }

    pub fn reset(&mut self) {
        // TODO(low): make a clean interface for generating this
        let standard_registers = vec![
            RegisterID(RegisterBank::R0, ProcessorMode::User),
            RegisterID(RegisterBank::R1, ProcessorMode::User),
            RegisterID(RegisterBank::R2, ProcessorMode::User),
            RegisterID(RegisterBank::R3, ProcessorMode::User),
            RegisterID(RegisterBank::R4, ProcessorMode::User),
            RegisterID(RegisterBank::R5, ProcessorMode::User),
            RegisterID(RegisterBank::R6, ProcessorMode::User),
            RegisterID(RegisterBank::R7, ProcessorMode::User),
            RegisterID(RegisterBank::R8, ProcessorMode::User),
            RegisterID(RegisterBank::R8, ProcessorMode::FastInterruptRequest),
            RegisterID(RegisterBank::R9, ProcessorMode::User),
            RegisterID(RegisterBank::R9, ProcessorMode::FastInterruptRequest),
            RegisterID(RegisterBank::R10, ProcessorMode::User),
            RegisterID(RegisterBank::R10, ProcessorMode::FastInterruptRequest),
            RegisterID(RegisterBank::R11, ProcessorMode::User),
            RegisterID(RegisterBank::R11, ProcessorMode::FastInterruptRequest),
            RegisterID(RegisterBank::R12, ProcessorMode::User),
            RegisterID(RegisterBank::R12, ProcessorMode::FastInterruptRequest),
            RegisterID(RegisterBank::R13, ProcessorMode::User),
            RegisterID(RegisterBank::R13, ProcessorMode::FastInterruptRequest),
            RegisterID(RegisterBank::R13, ProcessorMode::InterruptRequest),
            RegisterID(RegisterBank::R13, ProcessorMode::Supervisor),
            RegisterID(RegisterBank::R13, ProcessorMode::Undefined),
            RegisterID(RegisterBank::R13, ProcessorMode::Abort),
            RegisterID(RegisterBank::R14, ProcessorMode::User),
            RegisterID(RegisterBank::R14, ProcessorMode::FastInterruptRequest),
            RegisterID(RegisterBank::R14, ProcessorMode::InterruptRequest),
            RegisterID(RegisterBank::R14, ProcessorMode::Supervisor),
            RegisterID(RegisterBank::R14, ProcessorMode::Undefined),
            RegisterID(RegisterBank::R14, ProcessorMode::Abort),
            RegisterID(RegisterBank::R15, ProcessorMode::User),
            RegisterID(RegisterBank::CPSR, ProcessorMode::User),
            RegisterID(RegisterBank::SPSR, ProcessorMode::FastInterruptRequest),
            RegisterID(RegisterBank::SPSR, ProcessorMode::InterruptRequest),
            RegisterID(RegisterBank::SPSR, ProcessorMode::Supervisor),
            RegisterID(RegisterBank::SPSR, ProcessorMode::Undefined),
            RegisterID(RegisterBank::SPSR, ProcessorMode::Abort),
        ];

        for id in standard_registers {
            self.table.insert(id, Register32 { bits: 0u32 });
        }

        self.cpsr_mut().set_mode(ProcessorMode::Supervisor);
        self.cpsr_mut().set_instruction_set(InstructionSet::ARM);

        // TODO: any other initialization logic, like initial program
        // counter (PC)?
    }

    fn cpsr(&self) -> &Register32 {
        self.table
            .get(&RegisterID(RegisterBank::CPSR, ProcessorMode::User))
            .unwrap()
    }

    fn cpsr_mut(&mut self) -> &mut Register32 {
        self.table
            .get_mut(&RegisterID(RegisterBank::CPSR, ProcessorMode::User))
            .unwrap()
    }

    pub fn mode(&self) -> ProcessorMode {
        self.cpsr().mode().unwrap()
    }

    fn register_exists(&self, bank: RegisterBank) -> bool {
        self.table.get(&RegisterID(bank, self.mode())).is_some()
    }

    // TODO: DRY up lookup_mut and lookup
    pub fn lookup_mut(&mut self, bank: RegisterBank) ->
        Option<&mut Register32>
    {
        if self.register_exists(bank) {
            let mode = self.mode();
            self.table.get_mut(&RegisterID(bank, mode))
        } else {
            // only None for SPSR
            self.table.get_mut(&RegisterID(bank, ProcessorMode::User))
        }
    }

    pub fn lookup(&self, bank: RegisterBank) -> Option<&Register32> {
        if self.register_exists(bank) {
            self.table.get(&RegisterID(bank, self.mode()))
        } else {
            // only None for SPSR
            self.table.get(&RegisterID(bank, ProcessorMode::User))
        }
    }

    // TODO: ASDG says modes can change due to: "reset, interrupt
    // request, fast interrupt request, software interrupt, data
    // abort, prefetch abort, and undefined instruction"
    fn on_mode_change(&mut self, mode: ProcessorMode) {
        // let orig_mode = self.mode();
        self.cpsr_mut().set_mode(mode);

        // TODO: if orig_mode == User (and maybe only when new mode is
        // an IRQ or FIQ?), copy CPSR to SPSR for new mode.

        // TODO: adjust interrupt masks?
    }

    // TODO: Support special return instruction to go back to User
    // mode. Involves at least copying the current SPSR to CPSR.

    fn on_condition(&mut self, cf: ConditionFlag) {
        self.cpsr_mut().set_condition_flag(cf, true);
    }

    // TODO: Support conditional execution based on flags.

    // TODO: What mechanism clears condition flags? When?
}

// TODO: Pipelines (ASDG 2.3). Start with 3-stage.

// TODO(low): exceptions, interrupts, and vector table (ASDG 2.4)

// TODO(low): core extensions (ASDG 2.5)

// TODO(low): architecture revisions and ARM processor family (ASDG 2.6-7)


#[cfg(test)]
mod test {
    // TODO: verify that a new RegisterFile starts in supervisor mode
    // and using the ARM IS

    // TODO: verify that after writing to a CPSR's mode, reads match
    // the write, and that no write leads to a None read.

    #[test]
    fn it_works() {
        assert_eq!(2,2);
    }
}
