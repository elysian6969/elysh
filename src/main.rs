#![allow(dead_code)]
#![feature(char_indices_offset)]
#![feature(const_deref)]
#![feature(const_mut_refs)]
#![feature(const_ptr_read)]
#![feature(const_ptr_write)]
#![feature(const_trait_impl)]
#![feature(str_split_whitespace_as_str)]
#![feature(type_name_of_val)]

use context::{Context, Prompt};
use input::Input;
use paths::Summary;
use std::io;
use std::io::ErrorKind;
use std::path::PathBuf;
use tokio::fs::OpenOptions;

mod context;
mod history;
mod input;
mod paths;
mod session;

const WORD_CHARS: &[char] = &['/', '[', '&', '.', ';', '!', ']', '}', ':', '"', '|', ' '];

use elysh_syntax::Var;
use elysh_theme::{Color, DisplaySpaced, Style};
use std::fmt;
use std::fmt::Write;

pub struct DisplayArg<'a> {
    incomplete: bool,
    quote: char,
    string: &'a str,
}

impl<'a> DisplayArg<'a> {
    pub fn new(incomplete: bool, quote: char, string: &'a str) -> Self {
        Self {
            incomplete,
            quote,
            string,
        }
    }
}

impl<'a> fmt::Display for DisplayArg<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_char(self.quote)?;
        fmt.write_str(&self.string)?;

        if !self.incomplete {
            fmt.write_char(self.quote)?;
        }

        Ok(())
    }
}

pub struct DisplayVar<'a> {
    seperator_style: &'a Style,
    string_style: &'a Style,
    env: &'a Var<'a>,
}

impl<'a> DisplayVar<'a> {
    pub fn new(seperator_style: &'a Style, string_style: &'a Style, env: &'a Var<'a>) -> Self {
        Self {
            seperator_style,
            string_style,
            env,
        }
    }
}

impl<'a> fmt::Display for DisplayVar<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.env {
            Var::Pair(key, val) => {
                fmt.write_str(&key)?;

                let mut spaced = DisplaySpaced::new(fmt);

                spaced
                    .style(&self.seperator_style)
                    .entry(&'=')
                    .clear_style()
                    .finish()?;

                let mut spaced = DisplaySpaced::new(fmt);
                if let Some(quote) = val.quote() {
                    spaced
                        .style(&self.string_style)
                        .entry(&DisplayArg::new(
                            val.is_incomplete(),
                            quote.as_char(),
                            val.as_str(),
                        ))
                        .clear_style();
                } else {
                    spaced.entry(&val.as_str());
                }
            }
            Var::IncompletePair(key) => {
                fmt.write_str(&key)?;
            }
            _ => {}
        }

        Ok(())
    }
}

pub struct Display<'a> {
    prompt: &'a Prompt,
    command: Result<elysh_syntax::Command<'a>, elysh_syntax::CommandError<'a>>,
    string: &'a str,
    shift: usize,
    summary: &'a Summary,
}

impl<'a> Display<'a> {
    pub fn new(context: &'a Context, summary: &'a Summary) -> Self {
        let prompt = &context.prompt;
        let command = context.command();
        let string = &context.edit;
        let shift = context.edit.shift() + summary.shift();

        Self {
            prompt,
            command,
            string,
            shift,
            summary,
        }
    }
}

