use core::str::CharIndices;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Lexme<'input> {
    Equals,
    Newline,
    Semicolon,

    Backtick(&'input str),
    DoubleQuote(&'input str),
    Ident(&'input str),
    Quote(&'input str),
    Space(&'input str),
}

impl<'input> Lexme<'input> {
    pub fn to_string(&self) -> String {
        match self {
            Lexme::Equals => format!("\x1b[38;5;1m=\x1b[m"),
            Lexme::Newline => format!("\n"),
            Lexme::Semicolon => format!(";"),

            Lexme::Backtick(string) | Lexme::Quote(string) | Lexme::DoubleQuote(string) => {
                format!("\x1b[38;5;2m{string}\x1b[m")
            }

            Lexme::Ident(ident) => format!("{ident}"),
            Lexme::Space(space) => format!("{space}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Lexer<'input> {
    input: &'input str,
    chars: CharIndices<'input>,
    peek0: Option<(usize, char)>,
    peek1: Option<(usize, char)>,
}

pub trait Validate {
    fn validate(&self, character: char) -> bool;
}

impl Validate for char {
    fn validate(&self, character: char) -> bool {
        *self == character
    }
}

impl Validate for [char] {
    fn validate(&self, character: char) -> bool {
        self.contains(&character)
    }
}

impl<F> Validate for F
where
    F: Fn(char) -> bool,
{
    fn validate(&self, character: char) -> bool {
        self(character)
    }
}

impl<'input> Lexer<'input> {
    #[inline]
    pub fn new(input: &'input str) -> Self {
        let mut chars = input.char_indices();
        let peek0 = chars.next();
        let peek1 = chars.next();

        Lexer {
            input,
            chars,
            peek0,
            peek1,
        }
    }

    #[inline]
    fn step(&mut self) {
        self.peek0 = self.peek1;
        self.peek1 = self.chars.next();
    }

    #[inline]
    fn stepn(&mut self, n: usize) {
        for _ in 0..n {
            self.step();
        }
    }

    #[inline]
    fn peek(&self) -> Option<(usize, char)> {
        self.peek0
    }

    #[inline]
    fn peek2(&self) -> Option<(usize, char, char)> {
        self.peek0
            .and_then(|(start, char0)| self.peek1.map(|(_, char1)| (start, char0, char1)))
    }

    #[inline]
    fn consume_from_to<V, M, T>(
        &mut self,
        start: usize,
        validate: V,
        include_terminator: bool,
        map: M,
    ) -> Option<T>
    where
        V: Validate,
        M: Fn(&'input str) -> T,
    {
        let end = loop {
            match self.peek() {
                Some((_start, char0)) if !V::validate(&validate, char0) => self.step(),
                Some((start, _char0)) => {
                    if include_terminator {
                        self.step();
                    }

                    break start + include_terminator as usize;
                }
                None => return None,
            }
        };

        Some(map(&self.input[start..end]))
    }

    #[inline]
    fn ident(&mut self, start: usize) -> Option<Lexme<'input>> {
        fn isnt_ident_cont(character: char) -> bool {
            !matches!(character, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
        }

        self.consume_from_to(start, isnt_ident_cont, false, Lexme::Ident)
    }

    #[inline]
    fn space(&mut self, start: usize) -> Option<Lexme<'input>> {
        fn isnt_whitespace(character: char) -> bool {
            !character.is_whitespace()
        }

        self.consume_from_to(start, isnt_whitespace, false, Lexme::Space)
    }

    #[inline]
    fn backtick(&mut self, start: usize) -> Option<Lexme<'input>> {
        self.consume_from_to(start, '`', true, Lexme::Backtick)
    }

    #[inline]
    fn double_quote(&mut self, start: usize) -> Option<Lexme<'input>> {
        self.consume_from_to(start, '"', true, Lexme::DoubleQuote)
    }

    #[inline]
    fn quote(&mut self, start: usize) -> Option<Lexme<'input>> {
        self.consume_from_to(start, '\\', true, Lexme::Quote)
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Lexme<'input>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let lexme = match self.peek() {
            Some((start, char0)) => match char0 {
                '=' => Some(Lexme::Equals),
                '\n' => Some(Lexme::Newline),
                ';' => Some(Lexme::Semicolon),

                '`' => {
                    self.step();

                    return self.backtick(start);
                }
                '\'' => {
                    self.step();

                    return self.quote(start);
                }
                '"' => {
                    self.step();

                    return self.double_quote(start);
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.step();

                    return self.ident(start);
                }

                char0 if char0.is_whitespace() => {
                    self.step();

                    return self.space(start);
                }

                c => {
                    println!("\n\n!!! Encountered {c} !!!\n");
                    None
                }
            },
            _ => None,
        };

        if let Some(lexme) = lexme {
            self.step();

            return Some(lexme);
        }

        None
    }
}
