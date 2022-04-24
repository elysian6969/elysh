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

    pub fn insert_at_cursor(&mut self, character: char) {
        self.buffer.insert(self.column, character);
        self.move_right(1);
    }

    pub fn insert_str_at_cursor(&mut self, string: &str) {
        self.buffer.insert_str(self.column, string);
        self.move_right(string.len());
    }

    pub fn remove_at_cursor(&mut self) {
        if self.is_empty() {
            return;
        }

        if self.column == 0 {
            return;
        }

        if self.is_at_end() {
            self.buffer.pop();
        } else {
            self.buffer.remove(
                self.column
                    .saturating_sub(1)
                    .min(self.len().saturating_sub(1)),
            );
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

    pub fn is_at_start(&self) -> bool {
        self.column == 0
    }

    pub fn is_at_end(&self) -> bool {
        self.column == self.len()
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

    pub fn split_at_cursor(&mut self) -> (&str, &str) {
        let left = &self.buffer[..self.column.saturating_sub(1)];
        let right = &self.buffer[self.column..];

        (left, right)
    }

    pub fn move_to_whitespace_left(&mut self) {
        if let Some(index) = self.split_at_cursor().0.rfind(' ') {
            self.column = index + 1;
        } else {
            self.column = 0;
        }
    }

    pub fn move_to_whitespace_right(&mut self) {
        if let Some(index) = self.split_at_cursor().1.find(' ') {
            self.column += index + 1;
        } else {
            self.column = self.len();
        }
    }

    pub fn remove_word_at_cursor(&mut self) {
        self.move_to_whitespace_left();
        self.buffer.truncate(self.column);
    }

    pub fn remove_right_of_cursor(&mut self) {
        self.buffer.truncate(self.column);
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
