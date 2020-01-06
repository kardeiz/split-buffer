#[derive(Clone)]
pub struct Buffer(Vec<u8>);

impl Buffer {
    /// Build a buffer from parts that resolve to a slice of byte slices.
    ///
    /// A size hint will be calculated from the parts to preallocate the buffer.
    pub fn build<T: AsRef<[U]>, U: AsRef<[u8]>>(parts: T) -> Self {
        let parts = parts.as_ref();
        let parts_len = parts.len();
        let bytes_total = parts.into_iter().fold(0usize, |acc, part| acc + part.as_ref().len());
        Self::build_with_size_hint(parts, (parts_len * std::mem::size_of::<usize>()) + bytes_total)
    }

    /// Build a buffer from parts that resolve to a slice of byte slices.
    pub fn build_with_size_hint<T: AsRef<[U]>, U: AsRef<[u8]>>(parts: T, size_hint: usize) -> Self {
        let parts = parts.as_ref();

        let mut buffer = Vec::with_capacity(size_hint);

        for part in parts {
            let part = part.as_ref();
            let part_len = part.len();
            buffer.extend_from_slice(&part_len.to_le_bytes());
            buffer.extend_from_slice(part);
        }

        buffer.shrink_to_fit();

        Buffer(buffer)
    }

    /// Get the inner `Vec<u8>`
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

/// Iterator over parts of a `Buffer`
pub struct BufferIterator<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> IntoIterator for &'a Buffer {
    type Item = &'a [u8];
    type IntoIter = BufferIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BufferIterator { buffer: &self.0, offset: 0 }
    }
}

impl<'a> Iterator for BufferIterator<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        use std::convert::TryInto;

        if self.buffer[self.offset..].is_empty() {
            return None;
        }

        let bytes_start = self.offset + std::mem::size_of::<usize>();
        let len = usize::from_le_bytes(
            self.buffer[self.offset..bytes_start].try_into().expect("Must be `usize`"),
        ) as usize;
        self.offset = bytes_start + len;

        Some(&self.buffer[bytes_start..self.offset])
    }
}
