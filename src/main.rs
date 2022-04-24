#![allow(dead_code)]
#![feature(const_deref)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(str_split_whitespace_as_str)]

use buffer::Buffer;
use history::History;
use input::Input;
use line::Line;
use paths::Executables;
use session::Session;
use std::{env, io, mem};
use tokio::fs::OpenOptions;
use tokio::process::Command;

mod buffer;
mod history;
mod input;
mod line;
mod paths;
mod session;

use std::path::PathBuf;

async fn home() -> Option<PathBuf> {
    use tokio::fs;

    let home = env::var_os("HOME")?;
    let home = fs::canonicalize(home).await.ok()?;

    Some(home)
}

async fn before_prompt() -> String {
    let dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return "<unknown>".into(),
    };

    let display = home()
        .await
        .and_then(|home| {
            let stripped = dir.strip_prefix(home).ok()?.display().to_string();
            let seperator = if stripped.is_empty() { "" } else { "/" };

            Some(format!("~{seperator}{stripped}"))
        })
        .unwrap_or_else(|| dir.display().to_string());

    format!("\r\x1b[K {}\n\r", display)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    let session = Session::new(tty)?;

    let mut buffer = Buffer::new();

    let prompt = "\x1b[38;5;1m >\x1b[m";

    session.set_raw()?;
    session.set_nonblocking()?;

    let mut history = History::new();
    let mut last_buffer = None;

    let executables = paths::from_env().await?;
    let executables = Executables::new(&executables);
    let mut program = None;

    // enable bracketed paste mode
    session.write_all(b"\x1b[?2004h").await?;

    let status = before_prompt().await;
    session.write_str_all(&status).await?;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum Summary {
        Exact,
        Partial(String),
        NoMatch,
    }

    impl Summary {
        pub const fn is_exact(&self) -> bool {
            matches!(self, Summary::Exact)
        }

        pub const fn is_partial(&self) -> bool {
            matches!(self, Summary::Partial(_))
        }

        pub const fn is_no_match(&self) -> bool {
            matches!(self, Summary::NoMatch)
        }
    }

    loop {
        let summary = if let Some((program, args)) = buffer.split_program() {
            let executable = executables.search_one(program);
            let summary = match executable {
                Some(executable) => {
                    if program == executable {
                        Summary::Exact
                    } else {
                        let rest = args.as_str();
                        let rest = buffer.as_str().strip_suffix(rest).unwrap_or("");

                        if rest.is_empty() {
                            Summary::NoMatch
                        } else {
                            Summary::Partial(executable)
                        }
                    }
                }
                None => Summary::NoMatch,
            };

            let write = match &summary {
                Summary::Exact => {
                    let rest = args.as_str();
                    let space = if buffer
                        .as_str()
                        .strip_suffix(rest)
                        .unwrap_or("")
                        .chars()
                        .next_back()
                        .map(|character| character.is_whitespace())
                        .unwrap_or(false)
                    {
                        " "
                    } else {
                        ""
                    };

                    Line::new()
                        .clear_line()
                        .push(prompt)
                        .push(' ')
                        .red()
                        .push(program)
                        .reset()
                        .push(space)
                        .push(rest)
                }
                Summary::Partial(partial) => {
                    let remainder = unsafe { partial.strip_prefix(program).unwrap_unchecked() };
                    let length = remainder.len();
                    let rest = buffer.as_str().strip_prefix(program).unwrap_or("");

                    let mut line = Line::new().clear_line().push(prompt).push(' ');

                    if rest.is_empty() {
                        line = line
                            .push(program)
                            .grey()
                            .push(remainder)
                            .reset()
                            .move_left(length as u16);
                    } else {
                        line = line.push(&buffer);
                    }

                    line
                }
                Summary::NoMatch => Line::new()
                    .clear_line()
                    .push(prompt)
                    .push(' ')
                    .push(&buffer),
            };

            session.write_str_all(write.as_str()).await?;

            summary
        } else {
            let write = Line::new()
                .clear_line()
                .push(prompt)
                .push(' ')
                .push(&buffer);

            session.write_str_all(write.as_str()).await?;

            Summary::NoMatch
        };

        let column_shift = buffer.column_shift();
        let write = Line::new().move_left(column_shift as u16);
        session.write_str_all(write.as_str()).await?;

        let input = session.wait_for_user().await?;

        // debug start
        let debug = format!("\x1b[s\x1b[2;1H\x1b[K{:?}\x1b[u", unsafe {
            std::str::from_utf8_unchecked(&input)
        });
        session.write_str_all(&debug).await?;
        // debug end

        let input = match input::map(&input) {
            Some(input) => input,
            None => continue,
        };

        // debug start
        let debug = format!("\x1b[s\x1b[3;1H\x1b[K{:?}\x1b[u", input);
        session.write_str_all(&debug).await?;
        // debug end

        match input {
            Input::ArrowUp if input.none() => {
                history.next();

                if let Some(item) = history.get() {
                    if last_buffer.is_none() {
                        last_buffer = Some(mem::replace(&mut buffer, Buffer::from(item.clone())));
                    } else {
                        buffer = Buffer::from(item.clone());
                    }
                } else {
                    buffer = last_buffer.take().unwrap_or_default();
                }
            }
            // ctrl+p
            Input::Key('p') if input.ctrl() => {
                history.next();

                if let Some(item) = history.get() {
                    if last_buffer.is_none() {
                        last_buffer = Some(mem::replace(&mut buffer, Buffer::from(item.clone())));
                    } else {
                        buffer = Buffer::from(item.clone());
                    }
                } else {
                    buffer = last_buffer.take().unwrap_or_default();
                }
            }
            Input::ArrowDown if input.none() => {
                history.next_back();

                if let Some(item) = history.get() {
                    if last_buffer.is_none() {
                        last_buffer = Some(mem::replace(&mut buffer, Buffer::from(item.clone())));
                    } else {
                        buffer = Buffer::from(item.clone());
                    }
                } else {
                    buffer = last_buffer.take().unwrap_or_default();
                }
            }
            // ctrl+n
            Input::Key('n') if input.ctrl() => {
                history.next_back();

                if let Some(item) = history.get() {
                    if last_buffer.is_none() {
                        last_buffer = Some(mem::replace(&mut buffer, Buffer::from(item.clone())));
                    } else {
                        buffer = Buffer::from(item.clone());
                    }
                } else {
                    buffer = last_buffer.take().unwrap_or_default();
                }
            }
            Input::ArrowLeft if input.none() => buffer.move_left(1),
            Input::ArrowRight if input.none() => {
                if let Summary::Partial(partial) = &summary {
                    if buffer.is_at_end() {
                        buffer = partial.clone().into();
                        buffer.move_to_end();
                    } else {
                        buffer.move_right(1);
                    }
                } else {
                    buffer.move_right(1);
                }
            }

            // ctrl+c
            Input::Key('c') if input.ctrl() => buffer.clear(),

            // ctrl+d
            Input::Key('d') if input.ctrl() => break,

            // ctrl+m aka return
            Input::Key('m') if input.ctrl() => {
                if !buffer.is_empty() {
                    program = Some(mem::take(&mut buffer));
                }
            }

            // ctrl+i aka tab
            Input::Key('i') if input.ctrl() => {
                if let Summary::Partial(partial) = &summary {
                    buffer = partial.clone().into();
                    buffer.move_to_end();
                }
            }

            // ctrl+w
            Input::Key('w') if input.ctrl() => buffer.remove_word_at_cursor(),

            // ctrl+k
            Input::Key('k') if input.ctrl() => buffer.remove_right_of_cursor(),

            // shift+arrow left
            Input::ArrowLeft if input.shift() => buffer.move_to_whitespace_left(),

            // alt+b
            Input::Key('b') if input.meta() => buffer.move_to_whitespace_left(),

            // shift+arrow right
            Input::ArrowRight if input.shift() => buffer.move_to_whitespace_right(),

            // alt+f
            Input::Key('f') if input.meta() => buffer.move_to_whitespace_right(),

            Input::Backspace if input.none() => buffer.remove_at_cursor(),
            Input::Space if input.none() => {
                if !(buffer.is_empty() || buffer.ends_with_space()) {
                    buffer.insert_at_cursor(' ');
                }
            }

            // unfortunately it's `(pat | pat) if expr`, not `pat | (pat if expr)`
            Input::Home if input.none() => buffer.move_to_start(),

            // ctrl+a
            Input::Key('a') if input.ctrl() => buffer.move_to_start(),

            Input::End if input.none() => buffer.move_to_end(),

            // ctrl+e
            Input::Key('e') if input.ctrl() => buffer.move_to_end(),

            Input::Key(key) if input.none() => buffer.insert_at_cursor(key),
            Input::Paste(string) if input.none() => buffer.insert_str_at_cursor(&string),
            _ => {}
        }

        if let Some(program) = program.take() {
            let program_clone = program.clone();

            history.reset();
            history.push(program_clone.into());

            if let Some((program, mut args)) = program.split_program() {
                if program == "cd" {
                    if let Some(dir) = args.next() {
                        let dir = match env::var("HOME").ok() {
                            Some(home) => dir.replace("~", &home),
                            None => dir.to_string(),
                        };

                        if tokio::fs::metadata(&dir).await.is_ok() {
                            let _ = std::env::set_current_dir(dir);
                        }
                    } else if let Some(home) = env::var_os("HOME") {
                        let _ = std::env::set_current_dir(home);
                    }

                    session.write_all(b"\x1b[A").await?;
                    let status = before_prompt().await;
                    session.write_str_all(&status).await?;
                } else {
                    session.write_all(b"\n\r").await?;
                    session.set_cooked()?;
                    session.set_blocking()?;

                    let result = Command::new(program).args(args).spawn();
                    let result = match result {
                        Ok(mut child) => match child.wait().await {
                            Ok(_status) => Ok(()),
                            Err(error) => Err(error),
                        },
                        Err(error) => {
                            let dir = match env::var("HOME").ok() {
                                Some(home) => program.replace("~", &home),
                                None => program.to_string(),
                            };

                            if tokio::fs::metadata(&dir).await.is_ok() {
                                let _ = std::env::set_current_dir(dir);

                                session.set_raw()?;
                                session.set_nonblocking()?;

                                session.write_all(b"\x1b[2A").await?;
                                let status = before_prompt().await;
                                session.write_str_all(&status).await?;

                                continue;
                            } else {
                                Err(error)
                            }
                        }
                    };

                    if let Err(_result) = result {
                        session.write_all(b"\rchild process died\n\r").await?;
                    }

                    session.set_raw()?;
                    session.set_nonblocking()?;

                    session.write_all(b"\n").await?;
                    let status = before_prompt().await;
                    session.write_str_all(&status).await?;
                }
            }
        }

        // debug start
        let debug = format!("\x1b[s\x1b[1;1H\x1b[K{:?}\x1b[u", buffer);
        session.write_str_all(&debug).await?;
        // debug end
    }

    // disable bracketed paste mode
    session.write_all(b"\x1b[?2004l").await?;
    session.write_all(b"\n").await?;

    Ok(())
}
