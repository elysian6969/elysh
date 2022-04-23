use std::fmt;

pub struct Line {
    line: String,
}

impl Line {
    pub const fn new() -> Self {
        let line = String::new();

        Self { line }
    }

    pub fn clear_line(mut self) -> Self {
        self.line.push_str("\r\x1b[K");
        self
    }

    pub fn grey(mut self) -> Self {
        self.line.push_str("\x1b[38;5;8m");
        self
    }

    pub fn red(mut self) -> Self {
        self.line.push_str("\x1b[38;5;1m");
        self
    }

    pub fn reset(mut self) -> Self {
        self.line.push_str("\x1b[m");
        self
    }

    pub fn push<D>(mut self, text: D) -> Self
    where
        D: fmt::Display,
    {
        self.line.push_str(&format!("{text}"));
        self
    }

    pub fn move_left(mut self, amount: u16) -> Self {
        if amount == 0 {
            self
        } else {
            self.line.push_str(&format!("\x1b[{amount}D"));
            self
        }
    }

    pub fn as_str(&self) -> &str {
        self.line.as_str()
    }
}
