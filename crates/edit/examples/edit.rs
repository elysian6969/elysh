use elysh_edit::Edit;

const WORD_CHARS: &[char] = &['/', '[', '&', '.', ';', '!', ']', '}', ':', '"', '|', ' '];

fn main() {
    let mut edit = Edit::new();

    edit.insert_str("  hello   world  ");

    println!("{:?}", edit);
    println!("{:?}", edit.command());

    edit.prev_word(WORD_CHARS);

    println!("{:?}", edit.split());

    edit.prev_word(WORD_CHARS);

    println!("{:?}", edit.split());

    edit.to_start();
    edit.next_word(WORD_CHARS);

    println!("{:?}", edit.split());

    edit.next_word(WORD_CHARS);

    println!("{:?}", edit.split());

    edit.clear();
    edit.insert_str("foo bar");

    println!("{:?}", edit);

    edit.prev_word(WORD_CHARS);

    println!("{:?}", edit.split());

    edit.prev_word(WORD_CHARS);

    println!("{:?}", edit.split());

    edit.to_start();
    edit.next_word(WORD_CHARS);

    println!("{:?}", edit.split());

    edit.next_word(WORD_CHARS);

    println!("{:?}", edit.split());
}
