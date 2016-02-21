#![allow(dead_code)]

use address;
use processor;

// TODO: Support serializing and deserializing to a human-readable file.

// TODO: GUI that shows state of address space, CPU registers, and
// derivative information like the current mode. Support basic
// debugging features, like breakpoints, run, step, etc.

pub struct Computer {
    cpu: processor::Processor,
    mem: address::MemMap32,
}

impl Computer {
    pub fn new(boot_code: Vec<address::Cell>) -> Computer {
        Computer {
            cpu: Default::default(),
            mem: address::MemMap32::new(boot_code),
        }
    }

    pub fn execute_next_instruction(&mut self) {
        // TODO
    }
}
