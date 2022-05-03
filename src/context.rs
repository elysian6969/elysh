use crate::history::History;
use crate::input;
use crate::input::Input;
use crate::paths::{Exes, Summary};
use crate::session::Session;
use elysh_edit::Edit;
use elysh_syntax::Var;
use std::fmt;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::{env, io, mem};
use tokio::fs::File;
use tokio::process::Command;

pub struct Prompt {
    prompt_char: char,
}

impl Prompt {
    #[inline]
    pub const fn new(prompt_char: char) -> Self {
        Self { prompt_char }
    }
}

impl fmt::Display for Prompt {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_char(' ')?;
        fmt.write_str("\x1b[38;5;1m")?;
        fmt.write_char(self.prompt_char)?;
        fmt.write_str("\x1b[m")?;
        fmt.write_char(' ')?;

        Ok(())
    }
}

pub mod env2 {
    use std::env;
    use std::path::{Path, PathBuf};

    /// Returns the environment variable `HOME` or `/`.
    #[inline]
    pub fn home_dir() -> PathBuf {
        env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"))
    }

    /// Returns the environment variable `XDG_DATA_HOME/elysh` or `{home}/.local/share/elysh`.
    #[inline]
    pub fn data_dir<P>(home: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        let home = home.as_ref().to_path_buf();
        let data_dir = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"));

        data_dir.join("elysh")
    }
}

pub struct Context {
    pub edit: Edit,
    current_dir: PathBuf,
    data_dir: PathBuf,
    pub exact: char,
    pub execute_edit: bool,
    exes: Exes,
    history: History,
    pub home_dir: PathBuf,
    last_edit: Option<Edit>,
    pub prompt: Prompt,
    pub rest: char,
    pub session: Session,
    showkeys: bool,
}

impl Context {
    #[inline]
    pub async fn new(tty: File) -> io::Result<Self> {
        let current_dir = env::current_dir()?;
        let home_dir = env2::home_dir();
        let data_dir = env2::data_dir(&home_dir);

        let edit = Edit::new();
        let exact = '1';
        let execute_edit = false;
        let exes = Exes::from_env().await?;
        let history = History::new(&data_dir).await;
        let last_edit = None;
        let prompt = Prompt::new('>');
        let rest = '8';
        let session = Session::new(tty)?;
        let showkeys = false;

        Ok(Self {
            edit,
            current_dir,
            data_dir,
            exact,
            execute_edit,
            exes,
            history,
            home_dir,
            last_edit,
            prompt,
            rest,
            session,
            showkeys,
        })
    }

    /// Substitute `HOME` for `~`.
    ///
    /// Inverse of `expand_path`.
    #[inline]
    pub fn shorten_path<P>(&self, path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path = match path.strip_prefix(&self.home_dir) {
            Ok(rest) => Path::new("~").join(rest),
            Err(_error) => path.to_path_buf(),
        };

        path.components().collect()
    }

    /// Substitute `~` for `HOME`.
    ///
    /// Inverse of `shorten_path`.
    #[inline]
    pub fn expand_path<P>(&self, path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path = match path.strip_prefix("~") {
            Ok(rest) => self.home_dir.join(rest),
            Err(_error) => path.to_path_buf(),
        };
        
        path.components().collect()
    }

    /// Enable raw mode for all sorts of fancy terminalisms.
    #[inline]
    pub async fn enable_raw(&self) -> io::Result<()> {
        self.session.set_raw()?;
        self.session.set_nonblocking()?;
        // enable bracketed paste mode
        self.session.write_all(b"\x1b[?2004h").await?;

        Ok(())
    }

    /// Disable raw mode to run programs and such.
    #[inline]
    pub async fn disable_raw(&self) -> io::Result<()> {
        // disable bracketed paste mode
        self.session.write_all(b"\x1b[?2004l").await?;
        self.session.set_cooked()?;
        self.session.set_blocking()?;

        Ok(())
    }

