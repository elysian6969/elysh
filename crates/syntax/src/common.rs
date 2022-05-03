use crate::{Chars, Quote, Token, Value};

#[derive(Clone, Debug)]
pub struct Common<'a> {
    iter: Chars<'a>,
    pub(crate) string: &'a str,
}

impl<'a> Common<'a> {
    #[inline]
    pub fn new(string: &'a str) -> Self {
        let iter = Chars::new(string);

        Self { iter, string }
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.iter.offset()
    }

    #[inline]
    pub(crate) fn is_at_end(&self) -> bool {
        self.offset() == self.string.len()
    }

    /// Consume a quoted string.
    #[inline]
    pub fn next_string(&mut self, quote: Quote) -> Token<'a> {
        let iter = &mut self.iter;
        let start = iter.offset();
        let mut terminated = false;

        while let Some(character) = iter.peek() {
            if character == quote.as_char() {
                if !iter
                    .peek_back()
                    .map(|character| character == '\\')
                    .unwrap_or(false)
                {
                    terminated = true;

                    break;
                }
            }

            iter.next();
        }

        let end = iter.offset();
        let string = unsafe { self.string.get_unchecked(start..end) };

        if terminated {
            iter.next();

            Token::Value(Value::Quoted(quote, string))
        } else {
            Token::Value(Value::IncompleteQuoted(quote, string))
        }
    }

    /// Consume whitespace.
    #[inline]
    pub fn next_whitespace(&mut self) -> Token<'a> {
        let iter = &mut self.iter;
        let start = iter.offset().saturating_sub(1);

        while let Some(character) = iter.peek() {
            if !character.is_whitespace() {
                break;
            } else {
                iter.next();
            }
        }

        let end = iter.offset();
        let string = unsafe { self.string.get_unchecked(start..end) };

        Token::Whitespace(string)
    }

    /// Consume a word.
    #[inline]
    pub fn next_word(&mut self) -> Token<'a> {
        let iter = &mut self.iter;
        let start = iter.offset().saturating_sub(1);

        while let Some(character) = iter.peek() {
            if Quote::from_char(character).is_some() || character.is_whitespace() {
                break;
            } else {
                iter.next();
            }
        }

        let end = iter.offset();
        let string = unsafe { self.string.get_unchecked(start..end) };

        Token::Value(Value::Word(string))
    }

    /*pub fn current(&mut self) -> Option<char> {
        self.iter.current()
    }*/

    #[inline]
    pub fn peek(&mut self) -> Option<char> {
        self.iter.peek()
    }

    /*pub fn peek_back(&mut self) -> Option<char> {
        self.iter.peek_back()
    }*/
}

impl<'a> Iterator for Common<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        self.iter.next()
    }
}