impl<'a> fmt::Display for Display<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("\r\x1b[K")?;

        fmt::Display::fmt(&self.prompt, fmt)?;

        let seperator_style = Style::new(Color::Blue);
        let string_style = Style::new(Color::Green);
        let exact_style = Style::new(Color::Green);
        let partial_style = Style::new(Color::Black).bright(true);

        match &self.command {
            Ok(command) => {
                let mut spaced = DisplaySpaced::new(fmt);

                for env in &command.vars {
                    spaced.entry(&DisplayVar::new(&seperator_style, &string_style, &env));
                }

                spaced.finish()?;

                if !command.vars.is_empty() && !command.program.as_str().is_empty() {
                    fmt.write_char(' ')?;
                }

                if let Some(display) = self.summary.display(&exact_style, &partial_style) {
                    fmt::Display::fmt(&display, fmt)?;
                } else {
                    let arg = &command.program;
                    let mut spaced = DisplaySpaced::new(fmt);

                    if let Some(quote) = arg.quote() {
                        spaced
                            .style(&string_style)
                            .entry(&DisplayArg::new(
                                arg.is_incomplete(),
                                quote.as_char(),
                                arg.as_str(),
                            ))
                            .clear_style();
                    } else {
                        spaced.entry(&arg.as_str());
                    }

                    spaced.finish()?;
                }

                if !command.args.is_empty() {
                    fmt.write_char(' ')?;
                }

                let mut spaced = DisplaySpaced::new(fmt);

                for arg in &command.args {
                    if let Some(quote) = arg.quote() {
                        spaced
                            .style(&string_style)
                            .entry(&DisplayArg::new(
                                arg.is_incomplete(),
                                quote.as_char(),
                                arg.as_str(),
                            ))
                            .clear_style();
                    } else {
                        spaced.entry(&arg.as_str());
                    }
                }

                spaced.finish()?;
            }
            _ => {}
        }

        if self.string.ends_with(char::is_whitespace) {
            fmt.write_char(' ')?;
        }

        match self.shift {
            0 => {}
            1 => fmt.write_str("\x1b[D")?,
            n => {
                fmt.write_str("\x1b[")?;
                fmt::Display::fmt(&n, fmt)?;
                fmt.write_char('D')?;
            }
        }

        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    let mut context = Context::new(tty).await?;

    context.enable_raw().await?;
    context.pre_prompt().await?;

    loop {
        let summary = context.suggest();
        let display = Display::new(&context, &summary).to_string();

        context.session.write_all(display.as_bytes()).await?;

        let input = context.next_input().await?;

        match input {
            Input::ArrowUp if input.none() => context.history_up(),
            Input::Key('p') if input.ctrl() => context.history_up(),
            Input::ArrowDown if input.none() => context.history_down(),
            Input::Key('n') if input.ctrl() => context.history_down(),
            /*Input::ArrowRight if input.none() => {
                if let Summary::Partial(partial, rest) = &summary {
                    if context.edit.at_end() {
                        context.edit = format!("{partial}{rest}").into();
                        context.edit.to_end();
                    }
                }

                context.edit.right(1);
            }
            Input::Key('i') if input.ctrl() => {
                if let Summary::Partial(partial, rest) = &summary {
                    if context.edit.at_end() {
                        context.edit = format!("{partial}{rest}").into();
                        context.edit.to_end();
                    }
                }
                context.edit.right(1);
            }*/
            Input::ArrowLeft if input.none() => context.prev(),
            Input::ArrowRight if input.none() => context.next(),
            Input::Key('c') if input.ctrl() => context.clear(),
            Input::Key('d') if input.ctrl() => break,
            Input::Key('m') if input.ctrl() => {
                if !context.edit.is_empty() {
                    context.execute_edit = true;
                }
            }
            Input::Key('w') if input.ctrl() => context.remove_word(WORD_CHARS),
            Input::Key('k') if input.ctrl() => context.remove_end(),

            Input::ArrowLeft if input.ctrl() => context.prev_word(WORD_CHARS),
            Input::ArrowLeft if input.shift() => context.prev_word(WORD_CHARS),
            Input::Key('b') if input.meta() => context.prev_word(WORD_CHARS),

            Input::ArrowRight if input.ctrl() => context.next_word(WORD_CHARS),
            Input::ArrowRight if input.shift() => context.next_word(WORD_CHARS),
            Input::Key('f') if input.meta() => context.next_word(WORD_CHARS),

            Input::Backspace if input.none() => context.remove(),
            Input::Space if input.none() => context.insert(' '),
            Input::Home if input.none() => context.to_start(),
            Input::Key('a') if input.ctrl() => context.to_start(),
            Input::End if input.none() => context.to_end(),
            Input::Key('e') if input.ctrl() => context.to_end(),
            Input::Key(key) if input.none() => context.insert(key),
            Input::Paste(string) if input.none() => context.insert_str(&string),
            _ => {}
        }

        let command = context
            .should_execute()
            .and_then(|_| context.command().ok());

        if let Some(command) = command {
            match command.program.as_str() {
                "exit" => {
                    break;
                }
                "cd" => {
                    let target_dir = command
                        .args
                        .get(0)
                        .map(|arg| arg.as_str())
                        .map(PathBuf::from)
                        .unwrap_or_else(|| context.home_dir.clone());

                    let result = context.change_dir(&target_dir);

                    match result {
                        Err(error) if error.kind() == ErrorKind::NotFound => {
                            let edit = format!(
                                "\relysh: `{}` no such file or directory\r\n",
                                target_dir.display()
                            );

                            context.session.write_all(edit.as_bytes()).await?;
                        }
                        _ => {}
                    }

                    context.session.write_all(b"\x1b[2A").await?;
                    context.pre_prompt().await?;
                }
                "showkeys" => {
                    context.toggle_showkeys();
                }
                _ => {
                    let target_dir = PathBuf::from(command.program.as_str());

                    context.session.write_all(b"\r\n").await?;

                    let result = match context.spawn(&command).await {
                        Ok(ok) => Ok(ok),
                        Err(_error) => {
                            let target_dir = context.expand_path(&target_dir);

                            match context.change_dir(&target_dir) {
                                Ok(()) => {
                                    context.session.write_all(b"\x1b[3A").await?;

                                    Ok(Ok(()))
                                },
                                Err(error) => Err(error),
                            }
                        }
                    };

                    match result {
                        Err(error) if error.kind() == ErrorKind::NotFound => {
                            let edit = format!(
                                "\relysh: `{}` no such file or directory\r\n",
                                target_dir.display()
                            );

                            context.session.write_all(edit.as_bytes()).await?;
                        }
                        _ => {}
                    }

                    context.pre_prompt().await?;
                }
            }

            context.clear_and_record();
        }
    }

    context.save_history().await;
    context.disable_raw().await?;
    context.session.write_all(b"\r\n\n[elysh exited]\r\n").await?;

    Ok(())
}
