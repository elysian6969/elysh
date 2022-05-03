#[derive(Clone, Copy, Debug)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Color {
    #[inline]
    pub(crate) const fn as_normal_ansi(&self) -> &'static str {
        match self {
            Color::Black => "\x1b[38;5;0m",
            Color::Red => "\x1b[38;5;1m",
            Color::Green => "\x1b[38;5;2m",
            Color::Yellow => "\x1b[38;5;3m",
            Color::Blue => "\x1b[38;5;4m",
            Color::Magenta => "\x1b[38;5;5m",
            Color::Cyan => "\x1b[38;5;6m",
            Color::White => "\x1b[38;5;7m",
        }
    }

    #[inline]
    pub(crate) const fn as_bright_ansi(&self) -> &'static str {
        match self {
            Color::Black => "\x1b[38;5;8m",
            Color::Red => "\x1b[38;5;9m",
            Color::Green => "\x1b[38;5;10m",
            Color::Yellow => "\x1b[38;5;11m",
            Color::Blue => "\x1b[38;5;12m",
            Color::Magenta => "\x1b[38;5;13m",
            Color::Cyan => "\x1b[38;5;14m",
            Color::White => "\x1b[38;5;15m",
        }
    }

    #[inline]
    pub(crate) const fn as_normal_bg_ansi(&self) -> &'static str {
        match self {
            Color::Black => "\x1b[48;5;0m",
            Color::Red => "\x1b[48;5;1m",
            Color::Green => "\x1b[48;5;2m",
            Color::Yellow => "\x1b[48;5;3m",
            Color::Blue => "\x1b[48;5;4m",
            Color::Magenta => "\x1b[48;5;5m",
            Color::Cyan => "\x1b[48;5;6m",
            Color::White => "\x1b[48;5;7m",
        }
    }

    #[inline]
    pub(crate) const fn as_bright_bg_ansi(&self) -> &'static str {
        match self {
            Color::Black => "\x1b[48;5;8m",
            Color::Red => "\x1b[48;5;9m",
            Color::Green => "\x1b[48;5;10m",
            Color::Yellow => "\x1b[48;5;11m",
            Color::Blue => "\x1b[48;5;12m",
            Color::Magenta => "\x1b[48;5;13m",
            Color::Cyan => "\x1b[48;5;14m",
            Color::White => "\x1b[48;5;15m",
        }
    }
}
