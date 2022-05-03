use elysh_syntax::{Args, Chars, Command, CommandError};
use std::{fmt, ops};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Edit {
    buffer: String,
    cursor: usize,
    line: usize,
}

impl Edit {
    /// Creates a new empty `Edit`.
    #[inline]
    pub const fn new() -> Self {
        let buffer = String::new();
        let cursor = 0;
        let line = 0;

        Self {
            buffer,
            cursor,
            line,
        }
    }

    /// Returns a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.buffer.as_str()
    }

    /// Returns the length.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Is it empty!
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// If cursor is at the start.
    #[inline]
    pub fn is_at_start(&self) -> bool {
        self.cursor == 0
    }

    /// If cursor is at the end.
    #[inline]
    pub fn is_at_end(&self) -> bool {
        self.cursor == self.len()
    }

    /// Move cursor to start.
    #[inline]
    pub fn to_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end.
    #[inline]
    pub fn to_end(&mut self) {
        self.cursor = self.len();
    }

    /// Move left by `n`.
    #[inline]
    pub fn prev(&mut self, n: usize) {
        self.cursor = self.cursor.saturating_sub(n);
    }

    /// Move right by n.
    #[inline]
    pub fn next(&mut self, n: usize) {
        self.cursor = (self.cursor + n).min(self.len());
    }

    /// If the inner string ends with a space.
    #[inline]
    pub fn ends_with_space(&self) -> bool {
        self.ends_with(char::is_whitespace)
    }

    /// Insert a character at the current cursor position.
    #[inline]
    pub fn insert(&mut self, character: char) {
        match character {
            '`' | '"' | '\'' => {
                let (start, end) = self.split();
                let start_args = Args::new(start);
                let mut end_chars = end.chars();

                if let (Some(start), Some(end)) = (start_args.last(), end_chars.next()) {
                    if let Some(quote) = start.quote() {
                        if quote.as_char() == character && quote.as_char() == end {
                            self.next(1);

                            return;
                        }
                    }
                } else {
                    self.buffer.insert(self.cursor, character);
                }
            }
            character if character.is_whitespace() => {
                let (start, end) = self.split();
                let start_args = Args::new(start);
                let mut end_chars = end.chars();

                let start_ends_with_space = start_args
                    .last()
                    .map(|arg| arg.is_whitespace())
                    .unwrap_or(false);

                let end_starts_with_space = end_chars
                    .next()
                    .map(|arg| arg.is_whitespace())
                    .unwrap_or(false);

                if start_ends_with_space || end_starts_with_space {
                    return;
                } else {
                    self.buffer.insert(self.cursor, character);
                }
            }
            character => {
                self.buffer.insert(self.cursor, character);
            }
        }

        self.next(1);
    }

    /// Insert a string at the current cursor position.
    #[inline]
    pub fn insert_str(&mut self, string: &str) {
        for character in string.chars() {
            self.insert(character);
        }
    }

    /// Reduce code duplication in `remove` and prevent panics with the at_end check.
    #[inline]
    fn remove_internal(&mut self) {
        if self.is_empty() || self.is_at_end() {
            self.buffer.pop();
        } else {
            let start = self.split().0;
            let mut chars = Chars::new(start);

            if let Some(character) = chars.next_back() {
                let position = self.cursor - character.len_utf8();

                self.buffer.remove(position);
            }
        }

        self.prev(1);
    }

    /// Remove a character at the current cursor position.
    #[inline]
    pub fn remove(&mut self) {
        let (start, end) = self.split();

        if let (Some(start), Some(end)) = (start.chars().next_back(), end.chars().next()) {
            if start == end && matches!(start, '`' | '"' | '\'') {
                self.next(1);
                self.remove_internal();
            }
        }

        self.remove_internal();
    }

    /// Remove a word at the cursor position.
    #[inline]
    pub fn remove_word(&mut self, chars: &[char]) {
        self.prev_word(chars);
        self.buffer.truncate(self.cursor);
    }

    /// Remove everything right of the cursor position.
    #[inline]
    pub fn remove_end(&mut self) {
        self.buffer.truncate(self.cursor);
    }

    /// Remove everything.
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.to_start();
    }

    /// Remove everything from the cursor position.
    #[inline]
    pub fn clear_end(&mut self) {
        self.buffer.truncate(self.cursor);
        self.to_end();
    }

    /// Consume self into a vector of bytes.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer.into_bytes()
    }

    /// Split at the cursor position.
    #[inline]
    pub fn start(&self) -> &str {
        // SAFETY: cursor is always valid.
        unsafe {
            let mid = self.cursor;
            let start = self.buffer.get_unchecked(0..mid);

            start
        }
    }

    /// Try parsing as a command.
    #[inline]
    pub fn command(&self) -> Result<Command<'_>, CommandError> {
        Command::try_parse(&self.buffer)
    }

    /// Return start character iterator.
    #[inline]
    pub fn start_chars(&self) -> Chars<'_> {
        Chars::new(self.start())
    }

    /// Split at the cursor position.
    #[inline]
    pub fn end(&self) -> &str {
        // SAFETY: cursor is always valid.
        unsafe {
            let mid = self.cursor;
            let end = self.buffer.get_unchecked(mid..self.buffer.len());

            end
        }
    }

    /// Return end character iterator.
    #[inline]
    pub fn end_chars(&self) -> Chars<'_> {
        Chars::new(self.end())
    }

    /// Split at the cursor position.
    #[inline]
    pub fn split(&self) -> (&str, &str) {
        // SAFETY: cursor is always valid.
        unsafe {
            let mid = self.cursor;
            let start = self.buffer.get_unchecked(0..mid);
            let end = self.buffer.get_unchecked(mid..self.buffer.len());

            (start, end)
        }
    }

    /// Return first character in edit.
    #[inline]
    pub fn first_char(&self) -> Option<char> {
        self.buffer.chars().next()
    }

    /// Return last character in edit.
    #[inline]
    pub fn last_char(&self) -> Option<char> {
        self.buffer.chars().last()
    }

    /// Find the next word, splitting on `chars`.
    #[inline]
    pub fn next_word(&mut self, chars: &[char]) {
        let end = {
            let mut iter = self.end_chars();

            // skip to the next character else we'll keep finding the same character
            // "foo bar"
            //      ^
            iter.next_back();
            iter.end()
        };

        match end.find(chars) {
            Some(index) => self.cursor += index + 1,
            None => self.to_end(),
        }
    }

    /// Find the previous word, splitting on `chars`.
    #[inline]
    pub fn prev_word(&mut self, chars: &[char]) {
        let start = unsafe {
            let start = self.start();
            let mut iter = Chars::new(start);

            // skip to the previous character else we'll keep finding the same character
            // "foo bar"
            //    ^
            // TODO: fix elysh_syntax::Chars
            match iter.next_back() {
                Some(character) => {
                    start.get_unchecked(0..start.len().saturating_sub(character.len_utf8()))
                }
                None => "",
            }
        };

        match start.rfind(chars) {
            Some(index) => self.cursor = index + 1,
            None => self.to_start(),
        }
    }

    /// Returns the amount needed to shift the cursor into the correct position.
    #[inline]
    pub fn shift(&self) -> usize {
        self.len().saturating_sub(self.cursor)
    }
}

impl fmt::Display for Edit {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.buffer, fmt)
    }
}

impl ops::Deref for Edit {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Default for Edit {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Edit {
    #[inline]
    fn from(buffer: String) -> Self {
        Self {
            buffer,
            cursor: 0,
            line: 0,
        }
    }
}

impl Into<String> for Edit {
    #[inline]
    fn into(self) -> String {
        self.buffer
    }
}
