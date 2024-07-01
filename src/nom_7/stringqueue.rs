use nom_7 as nom;

impl<'a, 'b> nom::Compare<crate::ByteData<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn compare(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self.as_bytequeue(), t.as_slice())
    }

    #[inline]
    fn compare_no_case(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self.as_bytequeue(), t.as_slice())
    }
}

impl<'a, 'b: 'c, 'c> nom::Compare<&'c crate::ByteData<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn compare(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self.as_bytequeue(), t.as_slice())
    }

    #[inline]
    fn compare_no_case(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self.as_bytequeue(), t.as_slice())
    }
}

impl<'a, 'b> nom::Compare<crate::StringData<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn compare(&self, t: crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self, t.as_str())
    }

    #[inline]
    fn compare_no_case(&self, t: crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self, t.as_str())
    }
}

impl<'a, 'b: 'c, 'c> nom::Compare<&'c crate::StringData<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn compare(&self, t: &'c crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self, t.as_str())
    }

    #[inline]
    fn compare_no_case(&self, t: &'c crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self, t.as_str())
    }
}

impl<'a, 'b> nom::Compare<&'b [u8]> for crate::StringQueue<'a> {
    #[inline]
    fn compare(&self, t: &'b [u8]) -> nom::CompareResult {
        nom::Compare::compare(self.as_bytequeue(), t)
    }

    #[inline]
    fn compare_no_case(&self, t: &'b [u8]) -> nom::CompareResult {
        nom::Compare::compare_no_case(self.as_bytequeue(), t)
    }
}

impl<'a, 'b> nom::Compare<&'b str> for crate::StringQueue<'a> {
    #[inline]
    fn compare(&self, t: &'b str) -> nom::CompareResult {
        nom::Compare::compare(self, t.as_bytes())
    }

    #[inline]
    fn compare_no_case(&self, t: &'b str) -> nom::CompareResult {
        let pos = self
            .chars()
            .zip(t.chars())
            .position(|(a, b)| a.to_lowercase().ne(b.to_lowercase()));

        match pos {
            Some(_) => nom::CompareResult::Error,
            None => {
                if self.len() >= t.len() {
                    nom::CompareResult::Ok
                } else {
                    nom::CompareResult::Incomplete
                }
            }
        }
    }
}

impl<'a, 'b> nom::FindSubstring<crate::ByteData<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn find_substring(&self, substr: crate::ByteData<'b>) -> Option<usize> {
        self.as_bytequeue().find_slice(substr.as_slice())
    }
}

impl<'a, 'b: 'c, 'c> nom::FindSubstring<&'c crate::ByteData<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn find_substring(&self, substr: &'c crate::ByteData<'b>) -> Option<usize> {
        self.as_bytequeue().find_slice(substr.as_slice())
    }
}

impl<'a, 'b> nom::FindSubstring<&'b [u8]> for crate::StringQueue<'a> {
    #[inline]
    fn find_substring(&self, substr: &'b [u8]) -> Option<usize> {
        self.as_bytequeue().find_slice(substr)
    }
}

impl<'a, 'b> nom::FindSubstring<&'b str> for crate::StringQueue<'a> {
    #[inline]
    fn find_substring(&self, substr: &'b str) -> Option<usize> {
        self.as_bytequeue().find_slice(substr.as_bytes())
    }
}

impl<'a> nom::FindToken<u8> for crate::StringQueue<'a> {
    #[inline]
    fn find_token(&self, token: u8) -> bool {
        self.bytes().any(|b| b == token)
    }
}

impl<'a, 'b> nom::FindToken<&'b u8> for crate::StringQueue<'a> {
    #[inline]
    fn find_token(&self, token: &'b u8) -> bool {
        let token = *token;
        self.bytes().any(|b| b == token)
    }
}

impl<'a> nom::FindToken<char> for crate::StringQueue<'a> {
    #[inline]
    fn find_token(&self, token: char) -> bool {
        let mut utf8 = [0; 4];
        let utf8 = token.encode_utf8(&mut utf8);
        self.as_bytequeue().find_slice(utf8.as_bytes()).is_some()
    }
}

