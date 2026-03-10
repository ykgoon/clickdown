//! TUI helper utilities
//!
//! This module provides reusable helpers for TUI widget states and rendering.

use ratatui::widgets::ListState;

/// Generic selectable list helper for TUI widget states
///
/// This struct encapsulates common list selection logic used across
/// multiple widget states (sidebar, task list, etc.).
///
/// # Example
///
/// ```no_run
/// use clickdown::tui::helpers::SelectableList;
///
/// let mut list = SelectableList::empty();
/// list.items_mut().extend(vec!["Item 1", "Item 2", "Item 3"]);
/// list.select_first();
/// list.select_next();
/// assert_eq!(list.selected(), Some(&"Item 2"));
/// ```
#[derive(Debug, Clone)]
pub struct SelectableList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> SelectableList<T> {
    /// Create a new empty selectable list
    pub fn empty() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    /// Get the currently selected item
    pub fn selected(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    /// Select the first item
    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Move selection down (wraps to start)
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Move selection up (wraps to end)
    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Select item by index
    pub fn select(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            if i < self.items.len() {
                self.state.select(Some(i));
            }
        } else {
            self.state.select(None);
        }
    }

    /// Select item by predicate, returns true if found
    pub fn select_by<F>(&mut self, predicate: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        for (i, item) in self.items.iter().enumerate() {
            if predicate(item) {
                self.state.select(Some(i));
                return true;
            }
        }
        false
    }

    /// Get all items
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Get all items (mutable)
    pub fn items_mut(&mut self) -> &mut Vec<T> {
        &mut self.items
    }

    /// Get the internal ListState for rendering
    pub fn state(&self) -> &ListState {
        &self.state
    }
}

impl<T> Default for SelectableList<T> {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_list() {
        let list: SelectableList<i32> = SelectableList::empty();
        assert!(list.items().is_empty());
    }

    #[test]
    fn test_select_first() {
        let mut list = SelectableList::empty();
        list.items_mut().extend(vec![1, 2, 3]);
        list.select_first();
        assert_eq!(list.selected(), Some(&1));
    }

    #[test]
    fn test_select_next_wraps() {
        let mut list = SelectableList::empty();
        list.items_mut().extend(vec![1, 2, 3]);
        list.select_first();
        list.select_next();
        list.select_next();
        list.select_next();
        assert_eq!(list.selected(), Some(&1));
    }

    #[test]
    fn test_select_previous_wraps() {
        let mut list = SelectableList::empty();
        list.items_mut().extend(vec![1, 2, 3]);
        list.select_first();
        list.select_previous();
        assert_eq!(list.selected(), Some(&3));
    }

    #[test]
    fn test_select_by() {
        let mut list = SelectableList::empty();
        list.items_mut().extend(vec!["a", "b", "c"]);
        assert!(list.select_by(|&x| x == "b"));
        assert_eq!(list.selected(), Some(&"b"));

        assert!(!list.select_by(|&x| x == "z"));
    }

    #[test]
    fn test_select_none() {
        let mut list = SelectableList::empty();
        list.items_mut().extend(vec![1, 2, 3]);
        list.select_first();
        list.select(None);
        assert_eq!(list.selected(), None);
    }
}
