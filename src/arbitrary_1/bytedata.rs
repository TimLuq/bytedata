use arbitrary_1::{Arbitrary, Error, Unstructured};

use crate::ByteData;

enum Choice {
    Borrowed,
    Chunk,
    #[cfg(feature = "alloc")]
    Shared,
}

static CHOICES: &[Choice] = &[
    Choice::Borrowed,
    Choice::Chunk,
    #[cfg(feature = "alloc")]
    Choice::Shared,
];

#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary_1")))]
impl<'a> Arbitrary<'a> for ByteData<'a> {
    #[allow(clippy::missing_inline_in_public_items, clippy::min_ident_chars)]
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        if u.is_empty() {
            return Ok(ByteData::empty());
        }
        match u.choose(CHOICES)? {
            &Choice::Borrowed => {
                let dat_len = u.arbitrary_len::<u8>()?;
                let bdat = u.bytes(dat_len)?;
                Ok(ByteData::from_borrowed(bdat))
            }
            &Choice::Chunk => {
                let dat_len = u.arbitrary_len::<u8>()?.min(14);
                let bdat = u.bytes(dat_len)?;
                Ok(ByteData::from_chunk_slice(bdat))
            }
            #[cfg(feature = "alloc")]
            &Choice::Shared => {
                let buffer = <crate::SharedBytes as Arbitrary<'a>>::arbitrary(u)?;
                Ok(ByteData::from_shared(buffer))
            }
        }
    }
    #[allow(clippy::missing_inline_in_public_items, clippy::min_ident_chars)]
    fn arbitrary_take_rest(mut u: Unstructured<'a>) -> Result<Self, Error> {
        if u.is_empty() {
            return Ok(ByteData::empty());
        }
        match u.choose(CHOICES)? {
            &Choice::Borrowed => {
                let bdat = u.take_rest();
                Ok(ByteData::from_borrowed(bdat))
            }
            &Choice::Chunk => {
                let dat_len = u.len().min(14);
                let bdat = u.bytes(dat_len)?;
                Ok(ByteData::from_chunk_slice(bdat))
            }
            #[cfg(feature = "alloc")]
            &Choice::Shared => {
                let buffer = <crate::SharedBytes as Arbitrary<'a>>::arbitrary_take_rest(u)?;
                Ok(ByteData::from_shared(buffer))
            }
        }
    }
}
