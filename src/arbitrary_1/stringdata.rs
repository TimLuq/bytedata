use arbitrary_1::{Arbitrary, Error, Unstructured};

use crate::{ByteData, StringData};

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
impl<'a> Arbitrary<'a> for StringData<'a> {
    #[allow(
        clippy::missing_inline_in_public_items,
        clippy::unwrap_in_result,
        clippy::min_ident_chars
    )]
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        if u.is_empty() {
            return Ok(StringData::empty());
        }
        match u.choose(CHOICES)? {
            &Choice::Borrowed => {
                let sdat = u.arbitrary::<&'a str>()?;
                Ok(StringData::from_borrowed(sdat))
            }
            &Choice::Chunk => {
                let mut buf = [0_u8; 18];
                let mut len = 0;
                let it = u.arbitrary_iter::<char>()?;
                for ch in it {
                    let encoded = ch?.encode_utf8(&mut buf[len..]);
                    let new_len = len + encoded.len();
                    if new_len < 14 {
                        len = new_len;
                        continue;
                    }
                    if new_len == 14 {
                        len = new_len;
                    }
                    break;
                }
                #[allow(clippy::unwrap_used)]
                Ok(StringData::try_from_bytedata(ByteData::from_chunk_slice(&buf[..len])).unwrap())
            }
            #[cfg(feature = "alloc")]
            &Choice::Shared => {
                let mut buffer = crate::SharedBytesBuilder::new();
                let it = u.arbitrary_iter::<char>()?;
                for ch in it {
                    let ch = ch?;
                    buffer.reserve(4);
                    buffer.apply_unfilled(|buf| {
                        // SAFETY: The bytes may be uninitialized, but we are about to fill them.
                        let buf = unsafe {
                            &mut *(buf as *mut [core::mem::MaybeUninit<u8>] as *mut [u8])
                        };
                        let encoded = ch.encode_utf8(buf);
                        ((), encoded.len())
                    });
                }
                #[allow(clippy::unwrap_used)]
                Ok(StringData::try_from_bytedata(ByteData::from_shared(buffer.build())).unwrap())
            }
        }
    }
    #[allow(
        clippy::missing_inline_in_public_items,
        clippy::unwrap_in_result,
        clippy::min_ident_chars
    )]
    fn arbitrary_take_rest(mut u: Unstructured<'a>) -> Result<Self, Error> {
        if u.is_empty() {
            return Ok(StringData::empty());
        }
        match u.choose(CHOICES)? {
            &Choice::Borrowed => {
                let sdat = <&'a str as Arbitrary<'a>>::arbitrary_take_rest(u)?;
                Ok(StringData::from_borrowed(sdat))
            }
            &Choice::Chunk => {
                let mut buf = [0_u8; 18];
                let mut len = 0;
                while let Ok(ch) = <char as Arbitrary>::arbitrary(&mut u) {
                    let encoded = ch.encode_utf8(&mut buf[len..]);
                    let new_len = len + encoded.len();
                    if new_len < 14 {
                        len = new_len;
                        continue;
                    }
                    if new_len == 14 {
                        len = new_len;
                    }
                    break;
                }
                #[allow(clippy::unwrap_used)]
                Ok(StringData::try_from_bytedata(ByteData::from_chunk_slice(&buf[..len])).unwrap())
            }
            #[cfg(feature = "alloc")]
            &Choice::Shared => {
                let mut buffer = crate::SharedBytesBuilder::new();
                while let Ok(ch) = <char as Arbitrary>::arbitrary(&mut u) {
                    buffer.reserve(4);
                    buffer.apply_unfilled(|buf| {
                        // SAFETY: The bytes may be uninitialized, but we are about to fill them.
                        let buf = unsafe {
                            &mut *(buf as *mut [core::mem::MaybeUninit<u8>] as *mut [u8])
                        };
                        let encoded = ch.encode_utf8(buf);
                        ((), encoded.len())
                    });
                }
                #[allow(clippy::unwrap_used)]
                Ok(StringData::try_from_bytedata(ByteData::from_shared(buffer.build())).unwrap())
            }
        }
    }
}
