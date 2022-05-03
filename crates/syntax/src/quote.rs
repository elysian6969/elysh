use core::hint;

#[derive(Clone, Copy, Debug)]
pub enum Quote {
    Backtick,
    DoubleQuote,
    Quote,
}

impl Quote {
    /// Obtain the character value of the quote variant.
    #[inline]
    pub const fn as_char(&self) -> char {
        match self {
            Quote::Backtick => '`',
            Quote::DoubleQuote => '"',
            Quote::Quote => '\'',
        }
    }

    /// Construct a variant from a character.
    #[inline]
    pub const fn from_char(quote: char) -> Option<Self> {
        let quote = match quote {
            '`' => Quote::Backtick,
            '"' => Quote::DoubleQuote,
            '\'' => Quote::Quote,
            _ => return None,
        };

        Some(quote)
    }

    /// Construct a variant from a character, without checking the character.
    ///
    /// # Safety
    ///
    /// Caller must ensure `quote` is a valid quote.
    #[inline]
    pub const unsafe fn from_char_unchecked(quote: char) -> Self {
        match quote {
            '`' => Quote::Backtick,
            '"' => Quote::DoubleQuote,
            '\'' => Quote::Quote,
            _ => hint::unreachable_unchecked(),
        }
    }
}
