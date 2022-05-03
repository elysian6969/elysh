use crate::Style;
use core::fmt;
use core::fmt::Write;

struct DisplayInner<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    needs_seperator: bool,
}

impl<'a, 'b: 'a> DisplayInner<'a, 'b> {
    #[inline]
    fn entry(&mut self, entry: &dyn fmt::Display) {
        self.result = self.result.and_then(|_| {
            if self.needs_seperator {
                self.fmt.write_char(' ')?;
            }

            entry.fmt(self.fmt)
        });

        self.needs_seperator = true;
    }

    #[inline]
    fn ignored(&mut self, entry: &dyn fmt::Display) {
        self.result = self.result.and_then(|_| entry.fmt(self.fmt));
    }
}

pub struct DisplaySpaced<'a, 'b: 'a> {
    inner: DisplayInner<'a, 'b>,
}

impl<'a, 'b: 'a> DisplaySpaced<'a, 'b> {
    #[inline]
    pub fn new(fmt: &'a mut fmt::Formatter<'b>) -> Self {
        Self {
            inner: DisplayInner {
                fmt,
                result: Ok(()),
                needs_seperator: false,
            },
        }
    }

    #[inline]
    pub fn entry(&mut self, entry: &dyn fmt::Display) -> &mut Self {
        self.inner.entry(entry);
        self
    }

    #[inline]
    pub fn entries<D, I>(&mut self, entries: I) -> &mut Self
    where
        D: fmt::Display,
        I: IntoIterator<Item = D>,
    {
        for entry in entries {
            self.entry(&entry);
        }

        self
    }

    #[inline]
    pub fn style(&mut self, style: &Style) -> &mut Self {
        self.inner.ignored(&style.as_ansi());
        self
    }

    #[inline]
    pub fn clear_style(&mut self) -> &mut Self {
        self.inner.ignored(&"\x1b[m");
        self
    }

    #[inline]
    pub fn finish(&mut self) -> fmt::Result {
        self.inner.result
    }
}
