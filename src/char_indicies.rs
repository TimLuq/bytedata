/// An error that can occur when parsing a UTF-8 character in a byte sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Utf8CharError {
    /// The index in the input up to which valid UTF-8 was verified.
    pub valid_up_to: usize,
    /// The length of the invalid UTF-8 sequence in bytes.
    /// If the length is `0`, the error represent empty input.
    pub error_len: u8,
}

/// An iterator over the indices of the UTF-8 characters in a byte slice.
///
/// Can be iterated over like any iterator, or used with the [`Utf8CharIndices::next_const`] method to get the next character in a const context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Utf8CharIndices<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Utf8CharIndices<'a> {
    /// Create a new `Utf8CharIndices` from a byte slice.
    /// This a
    #[inline]
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Get the remaining byte sequence.
    #[inline]
    #[must_use]
    pub const fn remaining(self) -> &'a [u8] {
        self.data
    }

    /// Get the size hint of the remaining byte sequence.
    #[inline]
    #[must_use]
    pub const fn size_hint(&self) -> (usize, usize) {
        let len = self.data.len();
        ((len + 3) >> 2, len)
    }

    /// Get the next UTF-8 character from the remaining byte sequence.
    ///
    /// # Errors
    ///
    /// Returns an error if the next byte sequence is not a valid UTF-8 character.
    /// This also means that an error is returned if the input is empty, in which case the error length is `0`.
    #[inline]
    pub const fn next_const(mut self) -> Result<(Self, usize, char), (Self, Utf8CharError)> {
        /// Parse the next multi-byte UTF-8 character from the input.
        const fn inner(mut data: *const u8, max: usize, first: u8) -> Result<(char, u8), u8> {
            let (char, width) = if first & 0b1110_0000 == 0b1100_0000 {
                (first & 0b0001_1111, 2_u8)
            } else if first & 0b1111_0000 == 0b1110_0000 {
                (first & 0b0000_1111, 3_u8)
            } else if first & 0b1111_1000 == 0b1111_0000 {
                (first & 0b0000_0111, 4_u8)
            } else {
                return Err(1);
            };

            #[allow(clippy::cast_possible_truncation)]
            if max < 4 && (max as u8) < width {
                return Err(1);
            }

            let mut char = char as u32;
            let mut i = 1;
            while i < width {
                // SAFETY: `i` is always less than `max`
                data = unsafe { data.add(1) };
                // SAFETY: `data` is always within the bounds of `data`
                let byte = unsafe { data.read() };
                if byte & 0b1100_0000 != 0b1000_0000 {
                    return Err(i);
                }
                char = (char << 6_u8) | (byte & 0b0011_1111) as u32;
                i += 1;
            }

            if let Some(char) = core::char::from_u32(char) {
                Ok((char, width))
            } else {
                Err(width)
            }
        }

        if self.data.is_empty() {
            return Err((
                self,
                Utf8CharError {
                    valid_up_to: self.offset,
                    error_len: 0,
                },
            ));
        }

        let data = self.data.as_ptr();
        // SAFETY: `data` is always within the bounds as it is not empty
        let first = unsafe { data.read() };
        let (char, width) = if first < 128 {
            (first as char, 1)
        } else {
            match inner(data, self.data.len(), first) {
                Ok((char, width)) => (char, width as usize),
                Err(off) => {
                    self.data = crate::const_or_bytes(
                        crate::const_slice(self.data, 1..self.data.len()),
                        &[],
                    );
                    let valid_up_to = self.offset;
                    self.offset += 1;
                    return Err((
                        self,
                        Utf8CharError {
                            valid_up_to,
                            error_len: off,
                        },
                    ));
                }
            }
        };

        self.data =
            crate::const_or_bytes(crate::const_slice(self.data, width..self.data.len()), &[]);
        let old_offset = self.offset;
        self.offset += width;
        Ok((self, old_offset, char))
    }
}

#[allow(clippy::copy_iterator)]
impl core::iter::Iterator for Utf8CharIndices<'_> {
    type Item = Result<(usize, char), Utf8CharError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_const() {
            Ok((this, offset, char)) => {
                *self = this;
                Some(Ok((offset, char)))
            }
            Err((this, err)) => {
                *self = this;
                if err.error_len == 0 {
                    None
                } else {
                    Some(Err(err))
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.size_hint();
        (min, Some(max))
    }
}

impl core::iter::FusedIterator for Utf8CharIndices<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_char_indices() {
        let data = "Hello, 世界!";
        let data = data.as_bytes();
        let mut iter = Utf8CharIndices::new(data);
        assert_eq!(iter.size_hint(), (4, 14));
        assert_eq!(iter.next(), Some(Ok((0, 'H'))));
        assert_eq!(iter.size_hint(), (4, 13));
        assert_eq!(iter.next(), Some(Ok((1, 'e'))));
        assert_eq!(iter.size_hint(), (3, 12));
        assert_eq!(iter.next(), Some(Ok((2, 'l'))));
        assert_eq!(iter.next(), Some(Ok((3, 'l'))));
        assert_eq!(iter.next(), Some(Ok((4, 'o'))));
        assert_eq!(iter.next(), Some(Ok((5, ','))));
        assert_eq!(iter.next(), Some(Ok((6, ' '))));
        assert_eq!(iter.next(), Some(Ok((7, '世'))));
        assert_eq!(iter.next(), Some(Ok((10, '界'))));
        assert_eq!(iter.next(), Some(Ok((13, '!'))));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, 0));
    }

    #[test]
    fn utf8_char_indices_error() {
        let data = b"Hello, \xFF\xF0\x9F\x98\x8A!";
        let data = data.as_slice();
        let mut iter = Utf8CharIndices::new(data);
        assert_eq!(iter.size_hint(), (4, 13));
        assert_eq!(iter.next(), Some(Ok((0, 'H'))));
        assert_eq!(iter.next(), Some(Ok((1, 'e'))));
        assert_eq!(iter.next(), Some(Ok((2, 'l'))));
        assert_eq!(iter.next(), Some(Ok((3, 'l'))));
        assert_eq!(iter.next(), Some(Ok((4, 'o'))));
        assert_eq!(iter.next(), Some(Ok((5, ','))));
        assert_eq!(iter.next(), Some(Ok((6, ' '))));
        assert_eq!(
            iter.next(),
            Some(Err(Utf8CharError {
                valid_up_to: 7,
                error_len: 1
            }))
        );
        assert_eq!(iter.next(), Some(Ok((8, '😊'))));
        assert_eq!(iter.next(), Some(Ok((12, '!'))));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, 0));
    }

    #[test]
    fn utf8_char_indices_empty() {
        let data = b"".as_slice();
        let mut iter = Utf8CharIndices::new(data);
        assert_eq!(iter.size_hint(), (0, 0));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, 0));
    }
}
