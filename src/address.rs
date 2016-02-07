#![allow(dead_code)]

use std::cmp;
use std::collections::HashMap;


/// Unique identifier for a location in an address space.
pub type Address = u64;

/// Bits referenced by a single `Address` (u8 for byte-addressable
/// systems).
pub type Cell = u8;

/// Useful representation for the size of a segment of addressable
/// space.
pub type CellCount = u64;

/// A trait for looking up cells associated with an address.
trait Addressable {
    /// Lookup the cell at a particular address.
    fn get(&self, addr: Address) -> Option<&Cell>;

    /// Lookup a mutable cell reference at a particular address.
    fn get_mut(&mut self, addr: Address) -> Option<&mut Cell>;
}

/// A trait describing a contiguous region of addressable space.
trait Region : Addressable {
    /// Lowest address in the region.
    fn start(&self) -> Address;

    /// Highest address in the region.
    fn end(&self) -> Address;

    /// Number of addressable cells that fit in the region.
    fn size(&self) -> CellCount {
        self.end() - self.start() + 1
    }

    /// Check if an address is between the region's start and end
    /// address.
    fn contains_address(&self, addr: &Address) -> bool {
        *addr >= self.start() &&
            *addr <= self.end()
    }

    /// Check if another region is fully contained by this one.
    fn contains_region(&self, region: &Region) -> bool {
        self.contains_address(&(*region).start()) &&
            self.contains_address(&(*region).end())
    }

    /// Check if another region overlaps this one.
    fn overlaps_region(&self, region: &Region) -> bool {
        self.contains_address(&(*region).start()) ||
            self.contains_address(&(*region).end())
    }
}

// /// A trait for a region of addressable space that can lease control
// /// over its subregions.
// trait LeasableRegion : Region {
//     /// Check if a candidate's region is available for lease, meaning
//     /// it's fully contained by this region and doesn't overlap with
//     /// any of the currently leased subregions.
//     fn available_for_lease(&self, candidate: Box<Region>) -> bool;

//     /// Try to lease a subregion to a candidate region. Only succeeds
//     /// if `available_for_lease` is true.
//     fn lease(&mut self, candidate: Box<Region>) -> Option<&mut Box<Region>>;

//     fn leased_subregions(&self) -> Vec<&Region>;

//     fn leased_subregions_mut(&mut self) -> Vec<&mut Region>;

//     // /// Check if an address is controlled by this region, but not by
//     // /// one of its leased subregions.
//     // ///
//     // /// Typically, a region gains control of an address in its range
//     // /// by mapping it to an addressable interface, e.g. RAM.
//     // fn controls_address(&self, addr: Address) -> bool;

//     // /// Find the leased subregion that controls some address.
//     // fn lease_for_address<R : Region>(&self, addr: Address) -> Option<&R>;
// }

struct AddressSpace {
    start: Address,
    end: Address,
    mapped_regions: Vec<Box<Region>>,
}

impl AddressSpace {
    fn new() -> AddressSpace {
        AddressSpace {
            start: 0x00000000,
            end:   0xffffffff,
            mapped_regions: vec![],
        }
    }


    // TODO: extract into a LeasableRegion trait once the type and
    // borrow checker issues are resolved.

    fn available_for_lease(&self, candidate: &Region) -> bool {
        for subregion in self.mapped_regions.iter() {
            if subregion.overlaps_region(candidate) {
                return false
            }
        }
        return true
    }

    fn lease(&mut self, candidate: Box<Region>) -> Option<&mut Box<Region>> {
        if self.available_for_lease(&*candidate) {
            self.mapped_regions.push(candidate);
            self.mapped_regions.last_mut()
        } else {
            None
        }
    }

    fn leased_subregions(&self) -> &[Box<Region>] {
        &self.mapped_regions[..]
    }

    fn leased_subregions_mut(&mut self) -> &mut [Box<Region>] {
        &mut self.mapped_regions[..]
    }

    fn leased_subregion_at(&self, addr: Address) -> Option<&Box<Region>> {
        self.leased_subregions()
            .iter()
            .find(|ref r| r.contains_address(&addr))
    }

