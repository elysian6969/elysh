//! Debug printing of modifiers.

use core::fmt;
use super::Input;

#[derive(Debug)]
enum Modifier {
    CTRL,
    META,
    SHIFT,
}

pub struct Modifiers(bool, bool, bool);

impl Modifiers {
    pub const fn from(input: &Input) -> Self {
        let ctrl = input.ctrl();
        let meta = input.meta();
        let shift = input.shift();

        Self(ctrl, meta, shift)
    }
}

impl fmt::Debug for Modifiers {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut list = fmt.debug_list();

        if self.0 {
            list.entry(&Modifier::CTRL);
        }

        if self.1 {
            list.entry(&Modifier::META);
        }

        if self.2 {
            list.entry(&Modifier::SHIFT);
        }

        list.finish()
    }
}
