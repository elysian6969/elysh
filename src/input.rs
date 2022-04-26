//! Input mapping.

use core::fmt;
use modifiers::Modifiers;
use repr::InputRepr;

mod modifiers;
mod repr;

#[derive(Clone, Eq, PartialEq)]
#[repr(C)]
#[non_exhaustive]
pub enum Input {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Backspace,
    Delete,
    End,
    Home,
    Key(char),
    Paste(Box<str>),
    Space,

    /// THIS IS SUPER IMPORTANT!!!
    ///
    /// Informs the compiler to allocate enough space required to write bytes at the end of every
    /// variant.
    #[doc(hidden)]
    Padding([u8; 24]),
}

impl Input {
    #[inline]
    const fn repr(&self) -> &InputRepr {
        unsafe { &*(self as *const Self as *const InputRepr) }
    }

    #[inline]
    const fn repr_mut(&mut self) -> &mut InputRepr {
        unsafe { &mut *(self as *mut Self as *mut InputRepr) }
    }

    #[inline]
    const fn as_tag_str(&self) -> &str {
        self.repr().as_tag_str()
    }

    /// Constructing an `Input` variant will not initialize the modifiers
    /// You definitely do not want random junk data!
    #[inline]
    const fn with_none(mut self) -> Self {
        self.repr_mut().set_none();
        self
    }

    #[inline]
    const fn with_ctrl(mut self) -> Self {
        self.repr_mut().set_ctrl();
        self
    }

    #[inline]
    const fn with_meta(mut self) -> Self {
        self.repr_mut().set_meta();
        self
    }

    #[inline]
    const fn with_shift(mut self) -> Self {
        self.repr_mut().set_shift();
        self
    }

    #[inline]
    pub const fn none(&self) -> bool {
        self.repr().is_none()
    }

    #[inline]
    pub const fn ctrl(&self) -> bool {
        self.repr().has_ctrl()
    }

    #[inline]
    pub const fn meta(&self) -> bool {
        self.repr().has_meta()
    }

    #[inline]
    pub const fn shift(&self) -> bool {
        self.repr().has_shift()
    }
}

impl fmt::Debug for Input {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let has_fields = matches!(self, Input::Key(_) | Input::Paste(_));
        let has_mod = !self.none();

        // field-less and without modifiers
        if !has_fields && !has_mod {
            fmt.write_str(self.as_tag_str())?;

            return Ok(());
        }

        let mut tuple = fmt.debug_tuple(self.as_tag_str());

        // field handling
        match self {
            Input::Key(key) => {
                tuple.field(key);
            }
            Input::Paste(string) => {
                tuple.field(string);
            }
            _ => {}
        }

        if has_mod {
            tuple.field(&Modifiers::from(self));
        }

        tuple.finish()
    }
}

pub fn map(bytes: &[u8]) -> Option<Input> {
    let input = match bytes.len() {
        1 => match unsafe { bytes.get_unchecked(0) } {
            // ASCII: '`', 'a', 'b', 'c', ... 'z'
            code @ 0..=26 => Input::Key((code + b'`') as char).with_ctrl(),
            27 => Input::Key('[').with_ctrl(),
            28 => Input::Key('\\').with_ctrl(),
            29 => Input::Key(']').with_ctrl(),
            31 => Input::Key('/').with_ctrl(),
            127 => Input::Backspace.with_none(),
            b' ' => Input::Space.with_none(),
            character => {
                let character = *character as char;

                if character.is_ascii_control() {
                    return None;
                } else {
                    Input::Key(character).with_none()
                }
            }
        },
        2 => match unsafe { bytes.get_unchecked(..2) } {
            [b'\x1b', code] => Input::Key(*code as char).with_meta(),
            _ => return None,
        },
        3 => match unsafe { bytes.get_unchecked(..3) } {
            b"\x1b[A" => Input::ArrowUp.with_none(),
            b"\x1b[B" => Input::ArrowDown.with_none(),
            b"\x1b[C" => Input::ArrowRight.with_none(),
            b"\x1b[D" => Input::ArrowLeft.with_none(),
            b"\x1b[F" => Input::End.with_none(),
            b"\x1b[H" => Input::Home.with_none(),
            _ => return None,
        },
        4 => match unsafe { bytes.get_unchecked(..4) } {
            b"\x1b[3~" => Input::Delete.with_none(),
            _ => return None,
        },
        6 => match unsafe { bytes.get_unchecked(..6) } {
            b"\x1b[1;5A" => Input::ArrowUp.with_shift(),
            b"\x1b[1;5B" => Input::ArrowDown.with_shift(),
            b"\x1b[1;5C" => Input::ArrowRight.with_shift(),
            b"\x1b[1;5D" => Input::ArrowLeft.with_shift(),
            _ => return None,
        },
        _ => match bytes.strip_prefix(b"\x1b[200~") {
            Some(bytes) => match bytes.strip_suffix(b"\x1b[201~") {
                Some(bytes) => Input::Paste(Box::from(String::from_utf8_lossy(bytes))).with_none(),
                None => return None,
            },
            None => return None,
        },
    };

    Some(input)
}
