use core::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use ::http_body_04 as http_body;

use crate::ByteData;

impl<'a> http_body::Body for ByteData<'a> {
    type Data = ByteData<'a>;
    type Error = Infallible;

    fn poll_data(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let slf = core::ops::DerefMut::deref_mut(&mut self);
        if slf.is_empty() {
            Poll::Ready(None)
        } else if slf.len() > 65535 {
            let res = slf.sliced(0..65535);
            slf.make_sliced(65535..);
            Poll::Ready(Some(Ok(res)))
        } else {
            let res = core::mem::replace(slf, ByteData::empty());
            Poll::Ready(Some(Ok(res)))
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http_02::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        self.is_empty()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(self.len() as u64)
    }
}

#[cfg(feature = "alloc")]
impl http_body::Body for crate::SharedBytes {
    type Data = crate::SharedBytes;
    type Error = Infallible;

    fn poll_data(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let slf = core::ops::DerefMut::deref_mut(&mut self);
        if slf.is_empty() {
            Poll::Ready(None)
        } else if slf.len() > 65535 {
            let res = slf.sliced(0, 65535);
            slf.make_sliced(65535, slf.len() - 65535);
            Poll::Ready(Some(Ok(res)))
        } else {
            let res = core::mem::replace(slf, crate::SharedBytes::empty());
            Poll::Ready(Some(Ok(res)))
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http_02::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        self.is_empty()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(self.len() as u64)
    }
}
