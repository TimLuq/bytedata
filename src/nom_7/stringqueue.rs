use nom_7 as nom;

impl<'b> nom::Compare<crate::ByteData<'b>> for crate::StringQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self.as_bytequeue(), t.as_slice())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self.as_bytequeue(), t.as_slice())
    }
}

impl<'b: 'c, 'c> nom::Compare<&'c crate::ByteData<'b>> for crate::StringQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self.as_bytequeue(), t.as_slice())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self.as_bytequeue(), t.as_slice())
    }
}

impl<'b> nom::Compare<crate::StringData<'b>> for crate::StringQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self, t.as_str())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self, t.as_str())
    }
}

impl<'b: 'c, 'c> nom::Compare<&'c crate::StringData<'b>> for crate::StringQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'c crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self, t.as_str())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'c crate::StringData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self, t.as_str())
    }
}

impl<'b> nom::Compare<&'b [u8]> for crate::StringQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'b [u8]) -> nom::CompareResult {
        nom::Compare::compare(self.as_bytequeue(), t)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'b [u8]) -> nom::CompareResult {
        nom::Compare::compare_no_case(self.as_bytequeue(), t)
    }
}

impl<'b> nom::Compare<&'b str> for crate::StringQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'b str) -> nom::CompareResult {
        nom::Compare::compare(self, t.as_bytes())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
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

impl<'b> nom::FindSubstring<crate::ByteData<'b>> for crate::StringQueue<'_> {
    #[inline]
    fn find_substring(&self, substr: crate::ByteData<'b>) -> Option<usize> {
        self.as_bytequeue().find_slice(substr.as_slice())
    }
}

impl<'b: 'c, 'c> nom::FindSubstring<&'c crate::ByteData<'b>> for crate::StringQueue<'_> {
    #[inline]
    fn find_substring(&self, substr: &'c crate::ByteData<'b>) -> Option<usize> {
        self.as_bytequeue().find_slice(substr.as_slice())
    }
}

impl<'b> nom::FindSubstring<&'b [u8]> for crate::StringQueue<'_> {
    #[inline]
    fn find_substring(&self, substr: &'b [u8]) -> Option<usize> {
        self.as_bytequeue().find_slice(substr)
    }
}

impl<'b> nom::FindSubstring<&'b str> for crate::StringQueue<'_> {
    #[inline]
    fn find_substring(&self, substr: &'b str) -> Option<usize> {
        self.as_bytequeue().find_slice(substr.as_bytes())
    }
}

impl nom::FindToken<u8> for crate::StringQueue<'_> {
    #[inline]
    fn find_token(&self, token: u8) -> bool {
        self.bytes().any(|bv| bv == token)
    }
}

impl<'b> nom::FindToken<&'b u8> for crate::StringQueue<'_> {
    #[inline]
    fn find_token(&self, token: &'b u8) -> bool {
        let token = *token;
        self.bytes().any(|bv| bv == token)
    }
}

impl nom::FindToken<char> for crate::StringQueue<'_> {
    #[inline]
    fn find_token(&self, token: char) -> bool {
        let mut utf8 = [0; 4];
        let utf8 = token.encode_utf8(&mut utf8);
        self.as_bytequeue().find_slice(utf8.as_bytes()).is_some()
    }
}

impl<'b> nom::FindToken<&'b char> for crate::StringQueue<'_> {
    #[inline]
    fn find_token(&self, token: &'b char) -> bool {
        nom::FindToken::find_token(self, *token)
    }
}

#[cfg(feature = "alloc")]
impl nom::HexDisplay for crate::StringQueue<'_> {
    #[inline]
    fn to_hex(&self, chunk_size: usize) -> alloc::string::String {
        nom::HexDisplay::to_hex(self.as_bytequeue(), chunk_size)
    }

    #[inline]
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> alloc::string::String {
        nom::HexDisplay::to_hex_from(self.as_bytequeue(), chunk_size, from)
    }
}

impl<'a> nom::InputIter for crate::StringQueue<'a> {
    type Item = char;
    type Iter = core::iter::Enumerate<crate::queue::OwnedCharIter<'a>>;
    type IterElem = crate::queue::OwnedCharIter<'a>;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        Iterator::enumerate(self.clone().into_chars())
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.clone().into_chars()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.chars().position(predicate)
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if self.len() >= count {
            Ok(count)
        } else {
            Err(nom::Needed::new(count - self.len()))
        }
    }
}

impl nom::InputLength for crate::StringQueue<'_> {
    #[inline]
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl nom::InputTake for crate::StringQueue<'_> {
    #[inline]
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    #[inline]
    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl nom::InputTakeAtPosition for crate::StringQueue<'_> {
    type Item = char;

    #[inline]
    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let av = self
            .chars_indecies()
            .find(|&(_, ch)| predicate(ch))
            .map(|(i, _)| i);
        #[allow(clippy::option_if_let_else)]
        if let Some(av) = av {
            Ok((self.slice(av..), self.slice(..av)))
        } else {
            Err(nom::Err::Incomplete(nom::Needed::new(1)))
        }
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
        let a = self
            .chars_indecies()
            .find(|&(_, ch)| predicate(ch))
            .map(|(i, _)| i);
        match a {
            None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
            Some(0) => Err(nom::Err::Failure(E::from_error_kind(self.clone(), e))),
            Some(a) => Ok((self.slice(a..), self.slice(..a))),
        }
    }

    #[inline]
    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let av = self
            .chars_indecies()
            .find(|&(_, ch)| predicate(ch))
            .map(|(i, _)| i);
        #[allow(clippy::option_if_let_else)]
        if let Some(av) = av {
            Ok((self.slice(av..), self.slice(..av)))
        } else {
            Ok((Self::new(), self.clone()))
        }
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
        let a = self
            .chars_indecies()
            .find(|&(_, ch)| predicate(ch))
            .map(|(i, _)| i);
        match a {
            None => Ok((Self::new(), self.clone())),
            Some(0) => Err(nom::Err::Failure(E::from_error_kind(self.clone(), e))),
            Some(a) => Ok((self.slice(a..), self.slice(..a))),
        }
    }
}

impl nom::Slice<core::ops::Range<usize>> for crate::StringQueue<'_> {
    #[inline]
    fn slice(&self, range: core::ops::Range<usize>) -> Self {
        self.slice(range)
    }
}

impl nom::Slice<core::ops::RangeTo<usize>> for crate::StringQueue<'_> {
    #[inline]
    fn slice(&self, range: core::ops::RangeTo<usize>) -> Self {
        self.slice(range)
    }
}

impl nom::Slice<core::ops::RangeFrom<usize>> for crate::StringQueue<'_> {
    #[inline]
    fn slice(&self, range: core::ops::RangeFrom<usize>) -> Self {
        self.slice(range)
    }
}

impl nom::Slice<core::ops::RangeFull> for crate::StringQueue<'_> {
    #[inline]
    fn slice(&self, range: core::ops::RangeFull) -> Self {
        self.slice(range)
    }
}
