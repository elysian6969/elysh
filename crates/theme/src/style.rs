use crate::Color;

#[derive(Clone, Copy, Debug)]
pub struct Style {
    color: Color,
    bright: bool,
    ground: bool,
}

impl Style {
    #[inline]
    pub const fn new(color: Color) -> Self {
        Self {
            color,
            bright: false,
            ground: false,
        }
    }

    #[inline]
    pub const fn bright(mut self, bright: bool) -> Self {
        self.bright = bright;
        self
    }

    #[inline]
    pub const fn foreground(mut self) -> Self {
        self.ground = false;
        self
    }

    #[inline]
    pub const fn background(mut self) -> Self {
        self.ground = true;
        self
    }

    #[inline]
    pub const fn as_ansi(&self) -> &'static str {
        match (self.bright, self.ground) {
            (false, false) => self.color.as_normal_ansi(),
            (true, false) => self.color.as_bright_ansi(),
            (false, true) => self.color.as_normal_bg_ansi(),
            (true, true) => self.color.as_bright_bg_ansi(),
        }
    }
}

impl From<Color> for Style {
    #[inline]
    fn from(color: Color) -> Self {
        Self::new(color)
    }
}
