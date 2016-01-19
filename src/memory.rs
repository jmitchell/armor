#![allow(dead_code)]

// TODO: Change current `EmuMemory` to something like `AddressSpace`.

// TODO: Make a trait for anything that can be mapped to a portion of
// the address space. RAM, ROM, memory-mapped I/O, etc would implement
// it. Given a collection of them, identify whether there are any
// collisions. Assuming no collisions, map each of them to appropriate
// place in the address space. Eventually it may help to have a second
// trait for unmapping a memory-mappable item, e.g. memory mapped
// files.

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
//! address, and if that address has never been used, real memory is
//! dynamically allocated and associated with that address. Freshly
//! allocated memory is assigned to a default value. (Reads to novel
//! addresses don't incur any allocation because they can just return
//! this default value.) Writes are supported by getting a mutable
//! reference to the data associated with an address. There's no
//! mechanism for deallocation, so take care not to write (or acquire
//! mutable references) to more addresses than the real machine can
//! handle.

use std::collections::HashMap;
use std::hash::Hash;
// use std::ops::Add;

// trait Offset : Sized + Eq + Ord {

// }

// trait Address : Sized + Eq + Ord + Hash + Clone + Add<Offset> {

// }

pub struct EmuMemory<A,D> {
    segments: HashMap<A,D>,
    default_data: D,
}

// Each address `A` uniquely identifies a single byte in the memory
// space. Each datum `D` is the data that can be fetched via some
// address.
impl<A,D> EmuMemory<A,D>
    where A: Eq + Hash + Clone,
          D: Clone
{
    pub fn new(default: D) -> EmuMemory<A,D> {
        EmuMemory {
            segments: HashMap::new(),
            default_data: default,
        }
    }

    #[allow(unused_variables)]  // TODO: remove
    fn valid_address(&self, addr: &A) -> bool {
        // TODO: adhere to alignment constraints, and write test to
        // confirm.

        // TODO: accomodate memory map presented in
        // http://infocenter.arm.com/help/topic/com.arm.doc.den0001c/DEN0001C_principles_of_arm_memory_maps.pdf
        true
    }

    pub fn get(&mut self, addr: &A) -> Option<&D> {
        if !self.valid_address(addr) {
            return None;
        }

        if let Some(data) = self.segments.get(addr) {
            Some(data)
        } else {
            // TODO: Consider changing return type so it's possible to
            // tell clients they've read from memory which hasn't ever
            // been written to. Needs to be balanced against
            // constraints of real physical RAM.
            Some(&self.default_data)
        }
    }

    pub fn get_mut(&mut self, addr: &A) -> Option<&mut D> {
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
        let mut mem: EmuMemory<u32,u32> = EmuMemory::new(0u32);
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