    #[inline]
    pub fn command<'a>(&'a self) -> Result<elysh_syntax::Command, elysh_syntax::CommandError> {
        self.edit.command()
    }

    #[inline]
    pub fn search_program(&self, program: &str) -> Summary {
        self.exes.search_one(program)
    }

    #[inline]
    pub fn suggest(&self) -> Summary {
        if self.edit.is_empty() {
            return Summary::NoMatch;
        }

        match self.command() {
            Ok(command) => {
                if command.program.as_str().is_empty() {
                    Summary::NoMatch
                } else {
                    match self.search_program(command.program.as_str()) {
                        Summary::Partial(partial, rest) => {
                            if command.args.is_empty() && !self.edit.ends_with_space() {
                                Summary::Partial(partial, rest)
                            } else {
                                Summary::NoMatch
                            }
                        }
                        summary => summary,
                    }
                }
            }
            Err(_error) => Summary::NoMatch,
        }
    }

    #[inline]
    pub async fn showkeys(&self, string: &str, input: Option<&Input>) -> io::Result<()> {
        if !self.showkeys {
            return Ok(());
        }

        struct Showkeys<'a>(&'a str, Option<&'a Input>);

        impl<'a> fmt::Display for Showkeys<'a> {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("\x1b[s\x1b[1;1H\x1b[K[showkeys: ")?;
                fmt::Debug::fmt(&self.0, fmt)?;

                if let Some(input) = self.1 {
                    fmt.write_str(" -> ")?;
                    fmt::Debug::fmt(input, fmt)?;
                }

                fmt.write_str("]\x1b[u")?;

                Ok(())
            }
        }

        let showkeys = Showkeys(string, input);
        let edit = format!("{showkeys}");

        self.session.write_all(edit.as_bytes()).await?;

        Ok(())
    }

    #[inline]
    pub async fn next_input(&self) -> io::Result<Input> {
        let input = loop {
            let bytes = self.session.wait_for_user().await?;
            // SAFETY: `bytes` technically shouldn't be fucky?
            let string = unsafe { std::str::from_utf8_unchecked(&bytes) };

            let input = input::map(&bytes);

            match input {
                Some(input) => {
                    self.showkeys(string, Some(&input)).await?;

                    break input;
                }
                None => self.showkeys(string, None).await?,
            }
        };

        Ok(input)
    }

    #[inline]
    pub fn update_edit_with_history(&mut self) {
        if let Some(item) = self.history.get() {
            if self.last_edit.is_none() {
                let item_edit = Edit::from(item.clone());

                self.last_edit = Some(mem::replace(&mut self.edit, item_edit));
            } else {
                self.edit = Edit::from(item.clone());
            }
        } else {
            self.edit = self.last_edit.take().unwrap_or_default();
        }

        self.edit.to_end();
    }

    #[inline]
    pub fn history_up(&mut self) {
        self.history.next();
        self.update_edit_with_history();
    }

    #[inline]
    pub fn history_down(&mut self) {
        self.history.next_back();
        self.update_edit_with_history();
    }

    #[inline]
    pub fn clear(&mut self) {
        self.edit.clear();
    }

    #[inline]
    pub fn clear_and_record(&mut self) {
        let edit = mem::take(&mut self.edit);

        self.history.push(edit.into());
    }

    #[inline]
    pub fn next(&mut self) {
        self.edit.next(1);
    }

    #[inline]
    pub fn next_word(&mut self, chars: &[char]) {
        self.edit.next_word(chars);
    }

    #[inline]
    pub fn prev_word(&mut self, chars: &[char]) {
        self.edit.prev_word(chars);
    }

    #[inline]
    pub fn prev(&mut self) {
        self.edit.prev(1);
    }

    #[inline]
    pub fn to_end(&mut self) {
        self.edit.to_end();
    }

    #[inline]
    pub fn to_start(&mut self) {
        self.edit.to_start();
    }

    #[inline]
    pub fn insert(&mut self, character: char) {
        self.edit.insert(character);
    }

    #[inline]
    pub fn insert_str(&mut self, string: &str) {
        self.edit.insert_str(string);
    }

    #[inline]
    pub fn remove(&mut self) {
        self.edit.remove();
    }

    #[inline]
    pub fn remove_word(&mut self, chars: &[char]) {
        self.edit.remove_word(chars);
    }

    #[inline]
    pub fn remove_end(&mut self) {
        self.edit.remove_end();
    }

    #[inline]
    pub fn toggle_showkeys(&mut self) {
        self.showkeys = !self.showkeys;
    }

    #[inline]
    pub async fn save_history(&self) {
        let _ = self.history.save(&self.data_dir).await;
    }

    #[inline]
    pub fn change_dir<P>(&mut self, target_dir: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let target_dir = self.expand_path(target_dir);
        let result = std::env::set_current_dir(&target_dir);

        result
            .and_then(|_| std::env::current_dir())
            .map(|target_dir| self.current_dir = target_dir)
    }

    #[inline]
    pub async fn spawn(&self, command: &elysh_syntax::Command<'_>) -> io::Result<io::Result<()>> {
        self.disable_raw().await?;

        let result = Command::new(command.program.as_str())
            .args(command.args.iter().map(|arg| arg.as_str()))
            .envs(command.vars.iter().flat_map(|var| match var {
                Var::Pair(key, val) => Some((key, val.as_str())),
                _ => None,
            }))
            .spawn();

        let result = match result {
            Ok(mut child) => match child.wait().await {
                Ok(_status) => Ok(Ok(())),
                Err(error) => Ok(Err(error)),
            },
            Err(error) => Err(error),
        };

        self.enable_raw().await?;

        result
    }

    #[inline]
    pub async fn pre_prompt(&self) -> io::Result<()> {
        let current_dir = self.shorten_path(&self.current_dir);
        let edit = format!("\r\n \x1b[K{}\r\n", current_dir.display());

        self.session.write_all(edit.as_bytes()).await?;

        Ok(())
    }

    #[inline]
    pub fn should_execute(&mut self) -> Option<()> {
        mem::take(&mut self.execute_edit).then(|| ())
    }
}
