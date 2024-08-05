use nom_7 as nom;

impl<'b> nom::Compare<crate::ByteData<'b>> for crate::ByteQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self, t.as_slice())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self, t.as_slice())
    }
}

impl<'b: 'c, 'c> nom::Compare<&'c crate::ByteData<'b>> for crate::ByteQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare(self, t.as_slice())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'c crate::ByteData<'b>) -> nom::CompareResult {
        nom::Compare::compare_no_case(self, t.as_slice())
    }
}

impl<'b> nom::Compare<crate::StringData<'b>> for crate::ByteQueue<'_> {
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

impl<'b: 'c, 'c> nom::Compare<&'c crate::StringData<'b>> for crate::ByteQueue<'_> {
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

impl<'b> nom::Compare<&'b [u8]> for crate::ByteQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'b [u8]) -> nom::CompareResult {
        let mut rest = t;
        for s in self.chunks() {
            let (a, rest2) = rest.split_at(s.len().min(rest.len()));
            rest = rest2;
            match nom::Compare::compare(&s.as_slice(), a) {
                nom::CompareResult::Ok => continue,
                nom::CompareResult::Error => return nom::CompareResult::Error,
                nom::CompareResult::Incomplete => return nom::CompareResult::Incomplete,
            }
        }
        if rest.is_empty() {
            nom::CompareResult::Ok
        } else {
            nom::CompareResult::Incomplete
        }
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'b [u8]) -> nom::CompareResult {
        let mut rest = t;
        for s in self.chunks() {
            let (a, rest2) = rest.split_at(s.len().min(rest.len()));
            rest = rest2;
            match nom::Compare::compare_no_case(&s.as_slice(), a) {
                nom::CompareResult::Ok => continue,
                nom::CompareResult::Error => return nom::CompareResult::Error,
                nom::CompareResult::Incomplete => return nom::CompareResult::Incomplete,
            }
        }
        if rest.is_empty() {
            nom::CompareResult::Ok
        } else {
            nom::CompareResult::Incomplete
        }
    }
}

impl<'b> nom::Compare<&'b str> for crate::ByteQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare(&self, t: &'b str) -> nom::CompareResult {
        nom::Compare::compare(self, t.as_bytes())
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn compare_no_case(&self, t: &'b str) -> nom::CompareResult {
        nom::Compare::compare_no_case(self, t.as_bytes())
    }
}

impl<'b> nom::FindSubstring<crate::ByteData<'b>> for crate::ByteQueue<'_> {
    #[inline]
    fn find_substring(&self, substr: crate::ByteData<'b>) -> Option<usize> {
        self.find_slice(substr.as_slice())
    }
}

impl<'b: 'c, 'c> nom::FindSubstring<&'c crate::ByteData<'b>> for crate::ByteQueue<'_> {
    #[inline]
    fn find_substring(&self, substr: &'c crate::ByteData<'b>) -> Option<usize> {
        self.find_slice(substr.as_slice())
    }
}

impl<'b> nom::FindSubstring<&'b [u8]> for crate::ByteQueue<'_> {
    #[inline]
    fn find_substring(&self, substr: &'b [u8]) -> Option<usize> {
        self.find_slice(substr)
    }
}

impl<'b> nom::FindSubstring<&'b str> for crate::ByteQueue<'_> {
    #[inline]
    fn find_substring(&self, substr: &'b str) -> Option<usize> {
        self.find_slice(substr.as_bytes())
    }
}

impl nom::FindToken<u8> for crate::ByteQueue<'_> {
    #[inline]
    fn find_token(&self, token: u8) -> bool {
        self.bytes().any(|bv| bv == token)
    }
}

impl<'b> nom::FindToken<&'b u8> for crate::ByteQueue<'_> {
    #[inline]
    fn find_token(&self, token: &'b u8) -> bool {
        let token = *token;
        self.bytes().any(|bv| bv == token)
    }
}

impl nom::FindToken<char> for crate::ByteQueue<'_> {
    #[inline]
    fn find_token(&self, token: char) -> bool {
        let mut utf8 = [0; 4];
        let utf8 = token.encode_utf8(&mut utf8);
        self.find_slice(utf8.as_bytes()).is_some()
    }
}

impl<'b> nom::FindToken<&'b char> for crate::ByteQueue<'_> {
    #[inline]
    fn find_token(&self, token: &'b char) -> bool {
        nom::FindToken::find_token(self, *token)
    }
}

#[cfg(feature = "alloc")]
impl nom::HexDisplay for crate::ByteQueue<'_> {
    #[inline]
    fn to_hex(&self, chunk_size: usize) -> alloc::string::String {
        self.to_hex_from(chunk_size, 0)
    }

    #[inline]
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> alloc::string::String {
        use core::fmt::Write;
        let len = self.len() - from;
        #[allow(clippy::integer_division)]
        let mut st = alloc::string::String::with_capacity((len << 1) + 1 + (len / chunk_size));
        for (i, chunk) in self.bytes().skip(from).enumerate() {
            if i % chunk_size == 0 {
                st.push('\n');
            }
            write!(st, "{chunk:02x}").unwrap();
        }
        st
    }
}

impl<'a> nom::InputIter for crate::ByteQueue<'a> {
    type Item = u8;
    type Iter = core::iter::Enumerate<crate::queue::OwnedByteIter<'a>>;
    type IterElem = crate::queue::OwnedByteIter<'a>;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        Iterator::enumerate(self.clone().into_bytes())
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.clone().into_bytes()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.find_byte(predicate)
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

impl nom::InputLength for crate::ByteQueue<'_> {
    #[inline]
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl nom::InputTake for crate::ByteQueue<'_> {
    #[inline]
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    #[inline]
    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl nom::InputTakeAtPosition for crate::ByteQueue<'_> {
    type Item = u8;

    #[inline]
    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        #[allow(clippy::option_if_let_else)]
        if let Some(aa) = self.find_byte(predicate) {
            Ok((self.slice(aa..), self.slice(..aa)))
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
        match self.find_byte(predicate) {
            None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
            Some(0) => Err(nom::Err::Failure(E::from_error_kind(self.clone(), e))),
            Some(aa) => Ok((self.slice(aa..), self.slice(..aa))),
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
        #[allow(clippy::option_if_let_else)]
        if let Some(aa) = self.find_byte(predicate) {
            Ok((self.slice(aa..), self.slice(..aa)))
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
        match self.find_byte(predicate) {
            None => Ok((Self::new(), self.clone())),
            Some(0) => Err(nom::Err::Failure(E::from_error_kind(self.clone(), e))),
            Some(aa) => Ok((self.slice(aa..), self.slice(..aa))),
        }
    }
}

impl nom::Slice<core::ops::Range<usize>> for crate::ByteQueue<'_> {
    #[inline]
    fn slice(&self, range: core::ops::Range<usize>) -> Self {
        self.slice(range)
    }
}

impl nom::Slice<core::ops::RangeTo<usize>> for crate::ByteQueue<'_> {
    #[inline]
    fn slice(&self, range: core::ops::RangeTo<usize>) -> Self {
        self.slice(range)
    }
}

impl nom::Slice<core::ops::RangeFrom<usize>> for crate::ByteQueue<'_> {
    #[inline]
    fn slice(&self, range: core::ops::RangeFrom<usize>) -> Self {
        self.slice(range)
    }
}

impl nom::Slice<core::ops::RangeFull> for crate::ByteQueue<'_> {
    #[inline]
    fn slice(&self, range: core::ops::RangeFull) -> Self {
        self.slice(range)
    }
}