    fn leased_subregion_at_mut(&mut self, addr: Address) -> Option<&mut Box<Region>> {
        self.leased_subregions_mut()
            .iter_mut()
            .find(|ref r| r.contains_address(&addr))
    }
}

impl Addressable for AddressSpace {
    fn get(&self, addr: Address) -> Option<&Cell> {
        match self.leased_subregion_at(addr) {
            Some(region) => region.get(addr),
            None => None,
        }
    }

    fn get_mut(&mut self, addr: Address) -> Option<&mut Cell> {
        match self.leased_subregion_at_mut(addr) {
            Some(region) => region.get_mut(addr),
            None => None,
        }
    }
}

impl Region for AddressSpace {
    fn start(&self) -> Address {
        self.start
    }

    fn end(&self) -> Address {
        self.end
    }
}

// impl LeasableRegion for AddressSpace {
// }


struct RandomAccessMemory {
    start: Address,
    end: Address,
    cells: HashMap<Address, Cell>,
}

impl RandomAccessMemory {
    fn new(a: Address, b: Address) -> RandomAccessMemory {
        RandomAccessMemory {
            start: cmp::min(a, b),
            end: cmp::max(a, b),
            cells: HashMap::new(),
        }
    }
}

impl Addressable for RandomAccessMemory {
    fn get(&self, addr: Address) -> Option<&Cell> {
        self.cells.get(&addr)
    }

    fn get_mut(&mut self, addr: Address) -> Option<&mut Cell> {
        if let None = self.cells.get(&addr) {
            self.cells.insert(addr, Default::default());
        }
        self.cells.get_mut(&addr)
    }
}

impl Region for RandomAccessMemory {
    fn start(&self) -> Address {
        self.start
    }

    fn end(&self) -> Address {
        self.end
    }
}



// A white paper published in 2012 called "Principles of ARM Memory
// Maps" primary source guiding the implementation of this module.
//
// The white paper describes 32-bit, 36-bit, and 42-bit memory maps
// already in use, and proposes memory maps for 44-bit and 48-bit
// addresses. To leave room for future extension, the emulator
// represents addresses using u64's. However, the initial memory map
// implementation will only handle 32-bit addresses.



// TODO: Make a trait for anything that can be mapped to a portion of
// the address space. RAM, ROM, memory-mapped I/O, vector table
// interrupt handlers, etc would implement it.
//
// Given a collection of them, identify whether there are any
// collisions. Assuming no collisions, map each of them to appropriate
// place in the address space. Eventually it may help to have a second
// trait for unmapping a memory-mappable item, e.g. memory mapped
// files.


#[cfg(test)]
mod test {

    use address::Cell;
    use address::AddressSpace;
    use address::Addressable;
    use address::RandomAccessMemory;

    #[test]
    fn lease_portions_of_address_space() {
        let mut address_space = AddressSpace::new();
        assert_eq!(0, address_space.leased_subregions().len());

        let ram4k = RandomAccessMemory::new(0, 0xfff);
        assert!(address_space.available_for_lease(&ram4k));
        assert!(address_space.lease(Box::new(ram4k)).is_some());
        assert_eq!(1, address_space.leased_subregions().len());
        assert!(!address_space.available_for_lease(&RandomAccessMemory::new(0, 0)));
        assert!(address_space.leased_subregion_at(0).is_some());
        assert!(address_space.leased_subregion_at(0xffffffffffffffff).is_none());

        let mut orig_val: Cell = 0;
        let mut new_val: Cell = 0;
        {
            let mut inner_cell = address_space.get_mut(0).unwrap();
            orig_val = *inner_cell;
            *inner_cell += 1;
            assert!(*inner_cell != orig_val);
        }
        new_val = *address_space.get(0).unwrap();
        assert!(new_val != orig_val);

        // TODO: Evict ram4k.

        // TODO: Create a new region partly outside the bounds of
        // address_space. Demonstrate it can't acquire a lease.

        // TODO: Create 3 smaller regions, such that two can both
        // acquire leases from address_space, but the third can't.

        // TODO: Evict the region overlapping the third, and then
        // lease the third.
    }
}
