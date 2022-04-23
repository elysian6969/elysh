#![allow(dead_code)]
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
                        Summary::Partial(executable)
                    }
                }
                None => Summary::NoMatch,
            };

            let write = match &summary {
                Summary::Exact => {
                    let rest = args.as_str();

                    Line::new()
                        .clear_line()
                        .push(prompt)
                        .push(' ')
                        .red()
                        .push(program)
                        .reset()
                        .push(' ')
                        .push(rest)
                }
                Summary::Partial(partial) => {
                    let remainder = unsafe { partial.strip_prefix(program).unwrap_unchecked() };
                    let length = remainder.len();

                    Line::new()
                        .clear_line()
                        .push(prompt)
                        .push(' ')
                        .push(program)
                        .grey()
                        .push(remainder)
                        .reset()
                        .move_left(length as u16)
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

        let input = session.wait_for_user().await?;
        let input = match input::map(&input) {
            Some(input) => input,
            None => continue,
        };

        let dbg = format!("\x1b[s\x1b[2;1H\x1b[2K{input:?}\x1b[u");
        session.write_all(dbg.as_bytes()).await?;

        match input {
            Input::ArrowUp => {
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
            Input::ArrowDown => {
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
            Input::Ctrl('c') => buffer.clear(),
            Input::Ctrl('d') => break,
            // return is ctrl-m???
            Input::Ctrl('m') => {
                if !buffer.is_empty() {
                    program = Some(mem::take(&mut buffer));
                }
            }
            // tab is ctrl+i???
            Input::ArrowRight | Input::Ctrl('i') => {
                if let Summary::Partial(partial) = &summary {
                    buffer = partial.clone().into();
                }
            }
            Input::Backspace => {
                buffer.pop();
            }
            Input::Space => {
                if !(buffer.is_empty() || buffer.ends_with_space()) {
                    buffer.push(' ');
                }
            }
            Input::Key(key) => buffer.push(key),
            _ => {}
        }

        if let Some(program) = program.take() {
            let program_clone = program.clone();

            history.reset();
            history.push(program_clone.into());

            let test = program.split_program();
            let dbg = format!("\x1b[s\x1b[3;1H\x1b[2K{test:?}\x1b[u");
            session.write_all(dbg.as_bytes()).await?;

            if let Some((program, mut args)) = program.split_program() {
                if program == "cd" {
                    if let Some(dir) = args.next() {
                        let _ = std::env::set_current_dir(dir);
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
                        Err(error) => Err(error),
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

        let dbg = format!("\x1b[s\x1b[1;1H\x1b[2K{history:?}\x1b[u");
        session.write_all(dbg.as_bytes()).await?;
    }

    session.write_all(b"\n").await?;

    Ok(())
}
