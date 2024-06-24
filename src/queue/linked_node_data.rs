use core::mem::MaybeUninit;

use crate::ByteData;

pub(super) struct LinkedNodeData<'a> {
    pub(super) data: [MaybeUninit<crate::ByteData<'a>>; 8],
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
        let mut r = Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            beg: 0,
            len: 1,
        };
        r.data[0] = MaybeUninit::new(data);
        r
    }

    pub(super) fn push_back(&mut self, data: ByteData<'a>) -> Result<(), ByteData<'a>> {
        if self.len >= self.data.len() as u8 {
            return Err(data);
        }
        self.data[(self.beg as usize + self.len as usize) % self.data.len()].write(data);
        self.len += 1;
        Ok(())
    }

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

    pub(super) fn pop_back(&mut self) -> Option<ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let i = (self.beg as usize + self.len as usize) % self.data.len();
        Some(unsafe { self.data[i].as_mut_ptr().read() })
    }

    pub(super) fn pop_front(&mut self) -> Option<ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        let i = self.beg as usize;
        self.beg = (self.beg + 1) % self.data.len() as u8;
        self.len -= 1;
        Some(unsafe { self.data[i].as_mut_ptr().read() })
    }

    pub(super) fn front(&self) -> Option<&ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        Some(unsafe { self.data[self.beg as usize].assume_init_ref() })
    }

    pub(super) fn back(&self) -> Option<&ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        let i = (self.beg as usize + self.len as usize - 1) % self.data.len();
        Some(unsafe { self.data[i].assume_init_ref() })
    }
}

impl Drop for LinkedNodeData<'_> {
    fn drop(&mut self) {
        let mut b = self.beg as usize;
        for _ in 0..self.len {
            unsafe {
                self.data[b].as_mut_ptr().drop_in_place();
            }
            b = (b + 1) % self.data.len();
        }
    }
}
