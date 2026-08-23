//! Application state for the TUI, decoupled from any terminal I/O so it can
//! be unit tested directly.

use std::collections::HashMap;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};

use crate::engine::{hash_file, ComparisonEntry};

/// The user's resolution choice for a single file, keyed by its relative
/// path in [`App::tags`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tag {
    KeepSource,
    KeepDest,
    Ignore,
}

/// A cached BLAKE3 digest pair, computed on demand for the currently
/// selected file (source hash, destination hash). Either side may be
/// `None` if that side of the file doesn't exist or failed to hash.
pub type HashPair = (Option<blake3::Hash>, Option<blake3::Hash>);

/// Holds the data and UI state for the interactive review screen.
pub struct App {
    /// The classified files to review, as produced by the comparison engine.
    pub entries: Vec<ComparisonEntry>,
    /// Index into `entries` for the row shown in the attribute inspector.
    pub selected: usize,
    /// Set to `true` once the user has requested to exit.
    pub should_quit: bool,
    /// User-assigned resolution per file, keyed by relative path.
    pub tags: HashMap<PathBuf, Tag>,
    /// On-demand BLAKE3 hashes, keyed by relative path, computed via the
    /// 'h' keybinding rather than eagerly for every file.
    pub hash_cache: HashMap<PathBuf, HashPair>,
    source_root: PathBuf,
    dest_root: PathBuf,
}

impl App {
    pub fn new(source_root: PathBuf, dest_root: PathBuf, entries: Vec<ComparisonEntry>) -> Self {
        Self {
            entries,
            selected: 0,
            should_quit: false,
            tags: HashMap::new(),
            hash_cache: HashMap::new(),
            source_root,
            dest_root,
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
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection(1);
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection(-1);
                true
            }
            KeyCode::Char('s') => {
                self.tag_selected(Tag::KeepSource);
                true
            }
            KeyCode::Char('d') => {
                self.tag_selected(Tag::KeepDest);
                true
            }
            KeyCode::Char('i') => {
                self.tag_selected(Tag::Ignore);
                true
            }
            KeyCode::Char('h') => {
                self.hash_selected();
                true
            }
            _ => false,
        }
    }

    fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        let last = self.entries.len() as isize - 1;
        let next = (self.selected as isize + delta).clamp(0, last);
        self.selected = next as usize;
    }

    fn tag_selected(&mut self, tag: Tag) {
        if let Some(entry) = self.entries.get(self.selected) {
            self.tags.insert(entry.rel_path.clone(), tag);
        }
    }

    /// Computes and caches the BLAKE3 hash of the selected entry's source
    /// and/or destination file, if present. A no-op if already cached.
    fn hash_selected(&mut self) {
        let Some(entry) = self.entries.get(self.selected) else {
            return;
        };
        let rel_path = entry.rel_path.clone();
        if self.hash_cache.contains_key(&rel_path) {
            return;
        }

        let source_hash = entry
            .source
            .as_ref()
            .and_then(|_| hash_file(&self.source_root.join(&rel_path)).ok());
        let dest_hash = entry
            .dest
            .as_ref()
            .and_then(|_| hash_file(&self.dest_root.join(&rel_path)).ok());

        self.hash_cache.insert(rel_path, (source_hash, dest_hash));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{CompareStatus, FileEntry};
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};
    use std::fs;
    use std::time::SystemTime;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn entry(name: &str) -> ComparisonEntry {
        ComparisonEntry {
            rel_path: PathBuf::from(name),
            status: CompareStatus::Modified,
            source: Some(FileEntry {
                rel_path: PathBuf::from(name),
                size: 1,
                mtime: SystemTime::now(),
            }),
            dest: Some(FileEntry {
                rel_path: PathBuf::from(name),
                size: 1,
                mtime: SystemTime::now(),
            }),
        }
    }

    fn app_with(entries: Vec<ComparisonEntry>) -> App {
        App::new(PathBuf::from("/src"), PathBuf::from("/dst"), entries)
    }

    #[test]
    fn starts_with_provided_entries_and_does_not_quit() {
        let app = app_with(vec![]);
        assert!(app.entries.is_empty());
        assert_eq!(app.selected, 0);
        assert!(!app.should_quit);
        assert!(app.tags.is_empty());
    }

    #[test]
    fn q_and_esc_request_quit() {
        let mut app = app_with(vec![]);
        assert!(app.on_key(key(KeyCode::Char('q'))));
        assert!(app.should_quit);

        let mut app = app_with(vec![]);
        assert!(app.on_key(key(KeyCode::Esc)));
        assert!(app.should_quit);
    }

    #[test]
    fn other_keys_are_ignored() {
        let mut app = app_with(vec![]);
        assert!(!app.on_key(key(KeyCode::Char('x'))));
        assert!(!app.should_quit);
    }

    #[test]
    fn navigation_clamps_within_bounds() {
        let mut app = app_with(vec![entry("a"), entry("b"), entry("c")]);
        assert_eq!(app.selected, 0);

        app.on_key(key(KeyCode::Up)); // clamp at 0
        assert_eq!(app.selected, 0);

        app.on_key(key(KeyCode::Down));
        assert_eq!(app.selected, 1);
        app.on_key(key(KeyCode::Char('j')));
        assert_eq!(app.selected, 2);
        app.on_key(key(KeyCode::Down)); // clamp at last index
        assert_eq!(app.selected, 2);

        app.on_key(key(KeyCode::Char('k')));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn navigation_on_empty_entries_is_a_no_op() {
        let mut app = app_with(vec![]);
        app.on_key(key(KeyCode::Down));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn tagging_keys_record_resolution_for_selected_entry() {
        let mut app = app_with(vec![entry("a"), entry("b")]);

        app.on_key(key(KeyCode::Char('s')));
        assert_eq!(app.tags.get(&PathBuf::from("a")), Some(&Tag::KeepSource));

        app.on_key(key(KeyCode::Down));
        app.on_key(key(KeyCode::Char('d')));
        assert_eq!(app.tags.get(&PathBuf::from("b")), Some(&Tag::KeepDest));

        app.on_key(key(KeyCode::Char('i')));
        assert_eq!(app.tags.get(&PathBuf::from("b")), Some(&Tag::Ignore));
    }

    #[test]
    fn hash_key_computes_and_caches_hashes_for_selected_entry() {
        let dir = tempfile::tempdir().unwrap();
        let source_root = dir.path().join("source");
        let dest_root = dir.path().join("dest");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(&dest_root).unwrap();
        fs::write(source_root.join("f.txt"), b"same content").unwrap();
        fs::write(dest_root.join("f.txt"), b"same content").unwrap();

        let mut app = App::new(source_root, dest_root, vec![entry("f.txt")]);
        app.on_key(key(KeyCode::Char('h')));

        let (src_hash, dst_hash) = app.hash_cache.get(&PathBuf::from("f.txt")).unwrap();
        assert!(src_hash.is_some());
        assert_eq!(src_hash, dst_hash);
    }
}
