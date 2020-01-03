#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Endianness {
    Big,
    Little,
    Native,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum MaxLen {
    U8,
    U16,
    U32,
    Usize,
}

pub struct Config {
    max_len: MaxLen,
    endianness: Endianness,
    size_hint: Option<usize>,
}

impl Config {
    pub fn with_max_len(mut self, max_len: MaxLen) -> Self {
        self.max_len = max_len;
        self
    }

    pub fn with_endianness(mut self, endianness: Endianness) -> Self {
        self.endianness = endianness;
        self
    }

    pub fn with_size_hint(mut self, size_hint: usize) -> Self {
        self.size_hint = Some(size_hint);
        self
    }

    pub fn build<T: AsRef<[U]>, U: AsRef<[u8]>>(
        self,
        parts: T,
    ) -> Result<Buffer, std::num::TryFromIntError> {
        use std::convert::TryFrom;

        let parts = parts.as_ref();

        let size_hint = self.size_hint.unwrap_or_else(|| {
            let count = parts.len();
            let bytes_count = parts.into_iter().fold(0usize, |acc, part| acc + part.as_ref().len());
            let width = match self.max_len {
                MaxLen::U8 => std::mem::size_of::<u8>(),
                MaxLen::U16 => std::mem::size_of::<u16>(),
                MaxLen::U32 => std::mem::size_of::<u32>(),
                MaxLen::Usize => std::mem::size_of::<usize>(),
            };

            (count * width) + bytes_count + 2
        });

        let mut buffer = Vec::with_capacity(size_hint);

        buffer.push(self.endianness as u8);
        buffer.push(self.max_len as u8);

        for part in parts {
            let part = part.as_ref();
            let len = part.len();

            match (self.endianness, self.max_len) {
                (Endianness::Big, MaxLen::U8) => {
                    buffer.extend_from_slice(&u8::try_from(len)?.to_be_bytes())
                }
                (Endianness::Big, MaxLen::U16) => {
                    buffer.extend_from_slice(&u16::try_from(len)?.to_be_bytes())
                }
                (Endianness::Big, MaxLen::U32) => {
                    buffer.extend_from_slice(&u32::try_from(len)?.to_be_bytes())
                }
                (Endianness::Big, MaxLen::Usize) => {
                    buffer.extend_from_slice(&usize::try_from(len)?.to_be_bytes())
                }
                (Endianness::Little, MaxLen::U8) => {
                    buffer.extend_from_slice(&u8::try_from(len)?.to_le_bytes())
                }
                (Endianness::Little, MaxLen::U16) => {
                    buffer.extend_from_slice(&u16::try_from(len)?.to_le_bytes())
                }
                (Endianness::Little, MaxLen::U32) => {
                    buffer.extend_from_slice(&u32::try_from(len)?.to_le_bytes())
                }
                (Endianness::Little, MaxLen::Usize) => {
                    buffer.extend_from_slice(&usize::try_from(len)?.to_le_bytes())
                }
                (Endianness::Native, MaxLen::U8) => {
                    buffer.extend_from_slice(&u8::try_from(len)?.to_ne_bytes())
                }
                (Endianness::Native, MaxLen::U16) => {
                    buffer.extend_from_slice(&u16::try_from(len)?.to_ne_bytes())
                }
                (Endianness::Native, MaxLen::U32) => {
                    buffer.extend_from_slice(&u32::try_from(len)?.to_ne_bytes())
                }
                (Endianness::Native, MaxLen::Usize) => {
                    buffer.extend_from_slice(&usize::try_from(len)?.to_ne_bytes())
                }
            }

            buffer.extend_from_slice(part);
        }

        buffer.shrink_to_fit();

        Ok(Buffer(buffer))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config { max_len: MaxLen::Usize, endianness: Endianness::Big, size_hint: None }
    }
}

#[derive(Debug, Clone)]
pub struct Buffer(Vec<u8>);

pub struct BufferIterator<'a> {
    buffer: &'a [u8],
    offset: usize,
    endianness: Endianness,
    max_len: MaxLen,
}

impl<'a> IntoIterator for &'a Buffer {
    type Item = &'a [u8];
    type IntoIter = BufferIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let endianness = unsafe { std::mem::transmute::<_, Endianness>(self.0[0]) };
        let max_len = unsafe { std::mem::transmute::<_, MaxLen>(self.0[1]) };
        BufferIterator { buffer: &self.0[2..], offset: 0, endianness, max_len }
    }
}

impl<'a> Iterator for BufferIterator<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        use std::convert::TryInto;

        if self.buffer[self.offset..].is_empty() {
            return None;
        }

        let bytes_start;

        match (self.endianness, self.max_len) {
            (Endianness::Big, MaxLen::U8) => {
                bytes_start = self.offset + std::mem::size_of::<u8>();
                let len = u8::from_be_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `u8`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Big, MaxLen::U16) => {
                bytes_start = self.offset + std::mem::size_of::<u16>();
                let len = u16::from_be_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `u16`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Big, MaxLen::U32) => {
                bytes_start = self.offset + std::mem::size_of::<u32>();
                let len = u32::from_be_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `u32`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Big, MaxLen::Usize) => {
                bytes_start = self.offset + std::mem::size_of::<usize>();
                let len = usize::from_be_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `usize`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Little, MaxLen::U8) => {
                bytes_start = self.offset + std::mem::size_of::<u8>();
                let len = u8::from_le_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `u8`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Little, MaxLen::U16) => {
                bytes_start = self.offset + std::mem::size_of::<u16>();
                let len = u16::from_le_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `u16`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Little, MaxLen::U32) => {
                bytes_start = self.offset + std::mem::size_of::<u32>();
                let len = u32::from_le_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `u32`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Little, MaxLen::Usize) => {
                bytes_start = self.offset + std::mem::size_of::<usize>();
                let len = usize::from_le_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `usize`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Native, MaxLen::U8) => {
                bytes_start = self.offset + std::mem::size_of::<u8>();
                let len = u8::from_ne_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `u8`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Native, MaxLen::U16) => {
                bytes_start = self.offset + std::mem::size_of::<u16>();
                let len = u16::from_ne_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `u16`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Native, MaxLen::U32) => {
                bytes_start = self.offset + std::mem::size_of::<u32>();
                let len = u32::from_ne_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `u32`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
            (Endianness::Native, MaxLen::Usize) => {
                bytes_start = self.offset + std::mem::size_of::<usize>();
                let len = usize::from_ne_bytes(
                    self.buffer[self.offset..bytes_start].try_into().expect("Must be `usize`"),
                ) as usize;
                self.offset = bytes_start + len;
            }
        };

        Some(&self.buffer[bytes_start..self.offset])
    }
}
