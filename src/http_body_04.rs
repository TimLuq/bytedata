use core::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use ::http_body_04 as http_body;

use crate::ByteData;

impl http_body::Body for ByteData<'_> {
    type Data = Self;
    type Error = Infallible;

    #[inline]
    fn poll_data(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = core::ops::DerefMut::deref_mut(&mut self);
        if this.is_empty() {
            Poll::Ready(None)
        } else if this.len() > 0xFFFF {
            let res = this.sliced(0..0xFFFF);
            this.make_sliced(0xFFFF..);
            Poll::Ready(Some(Ok(res)))
        } else {
            let res = core::mem::replace(this, ByteData::empty());
            Poll::Ready(Some(Ok(res)))
        }
    }

    #[inline]
    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http_02::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(self.len() as u64)
    }
}

#[cfg(feature = "alloc")]
impl http_body::Body for crate::SharedBytes {
    type Data = Self;
    type Error = Infallible;

    #[inline]
    fn poll_data(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = core::ops::DerefMut::deref_mut(&mut self);
        if this.is_empty() {
            Poll::Ready(None)
        } else if this.len() > 0xFFFF {
            let res = this.sliced(0, 0xFFFF);
            this.make_sliced(0xFFFF, this.len() - 0xFFFF);
            Poll::Ready(Some(Ok(res)))
        } else {
            let res = core::mem::replace(this, Self::empty());
            Poll::Ready(Some(Ok(res)))
        }
    }

    #[inline]
    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http_02::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(self.len() as u64)
    }
}
