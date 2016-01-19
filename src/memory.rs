#![allow(dead_code)]

//! Processors are simply machines that consume bits from a memory
//! space, compute based on those bits, and emit results to the memory
//! space. A memory space can be thought of as a rectangular grid of
//! bits where each row is identified by a unique address. The bits
//! accessible at an address might represent an instruction for the
//! processor, information to be processed by instructions (machine
//! code) loaded from elsewhere, or data from another device in the
//! machine.
//!
//! An emulated processor runs on a real computer which has its own
//! real physical memory. As tempting as it might be, the emulated
//! processor can't use the real physical memory as its own, at least
//! not directly. That would interfere with the real processor,
//! operating system, and potentially any part of the entire machine.
//! One way around that is to emulate the memory too. If there's a
//! convincing enough illusion of a memory space, the emulated
//! processor won't know the difference.
//!
//! There are a couple high-level strategies for emulating a memory
//! space.
//!
//!   * Preallocate a large fixed amount of memory
//!   * Dynamically allocate memory as needed
//!
//! Both have their pros and cons. Preallocation has a higher up front
//! cost, but would generally go faster once it's ready. It's also
//! more accurate from an emulation perspective; the size of a
//! computer's RAM chips don't change while it's running. Dynamic
//! allocation can scale up to handle memory-consuming tasks,
//! potentially beyond the fixed limits of a preallocated space.
//!
//!
//! # Initial Naive Design
//!
//! At this point I'd like avoid deciding on a fixed amount and scale
//! dynamically, so I'm starting with a dynamic allocation design.
//! Until the system is working, performance doesn't matter at all.
//! The initial design is embarassingly naive.
//!
//! Initially no memory is allocated. When there's a write to a valid
//! address, if that address has never been used, real memory is
//! dynamically allocated and associated with that address. Freshly
//! allocated memory is assigned to a default value. (Thus, reads to
//! novel addresses can simply return this default value.) Writes are
//! supported by getting a mutable reference to the data associated
//! with an address. For now there is no mechanism for deallocation,
//! so take care not to write to more addresses than the real machine
//! can handle.

use std::collections::HashMap;
use std::hash::Hash;

pub struct EmuMemory<A,D> {
    segments: HashMap<A,D>,
    default_data: D,
}

impl<A,D> EmuMemory<A,D>
    where A: Eq + Hash + Clone,
          D: Default + Clone
{
    pub fn new() -> EmuMemory<A,D> {
        EmuMemory {
            segments: HashMap::new(),
            default_data: Default::default(),
        }
    }

    fn init_address(&mut self, addr: &A) {
        if let None = self.segments.get(addr) {
            self.segments.insert((*addr).clone(), self.default_data.clone());
        }
    }

    pub fn get(&mut self, addr: &A) -> Option<&D> {
        if let Some(data) = self.segments.get(addr) {
            Some(data)
        } else {
            Some(&self.default_data)
        }
    }

    pub fn get_mut(&mut self, addr: &A) -> Option<&mut D> {
        self.init_address(addr);
        self.segments.get_mut(addr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_and_write_at_one_address() {
        let mut mem: EmuMemory<u32,u32> = EmuMemory::new();
        let addr: u32 = 1024;
        let data: u32 = 99;

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
