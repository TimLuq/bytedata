use core::mem::MaybeUninit;

use crate::ByteData;

#[cfg(feature = "alloc")]
const NODE_SIZE: usize = 15;

#[cfg(not(feature = "alloc"))]
const NODE_SIZE: usize = 23;

pub(super) struct LinkedNodeData<'a> {
    pub(super) data: [MaybeUninit<crate::ByteData<'a>>; NODE_SIZE],
    pub(super) beg: u8,
    pub(super) len: u8,
}

impl<'a> LinkedNodeData<'a> {
    #[cfg(not(feature = "alloc"))]
    pub(super) const fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            beg: 0,
            len: 0,
        }
    }

    pub(super) const fn with_item(data: ByteData<'a>) -> Self {
        let mut ret = Self {
            // SAFETY: data can be uninitialized because it will be written to before being read
            data: unsafe { MaybeUninit::uninit().assume_init() },
            beg: 0,
            len: 1,
        };
        ret.data[0] = MaybeUninit::new(data);
        ret
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::integer_division_remainder_used
    )]
    pub(super) fn push_back(&mut self, data: ByteData<'a>) -> Result<(), ByteData<'a>> {
        if self.len >= self.data.len() as u8 {
            return Err(data);
        }
        self.data[(self.beg as usize + self.len as usize) % self.data.len()].write(data);
        self.len += 1;
        Ok(())
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::integer_division_remainder_used
    )]
    pub(super) fn push_front(&mut self, data: ByteData<'a>) -> Result<(), ByteData<'a>> {
        if self.len >= self.data.len() as u8 {
            return Err(data);
        }
        let i = (self.beg as usize + (self.data.len() - 1)) % self.data.len();
        self.data[i].write(data);
        self.beg = i as u8;
        self.len += 1;
        Ok(())
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::integer_division_remainder_used
    )]
    pub(super) fn pop_back(&mut self) -> Option<ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let i = (self.beg as usize + self.len as usize) % self.data.len();
        // SAFETY: `i` is a valid index and is already marked as consumed so we can steal it
        Some(unsafe { self.data[i].as_mut_ptr().read() })
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::integer_division_remainder_used
    )]
    pub(super) fn pop_front(&mut self) -> Option<ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        let i = self.beg as usize;
        self.beg = (self.beg + 1) % self.data.len() as u8;
        self.len -= 1;
        // SAFETY: `i` is a valid index and is already marked as consumed so we can steal it
        Some(unsafe { self.data[i].as_mut_ptr().read() })
    }

    pub(super) const fn front(&self) -> Option<&ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: `self.beg` is a valid index so we can safely ref it
        Some(unsafe { self.data[self.beg as usize].assume_init_ref() })
    }

    #[allow(clippy::integer_division_remainder_used)]
    pub(super) const fn back(&self) -> Option<&ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        let i = (self.beg as usize + self.len as usize - 1) % self.data.len();
        // SAFETY: `i` is a valid index so we can safely ref it
        Some(unsafe { self.data[i].assume_init_ref() })
    }
}

impl Drop for LinkedNodeData<'_> {
    fn drop(&mut self) {
        let mut beg = self.beg as usize;

        #[allow(clippy::integer_division_remainder_used)]
        for _ in 0..self.len {
            // SAFETY: `b` is a valid index so we must drop it
            unsafe {
                self.data[beg].as_mut_ptr().drop_in_place();
            };
            beg = (beg + 1) % self.data.len();
        }
    }
}
