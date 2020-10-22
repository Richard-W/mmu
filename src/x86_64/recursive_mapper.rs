use super::*;

const PAGE_SIZE: u64 = 0x1000;
const PT_SIZE: u64 = 512;

const PT1_EXTENT: u64 = PAGE_SIZE * PT_SIZE;
const PT2_EXTENT: u64 = PT_SIZE * PT1_EXTENT;
const PT3_EXTENT: u64 = PT_SIZE * PT2_EXTENT;

/// Mapper implementation for this architecture
pub struct RecursiveMapper<AllocFrame, TranslateAddress>
where
    AllocFrame: FnMut() -> Result<PhysicalAddress>,
    TranslateAddress: FnMut(PhysicalAddress) -> VirtualAddress,
{
    pt4: *mut PageTable,
    alloc_frame: AllocFrame,
    translate_address: TranslateAddress,
}

impl<AllocFrame, TranslateAddress> RecursiveMapper<AllocFrame, TranslateAddress>
where
    AllocFrame: FnMut() -> Result<PhysicalAddress>,
    TranslateAddress: FnMut(PhysicalAddress) -> VirtualAddress,
{
    /// Create a new `RecursiveMapper` object.
    ///
    /// # Safety
    ///
    /// Safety assumptions:
    /// * `pt4` is a pointer to a valid free memory page
    /// * `alloc_frame` returns pointers to valid free memory pages
    /// * `translate_address` correctly translates physical addresses to virtual addresses
    pub unsafe fn new(
        pt4: *mut PageTable,
        alloc_frame: AllocFrame,
        translate_address: TranslateAddress,
    ) -> Self {
        assert_eq!(pt4 as u64 % PAGE_SIZE, 0);
        RecursiveMapper {
            pt4,
            alloc_frame,
            translate_address,
        }
    }

    unsafe fn ensure_subtable(&mut self, entry: &mut Entry) -> Result<()> {
        if !entry.bit(Bit::Present) {
            let frame = (self.alloc_frame)()?;
            let addr = (self.translate_address)(frame);
            let table = &mut *(addr.as_u64() as *mut PageTable);
            table.clear();
            entry.set_address(frame);

            // Mark table present.
            entry.set_bit(Bit::Present);

            // Set writable and user bit. If we didn't set these bits
            // the user wouldn't be able to.
            entry.set_bit(Bit::Writable);
            entry.set_bit(Bit::User);
            Ok(())
        } else if entry.bit(Bit::Huge) {
            Err(Error::Overlap)
        } else {
            Ok(())
        }
    }

    unsafe fn descend_entry(&mut self, entry: &mut Entry) -> Result<&'static mut PageTable> {
        self.ensure_subtable(entry)?;
        let phys_addr = entry.address();
        let virt_addr = (self.translate_address)(phys_addr);
        Ok(&mut *(virt_addr.as_u64() as *mut PageTable))
    }
}

impl<AllocFrame, TranslateAddress> Mapper for RecursiveMapper<AllocFrame, TranslateAddress>
where
    AllocFrame: FnMut() -> Result<PhysicalAddress>,
    TranslateAddress: FnMut(PhysicalAddress) -> VirtualAddress,
{
    type Entry = Entry;

    unsafe fn entry(&mut self, virt_addr: VirtualAddress, level: u8) -> Result<&'static mut Entry> {
        assert!(!(level < 1 && level > 4));
        assert!(level != 1 || virt_addr % PAGE_SIZE == 0);
        assert!(level != 2 || virt_addr % PT1_EXTENT == 0);
        assert!(level != 3 || virt_addr % PT2_EXTENT == 0);
        assert!(level != 4 || virt_addr % PT3_EXTENT == 0);

        let virt_addr = virt_addr.as_u64() & !0xffff_0000_0000_0000;

        let pt4 = &mut *self.pt4;
        let pt4_idx = virt_addr / PT3_EXTENT;
        let pt4_entry = &mut pt4[pt4_idx as usize];
        if level == 4 {
            return Ok(pt4_entry);
        }

        let pt3 = self.descend_entry(pt4_entry)?;
        let pt3_idx = (virt_addr % PT3_EXTENT) / PT2_EXTENT;
        let pt3_entry = &mut pt3[pt3_idx as usize];
        if level == 3 {
            return Ok(pt3_entry);
        }

        let pt2 = self.descend_entry(pt3_entry)?;
        let pt2_idx = (virt_addr % PT2_EXTENT) / PT1_EXTENT;
        let pt2_entry = &mut pt2[pt2_idx as usize];
        if level == 2 {
            return Ok(pt2_entry);
        }

        let pt1 = self.descend_entry(pt2_entry)?;
        let pt1_idx = (virt_addr % PT1_EXTENT) / PAGE_SIZE;
        Ok(&mut pt1[pt1_idx as usize])
    }
}

#[test]
fn map_tables() {
    unsafe {
        let layout = std::alloc::Layout::from_size_align(0x100_0000, 0x1000).unwrap();
        let memory_addr = std::alloc::alloc(layout.clone());

        let pt4_addr = memory_addr as *mut PageTable;

        let mut current_addr = 0x1000;

        let mut mapper = RecursiveMapper::new(
            pt4_addr,
            || {
                let result = current_addr;
                current_addr += 0x1000;
                println!("ALLOC: {:#x}", result);
                Ok(result.into())
            },
            |phys_addr| (memory_addr as u64 + phys_addr.as_u64()).into(),
        );

        let entry = mapper.entry(0xffff_8000_0000_0000.into(), 1).unwrap();

        let pt4 = &mut *pt4_addr;
        let pt3_addr = (memory_addr as u64 + pt4[256].address().as_u64()) as *mut PageTable;
        let pt3 = &mut *pt3_addr;
        let pt2_addr = (memory_addr as u64 + pt3[0].address().as_u64()) as *mut PageTable;
        let pt2 = &mut *pt2_addr;
        let pt1_addr = (memory_addr as u64 + pt2[0].address().as_u64()) as *mut PageTable;

        assert_eq!(pt4_addr.offset(1), pt3_addr);
        assert_eq!(pt3_addr.offset(1), pt2_addr);
        assert_eq!(pt2_addr.offset(1), pt1_addr);
        assert_eq!(pt1_addr, entry as *mut _ as _);

        std::alloc::dealloc(memory_addr as _, layout);
    }
}
