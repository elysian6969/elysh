use core::hint;
use core::mem::ManuallyDrop;

const CTRL: u8 = 0b001;
const META: u8 = 0b010;
const SHIFT: u8 = 0b100;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Tag {
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

impl Tag {
    #[inline]
    pub const fn is_tag_only(&self) -> bool {
        matches!(*self as u8, ARROW_DOWN..=HOME | SPACE)
    }

    #[inline]
    pub const fn is_key(&self) -> bool {
        matches!(self, Tag::Key)
    }

    #[inline]
    pub const fn is_paste(&self) -> bool {
        matches!(self, Tag::Paste)
    }

    #[inline]
    pub const fn as_str(&self) -> &'static str {
        const TAG: [&str; 11] = [
            "ArrowDown",
            "ArrowLeft",
            "ArrowRight",
            "ArrowUp",
            "Backspace",
            "Delete",
            "End",
            "Home",
            "Key",
            "Paste",
            "Space",
        ];

        TAG[*self as usize]
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TagOnly {
    modifiers: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct Key {
    code: char,
    modifiers: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct Paste {
    string: Box<str>,
    modifiers: u8,
}

#[repr(C)]
pub union Data {
    pub tag_only: ManuallyDrop<TagOnly>,
    pub key: ManuallyDrop<Key>,
    pub paste: ManuallyDrop<Paste>,
}

#[repr(C)]
pub struct InputRepr {
    pub tag: Tag,
    pub data: Data,
}

impl TagOnly {
    #[inline]
    pub const fn modifiers(&self) -> &u8 {
        &self.modifiers
    }
}

impl Key {
    #[inline]
    pub const fn modifiers(&self) -> &u8 {
        &self.modifiers
    }
}

impl Paste {
    #[inline]
    pub const fn modifiers(&self) -> &u8 {
        &self.modifiers
    }
}

impl Data {
    /// Returns a pointer to the modifiers from a tag only variant.
    #[inline]
    pub const fn tag_only(&self) -> &u8 {
        // SAFETY: only called if it's a tag_only
        unsafe { self.tag_only.modifiers() }
    }

    /// Returns a pointer to the modifiers from a key variant.
    #[inline]
    pub const fn key(&self) -> &u8 {
        // SAFETY: only called if it's a key
        unsafe { self.key.modifiers() }
    }

    /// Returns a pointer to the modifiers from a paste variant.
    #[inline]
    pub const fn paste(&self) -> &u8 {
        // SAFETY: only called if it's a paste
        unsafe { self.paste.modifiers() }
    }
}

impl InputRepr {
    #[inline]
    pub const fn as_tag_str(&self) -> &'static str {
        self.tag.as_str()
    }

    #[inline]
    pub const fn is_tag_only(&self) -> bool {
        self.tag.is_tag_only()
    }

    #[inline]
    pub const fn is_key(&self) -> bool {
        self.tag.is_key()
    }

    #[inline]
    pub const fn is_paste(&self) -> bool {
        self.tag.is_paste()
    }

    #[inline]
    pub const fn modifiers(&self) -> &u8 {
        let data = &self.data;

        if self.is_tag_only() {
            data.tag_only()
        } else if self.is_key() {
            data.key()
        } else if self.is_paste() {
            data.paste()
        } else {
            unsafe { hint::unreachable_unchecked() }
        }
    }

    #[inline]
    pub const fn modifiers_mut(&mut self) -> &mut u8 {
        // SAFETY: just a reborrow
        unsafe { &mut *(self.modifiers() as *const u8 as *mut u8) }
    }

    #[inline]
    pub const fn set_modifiers(&mut self, modifiers: u8) {
        *self.modifiers_mut() = modifiers;
    }

    #[inline]
    pub const fn has_modifier(&self, flag: u8) -> bool {
        *self.modifiers() & flag != 0
    }

    #[inline]
    pub const fn set_none(&mut self) {
        self.set_modifiers(0);
    }

    #[inline]
    pub const fn set_ctrl(&mut self) {
        self.set_modifiers(CTRL);
    }

    #[inline]
    pub const fn set_meta(&mut self) {
        self.set_modifiers(META);
    }

    #[inline]
    pub const fn set_shift(&mut self) {
        self.set_modifiers(SHIFT);
    }

    #[inline]
    pub const fn is_none(&self) -> bool {
        *self.modifiers() == 0
    }

    #[inline]
    pub const fn has_ctrl(&self) -> bool {
        self.has_modifier(CTRL)
    }

    #[inline]
    pub const fn has_meta(&self) -> bool {
        self.has_modifier(META)
    }

    #[inline]
    pub const fn has_shift(&self) -> bool {
        self.has_modifier(SHIFT)
    }
}
