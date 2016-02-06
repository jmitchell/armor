#![allow(dead_code)]

// TODO: Make a trait for anything that can be mapped to a portion of
// the address space. RAM, ROM, memory-mapped I/O, vector table
// interrupt handlers, etc would implement it.
//
// Given a collection of them, identify whether there are any
// collisions. Assuming no collisions, map each of them to appropriate
// place in the address space. Eventually it may help to have a second
// trait for unmapping a memory-mappable item, e.g. memory mapped
// files.

use std::collections::HashMap;

pub struct AddressSpace {
    segments: HashMap<u64,u8>,
    default_data: u8,
}

impl AddressSpace
{
    pub fn new() -> AddressSpace {
        AddressSpace {
            segments: HashMap::new(),
            default_data: Default::default(),
        }
    }

    #[allow(unused_variables)]
    fn valid_address(&self, addr: &u64) -> bool {
        // TODO: adhere to alignment constraints, and write test to
        // confirm.

        // TODO: accomodate memory map presented in
        // http://infocenter.arm.com/help/topic/com.arm.doc.den0001c/DEN0001C_principles_of_arm_memory_maps.pdf
        true
    }

    pub fn get(&mut self, addr: &u64) -> Option<&u8> {
        if !self.valid_address(addr) {
            return None;
        }

        if let Some(data) = self.segments.get(addr) {
            Some(data)
        } else {
            // TODO: Indicate uninitialized reads.
            Some(&self.default_data)
        }
    }

    pub fn get_mut(&mut self, addr: &u64) -> Option<&mut u8> {
        if !self.valid_address(addr) {
            return None;
        }

        if let None = self.segments.get(addr) {
            self.segments.insert((*addr).clone(), self.default_data.clone());
        }
        self.segments.get_mut(addr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_and_write_at_one_address() {
        let mut mem = AddressSpace::new();
        let addr = 1024;
        let data = 99;

        if let Some(d) = mem.get(&addr) {
            assert_eq!(*d, Default::default());
        } else {
            // TODO: check whether a panic! in a test can prematurely
            // stop a test suite.
            panic!("Failed to fetch memory reference.");
        }
        if let Some(d) = mem.get_mut(&addr) {
            *d = data;
        } else {
            panic!("Failed to fetch mutable memory reference.");
        }
        if let Some(d) = mem.get(&addr) {
            assert_eq!(*d, data);
        } else {
            panic!("Failed to fetch memory reference.");
        }
    }
}
