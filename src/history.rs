use core::cmp::Ordering;

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
                Some(unsafe { self.history.get_unchecked(self.position as usize - 1) })
            }
            Ordering::Equal => None,
            Ordering::Less => {
                // SAFETY: self.position is always valid.
                Some(unsafe { self.history.get_unchecked(self.position.abs() as usize - 1) })
            }
        }
    }
}
