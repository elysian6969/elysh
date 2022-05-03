use crate::Value;
use core::hint;

#[derive(Clone, Copy, Debug)]
pub enum Token<'a> {
    Value(Value<'a>),
    Whitespace(&'a str),
}

impl<'a> Token<'a> {
    /// Return this token as a string.
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        match self {
            Token::Value(arg) => arg.as_str(),
            Token::Whitespace(string) => string,
        }
    }

    /// Is this token a value?
    #[inline]
    pub const fn is_value(&self) -> bool {
        matches!(self, Token::Value(_))
    }

    /// Is this token whitespace?
    #[inline]
    pub const fn is_whitespace(&self) -> bool {
        matches!(self, Token::Whitespace(_))
    }

    /// Extracts the value out of this token.
    ///
    /// # Safety
    ///
    /// Caller must ensure the token is a value.
    #[inline]
    pub const unsafe fn value_unchecked(self) -> Value<'a> {
        match self {
            Token::Value(value) => value,
            _ => hint::unreachable_unchecked(),
        }
    }

    /// Extracts the whitespace out of this token.
    ///
    /// # Safety
    ///
    /// Caller must ensure the token is whitespace.
    #[inline]
    pub const unsafe fn whitespace_unchecked(self) -> &'a str {
        match self {
            Token::Whitespace(whitespace) => whitespace,
            _ => hint::unreachable_unchecked(),
        }
    }

    /// Extracts the word out of the value within this token.
    ///
    /// Equivalent to `self.value_unchecked().word_unchecked()`.
    ///
    /// # Safety
    ///
    /// Caller must ensure the token is a word value.
    #[inline]
    pub const unsafe fn word_unchecked(self) -> &'a str {
        self.value_unchecked().word_unchecked()
    }
}
