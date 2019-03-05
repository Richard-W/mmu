macro_rules! get_bit {
    ($field:expr, $bit:expr) => {
        ($field & (1 << $bit)) > 0
    }
}

macro_rules! set_bit {
    ($field:expr, $bit:expr, $enabled:expr) => {
        if $enabled {
            $field |= 1 << $bit;
        }
        else {
            $field &= !(1 << $bit)
        }
    }
}

macro_rules! add_indexing {
    ($ty:ty, $output:ty) => {
        use core::ops::{Index, IndexMut};

        impl Index<usize> for $ty {
            type Output = $output;
            fn index(&self, index: usize) -> &Self::Output {
                &self.entries[index]
            }
        }

        impl IndexMut<usize> for $ty {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.entries[index]
            }
        }
    }
}

