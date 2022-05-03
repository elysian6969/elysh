use crate::Quote;
use core::hint;

#[derive(Clone, Copy, Debug)]
pub enum Value<'a> {
    Quoted(Quote, &'a str),
    IncompleteQuoted(Quote, &'a str),
    Word(&'a str),
}

impl<'a> Value<'a> {
    /// Returns the argument as a string.
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        match self {
            Value::Quoted(_quote, string) => string,
            Value::IncompleteQuoted(_quote, string) => string,
            Value::Word(string) => string,
        }
    }

    /// Returns the quote of the string argument, if present.
    #[inline]
    pub const fn quote(&self) -> Option<Quote> {
        let quote = match self {
            Value::Quoted(quote, _string) => quote,
            Value::IncompleteQuoted(quote, _string) => quote,
            Value::Word(_string) => return None,
        };

        Some(*quote)
    }

    /// Returns the quote of the string argument, withour checking.
    ///
    /// # Safety
    ///
    /// Caller must ensure this value is a string.
    #[inline]
    pub const unsafe fn quote_unchecked(&self) -> Quote {
        match self {
            Value::Quoted(quote, _string) => *quote,
            Value::IncompleteQuoted(quote, _string) => *quote,
            _ => hint::unreachable_unchecked(),
        }
    }

    /// Is this value a quoted string?
    #[inline]
    pub const fn is_quoted(&self) -> bool {
        matches!(self, Value::Quoted(_, _) | Value::IncompleteQuoted(_, _))
    }

    // Is this value an incomplete quoted string?
    #[inline]
    pub const fn is_incomplete(&self) -> bool {
        matches!(self, Value::IncompleteQuoted(_, _))
    }

    /// Returns the word of the string argument, withour checking.
    ///
    /// # Safety
    ///
    /// Caller must ensure this value is a word.
    #[inline]
    pub const unsafe fn word_unchecked(&self) -> &'a str {
        match self {
            Value::Word(string) => string,
            _ => hint::unreachable_unchecked(),
        }
    }
}