impl<'a, 'b> nom::FindToken<&'b char> for crate::StringQueue<'a> {
    #[inline]
    fn find_token(&self, token: &'b char) -> bool {
        nom::FindToken::find_token(self, *token)
    }
}

#[cfg(feature = "alloc")]
impl<'a, 'b> nom::HexDisplay for crate::StringQueue<'a> {
    fn to_hex(&self, chunk_size: usize) -> alloc::string::String {
        nom::HexDisplay::to_hex(self.as_bytequeue(), chunk_size)
    }

    fn to_hex_from(&self, chunk_size: usize, from: usize) -> alloc::string::String {
        nom::HexDisplay::to_hex_from(self.as_bytequeue(), chunk_size, from)
    }
}

impl<'a> nom::InputIter for crate::StringQueue<'a> {
    type Item = char;

    type Iter = core::iter::Enumerate<crate::queue::OwnedCharIter<'a>>;

    type IterElem = crate::queue::OwnedCharIter<'a>;

    fn iter_indices(&self) -> Self::Iter {
        Iterator::enumerate(self.clone().into_chars())
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.clone().into_chars()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.chars().position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if self.len() >= count {
            Ok(count)
        } else {
            Err(nom::Needed::new(count - self.len()))
        }
    }
}

impl<'a> nom::InputLength for crate::StringQueue<'a> {
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl<'a> nom::InputTake for crate::StringQueue<'a> {
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl<'a> nom::InputTakeAtPosition for crate::StringQueue<'a> {
    type Item = char;

    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let a = self
            .chars_indecies()
            .map(|(i, c)| (i, c))
            .find(|(_, c)| predicate(*c))
            .map(|(i, _)| i);
        if let Some(a) = a {
            Ok((self.slice(a..), self.slice(..a)))
        } else {
            Err(nom::Err::Incomplete(nom::Needed::new(1)))
        }
    }

    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let a = self
            .chars_indecies()
            .map(|(i, c)| (i, c))
            .find(|(_, c)| predicate(*c))
            .map(|(i, _)| i);
        match a {
            None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
            Some(0) => Err(nom::Err::Failure(E::from_error_kind(self.clone(), e))),
            Some(a) => Ok((self.slice(a..), self.slice(..a))),
        }
    }

    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let a = self
            .chars_indecies()
            .map(|(i, c)| (i, c))
            .find(|(_, c)| predicate(*c))
            .map(|(i, _)| i);
        if let Some(a) = a {
            Ok((self.slice(a..), self.slice(..a)))
        } else {
            Ok((Self::new(), self.clone()))
        }
    }

    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let a = self
            .chars_indecies()
            .map(|(i, c)| (i, c))
            .find(|(_, c)| predicate(*c))
            .map(|(i, _)| i);
        match a {
            None => Ok((Self::new(), self.clone())),
            Some(0) => Err(nom::Err::Failure(E::from_error_kind(self.clone(), e))),
            Some(a) => Ok((self.slice(a..), self.slice(..a))),
        }
    }
}

impl<'a> nom::Slice<core::ops::Range<usize>> for crate::StringQueue<'a> {
    fn slice(&self, range: core::ops::Range<usize>) -> Self {
        self.slice(range)
    }
}

impl<'a> nom::Slice<core::ops::RangeTo<usize>> for crate::StringQueue<'a> {
    fn slice(&self, range: core::ops::RangeTo<usize>) -> Self {
        self.slice(range)
    }
}

impl<'a> nom::Slice<core::ops::RangeFrom<usize>> for crate::StringQueue<'a> {
    fn slice(&self, range: core::ops::RangeFrom<usize>) -> Self {
        self.slice(range)
    }
}

impl<'a> nom::Slice<core::ops::RangeFull> for crate::StringQueue<'a> {
    fn slice(&self, range: core::ops::RangeFull) -> Self {
        self.slice(range)
    }
}
