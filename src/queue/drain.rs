/// A draining iterator over the elements of a `ByteQueue`.
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

unsafe impl<'a, 'b> Send for DrainBytes<'a, 'b> {}
unsafe impl<'a, 'b> Sync for DrainBytes<'a, 'b> {}

impl<'a, 'b> Drop for DrainBytes<'a, 'b> {
    fn drop(&mut self) {
        if self.start == self.end {
            return;
        }
        core::mem::drop(self.queue.take());
        let queue = unsafe { &mut *self.ptr };
        do_drain(queue, self.start, self.end);
    }
}

impl<'a, 'b> Iterator for DrainBytes<'a, 'b> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.as_mut().and_then(super::ByteIter::next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.queue
            .as_ref()
            .map_or((0, Some(0)), super::ByteIter::size_hint)
    }

    fn count(mut self) -> usize {
        self.queue.take().map_or(0, super::ByteIter::count)
    }

    fn last(mut self) -> Option<Self::Item> {
        self.queue.take().and_then(super::ByteIter::last)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.queue.as_mut().and_then(|q| q.nth(n))
    }
}

impl<'a, 'b> ExactSizeIterator for DrainBytes<'a, 'b> {
    fn len(&self) -> usize {
        self.queue.as_ref().map_or(0, super::ByteIter::len)
    }
}

/// A draining iterator over the characters of a `StringQueue`.
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

unsafe impl<'a, 'b> Send for DrainChars<'a, 'b> {}
unsafe impl<'a, 'b> Sync for DrainChars<'a, 'b> {}

impl<'a, 'b> Drop for DrainChars<'a, 'b> {
    fn drop(&mut self) {
        if self.start == self.end {
            return;
        }
        core::mem::drop(self.queue.take());
        let queue = unsafe { &mut *self.ptr };
        do_drain(queue.as_bytequeue_mut(), self.start, self.end);
    }
}

impl<'a, 'b> Iterator for DrainChars<'a, 'b> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.as_mut().and_then(super::CharIter::next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.queue
            .as_ref()
            .map_or((0, Some(0)), super::CharIter::size_hint)
    }

    fn count(mut self) -> usize {
        self.queue.take().map_or(0, super::CharIter::count)
    }

    fn last(mut self) -> Option<Self::Item> {
        self.queue.take().and_then(super::CharIter::last)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.queue.as_mut().and_then(|q| q.nth(n))
    }
}

fn do_drain(queue: &mut super::ByteQueue, start: usize, mut end: usize) {
    if start == 0 {
        while let Some(mut a) = queue.pop_front() {
            if a.len() > end {
                a.make_sliced(end..);
                queue.push_front(a);
                return;
            }
            end -= a.len();
            if end != 0 {
                continue;
            }
            return;
        }
        return;
    }
    if end == queue.len() {
        while let Some(mut a) = queue.pop_back() {
            if queue.len() < start {
                a.make_sliced(..(start - queue.len()));
                queue.push_back(a);
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
    while let Some(mut a) = queue2.pop_front() {
        if a.len() > remove {
            a.make_sliced(remove..);
            queue2.push_front(a);
            break;
        }
        remove -= a.len();
        if remove != 0 {
            continue;
        }
        break;
    }
    queue.append(queue2);
}
