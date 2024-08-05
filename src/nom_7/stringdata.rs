use nom_7 as nom;

impl nom::AsBytes for crate::StringData<'_> {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'b> nom::Compare<crate::ByteData<'b>> for crate::StringData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(&self.as_bytes(), t.as_slice())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(&self.as_bytes(), t.as_slice())
    }
}

impl<'b> nom::Compare<crate::StringData<'b>> for crate::StringData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare(&self.as_str(), t.as_str())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(&self.as_str(), t.as_str())
    }
}

impl<'b: 'c, 'c> nom::Compare<&'c crate::ByteData<'b>> for crate::StringData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(&self.as_bytes(), t.as_slice())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(&self.as_bytes(), t.as_slice())
    }
}

impl<'b: 'c, 'c> nom::Compare<&'c crate::StringData<'b>> for crate::StringData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'c crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare(&self.as_str(), t.as_str())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'c crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(&self.as_str(), t.as_str())
    }
}

impl<'b> nom::Compare<&'b [u8]> for crate::StringData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'b [u8]) -> nom::CompareResult {
        nom::Compare::<&'b [u8]>::compare(&self.as_bytes(), t)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'b [u8]) -> nom::CompareResult {
        nom::Compare::<&'b [u8]>::compare_no_case(&self.as_bytes(), t)
    }
}

impl<'b> nom::Compare<&'b str> for crate::StringData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'b str) -> nom::CompareResult {
        nom::Compare::<&'b str>::compare(&self.as_str(), t)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'b str) -> nom::CompareResult {
        nom::Compare::<&'b str>::compare_no_case(&self.as_str(), t)
    }
}

impl<'b> nom::FindSubstring<crate::ByteData<'b>> for crate::StringData<'_> {
    #[inline]
    fn find_substring(&self, substr: crate::ByteData<'b>) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_bytes(), substr.as_slice())
    }
}

impl<'b> nom::FindSubstring<crate::StringData<'b>> for crate::StringData<'_> {
    #[inline]
    fn find_substring(&self, substr: crate::StringData<'b>) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_str(), substr.as_str())
    }
}

impl<'b: 'c, 'c> nom::FindSubstring<&'c crate::ByteData<'b>> for crate::StringData<'_> {
    #[inline]
    fn find_substring(&self, substr: &'c crate::ByteData<'b>) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_bytes(), substr.as_slice())
    }
}

impl<'b: 'c, 'c> nom::FindSubstring<&'c crate::StringData<'b>> for crate::StringData<'_> {
    #[inline]
    fn find_substring(&self, substr: &'c crate::StringData<'b>) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_str(), substr.as_str())
    }
}

impl<'b> nom::FindSubstring<&'b [u8]> for crate::StringData<'_> {
    #[inline]
    fn find_substring(&self, substr: &'b [u8]) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_bytes(), substr)
    }
}

impl<'b> nom::FindSubstring<&'b str> for crate::StringData<'_> {
    #[inline]
    fn find_substring(&self, substr: &'b str) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.as_str(), substr)
    }
}

impl nom::FindToken<u8> for crate::StringData<'_> {
    #[inline]
    fn find_token(&self, token: u8) -> bool {
        nom::FindToken::find_token(&self.as_bytes(), token)
    }
}

impl<'b> nom::FindToken<&'b u8> for crate::StringData<'_> {
    #[inline]
    fn find_token(&self, token: &'b u8) -> bool {
        nom::FindToken::find_token(&self.as_bytes(), token)
    }
}

impl nom::FindToken<char> for crate::StringData<'_> {
    #[inline]
    fn find_token(&self, token: char) -> bool {
        nom::FindToken::find_token(&self.as_str(), token)
    }
}

impl<'b> nom::FindToken<&'b char> for crate::StringData<'_> {
    #[inline]
    fn find_token(&self, token: &'b char) -> bool {
        nom::FindToken::find_token(&self.as_str(), *token)
    }
}

#[cfg(feature = "alloc")]
impl nom::HexDisplay for crate::StringData<'_> {
    #[inline]
    fn to_hex(&self, chunk_size: usize) -> alloc::string::String {
        nom::HexDisplay::to_hex(self.as_str(), chunk_size)
    }

    #[inline]
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> alloc::string::String {
        nom::HexDisplay::to_hex_from(self.as_str(), chunk_size, from)
    }
}

impl nom::InputIter for crate::StringData<'_> {
    type Item = char;
    type Iter = core::iter::Enumerate<Self>;
    type IterElem = Self;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        Iterator::enumerate(self.clone())
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.clone()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        nom::InputIter::position(&self.as_str(), predicate)
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        nom::InputIter::slice_index(&self.as_str(), count)
    }
}

impl nom::InputLength for crate::StringData<'_> {
    #[inline]
    fn input_len(&self) -> usize {
        nom::InputLength::input_len(&self.as_str())
    }
}

impl nom::InputTake for crate::StringData<'_> {
    #[inline]
    fn take(&self, count: usize) -> Self {
        self.sliced(..count)
    }

    #[inline]
    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.sliced(count..), self.sliced(..count))
    }
}

impl nom::InputTakeAtPosition for crate::StringData<'_> {
    type Item = char;

    #[inline]
    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        for (i, bv) in self.as_str().char_indices() {
            if predicate(bv) {
                return Ok((self.sliced(i..), self.sliced(..i)));
            }
        }
        Err(nom::Err::Incomplete(nom::Needed::new(1)))
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        for (i, b) in self.as_str().char_indices() {
            if predicate(b) {
                if i == 0 {
                    return Err(nom::Err::Failure(E::from_error_kind(self.clone(), e)));
                }
                return Ok((self.sliced(i..), self.sliced(..i)));
            }
        }
        Err(nom::Err::Incomplete(nom::Needed::new(1)))
    }

    #[inline]
    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        for (i, bv) in self.as_str().char_indices() {
            if predicate(bv) {
                return Ok((self.sliced(i..), self.sliced(..i)));
            }
        }
        Ok((Self::empty(), self.clone()))
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        for (i, b) in self.as_str().char_indices() {
            if predicate(b) {
                if i == 0 {
                    return Err(nom::Err::Failure(E::from_error_kind(self.clone(), e)));
                }
                return Ok((self.sliced(i..), self.sliced(..i)));
            }
        }
        Ok((Self::empty(), self.clone()))
    }
}

impl nom::Slice<core::ops::Range<usize>> for crate::StringData<'_> {
    #[inline]
    fn slice(&self, range: core::ops::Range<usize>) -> Self {
        self.sliced(range)
    }
}

impl nom::Slice<core::ops::RangeTo<usize>> for crate::StringData<'_> {
    #[inline]
    fn slice(&self, range: core::ops::RangeTo<usize>) -> Self {
        self.sliced(range)
    }
}

impl nom::Slice<core::ops::RangeFrom<usize>> for crate::StringData<'_> {
    #[inline]
    fn slice(&self, range: core::ops::RangeFrom<usize>) -> Self {
        self.sliced(range)
    }
}

impl nom::Slice<core::ops::RangeFull> for crate::StringData<'_> {
    #[inline]
    fn slice(&self, range: core::ops::RangeFull) -> Self {
        self.sliced(range)
    }
}
