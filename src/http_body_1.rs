use core::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use ::http_body_1 as http_body;

use crate::ByteData;

#[cfg_attr(docsrs, doc(cfg(feature = "http-body_1")))]
impl<'a> http_body::Body for ByteData<'a> {
    type Data = ByteData<'a>;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let slf = core::ops::DerefMut::deref_mut(&mut self);
        if slf.is_empty() {
            Poll::Ready(None)
        } else if slf.len() > 65535 {
            let res = slf.sliced(0..65535);
            slf.make_sliced(65535..);
            Poll::Ready(Some(Ok(http_body::Frame::data(res))))
        } else {
            let res = core::mem::replace(slf, ByteData::empty());
            Poll::Ready(Some(Ok(http_body::Frame::data(res))))
        }
    }

    fn is_end_stream(&self) -> bool {
        self.is_empty()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(self.len() as u64)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "http-body_1")))]
impl http_body::Body for crate::SharedBytes {
    type Data = crate::SharedBytes;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let slf = core::ops::DerefMut::deref_mut(&mut self);
        if slf.is_empty() {
            Poll::Ready(None)
        } else if slf.len() > 65535 {
            let res = slf.sliced(0, 65535);
            slf.make_sliced(65535, slf.len() - 65535);
            Poll::Ready(Some(Ok(http_body::Frame::data(res))))
        } else {
            let res = core::mem::replace(slf, crate::SharedBytes::empty());
            Poll::Ready(Some(Ok(http_body::Frame::data(res))))
        }
    }

    fn is_end_stream(&self) -> bool {
        self.is_empty()
    }

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

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let slf = core::ops::DerefMut::deref_mut(&mut self);
        if let Some(mut a) = slf.pop_front() {
            if slf.len() > 65535 {
                slf.push_front(a.sliced(65535..));
                a.make_sliced(0..65535);
            }
            Poll::Ready(Some(Ok(http_body::Frame::data(a))))
        } else {
            Poll::Ready(None)
        }
    }

    fn is_end_stream(&self) -> bool {
        self.is_empty()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(self.len() as u64)
    }
}
