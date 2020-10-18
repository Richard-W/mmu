use core::ops;

/// Physical memory address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalAddress(u64);

/// Virtual memory address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualAddress(u64);

macro_rules! address_newtype {
    ($ty:tt) => {
        impl $ty {
            /// Create a new address
            pub fn new(numeric: u64) -> Self {
                Self(numeric)
            }

            /// Get the numeric value of the address
            pub fn as_u64(self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $ty {
            fn from(addr: u64) -> Self {
                Self(addr)
            }
        }

        impl Into<u64> for $ty {
            fn into(self) -> u64 {
                self.as_u64()
            }
        }

        impl ops::Rem<u64> for $ty {
            type Output = u64;
            fn rem(self, rhs: u64) -> u64 {
                self.as_u64() % rhs
            }
        }
    };
}

address_newtype!(PhysicalAddress);
address_newtype!(VirtualAddress);
