use nom_7 as nom;

impl<'a> nom::AsBytes for crate::ByteData<'a> {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a, 'b> nom::Compare<crate::ByteData<'b>> for crate::ByteData<'a> {
    #[inline]
    fn compare(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(&self.as_slice(), t.as_slice())
    }

    #[inline]
    fn compare_no_case(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(&self.as_slice(), t.as_slice())
    }
}

impl<'a, 'b: 'c, 'c> nom::Compare<&'c crate::ByteData<'b>> for crate::ByteData<'a> {
    #[inline]
    fn compare(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(&self.as_slice(), t.as_slice())
    }

    #[inline]
    fn compare_no_case(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(&self.as_slice(), t.as_slice())
    }
}

impl<'a, 'b> nom::Compare<&'b [u8]> for crate::ByteData<'a> {
    #[inline]
    fn compare(&self, t: &'b [u8]) -> nom::CompareResult {
        nom::Compare::<&'b [u8]>::compare(&self.as_slice(), t)
    }

    #[inline]
    fn compare_no_case(&self, t: &'b [u8]) -> nom::CompareResult {
        nom::Compare::<&'b [u8]>::compare_no_case(&self.as_slice(), t)
    }
}

impl<'a, 'b> nom::Compare<&'b str> for crate::ByteData<'a> {
    #[inline]
    fn compare(&self, t: &'b str) -> nom::CompareResult {
        nom::Compare::<&'b str>::compare(&self.as_slice(), t)
    }

    #[inline]
    fn compare_no_case(&self, t: &'b str) -> nom::CompareResult {
        nom::Compare::<&'b str>::compare_no_case(&self.as_slice(), t)
    }
}

impl<'a, 'b> nom::FindSubstring<crate::ByteData<'b>> for crate::ByteData<'a> {
    #[inline]
    fn find_substring(&self, substr: crate::ByteData<'b>) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_slice(), substr.as_slice())
    }
}

impl<'a, 'b: 'c, 'c> nom::FindSubstring<&'c crate::ByteData<'b>> for crate::ByteData<'a> {
    #[inline]
    fn find_substring(&self, substr: &'c crate::ByteData<'b>) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_slice(), substr.as_slice())
    }
}

impl<'a, 'b> nom::FindSubstring<&'b [u8]> for crate::ByteData<'a> {
    #[inline]
    fn find_substring(&self, substr: &'b [u8]) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_slice(), substr)
    }
}

impl<'a, 'b> nom::FindSubstring<&'b str> for crate::ByteData<'a> {
    #[inline]
    fn find_substring(&self, substr: &'b str) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_slice(), substr)
    }
}

impl<'a> nom::FindToken<u8> for crate::ByteData<'a> {
    #[inline]
    fn find_token(&self, token: u8) -> bool {
        nom::FindToken::find_token(&self.as_slice(), token)
    }
}

impl<'a, 'b> nom::FindToken<&'b u8> for crate::ByteData<'a> {
    #[inline]
    fn find_token(&self, token: &'b u8) -> bool {
        nom::FindToken::find_token(&self.as_slice(), token)
    }
}

impl<'a> nom::FindToken<char> for crate::ByteData<'a> {
    #[inline]
    fn find_token(&self, token: char) -> bool {
        nom::FindToken::find_token(&self.as_slice(), token)
    }
}

impl<'a, 'b> nom::FindToken<&'b char> for crate::ByteData<'a> {
    #[inline]
    fn find_token(&self, token: &'b char) -> bool {
        nom::FindToken::find_token(&self.as_slice(), *token)
    }
}

#[cfg(feature = "alloc")]
impl<'a, 'b> nom::HexDisplay for crate::ByteData<'a> {
    fn to_hex(&self, chunk_size: usize) -> alloc::string::String {
        nom::HexDisplay::to_hex(self.as_slice(), chunk_size)
    }

    fn to_hex_from(&self, chunk_size: usize, from: usize) -> alloc::string::String {
        nom::HexDisplay::to_hex_from(self.as_slice(), chunk_size, from)
    }
}

impl<'a> nom::InputIter for crate::ByteData<'a> {
    type Item = u8;

    type Iter = core::iter::Enumerate<Self>;

    type IterElem = Self;

    fn iter_indices(&self) -> Self::Iter {
        Iterator::enumerate(self.clone())
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.clone()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        nom::InputIter::position(&self.as_slice(), predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        nom::InputIter::slice_index(&self.as_slice(), count)
    }
}

impl<'a> nom::InputLength for crate::ByteData<'a> {
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl<'a> nom::InputTake for crate::ByteData<'a> {
    fn take(&self, count: usize) -> Self {
        self.sliced(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.sliced(count..), self.sliced(..count))
    }
}

impl<'a> nom::InputTakeAtPosition for crate::ByteData<'a> {
    type Item = u8;

    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let a = self.as_slice();
        let mut i = 0;
        for b in a {
            if predicate(*b) {
                return Ok((self.sliced(i..), self.sliced(..i)));
            }
            i += 1;
        }
        Err(nom::Err::Incomplete(nom::Needed::new(1)))
    }

    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let a = self.as_slice();
        let mut i = 0;
        for b in a {
            if predicate(*b) {
                if i == 0 {
                    return Err(nom::Err::Failure(E::from_error_kind(self.clone(), e)));
                }
                return Ok((self.sliced(i..), self.sliced(..i)));
            }
            i += 1;
        }
        Err(nom::Err::Incomplete(nom::Needed::new(1)))
    }

    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let a = self.as_slice();
        let mut i = 0;
        for b in a {
            if predicate(*b) {
                return Ok((self.sliced(i..), self.sliced(..i)));
            }
            i += 1;
        }
        Ok((Self::empty(), self.clone()))
    }

    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let a = self.as_slice();
        let mut i = 0;
        for b in a {
            if predicate(*b) {
                if i == 0 {
                    return Err(nom::Err::Failure(E::from_error_kind(self.clone(), e)));
                }
                return Ok((self.sliced(i..), self.sliced(..i)));
            }
            i += 1;
        }
        Ok((Self::empty(), self.clone()))
    }
}

impl<'a> nom::Slice<core::ops::Range<usize>> for crate::ByteData<'a> {
    fn slice(&self, range: core::ops::Range<usize>) -> Self {
        self.sliced(range)
    }
}

impl<'a> nom::Slice<core::ops::RangeTo<usize>> for crate::ByteData<'a> {
    fn slice(&self, range: core::ops::RangeTo<usize>) -> Self {
        self.sliced(range)
    }
}

impl<'a> nom::Slice<core::ops::RangeFrom<usize>> for crate::ByteData<'a> {
    fn slice(&self, range: core::ops::RangeFrom<usize>) -> Self {
        self.sliced(range)
    }
}

impl<'a> nom::Slice<core::ops::RangeFull> for crate::ByteData<'a> {
    fn slice(&self, range: core::ops::RangeFull) -> Self {
        self.sliced(range)
    }
}
