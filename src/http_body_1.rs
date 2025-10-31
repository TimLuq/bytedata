use core::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use ::http_body_1 as http_body;

use crate::ByteData;

#[cfg_attr(docsrs, doc(cfg(feature = "http-body_1")))]
impl http_body::Body for ByteData<'_> {
    type Data = Self;
    type Error = Infallible;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = &mut *self;
        if this.is_empty() {
            Poll::Ready(None)
        } else if this.len() > 0xFFFF {
            let res = this.sliced(0..0xFFFF);
            this.make_sliced(0xFFFF..);
            Poll::Ready(Some(Ok(http_body::Frame::data(res))))
        } else {
            let res = core::mem::replace(this, const { ByteData::empty() });
            Poll::Ready(Some(Ok(http_body::Frame::data(res))))
        }
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
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "http-body_1")))]
impl http_body::Body for crate::SharedBytes {
    type Data = Self;
    type Error = Infallible;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = &mut *self;
        if this.is_empty() {
            Poll::Ready(None)
        } else if this.len() > 0xFFFF {
            let res = this.sliced(0, 0xFFFF);
            this.make_sliced(0xFFFF, this.len() - 0xFFFF);
            Poll::Ready(Some(Ok(http_body::Frame::data(res))))
        } else {
            let res = core::mem::replace(this, const { Self::empty() });
            Poll::Ready(Some(Ok(http_body::Frame::data(res))))
        }
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

#[cfg(feature = "queue")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "http-body_1")))]
impl<'a> http_body::Body for crate::ByteQueue<'a> {
    type Data = ByteData<'a>;
    type Error = Infallible;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = &mut *self;
        let Some(mut aa) = this.pop_front() else {
            return Poll::Ready(None);
        };
        if aa.len() > 0xFFFF {
            this.push_front(aa.sliced(0xFFFF..));
            aa.make_sliced(0..0xFFFF);
        }
        Poll::Ready(Some(Ok(http_body::Frame::data(aa))))
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
