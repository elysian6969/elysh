use std::slice::SliceIndex;
use std::{fmt, ops};

#[derive(Clone, Eq, PartialEq)]
pub struct Buffer {
    buffer: String,
}

impl Buffer {
    pub const fn new() -> Self {
        let buffer = String::new();

        Self { buffer }
    }

    pub fn as_str(&self) -> &str {
        self.buffer.as_str()
    }

    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.buffer.as_bytes().get(index)
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn ends_with_space(&self) -> bool {
        self.ends_with(' ')
    }

    pub fn push(&mut self, character: char) {
        self.buffer.push(character);
    }

    pub fn pop(&mut self) {
        self.buffer.pop();
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer.into_bytes()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.buffer, fmt)
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.buffer, fmt)
    }
}

impl ops::Deref for Buffer {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Buffer {
    fn from(buffer: String) -> Self {
        Self { buffer }
    }
}

impl Into<String> for Buffer {
    fn into(self) -> String {
        self.buffer
    }
}
