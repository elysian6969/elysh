//! Input mapping.

use core::hint;
use core::mem::ManuallyDrop;

const CTRL: u8 = 0b001;
const META: u8 = 0b010;
const SHIFT: u8 = 0b100;

#[derive(Clone, Copy)]
#[repr(u32)]
enum Tag {
    ArrowDown = 0,
    ArrowLeft = 1,
    ArrowRight = 2,
    ArrowUp = 3,
    Backspace = 4,
    Delete = 5,
    End = 6,
    Home = 7,
    Key = 8,
    Paste = 9,
    Space = 10,
}

const ARROW_DOWN: u8 = Tag::ArrowDown as u8;
const HOME: u8 = Tag::Home as u8;
const KEY: u8 = Tag::Key as u8;
const PASTE: u8 = Tag::Paste as u8;
const SPACE: u8 = Tag::Space as u8;

#[repr(C)]
struct TagOnly {
    modifiers: u8,
}

#[repr(C)]
struct Key {
    code: char,
    modifiers: u8,
}

#[repr(C)]
struct Paste {
    string: Box<str>,
    modifiers: u8,
}

#[repr(C)]
union Data {
    tag_only: ManuallyDrop<TagOnly>,
    key: ManuallyDrop<Key>,
    paste: ManuallyDrop<Paste>,
}

#[repr(C)]
struct InputRepr {
    tag: Tag,
    data: Data,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
}

impl Input {
    const fn repr(&self) -> &mut InputRepr {
        unsafe { &mut *(self as *const Self as *mut InputRepr) }
    }

    const fn modifiers(&self) -> &mut u8 {
        let repr = self.repr();
        let modifiers = unsafe {
            match repr.tag as u8 {
                ARROW_DOWN..=HOME | SPACE => &repr.data.tag_only.modifiers,
                KEY => &repr.data.key.modifiers,
                PASTE => &repr.data.paste.modifiers,
                _ => hint::unreachable_unchecked(),
            }
        };

        unsafe { &mut *(modifiers as *const u8 as *mut u8) }
    }

    const fn set_modifiers(&mut self, modifiers: u8) {
        *self.modifiers() = modifiers;
    }

    /// Constructing an `Input` variant will not initialize the modifiers
    /// You definitely do not want random junk data!
    const fn with_none(mut self) -> Self {
        self.set_modifiers(0);
        self
    }

    const fn with_ctrl(mut self) -> Self {
        self.set_modifiers(CTRL);
        self
    }

    const fn with_meta(mut self) -> Self {
        self.set_modifiers(META);
        self
    }

    const fn with_shift(mut self) -> Self {
        self.set_modifiers(SHIFT);
        self
    }

    pub const fn none(&self) -> bool {
        *self.modifiers() == 0
    }

    pub const fn ctrl(&self) -> bool {
        *self.modifiers() & CTRL != 0
    }

    pub const fn meta(&self) -> bool {
        *self.modifiers() & META != 0
    }

    pub const fn shift(&self) -> bool {
        *self.modifiers() & SHIFT != 0
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
                Some(bytes) => Input::Paste(String::from_utf8_lossy(bytes).into()).with_none(),
                None => return None,
            },
            None => return None,
        },
    };

    Some(input)
}
