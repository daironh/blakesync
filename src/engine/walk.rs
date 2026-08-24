//! Multithreaded directory traversal built on the `ignore` crate.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use ignore::{WalkBuilder, WalkState};

/// Metadata for a single regular file discovered during a traversal,
/// captured relative to the root that was walked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// Path relative to the traversal root (used as the comparison key).
    pub rel_path: PathBuf,
    /// File size in bytes.
    pub size: u64,
    /// Last-modified time reported by the filesystem.
    pub mtime: SystemTime,
}

/// Recursively walks `root` using a thread pool (via the `ignore` crate's
/// parallel walker) and returns metadata for every regular file found.
///
/// Hidden files and VCS-ignored files are intentionally included: a file
/// synchronizer must consider every file on disk, regardless of
/// `.gitignore` rules.
pub fn walk_dir(root: &Path) -> anyhow::Result<Vec<FileEntry>> {
    let entries: Arc<Mutex<Vec<FileEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let first_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(false)
        .git_exclude(false)
        .git_global(false)
        .ignore(false)
        .threads(threads)
        .build_parallel();

    let root = root.to_path_buf();
    walker.run(|| {
        let root = root.clone();
        let entries = Arc::clone(&entries);
        let first_error = Arc::clone(&first_error);
        Box::new(move |result| {
            match result {
                Ok(entry) => {
                    let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                    if is_file {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(rel_path) = entry.path().strip_prefix(&root) {
                                entries.lock().unwrap().push(FileEntry {
                                    rel_path: rel_path.to_path_buf(),
                                    size: metadata.len(),
                                    mtime: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                                });
                            }
                        }
                    }
                }
                Err(err) => {
                    let mut slot = first_error.lock().unwrap();
                    if slot.is_none() {
                        *slot = Some(err.to_string());
                    }
                    return WalkState::Quit;
                }
            }
            WalkState::Continue
        })
    });

    let maybe_error = Arc::try_unwrap(first_error)
        .map_err(|_| anyhow::anyhow!("walker threads did not release traversal error state"))?
        .into_inner()
        .map_err(|_| anyhow::anyhow!("traversal error mutex was poisoned"))?;
    if let Some(error) = maybe_error {
        anyhow::bail!("failed to walk '{}': {}", root.display(), error);
    }

    let entries = Arc::try_unwrap(entries)
        .map_err(|_| anyhow::anyhow!("walker threads did not release entries"))?
        .into_inner()
        .map_err(|_| anyhow::anyhow!("entries mutex was poisoned"))?;

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn walks_nested_files_and_reports_size() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("a.txt"), b"hello").unwrap();
        fs::write(dir.path().join("sub/b.txt"), b"world!").unwrap();

        let mut entries = walk_dir(dir.path()).unwrap();
        entries.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].rel_path, PathBuf::from("a.txt"));
        assert_eq!(entries[0].size, 5);
        assert_eq!(entries[1].rel_path, PathBuf::from("sub/b.txt"));
        assert_eq!(entries[1].size, 6);
    }

    #[test]
    fn empty_directory_yields_no_entries() {
        let dir = tempfile::tempdir().unwrap();
        let entries = walk_dir(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn includes_hidden_and_gitignored_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), b"ignored.txt\n").unwrap();
        fs::write(dir.path().join("ignored.txt"), b"should still appear").unwrap();
        fs::write(dir.path().join(".hidden"), b"also present").unwrap();

        let entries = walk_dir(dir.path()).unwrap();
        let names: Vec<_> = entries
            .iter()
            .map(|e| e.rel_path.to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"ignored.txt".to_string()));
        assert!(names.contains(&".hidden".to_string()));
    }
}
