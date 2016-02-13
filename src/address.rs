#![allow(dead_code)]

use std::{
    cmp,
    usize,
};
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

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

impl Index<Address> for Addressable {
    type Output = Cell;

    fn index<'a>(&'a self, index: Address) -> &'a Cell {
        self.get(index).unwrap()
    }
}

impl IndexMut<Address> for Addressable {
    fn index_mut<'a>(&'a mut self, index: Address) -> &'a mut Cell {
        self.get_mut(index).unwrap()
    }
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

    fn write_cells(&mut self, data: Vec<Cell>, addr: Address) {
        assert!(addr >= self.start());

        // TODO: clean up using CellCount
        let last_addr = addr + data.len() as u64 - 1;
        assert!(last_addr <= self.end());

        for i in addr..(last_addr + 1) {
            match self.get_mut(i) {
                Some(cell) => {
                    assert!(i <= usize::MAX as u64);
                    *cell = data[i as usize];
                },
                None => panic!(),
            }
        }
    }

    fn read_cells(&self, low: Address, high: Address) -> Option<Vec<Cell>> {
        assert!(low <= high);
        let size = (high - low + 1) as usize;
        let mut ret = Vec::with_capacity(size);
        for i in low..(high + 1) {
            match self.get(i) {
                Some(&cell) => {
                    assert!(i <= usize::MAX as u64);
                    ret.push(cell);
                },
                None => return None
            }
        }
        Some(ret)
    }
}

/// A trait for a region of addressable space that can lease control
/// over its subregions.
trait LeasableRegion : Region {
    /// Check if a candidate's region is available for lease, meaning
    /// it's fully contained by this region and doesn't overlap with
    /// any of the currently leased subregions.
    fn available_for_lease(&self, candidate: &Region) -> bool;

    /// Try to lease a subregion to a candidate region. Only succeeds
    /// if `available_for_lease` is true.
    fn lease(&mut self, candidate: Box<Region>) ->
        Option<&mut Box<Region>>;

    fn leased_subregions(&self) -> &[Box<Region>];

    fn leased_subregions_mut(&mut self) -> &mut [Box<Region>];

    fn leased_subregion_at(&self, addr: Address) ->
        Option<&Box<Region>>;

    fn leased_subregion_at_mut(&mut self, addr: Address) ->
        Option<&mut Box<Region>>;
}

pub struct AddressSpace {
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

    fn from_range(a: Address, b: Address) -> AddressSpace {
        AddressSpace {
            start: cmp::min(a, b),
            end: cmp::max(a, b),
            mapped_regions: vec![],
        }
    }
}

impl LeasableRegion for AddressSpace {
    fn available_for_lease(&self, candidate: &Region) -> bool {
        if !self.contains_region(candidate) {
            return false
        }
        for subregion in self.mapped_regions.iter() {
            if subregion.overlaps_region(candidate) {
                return false
            }
        }
        return true
    }

