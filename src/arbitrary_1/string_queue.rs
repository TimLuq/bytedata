use arbitrary_1::{Arbitrary, Error, Unstructured};

use crate::{StringData, StringQueue};

#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary_1")))]
impl<'a> Arbitrary<'a> for StringQueue<'a> {
    #[allow(
        clippy::missing_inline_in_public_items,
        clippy::unwrap_in_result,
        clippy::min_ident_chars
    )]
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        let mut buffer = Self::new();
        let mut maxlen = u.arbitrary_len::<StringData<'a>>()?;
        while !u.is_empty() && maxlen != 0 {
            let data = u.arbitrary::<StringData<'a>>()?;
            maxlen -= 1;
            let dir = u.arbitrary::<bool>().unwrap_or_default();
            if dir {
                buffer.push_front(data);
            } else {
                buffer.push_back(data);
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
        while !u.is_empty() {
            let data = u.arbitrary::<StringData<'a>>()?;
            let dir = u.arbitrary::<bool>().unwrap_or_default();
            if dir {
                buffer.push_front(data);
            } else {
                buffer.push_back(data);
            }
        }
        Ok(buffer)
    }
}
