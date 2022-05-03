use core::cmp::Ordering;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug)]
pub struct History {
    history: Vec<String>,
    position: isize,
}

impl History {
    const NEW: Self = {
        let history = Vec::new();
        let position = 0;

        Self { history, position }
    };

    /// Construct a new history!
    #[inline]
    pub async fn new<P>(data_dir: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self::load(data_dir).await.unwrap_or_else(|_| Self::NEW)
    }

    /// Returns the count of history items.
    #[inline]
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Returns the current position within the history list.
    #[inline]
    pub fn position(&self) -> isize {
        self.position
    }

    /// Append a new item to the history list.
    #[inline]
    pub fn push(&mut self, item: String) {
        self.history.push(item);
    }

    /// Increment the position within the history list.
    #[inline]
    pub fn next(&mut self) {
        self.position = self.position.saturating_add(1).min(self.len() as isize);
    }

    /// Decrement the position within the history list.
    #[inline]
    pub fn next_back(&mut self) {
        self.position = self.position.saturating_sub(1).max(-(self.len() as isize));
    }

    /// Reset the position.
    #[inline]
    pub fn reset(&mut self) {
        self.position = 0;
    }

    #[inline]
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

    #[inline]
    async fn load<P>(data_dir: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = history_path(data_dir);
        let history = fs::read_to_string(path)
            .await?
            .lines()
            .map(String::from)
            .collect();

        Ok(Self {
            history,
            position: 0,
        })
    }

    #[inline]
    pub async fn save<P>(&self, data_dir: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let path = history_path(data_dir.as_ref());
        let history = self.history.join("\n");
        let _ = fs::create_dir_all(data_dir).await;

        fs::write(path, history.as_bytes()).await?;

        Ok(())
    }
}

#[inline]
fn history_path<P>(data_dir: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let mut path = data_dir.as_ref().to_path_buf();

    path.push("history");
    path
}