    fn lease(&mut self, candidate: Box<Region>) ->
        Option<&mut Box<Region>>
    {
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

    fn leased_subregion_at(&self, addr: Address) ->
        Option<&Box<Region>>
    {
        self.leased_subregions()
            .iter()
            .find(|ref r| r.contains_address(&addr))
    }

    fn leased_subregion_at_mut(&mut self, addr: Address) ->
        Option<&mut Box<Region>>
    {
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
//
// TODO: Use region and leasing features to build the standard ARM
// 32-bit memory map.

// TODO: What is HiVECs? It's referenced in "Principles of ARM Memory
// Maps", sec 3.1.6, P15.

// 32-bit memory map, according to section 4.1.
//
// 4GB  +-----------------+ <- 32-bit
//      | DRAM            |
//      |                 |
// 2GB  +-----------------+
//      | Mapped I/O      |
// 1GB  +-----------------+
//      | ROM & RAM & I/O |
// 0GB  +-----------------+ 0

struct MemMap32 {
    address_space: AddressSpace,
}

impl MemMap32 {
    fn new() -> MemMap32 {
        let rom_ram_io = AddressSpace::from_range(0x00000000, 0x3fffffff);
        let mapped_io = AddressSpace::from_range(0x40000000, 0x7fffffff);
        let dram = RandomAccessMemory::new(0x80000000, 0xffffffff);

        let mut map = AddressSpace::from_range(0x00000000, 0xffffffff);
        map.lease(Box::new(rom_ram_io));
        map.lease(Box::new(mapped_io));
        map.lease(Box::new(dram));

        MemMap32 {
            address_space: map,
        }
    }
}


#[cfg(test)]
mod test {
    use address::{
        Address,
        Cell,
        Addressable,
        Region,
        LeasableRegion,
        AddressSpace,
        RandomAccessMemory,
        MemMap32,
    };

    #[test]
    fn lease_first_4k_of_address_space() {
        let mut address_space = AddressSpace::new();
        assert_eq!(0, address_space.leased_subregions().len());

        let ram4k = RandomAccessMemory::new(0, 0xfff);
        assert!(address_space.available_for_lease(&ram4k));
        assert!(address_space.lease(Box::new(ram4k)).is_some());
        assert_eq!(1, address_space.leased_subregions().len());
        assert!(!address_space
                .available_for_lease(&RandomAccessMemory::new(0, 0)));
        assert!(address_space.leased_subregion_at(0).is_some());
        assert!(address_space
                .leased_subregion_at(0xffffffffffffffff).is_none());

        let mut orig_val: Cell = 0;
        let mut new_val: Cell = 0;
        let addr: Address = 50;
        {
            let mut inner_cell = address_space.get_mut(addr).unwrap();
            orig_val = *inner_cell;
            *inner_cell += 1;
            assert!(*inner_cell != orig_val);
        }
        new_val = *address_space.get(addr).unwrap();
        assert!(new_val != orig_val);
    }

    #[test]
    fn fail_to_sublease_region_partway_out_of_bounds() {
        let mut address_space = AddressSpace::new();
        assert_eq!(address_space.end, 0xffffffff);
        let ram_oob = RandomAccessMemory::new(address_space.end,
                                              address_space.end + 1);
        assert_eq!(0, address_space.leased_subregions().len());
        assert!(!address_space.available_for_lease(&ram_oob));
        assert!(address_space.lease(Box::new(ram_oob)).is_none());
        assert_eq!(0, address_space.leased_subregions().len());
        assert!(address_space.available_for_lease(
            &RandomAccessMemory::new(address_space.start,
                                     address_space.end)));
        assert!(address_space.leased_subregion_at(address_space.start).is_none());
        assert!(address_space.leased_subregion_at(address_space.end).is_none());
    }

    #[test]
    fn fail_to_sublease_to_two_overlapping_subregions() {
        let mut address_space = AddressSpace::new();
        let upper_bound = 1024;
        assert!(upper_bound <= address_space.end());
        let ram_x = RandomAccessMemory::new(0, 63);
        let ram_y = RandomAccessMemory::new(63, upper_bound);
        assert!(ram_x.overlaps_region(&ram_y));

        assert!(address_space.lease(Box::new(ram_x)).is_some());
        assert!(!address_space.available_for_lease(&ram_y));
        assert!(address_space.lease(Box::new(ram_y)).is_none());
        assert_eq!(1, address_space.leased_subregions().len());
        assert!(address_space.leased_subregion_at(63).is_some());
        assert!(address_space.leased_subregion_at(64).is_none());
    }

    #[test]
    fn successfully_sublease_to_two_nonoverlapping_regions() {
        let mut address_space = AddressSpace::new();
        let upper_bound = 1024;
        assert!(upper_bound <= address_space.end());
        let ram_x = RandomAccessMemory::new(0, 63);
        let ram_y = RandomAccessMemory::new(64, upper_bound);
        assert!(!ram_x.overlaps_region(&ram_y));

        assert!(address_space.lease(Box::new(ram_x)).is_some());
        assert!(address_space.available_for_lease(&ram_y));
        assert!(address_space.lease(Box::new(ram_y)).is_some());
        assert_eq!(2, address_space.leased_subregions().len());
        assert!(address_space.leased_subregion_at(63).is_some());
        assert!(address_space.leased_subregion_at(64).is_some());
    }

    #[test]
    fn build_deep_tree_of_ram_chips_and_write_to_all_cells() {
        let mut address_space = AddressSpace::new();

        let mut ram512 = AddressSpace::from_range(0, 511);

        let mut ram256_low = AddressSpace::from_range(0, 255);
        let mut ram256_high = AddressSpace::from_range(256, 511);

        assert!(ram256_low.lease(
            Box::new(RandomAccessMemory::new(0, 63))).is_some());
        assert!(ram256_low.lease(
            Box::new(RandomAccessMemory::new(64, 127))).is_some());
        assert!(ram256_low.lease(
            Box::new(RandomAccessMemory::new(128, 191))).is_some());
        assert!(ram256_low.lease(
            Box::new(RandomAccessMemory::new(192, 255))).is_some());

        assert!(ram256_high.lease(
            Box::new(RandomAccessMemory::new(256, 319))).is_some());
        assert!(ram256_high.lease(
            Box::new(RandomAccessMemory::new(320, 383))).is_some());
        assert!(ram256_high.lease(
            Box::new(RandomAccessMemory::new(384, 447))).is_some());
        assert!(ram256_high.lease(
            Box::new(RandomAccessMemory::new(448, 511))).is_some());

        assert!(ram512.lease(Box::new(ram256_low)).is_some());
        assert!(ram512.lease(Box::new(ram256_high)).is_some());

        assert!(address_space.lease(Box::new(ram512)).is_some());

        fn val_for_address(addr: Address) -> u8 {
            (addr % 256) as u8
        }
        fn data() -> Vec<Cell> {
            (0..512).map(val_for_address).collect()
        }
        address_space.write_cells(data(), 0);

        let recorded_data = address_space.read_cells(0, 511);
        assert!(recorded_data.is_some());
        assert_eq!(recorded_data.unwrap(), data());
    }

    #[test]
    fn create_32_bit_arm_memory_map() {
        let mm = MemMap32::new();
        assert_eq!(mm.address_space.start(), 0x00000000);
        assert_eq!(mm.address_space.end(), 0xffffffff);
    }
}
