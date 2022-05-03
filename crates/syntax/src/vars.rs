use crate::{Common, Quote, Token, Value};
use core::hint;

#[derive(Clone, Debug)]
pub enum Var<'a> {
    IncompletePair(&'a str),
    Pair(&'a str, Value<'a>),
    UnexpectedChar(char),
    Whitespace(&'a str),
}

#[derive(Clone, Debug)]
pub struct Vars<'a> {
    iter: Common<'a>,
    error: bool,
}

impl<'a> Var<'a> {
    #[inline]
    pub const fn is_incomplete(&self) -> bool {
        matches!(self, Var::IncompletePair(_))
    }

    #[inline]
    pub const fn is_whitespace(&self) -> bool {
        matches!(self, Var::Whitespace(_))
    }
}

impl<'a> Vars<'a> {
    #[inline]
    pub fn new(string: &'a str) -> Self {
        let iter = Common::new(string);

        Self { iter, error: false }
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.iter.offset()
    }

    /// Consume a word.
    #[inline]
    fn next_word(&mut self) -> &'a str {
        let iter = &mut self.iter;
        let start = iter.offset().saturating_sub(1);

        while let Some(character) = iter.peek() {
            if Quote::from_char(character).is_some()
                || character == '='
                || character.is_whitespace()
            {
                break;
            } else {
                iter.next();
            }
        }

        let end = iter.offset();
        let string = unsafe { iter.string.get_unchecked(start..end) };

        string
    }

    /// Consume a pair.
    #[inline]
    fn next_pair(&mut self) -> Option<Var<'a>> {
        let key = self.next_word();

        if self.iter.is_at_end() {
            return None;
        }

        let iter = &mut self.iter;
        let character = iter.peek()?;
        let token = match character {
            '=' => {
                iter.next();

                let character = iter.peek()?;

                // we match it all so
                iter.next();

                let token = match character {
                    '`' | '"' | '\'' => {
                        // SAFETY: match arm ensures `quote` is valid.
                        let quote = unsafe { Quote::from_char_unchecked(character) };

                        self.iter.next_string(quote)
                    }
                    character if character.is_whitespace() => {
                        self.error = true;

                        return Some(Var::IncompletePair(key));
                    }
                    _word => self.iter.next_word(),
                };

                token
            }
            character if character.is_whitespace() => {
                self.error = true;

                return Some(Var::IncompletePair(key));
            }
            // SAFETY: this cannot be a word as we just consumed a word!
            _ => unsafe { hint::unreachable_unchecked() },
        };

        // SAFETY: we never reach here without a value.
        let val = unsafe { token.value_unchecked() };

        Some(Var::Pair(key, val))
    }
}

impl<'a> Iterator for Vars<'a> {
    type Item = Var<'a>;

    #[inline]
    fn next(&mut self) -> Option<Var<'a>> {
        if self.error {
            return None;
        }

        let iter = &mut self.iter;
        let character = iter.next()?;
        let token = match character {
            '`' | '\'' | '"' => {
                self.error = true;

                return Some(Var::UnexpectedChar(character));
            }
            character if character.is_whitespace() => self.iter.next_whitespace(),
            _word => return self.next_pair(),
        };

        let arg = match token {
            Token::Whitespace(string) => Var::Whitespace(string),
            _ => unsafe { hint::unreachable_unchecked() },
        };

        Some(arg)
    }
}
