/// An iterator over chunks of a `ByteQueue` separated by a byte sequence.
pub struct SplitOn<'a, 'b> {
    queue: &'b super::ByteQueue<'a>,
    needle: &'b [u8],
    start_chunk: usize,
    start_offset: usize,
    start_byte: usize,
    max: usize,
    done: bool,
}

impl<'a, 'b> SplitOn<'a, 'b> {
    pub(super) const fn new(queue: &'b super::ByteQueue<'a>, needle: &'b [u8], max: usize) -> Self {
        let done = queue.is_empty();
        Self {
            queue,
            needle,
            start_chunk: 0,
            start_offset: 0,
            start_byte: 0,
            max,
            done,
        }
    }
}

impl<'a, 'b> Iterator for SplitOn<'a, 'b> {
    type Item = super::ByteQueue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if self.start_byte == self.queue.len() {
            self.done = true;
            return Some(super::ByteQueue::new());
        }
        if self.max == 1 {
            self.done = true;
            return Some(self.queue.slice(self.start_byte..));
        }
        match self.queue.find_slice_pos(
            self.needle,
            self.start_chunk,
            self.start_offset,
            self.start_byte,
        ) {
            Some((chunk, offset, byte)) => {
                let mut chunks = self.queue.chunks().skip(self.start_chunk);
                let new_q = if self.start_chunk == chunk {
                    super::ByteQueue::with_item(
                        chunks.next().unwrap().sliced(self.start_offset..offset),
                    )
                } else {
                    let mut new_q = super::ByteQueue::new();
                    new_q.push_back(chunks.next().unwrap().sliced(self.start_offset..));
                    for chunk in chunks.take(chunk - self.start_chunk - 1) {
                        new_q.push_back(chunk.clone());
                    }
                    if offset != 0 {
                        chunks = self.queue.chunks().skip(chunk);
                        new_q.push_back(chunks.next().unwrap().sliced(..offset));
                    }
                    new_q
                };
                self.start_chunk = chunk;
                self.start_offset = offset;
                self.start_byte = byte;
                if self.max != 0 {
                    self.max -= 1;
                }
                Some(new_q)
            }
            None => {
                self.done = true;
                Some(self.queue.slice(self.start_byte..))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            let max = if self.max == 0 { None } else { Some(self.max) };
            (1, max)
        }
    }
}

/// An iterator over chunks of a `StringQueue` separated by a str sequence.
#[repr(transparent)]
pub struct SplitOnStr<'a, 'b> {
    inner: SplitOn<'a, 'b>,
}

impl<'a, 'b> SplitOnStr<'a, 'b> {
    pub(super) const fn new(
        queue: &'b super::StringQueue<'a>,
        needle: &'b str,
        max: usize,
    ) -> Self {
        Self {
            inner: SplitOn::new(queue.as_bytequeue(), needle.as_bytes(), max),
        }
    }
}

impl<'a, 'b> Iterator for SplitOnStr<'a, 'b> {
    type Item = super::StringQueue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.inner.next()?;
        Some(unsafe { super::StringQueue::from_bytequeue(n) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
