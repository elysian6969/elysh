use core::iter::FusedIterator;
use core::str::CharIndices;

#[derive(Clone, Debug)]
pub struct Chars<'a> {
    iter: CharIndices<'a>,
    last_offset: usize,
    string: &'a str,
}

impl<'a> Chars<'a> {
    /// Create a new Chars.
    #[inline]
    pub fn new(string: &'a str) -> Self {
        let iter = string.char_indices();

        Self {
            iter,
            last_offset: 0,
            string,
        }
    }

    /// Returns the byte position of the next character, or the length of the underlying string if
    /// there are no more characters.
    #[inline]
    pub fn offset(&self) -> usize {
        self.iter.offset()
    }

    /// Provides a reference to the start of the underlying string.
    #[inline]
    pub fn start(&self) -> &'a str {
        // SAFETY: offset is always on a char boundary!
        unsafe {
            let mid = self.offset();
            let start = self.string.get_unchecked(0..mid);

            start
        }
    }

    /// Provides a reference to the end of the underlying string.
    #[inline]
    pub fn end(&self) -> &'a str {
        // SAFETY: offset is always on a char boundary!
        unsafe {
            let mid = self.offset();
            let end = self.string.get_unchecked(mid..self.string.len());

            end
        }
    }

    /// Divide the underlying string slice into two by the internal offset.
    #[inline]
    pub fn split(&self) -> (&'a str, &'a str) {
        // SAFETY: offset is always on a char boundary!
        unsafe {
            let mid = self.offset();
            let left = self.string.get_unchecked(0..mid);
            let right = self.string.get_unchecked(mid..self.string.len());

            (left, right)
        }
    }

    /// Returns the next character without advancing the iterator.
    #[inline]
    pub fn current(&mut self) -> Option<char> {
        // SAFETY: last_offset is always on a char boundary!
        let end = unsafe {
            let mid = self.last_offset;
            let end = self.string.get_unchecked(mid..self.string.len());

            end
        };

        end.chars().next()
    }

    /// Returns the next character without advancing the iterator.
    #[inline]
    pub fn peek(&mut self) -> Option<char> {
        self.end().chars().next()
    }

    /// Returns the nth character without advancing the iterator.
    #[inline]
    pub fn peek_nth(&mut self, n: usize) -> Option<char> {
        self.end().chars().nth(n)
    }

    /// Returns the previous next character.
    #[inline]
    pub fn peek_back(&mut self) -> Option<char> {
        self.start().chars().next_back()
    }

    /// Returns the previous nth character.
    #[inline]
    pub fn peek_nth_back(&mut self, n: usize) -> Option<char> {
        self.start().chars().nth_back(n)
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        self.last_offset = self.offset();
        self.iter.next().map(|(_index, character)| character)
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<char> {
        self.next_back()
    }
}

impl<'a> DoubleEndedIterator for Chars<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<char> {
        let item = self.iter.next_back().map(|(_index, character)| character);

        self.last_offset = self.offset();

        item
    }
}

impl FusedIterator for Chars<'_> {}
