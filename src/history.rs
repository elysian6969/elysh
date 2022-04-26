use core::cmp::Ordering;
use std::path::PathBuf;
use std::{env, io};
use tokio::fs;

#[derive(Debug)]
pub struct History {
    history: Vec<String>,
    position: isize,
}

impl History {
    /// Construct a new instance of History.
    pub const fn new() -> Self {
        let history = Vec::new();
        let position = 0;

        Self { history, position }
    }

    /// Returns the count of history items.
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Returns the current position within the history list.
    pub fn position(&self) -> isize {
        self.position
    }

    /// Append a new item to the history list.
    pub fn push(&mut self, item: String) {
        self.history.push(item);
    }

    /// Increment the position within the history list.
    pub fn next(&mut self) {
        self.position = self.position.saturating_add(1).min(self.len() as isize);
    }

    /// Decrement the position within the history list.
    pub fn next_back(&mut self) {
        self.position = self.position.saturating_sub(1).max(-(self.len() as isize));
    }

    /// Reset the position.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    pub fn get(&self) -> Option<&String> {
        match self.position.cmp(&0) {
            Ordering::Greater => {
                // SAFETY: self.position is always valid.
                Some(unsafe {
                    self.history
                        .get_unchecked(self.len().saturating_sub(self.position as usize))
                })
            }
            Ordering::Equal => None,
            Ordering::Less => {
                // SAFETY: self.position is always valid.
                Some(unsafe {
                    self.history
                        .get_unchecked(self.len().saturating_sub(self.position.abs() as usize))
                })
            }
        }
    }

    pub fn path() -> PathBuf {
        let mut home = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));

        let mut xdg_data_home = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                home.push(".local/share");
                home
            });

        xdg_data_home.push("elysh/history");
        xdg_data_home
    }

    pub async fn load() -> io::Result<Self> {
        let path = History::path();
        let history = fs::read_to_string(&path)
            .await?
            .lines()
            .map(String::from)
            .collect();

        Ok(Self {
            history,
            position: 0,
        })
    }

    pub async fn save(&self) -> io::Result<()> {
        let path = History::path();
        let parent = unsafe { path.parent().unwrap_unchecked() };
        let history = self.history.join("\n");

        let _ = fs::create_dir_all(&parent).await;
        fs::write(&path, history.as_bytes()).await?;

        Ok(())
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
