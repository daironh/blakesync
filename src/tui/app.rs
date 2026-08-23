//! Application state for the TUI, decoupled from any terminal I/O so it can
//! be unit tested directly.

use crossterm::event::{KeyCode, KeyEvent};

use crate::engine::ComparisonEntry;

/// Holds the data and UI state for the interactive review screen.
pub struct App {
    /// The classified files to review, as produced by the comparison engine.
    pub entries: Vec<ComparisonEntry>,
    /// Set to `true` once the user has requested to exit.
    pub should_quit: bool,
}

impl App {
    pub fn new(entries: Vec<ComparisonEntry>) -> Self {
        Self {
            entries,
            should_quit: false,
        }
    }

    /// Updates state in response to a key press. Returns `true` if the key
    /// was handled.
    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn starts_with_provided_entries_and_does_not_quit() {
        let app = App::new(vec![]);
        assert!(app.entries.is_empty());
        assert!(!app.should_quit);
    }

    #[test]
    fn q_requests_quit() {
        let mut app = App::new(vec![]);
        assert!(app.on_key(key(KeyCode::Char('q'))));
        assert!(app.should_quit);
    }

    #[test]
    fn esc_requests_quit() {
        let mut app = App::new(vec![]);
        assert!(app.on_key(key(KeyCode::Esc)));
        assert!(app.should_quit);
    }

    #[test]
    fn other_keys_are_ignored() {
        let mut app = App::new(vec![]);
        assert!(!app.on_key(key(KeyCode::Char('x'))));
        assert!(!app.should_quit);
    }
}
