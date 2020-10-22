#![cfg_attr(not(test), no_std)]

#[macro_use]
mod macros;

mod result;
pub use result::*;

mod types;
pub use types::*;

pub mod x86_64;

/// Provides access to page table entries.
pub trait Mapper {
    /// Page table entry type
    type Entry;

    /// Get the page table entry for a virtual address.
    ///
    /// # Safety
    ///
    /// Function itself is safe but modifying entries is inherently unsafe.
    unsafe fn entry(
        &mut self,
        virt_addr: VirtualAddress,
        level: u8,
    ) -> Result<&'static mut Self::Entry>;
}
