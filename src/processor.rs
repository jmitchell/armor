#![allow(dead_code)]

use registers;

pub struct Processor {
    register_file: registers::RegisterFile,
    // TODO: add other parts as needed
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            register_file: Default::default(),
        }
    }
}

impl Default for Processor {
    fn default() -> Processor {
        Processor::new()
    }
}
