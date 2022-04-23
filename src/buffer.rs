use std::slice::SliceIndex;
use std::str::SplitWhitespace;
use std::{fmt, ops};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Buffer {
    buffer: String,
    column: usize,
}

impl Buffer {
    pub const fn new() -> Self {
        let buffer = String::new();
        let column = 0;

        Self { buffer, column }
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
        self.buffer.insert(self.column, character);
        self.move_right(1);
    }

    pub fn pop(&mut self) {
        if self.is_empty() {
            return;
        }

        if self.column == 0 {
            return
        }

        if self.column == self.len() {
            self.buffer.pop();
        } else {
            self.buffer
                .remove(self.column.saturating_sub(1).min(self.len().saturating_sub(1)));
        }

        self.move_left(1);
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer.into_bytes()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.move_to_start();
    }

    pub fn split_program<'a>(&'a self) -> Option<(&'a str, SplitWhitespace<'a>)> {
        let mut args = self.buffer.split_whitespace();
        let program = args.next()?;

        Some((program, args))
    }

    pub fn move_to_start(&mut self) {
        self.column = 0;
    }

    pub fn move_to_end(&mut self) {
        self.column = self.len();
    }

    pub fn move_left(&mut self, amount: usize) {
        self.column = self.column.saturating_sub(amount);
    }

    pub fn move_right(&mut self, amount: usize) {
        self.column = self.column.saturating_add(amount).min(self.len());
    }

    pub fn column_shift(&self) -> usize {
        self.len().saturating_sub(self.column)
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
        Self { buffer, column: 0 }
    }
}

impl Into<String> for Buffer {
    fn into(self) -> String {
        self.buffer
    }
}
