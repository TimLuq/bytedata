/// An iterator over chunks of a `ByteQueue` separated by a byte sequence.
#[allow(missing_debug_implementations)]
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

impl<'a> Iterator for SplitOn<'a, '_> {
    type Item = super::ByteQueue<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        fn inner<'a>(this: &mut SplitOn<'a, '_>) -> super::ByteQueue<'a> {
            let val = this.queue.find_slice_pos(
                this.needle,
                this.start_chunk,
                this.start_offset,
                this.start_byte,
            );
            if let Some((chunk, offset, byte)) = val {
                let mut chunks = this.queue.chunks().skip(this.start_chunk);
                let new_q = if this.start_chunk == chunk {
                    #[allow(clippy::unwrap_used)]
                    let chunk_next = chunks.next().unwrap();
                    super::ByteQueue::with_item(chunk_next.sliced(this.start_offset..offset))
                } else {
                    let mut new_q = super::ByteQueue::new();
                    #[allow(clippy::unwrap_used)]
                    let chunk_next = chunks.next().unwrap();
                    new_q.push_back(chunk_next.sliced(this.start_offset..));
                    for chunkdata in chunks.take(chunk - this.start_chunk - 1) {
                        new_q.push_back(chunkdata.clone());
                    }
                    if offset != 0 {
                        chunks = this.queue.chunks().skip(chunk);
                        #[allow(clippy::unwrap_used)]
                        let chunk_nexter = chunks.next().unwrap();
                        new_q.push_back(chunk_nexter.sliced(..offset));
                    }
                    new_q
                };
                this.start_chunk = chunk;
                this.start_offset = offset;
                this.start_byte = byte;
                if this.max != 0 {
                    this.max -= 1;
                }
                new_q
            } else {
                this.done = true;
                this.queue.slice(this.start_byte..)
            }
        }

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
        Some(inner(self))
    }

    #[inline]
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
#[allow(missing_debug_implementations)]
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

impl<'a> Iterator for SplitOnStr<'a, '_> {
    type Item = super::StringQueue<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let n = self.inner.next()?;
        // SAFETY: as the needle is a valid UTF-8 sequence, the split chunks are also valid UTF-8
        Some(unsafe { super::StringQueue::from_bytequeue(n) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
