use crate::{Common, Quote, Token, Value};

#[derive(Clone, Debug)]
pub enum Arg<'a> {
    Value(Value<'a>),
    Whitespace(&'a str),
}

#[derive(Clone, Debug)]
pub struct Args<'a> {
    iter: Common<'a>,
}

impl<'a> Arg<'a> {
    #[inline]
    pub const fn is_value(&self) -> bool {
        matches!(self, Arg::Value(_))
    }

    #[inline]
    pub const fn is_whitespace(&self) -> bool {
        matches!(self, Arg::Whitespace(_))
    }

    #[inline]
    pub const fn is_incomplete(&self) -> bool {
        match self {
            Arg::Value(value) => value.is_incomplete(),
            _ => false,
        }
    }

    #[inline]
    pub const fn as_str(&self) -> &'a str {
        match self {
            Arg::Value(value) => value.as_str(),
            Arg::Whitespace(whitespace) => whitespace,
        }
    }

    #[inline]
    pub const fn quote(&self) -> Option<Quote> {
        match self {
            Arg::Value(value) => value.quote(),
            _ => None,
        }
    }
}

impl<'a> Args<'a> {
    #[inline]
    pub fn new(string: &'a str) -> Self {
        let iter = Common::new(string);

        Self { iter }
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.iter.offset()
    }
}

impl<'a> Iterator for Args<'a> {
    type Item = Arg<'a>;

    #[inline]
    fn next(&mut self) -> Option<Arg<'a>> {
        let iter = &mut self.iter;
        let character = iter.next()?;
        let token = match character {
            '`' | '\'' | '"' => {
                // SAFETY: match arm ensures `quote` is valid.
                let quote = unsafe { Quote::from_char_unchecked(character) };

                self.iter.next_string(quote)
            }
            character if character.is_whitespace() => self.iter.next_whitespace(),
            _word => self.iter.next_word(),
        };

        let arg = match token {
            Token::Value(value) => Arg::Value(value),
            Token::Whitespace(string) => Arg::Whitespace(string),
        };

        Some(arg)
    }
}
