use super::*;

/// Defines a mapping from virtual to physical address space.
#[repr(align(4096))]
pub struct PageTable {
    pub entries: [Entry; 512],
}

/// Page table entry.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    entry: u64,
}

/// Properties of a page table entry.
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum Bit {
    Present = 0,
    Writable = 1,
    User = 2,
    Direct = 3,
    Nocache = 4,
    Accessed = 5,
    Dirty = 6,
    Huge = 7,
    Global = 8,
    Noexec = 63,
}

add_indexing!(PageTable, Entry);

impl PageTable {
    /// Create a new PageTable.
    pub const fn new() -> Self {
        PageTable {
            entries: [Entry::new(); 512],
        }
    }

    /// Clear the page table.
    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.clear();
        }
    }
}

impl Entry {
    /// Create a new page table entry.
    pub const fn new() -> Self {
        Entry { entry: 0 }
    }

    /// Volatile read of the entry.
    fn read(&self) -> u64 {
        let entry_ptr = &self.entry as *const u64;
        unsafe { core::ptr::read_volatile(entry_ptr) }
    }

    /// Volatile write of the entry.
    fn write(&mut self, entry: u64) {
        let entry_ptr = &mut self.entry as *mut u64;
        unsafe {
            core::ptr::write_volatile(entry_ptr, entry);
        }
    }

    /// Update the entry value using volatile read/write.
    fn update<F>(&mut self, f: F)
    where
        F: Fn(u64) -> u64,
    {
        self.write(f(self.read()));
    }

    /// Set the entry to 0.
    pub fn clear(&mut self) {
        self.write(0);
    }

    /// Physical memory address referenced by this entry.
    pub fn address(&self) -> PhysicalAddress {
        (self.read() & 0x000ffffffffff000).into()
    }

    /// Set the physical memory address of this entry.
    pub fn set_address(&mut self, address: PhysicalAddress) -> &mut Self {
        assert!(address % 0x1000 == 0);
        assert!(address.as_u64() < 0xfff0_0000_0000_0000);
        self.update(|entry| (entry & !0x000ffffffffff000) | address.as_u64());
        self
    }

    /// Bits available to operating system.
    pub fn avail(&self) -> u8 {
        ((self.read() & 0x0e00) >> 9) as u8
    }

    /// Set available bits.
    pub fn set_avail(&mut self, val: u8) -> &mut Self {
        if val > 7 {
            panic!("Avail value out ouf bounds");
        }
        self.update(|entry| (entry & !0x0e00) | ((val as u64) << 9));
        self
    }

    /// Whether a certain bit is set.
    pub fn bit(&self, bit: Bit) -> bool {
        get_bit!(self.read(), bit as u64)
    }

    /// Set or unset a bit.
    fn modify_bit(&mut self, bit: Bit, val: bool) {
        self.update(|mut entry| {
            set_bit!(entry, bit as u64, val);
            entry
        });
    }

    /// Set a bit.
    pub fn set_bit(&mut self, bit: Bit) -> &mut Self {
        self.modify_bit(bit, true);
        self
    }

    /// Unset a bit.
    pub fn unset_bit(&mut self, bit: Bit) -> &mut Self {
        self.modify_bit(bit, false);
        self
    }
}

#[test]
fn int_consistency() {
    let mut entry = Entry::new();
    entry.set_address(0x4242000.into());
    assert_eq!(entry.address().as_u64(), 0x4242000);

    entry.set_avail(7);
    assert_eq!(entry.avail(), 7);
    entry.set_avail(0);
    assert_eq!(entry.avail(), 0);
}

#[test]
fn bit_consistency() {
    let mut entry = Entry::new();
    entry.set_address(0x000ffffffffff000.into());
    assert!(!entry.bit(Bit::Present));
    assert!(!entry.bit(Bit::Writable));
    assert!(!entry.bit(Bit::User));
    assert!(!entry.bit(Bit::Direct));
    assert!(!entry.bit(Bit::Nocache));
    assert!(!entry.bit(Bit::Accessed));
    assert!(!entry.bit(Bit::Dirty));
    assert!(!entry.bit(Bit::Huge));
    assert!(!entry.bit(Bit::Global));
    assert!(!entry.bit(Bit::Noexec));

    entry.set_bit(Bit::Present);
    assert!(entry.bit(Bit::Present));
    entry.unset_bit(Bit::Present);

    entry.set_bit(Bit::Writable);
    assert!(entry.bit(Bit::Writable));
    entry.unset_bit(Bit::Writable);

    entry.set_bit(Bit::User);
    assert!(entry.bit(Bit::User));
    entry.unset_bit(Bit::User);

    entry.set_bit(Bit::Direct);
    assert!(entry.bit(Bit::Direct));
    entry.unset_bit(Bit::Direct);

    entry.set_bit(Bit::Nocache);
    assert!(entry.bit(Bit::Nocache));
    entry.unset_bit(Bit::Nocache);

    entry.set_bit(Bit::Accessed);
    assert!(entry.bit(Bit::Accessed));
    entry.unset_bit(Bit::Accessed);

    entry.set_bit(Bit::Dirty);
    assert!(entry.bit(Bit::Dirty));
    entry.unset_bit(Bit::Dirty);

    entry.set_bit(Bit::Huge);
    assert!(entry.bit(Bit::Huge));
    entry.unset_bit(Bit::Huge);

    entry.set_bit(Bit::Global);
    assert!(entry.bit(Bit::Global));
    entry.unset_bit(Bit::Global);

    entry.set_bit(Bit::Noexec);
    assert!(entry.bit(Bit::Noexec));
    entry.unset_bit(Bit::Noexec);
}
