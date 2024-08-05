/// A draining iterator over the elements of a `ByteQueue`.
#[allow(missing_debug_implementations)]
pub struct DrainBytes<'a, 'b> {
    ptr: *mut super::ByteQueue<'a>,
    queue: Option<super::ByteIter<'a, 'b>>,
    start: usize,
    end: usize,
}

impl<'a, 'b> DrainBytes<'a, 'b> {
    pub(super) fn new(queue: &'b mut super::ByteQueue<'a>, start: usize, end: usize) -> Self {
        let ptr = queue as *mut super::ByteQueue<'a>;
        let queue = Some(queue.bytes().skip(start).take(end - start));
        Self {
            ptr,
            queue,
            start,
            end,
        }
    }
}

#[allow(single_use_lifetimes)]
// SAFETY: `DrainBytes` is `Send` and `Sync` because points to data valid for the lifetimes
unsafe impl<'a, 'b> Send for DrainBytes<'a, 'b> {}
#[allow(single_use_lifetimes)]
// SAFETY: `DrainBytes` is `Send` and `Sync` because points to data valid for the lifetimes
unsafe impl<'a, 'b> Sync for DrainBytes<'a, 'b> {}

#[allow(single_use_lifetimes)]
impl<'a, 'b> Drop for DrainBytes<'a, 'b> {
    #[inline]
    fn drop(&mut self) {
        if self.start == self.end {
            return;
        }
        _ = self.queue.take();
        // SAFETY: `ptr` is a valid pointer to a `ByteQueue`.
        let queue = unsafe { &mut *self.ptr };
        do_drain(queue, self.start, self.end);
    }
}

#[allow(single_use_lifetimes)]
impl<'a, 'b> Iterator for DrainBytes<'a, 'b> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.as_mut().and_then(super::ByteIter::next)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.queue
            .as_ref()
            .map_or((0, Some(0)), super::ByteIter::size_hint)
    }

    #[inline]
    fn count(mut self) -> usize {
        self.queue.take().map_or(0, super::ByteIter::count)
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.queue.take().and_then(super::ByteIter::last)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.queue.as_mut().and_then(|qu| qu.nth(n))
    }
}

#[allow(single_use_lifetimes)]
impl<'a, 'b> ExactSizeIterator for DrainBytes<'a, 'b> {
    #[inline]
    fn len(&self) -> usize {
        self.queue.as_ref().map_or(0, super::ByteIter::len)
    }
}

/// A draining iterator over the characters of a `StringQueue`.
#[allow(missing_debug_implementations)]
pub struct DrainChars<'a, 'b> {
    ptr: *mut super::StringQueue<'a>,
    queue: Option<super::CharIter<'a, 'b>>,
    start: usize,
    end: usize,
}

impl<'a, 'b> DrainChars<'a, 'b> {
    pub(super) unsafe fn new(
        queue: &'b mut super::StringQueue<'a>,
        start: usize,
        end: usize,
    ) -> Self {
        let ptr = queue as *mut super::StringQueue<'a>;
        let queue = queue.bytes().skip(start).take(end - start);
        let queue = super::CharIter::from_byte_iter(queue);
        let queue = Some(queue);
        Self {
            ptr,
            queue,
            start,
            end,
        }
    }
}

#[allow(single_use_lifetimes)]
// SAFETY: `DrainChars` is `Send` and `Sync` because points to data valid for the lifetimes
unsafe impl<'a, 'b> Send for DrainChars<'a, 'b> {}
#[allow(single_use_lifetimes)]
// SAFETY: `DrainChars` is `Send` and `Sync` because points to data valid for the lifetimes
unsafe impl<'a, 'b> Sync for DrainChars<'a, 'b> {}

#[allow(single_use_lifetimes)]
impl<'a, 'b> Drop for DrainChars<'a, 'b> {
    #[inline]
    fn drop(&mut self) {
        if self.start == self.end {
            return;
        }
        _ = self.queue.take();
        // SAFETY: `ptr` is a valid pointer to a `StringQueue`.
        let queue = unsafe { &mut *self.ptr };
        do_drain(queue.as_bytequeue_mut(), self.start, self.end);
    }
}

#[allow(single_use_lifetimes)]
impl<'a, 'b> Iterator for DrainChars<'a, 'b> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.as_mut().and_then(super::CharIter::next)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.queue
            .as_ref()
            .map_or((0, Some(0)), super::CharIter::size_hint)
    }

    #[inline]
    fn count(mut self) -> usize {
        self.queue.take().map_or(0, super::CharIter::count)
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.queue.take().and_then(super::CharIter::last)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.queue.as_mut().and_then(|qu| qu.nth(n))
    }
}

fn do_drain(queue: &mut super::ByteQueue<'_>, start: usize, mut end: usize) {
    if start == 0 {
        while let Some(mut aa) = queue.pop_front() {
            if aa.len() > end {
                aa.make_sliced(end..);
                queue.push_front(aa);
                return;
            }
            end -= aa.len();
            if end != 0 {
                continue;
            }
            return;
        }
        return;
    }
    if end == queue.len() {
        while let Some(mut av) = queue.pop_back() {
            if queue.len() < start {
                av.make_sliced(..(start - queue.len()));
                queue.push_back(av);
                return;
            }
            if start != queue.len() {
                continue;
            }
            return;
        }
        return;
    }
    let mut queue2 = queue.split_off(start);
    let mut remove = end - start;
    while let Some(mut av) = queue2.pop_front() {
        if av.len() > remove {
            av.make_sliced(remove..);
            queue2.push_front(av);
            break;
        }
        remove -= av.len();
        if remove != 0 {
            continue;
        }
        break;
    }
    queue.append(queue2);
}
