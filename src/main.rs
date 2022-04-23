#![feature(str_split_whitespace_as_str)]

use buffer::Buffer;
use history::History;
use input::Input;
use paths::Executables;
use session::Session;
use std::{env, io, mem};
use tokio::fs::OpenOptions;
use tokio::process::Command;

mod buffer;
mod history;
mod input;
mod paths;
mod session;

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

    let current_dir = env::current_dir()?;
    let status = format!(" {}\n\r", current_dir.display());
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
        let buffer2 = buffer.as_str();
        let mut args = buffer2.split_whitespace();

        let summary = if let Some(program) = args.next() {
            let executable = executables.search_one(&program);
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

            summary
        } else {
            Summary::NoMatch
        };

        let mut args = buffer2.split_whitespace();
        let write = match &summary {
            Summary::Exact => {
                let program = unsafe { args.next().unwrap_unchecked() };
                let rest = args.as_str();

                format!("\r\x1b[K{prompt} \x1b[38;5;1m{program}\x1b[m {rest}")
            },
            Summary::Partial(partial) => {
                let rem = unsafe { partial.strip_prefix(buffer.as_str()).unwrap_unchecked() };
                let len = rem.len();

                format!("\r\x1b[K{prompt} {buffer}\x1b[38;5;8m{rem}\x1b[m\x1b[{len}D")
            }
            Summary::NoMatch => format!("\r\x1b[K{prompt} {buffer}"),
        };

        session.write_str_all(&write).await?;

        let input = session.wait_for_user().await?;
        let input = match input::map(&input) {
            Some(input) => input,
            None => continue,
        };

        match input {
            Input::ArrowUp => {
                history.next();

                if let Some(item) = history.get() {
                    if last_buffer.is_none() {
                        last_buffer = Some(mem::replace(&mut buffer, Buffer::from(item.clone())));
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
                    }
                } else {
                    buffer = last_buffer.take().unwrap_or_default();
                }
            }
            Input::Ctrl('c') => buffer.clear(),
            Input::Ctrl('d') => break,
            Input::Return => {
                program = Some(mem::take(&mut buffer));
            }
            Input::ArrowRight | Input::Tab => {
                if let Summary::Partial(partial) = &summary {
                    buffer = partial.clone().into();
                }
            }
            Input::Backspace => {
                buffer.pop();
            }
            Input::Space => {
                if !buffer.is_empty() && !buffer.ends_with_space() {
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

            let args = program.as_str();
            let mut args = args.split_whitespace();

            if let Some(program) = args.next() {
                if program == "cd" {
                    if let Some(dir) = args.next() {
                        let _ = std::env::set_current_dir(dir);
                    }
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

                    session.write_all(b"\n\r").await?;
                    let current_dir = env::current_dir()?;
                    let status = format!(" {}\n", current_dir.display());
                    session.write_str_all(&status).await?;
                }
            }
        }
    }

    session.write_all(b"\n").await?;

    Ok(())
}
