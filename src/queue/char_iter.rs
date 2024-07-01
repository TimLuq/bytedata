use super::StringQueue;

/// An iterator over the characters of a [`StringQueue`].
pub struct CharIter<'a, 'b> {
    bytes: super::ByteIter<'a, 'b>,
}

impl<'a, 'b> CharIter<'a, 'b> {
    #[inline]
    pub(super) fn new(queue: &'b StringQueue<'a>) -> Self {
        Self {
            bytes: super::ByteIter::new(queue.as_bytequeue()),
        }
    }
}

impl<'a, 'b> Iterator for CharIter<'a, 'b> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let b0 = self.bytes.next()?;
        let (mut ch, expects) = match b0 {
            b0 if b0 & 0b1000_0000 == 0 => (b0 as u32, 0),
            b0 if b0 & 0b1110_0000 == 0b1100_0000 => (b0 as u32 & 0b0001_1111, 1),
            b0 if b0 & 0b1111_0000 == 0b1110_0000 => (b0 as u32 & 0b0000_1111, 2),
            b0 if b0 & 0b1111_1000 == 0b1111_0000 => (b0 as u32 & 0b0000_0111, 3),
            _ => return None,
        };
        for _ in 0..expects {
            let b = self.bytes.next()?;
            if b & 0b1100_0000 != 0b1000_0000 {
                panic!("CharIter: Invalid UTF-8 continuation byte");
            }
            ch = (ch << 6) | (b as u32 & 0b0011_1111);
        }
        Some(unsafe { core::char::from_u32_unchecked(ch) })
    }
}

/// An iterator over the characters of a [`StringQueue`].
pub struct CharIndecies<'a, 'b> {
    bytes: super::ByteIter<'a, 'b>,
    offset: usize,
}

impl<'a, 'b> CharIndecies<'a, 'b> {
    #[inline]
    pub(super) fn new(queue: &'b StringQueue<'a>) -> Self {
        Self {
            bytes: super::ByteIter::new(queue.as_bytequeue()),
            offset: 0,
        }
    }
}

impl<'a, 'b> Iterator for CharIndecies<'a, 'b> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let b0 = self.bytes.next()?;
        let pre = self.offset;
        self.offset += 1;
        let (mut ch, expects) = match b0 {
            b0 if b0 & 0b1000_0000 == 0 => (b0 as u32, 0),
            b0 if b0 & 0b1110_0000 == 0b1100_0000 => (b0 as u32 & 0b0001_1111, 1),
            b0 if b0 & 0b1111_0000 == 0b1110_0000 => (b0 as u32 & 0b0000_1111, 2),
            b0 if b0 & 0b1111_1000 == 0b1111_0000 => (b0 as u32 & 0b0000_0111, 3),
            _ => return None,
        };
        for _ in 0..expects {
            let b = self.bytes.next()?;
            self.offset += 1;
            if b & 0b1100_0000 != 0b1000_0000 {
                panic!("CharIter: Invalid UTF-8 continuation byte");
            }
            ch = (ch << 6) | (b as u32 & 0b0011_1111);
        }
        Some((pre, unsafe { core::char::from_u32_unchecked(ch) }))
    }
}

/// An iterator over the characters of a [`StringQueue`].
pub struct OwnedCharIter<'a> {
    bytes: super::OwnedByteIter<'a>,
}

impl<'a> OwnedCharIter<'a> {
    #[inline]
    pub(super) fn new(queue: StringQueue<'a>) -> Self {
        Self {
            bytes: super::OwnedByteIter::new(queue.into_bytequeue()),
        }
    }
}

impl<'a> Iterator for OwnedCharIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let b0 = self.bytes.next()?;
        let (mut ch, expects) = match b0 {
            b0 if b0 & 0b1000_0000 == 0 => (b0 as u32, 0),
            b0 if b0 & 0b1110_0000 == 0b1100_0000 => (b0 as u32 & 0b0001_1111, 1),
            b0 if b0 & 0b1111_0000 == 0b1110_0000 => (b0 as u32 & 0b0000_1111, 2),
            b0 if b0 & 0b1111_1000 == 0b1111_0000 => (b0 as u32 & 0b0000_0111, 3),
            _ => return None,
        };
        for _ in 0..expects {
            let b = self.bytes.next()?;
            if b & 0b1100_0000 != 0b1000_0000 {
                panic!("CharIter: Invalid UTF-8 continuation byte");
            }
            ch = (ch << 6) | (b as u32 & 0b0011_1111);
        }
        Some(unsafe { core::char::from_u32_unchecked(ch) })
    }
}
