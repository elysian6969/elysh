//! Input mapping.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Input {
    Key(char),
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
    Return,
    Backspace,
    Tab,
    Space,
    Ctrl(char),
}

pub fn map(bytes: &[u8]) -> Option<Input> {
    let input = match bytes.len() {
        1 => match unsafe { bytes.get_unchecked(0) } {
            code @ 1..=26 => Input::Ctrl((code - 1 + b'a') as char),
            127 => Input::Backspace,
            b' ' => Input::Space,
            character => {
                let character = *character as char;

                if character.is_ascii_control() {
                    return None;
                } else {
                    Input::Key(character)
                }
            }
            _ => return None,
        },
        3 => match unsafe { bytes.get_unchecked(..3) } {
            b"\x1b[A" => Input::ArrowUp,
            b"\x1b[B" => Input::ArrowDown,
            b"\x1b[C" => Input::ArrowRight,
            b"\x1b[D" => Input::ArrowLeft,
            _ => return None,
        },
        _ => return None,
    };

    Some(input)
}
