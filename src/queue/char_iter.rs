use super::StringQueue;

/// An iterator over the characters of a [`StringQueue`].
#[allow(missing_debug_implementations)]
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
    #[inline]
    pub(super) const unsafe fn from_byte_iter(bytes: super::ByteIter<'a, 'b>) -> Self {
        Self { bytes }
    }
}

#[allow(single_use_lifetimes, clippy::needless_lifetimes)]
impl<'a, 'b> Iterator for CharIter<'a, 'b> {
    type Item = char;

    #[allow(clippy::missing_inline_in_public_items)]
    fn next(&mut self) -> Option<Self::Item> {
        let b0 = self.bytes.next()?;
        #[allow(clippy::cast_lossless)]
        let (mut ch, expects) = match b0 {
            b0 if b0 & 0b1000_0000 == 0 => (b0 as u32, 0_u8),
            b0 if b0 & 0b1110_0000 == 0b1100_0000 => (b0 as u32 & 0b0001_1111, 1_u8),
            b0 if b0 & 0b1111_0000 == 0b1110_0000 => (b0 as u32 & 0b0000_1111, 2_u8),
            b0 if b0 & 0b1111_1000 == 0b1111_0000 => (b0 as u32 & 0b0000_0111, 3_u8),
            _ => return None,
        };
        #[allow(clippy::cast_lossless)]
        for _ in 0..expects {
            let by: u8 = self.bytes.next()?;
            debug_assert!(
                by & 0b1100_0000 == 0b1000_0000,
                "CharIter: Invalid UTF-8 continuation byte"
            );
            ch = (ch << 6_u8) | (by as u32 & 0b0011_1111);
        }
        // SAFETY: `ch` is a valid Unicode code point.
        Some(unsafe { core::char::from_u32_unchecked(ch) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // this is the absolute bounds where the lower bound is the number of chars if all bytes were 4 byte chars
        // and the upper bound assumes all bytes are ASCII-7 chars.
        let av = self.bytes.len();
        ((av + 3) >> 2, Some(av))
    }
}

/// An iterator over the characters of a [`StringQueue`].
#[allow(missing_debug_implementations)]
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

#[allow(single_use_lifetimes, clippy::needless_lifetimes)]
impl<'a, 'b> Iterator for CharIndecies<'a, 'b> {
    type Item = (usize, char);

    #[allow(clippy::missing_inline_in_public_items)]
    fn next(&mut self) -> Option<Self::Item> {
        let b0 = self.bytes.next()?;
        let pre = self.offset;
        self.offset += 1;
        #[allow(clippy::cast_lossless)]
        let (mut ch, expects) = match b0 {
            b0 if b0 & 0b1000_0000 == 0 => (b0 as u32, 0_u8),
            b0 if b0 & 0b1110_0000 == 0b1100_0000 => (b0 as u32 & 0b0001_1111, 1),
            b0 if b0 & 0b1111_0000 == 0b1110_0000 => (b0 as u32 & 0b0000_1111, 2),
            b0 if b0 & 0b1111_1000 == 0b1111_0000 => (b0 as u32 & 0b0000_0111, 3),
            _ => return None,
        };
        #[allow(clippy::cast_lossless)]
        for _ in 0..expects {
            let by: u8 = self.bytes.next()?;
            self.offset += 1;
            debug_assert!(
                by & 0b1100_0000 == 0b1000_0000,
                "CharIter: Invalid UTF-8 continuation byte"
            );
            ch = (ch << 6_u8) | (by as u32 & 0b0011_1111);
        }
        // SAFETY: `ch` is a valid Unicode code point.
        Some((pre, unsafe { core::char::from_u32_unchecked(ch) }))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // this is the absolute bounds where the lower bound is the number of chars if all bytes were 4 byte chars
        // and the upper bound assumes all bytes are ASCII-7 chars.
        let aa = self.bytes.len();
        ((aa + 3) >> 2, Some(aa))
    }
}

/// An iterator over the characters of a [`StringQueue`].
#[allow(missing_debug_implementations)]
pub struct OwnedCharIter<'a> {
    bytes: super::OwnedByteIter<'a>,
}

impl<'a> OwnedCharIter<'a> {
    #[inline]
    pub(super) const fn new(queue: StringQueue<'a>) -> Self {
        Self {
            bytes: super::OwnedByteIter::new(queue.into_bytequeue()),
        }
    }
}

impl Iterator for OwnedCharIter<'_> {
    type Item = char;

    #[allow(clippy::missing_inline_in_public_items)]
    fn next(&mut self) -> Option<Self::Item> {
        let b0 = self.bytes.next()?;
        #[allow(clippy::cast_lossless)]
        let (mut ch, expects) = match b0 {
            b0 if b0 & 0b1000_0000 == 0 => (b0 as u32, 0_u8),
            b0 if b0 & 0b1110_0000 == 0b1100_0000 => (b0 as u32 & 0b0001_1111, 1_u8),
            b0 if b0 & 0b1111_0000 == 0b1110_0000 => (b0 as u32 & 0b0000_1111, 2_u8),
            b0 if b0 & 0b1111_1000 == 0b1111_0000 => (b0 as u32 & 0b0000_0111, 3_u8),
            _ => return None,
        };
        #[allow(clippy::cast_lossless)]
        for _ in 0..expects {
            let by = self.bytes.next()?;
            debug_assert!(
                by & 0b1100_0000 == 0b1000_0000,
                "CharIter: Invalid UTF-8 continuation byte"
            );
            ch = (ch << 6_u8) | (by as u32 & 0b0011_1111);
        }
        // SAFETY: `ch` is a valid Unicode code point.
        Some(unsafe { core::char::from_u32_unchecked(ch) })
    }
}
