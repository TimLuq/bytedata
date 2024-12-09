use arbitrary_1::{Arbitrary, Error, Unstructured};

use crate::{SharedBytes, SharedBytesBuilder};

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary_1")))]
impl<'a> Arbitrary<'a> for SharedBytesBuilder {
    #[allow(
        clippy::missing_inline_in_public_items,
        clippy::unwrap_in_result,
        clippy::min_ident_chars
    )]
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        let mut buffer = Self::new();
        if u.is_empty() {
            return Ok(buffer);
        }
        let mut maxlen = u.arbitrary_len::<u8>()?;
        while !u.is_empty() {
            buffer.reserve(4);
            let res = buffer.apply_unfilled(|buf| {
                // SAFETY: The bytes may be uninitialized, but we are about to fill them.
                let buf = unsafe { &mut *(buf as *mut [core::mem::MaybeUninit<u8>] as *mut [u8]) };
                let len = maxlen.min(buf.len());
                if len == 0 {
                    return (Ok(false), 0);
                }
                maxlen -= len;
                match u.fill_buffer(&mut buf[..len]) {
                    Ok(()) => (Ok(true), len),
                    Err(e) => (Err(e), 0),
                }
            })?;
            if !res {
                break;
            }
        }
        Ok(buffer)
    }
    #[allow(
        clippy::missing_inline_in_public_items,
        clippy::unwrap_in_result,
        clippy::min_ident_chars
    )]
    fn arbitrary_take_rest(mut u: Unstructured<'a>) -> Result<Self, Error> {
        let mut buffer = Self::new();
        if u.is_empty() {
            return Ok(buffer);
        }
        let mut maxlen = u.len();
        while !u.is_empty() {
            buffer.reserve(4);
            let res = buffer.apply_unfilled(|buf| {
                // SAFETY: The bytes may be uninitialized, but we are about to fill them.
                let buf = unsafe { &mut *(buf as *mut [core::mem::MaybeUninit<u8>] as *mut [u8]) };
                let len = maxlen.min(buf.len());
                if len == 0 {
                    return (Ok(false), 0);
                }
                maxlen -= len;
                match u.fill_buffer(&mut buf[..len]) {
                    Ok(()) => (Ok(true), len),
                    Err(e) => (Err(e), 0),
                }
            })?;
            if !res {
                break;
            }
        }
        Ok(buffer)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary_1")))]
impl<'a> Arbitrary<'a> for SharedBytes {
    #[allow(
        clippy::missing_inline_in_public_items,
        clippy::unwrap_in_result,
        clippy::min_ident_chars
    )]
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        let buffer = SharedBytesBuilder::arbitrary(u)?;
        Ok(buffer.build())
    }
}
