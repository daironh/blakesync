//! Executes the user's tagged resolutions: copies files between the
//! source and destination trees, preserving mtimes, and reports progress
//! as it runs so callers (e.g. the TUI) can render a progress bar.
//!
//! Not yet wired into the TUI's event loop or the `--dry-run`/`--log-file`
//! CLI flags; that hookup lands in Task 9.
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::engine::{ComparisonEntry, Resolution};

/// A single planned filesystem action, derived from a comparison entry and
/// its user-assigned resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Copy the file from the source tree to the destination tree.
    CopySourceToDest(PathBuf),
    /// Copy the file from the destination tree to the source tree.
    CopyDestToSource(PathBuf),
    /// No filesystem change requested (untagged or explicitly ignored).
    Skip(PathBuf),
}

/// Outcome of executing a single action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResult {
    pub rel_path: PathBuf,
    pub applied: bool,
    pub message: String,
}

/// Builds the ordered list of actions to perform from the comparison
/// results and the user's tags. Untagged entries are skipped, since the
/// user hasn't made a decision for them yet.
pub fn plan_actions(entries: &[ComparisonEntry], tags: &HashMap<PathBuf, Resolution>) -> Vec<Action> {
    entries
        .iter()
        .map(|entry| match tags.get(&entry.rel_path) {
            Some(Resolution::KeepSource) => Action::CopySourceToDest(entry.rel_path.clone()),
            Some(Resolution::KeepDest) => Action::CopyDestToSource(entry.rel_path.clone()),
            Some(Resolution::Ignore) | None => Action::Skip(entry.rel_path.clone()),
        })
        .collect()
}

/// Applies a single action against the given roots. When `dry_run` is
/// `true`, no filesystem writes are performed; the result still reports
/// what *would* have happened.
pub fn apply_action(
    action: &Action,
    source_root: &Path,
    dest_root: &Path,
    dry_run: bool,
) -> anyhow::Result<ActionResult> {
    match action {
        Action::Skip(rel_path) => Ok(ActionResult {
            rel_path: rel_path.clone(),
            applied: false,
            message: "skipped".to_string(),
        }),
        Action::CopySourceToDest(rel_path) => {
            copy_preserving_mtime(&source_root.join(rel_path), &dest_root.join(rel_path), rel_path, dry_run)
        }
        Action::CopyDestToSource(rel_path) => {
            copy_preserving_mtime(&dest_root.join(rel_path), &source_root.join(rel_path), rel_path, dry_run)
        }
    }
}

/// Runs every action in `actions` in order, invoking `on_progress(completed,
/// total)` after each one completes.
pub fn execute_plan(
    actions: &[Action],
    source_root: &Path,
    dest_root: &Path,
    dry_run: bool,
    mut on_progress: impl FnMut(usize, usize),
) -> anyhow::Result<Vec<ActionResult>> {
    let total = actions.len();
    let mut results = Vec::with_capacity(total);
    for (i, action) in actions.iter().enumerate() {
        results.push(apply_action(action, source_root, dest_root, dry_run)?);
        on_progress(i + 1, total);
    }
    Ok(results)
}

fn copy_preserving_mtime(from: &Path, to: &Path, rel_path: &Path, dry_run: bool) -> anyhow::Result<ActionResult> {
    if dry_run {
        return Ok(ActionResult {
            rel_path: rel_path.to_path_buf(),
            applied: false,
            message: format!("dry-run: would copy {} -> {}", from.display(), to.display()),
        });
    }

    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(from, to)?;
    let mtime = fs::metadata(from)?.modified()?;
    fs::File::open(to)?.set_modified(mtime)?;

    Ok(ActionResult {
        rel_path: rel_path.to_path_buf(),
        applied: true,
        message: format!("copied {} -> {}", from.display(), to.display()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn entry(name: &str) -> ComparisonEntry {
        use crate::engine::{CompareStatus, FileEntry};
        use std::time::SystemTime;
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

    #[test]
    fn plan_actions_maps_tags_to_actions_and_defaults_untagged_to_skip() {
        let entries = vec![entry("keep_source.txt"), entry("keep_dest.txt"), entry("ignored.txt"), entry("untagged.txt")];
        let mut tags = HashMap::new();
        tags.insert(PathBuf::from("keep_source.txt"), Resolution::KeepSource);
        tags.insert(PathBuf::from("keep_dest.txt"), Resolution::KeepDest);
        tags.insert(PathBuf::from("ignored.txt"), Resolution::Ignore);

        let actions = plan_actions(&entries, &tags);

        assert_eq!(actions[0], Action::CopySourceToDest(PathBuf::from("keep_source.txt")));
        assert_eq!(actions[1], Action::CopyDestToSource(PathBuf::from("keep_dest.txt")));
        assert_eq!(actions[2], Action::Skip(PathBuf::from("ignored.txt")));
        assert_eq!(actions[3], Action::Skip(PathBuf::from("untagged.txt")));
    }

    #[test]
    fn copy_source_to_dest_overwrites_dest_and_preserves_mtime() {
        let dir = tempfile::tempdir().unwrap();
        let source_root = dir.path().join("source");
        let dest_root = dir.path().join("dest");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(&dest_root).unwrap();
        fs::write(source_root.join("f.txt"), b"source content").unwrap();
        fs::write(dest_root.join("f.txt"), b"stale dest content").unwrap();

        let mtime = std::time::SystemTime::now() - Duration::from_secs(3600);
        fs::File::open(source_root.join("f.txt")).unwrap().set_modified(mtime).unwrap();

        let action = Action::CopySourceToDest(PathBuf::from("f.txt"));
        let result = apply_action(&action, &source_root, &dest_root, false).unwrap();

        assert!(result.applied);
        assert_eq!(fs::read(dest_root.join("f.txt")).unwrap(), b"source content");
        let dest_mtime = fs::metadata(dest_root.join("f.txt")).unwrap().modified().unwrap();
        assert_eq!(dest_mtime, mtime);
    }

    #[test]
    fn copy_dest_to_source_creates_missing_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let source_root = dir.path().join("source");
        let dest_root = dir.path().join("dest");
        fs::create_dir_all(&dest_root.join("nested")).unwrap();
        fs::create_dir_all(&source_root).unwrap();
        fs::write(dest_root.join("nested/f.txt"), b"dest content").unwrap();

        let action = Action::CopyDestToSource(PathBuf::from("nested/f.txt"));
        let result = apply_action(&action, &source_root, &dest_root, false).unwrap();

        assert!(result.applied);
        assert_eq!(fs::read(source_root.join("nested/f.txt")).unwrap(), b"dest content");
    }

    #[test]
    fn dry_run_does_not_touch_the_filesystem() {
        let dir = tempfile::tempdir().unwrap();
        let source_root = dir.path().join("source");
        let dest_root = dir.path().join("dest");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(&dest_root).unwrap();
        fs::write(source_root.join("f.txt"), b"source content").unwrap();

        let action = Action::CopySourceToDest(PathBuf::from("f.txt"));
        let result = apply_action(&action, &source_root, &dest_root, true).unwrap();

        assert!(!result.applied);
        assert!(!dest_root.join("f.txt").exists());
    }

    #[test]
    fn skip_action_never_touches_the_filesystem() {
        let dir = tempfile::tempdir().unwrap();
        let result = apply_action(&Action::Skip(PathBuf::from("x.txt")), dir.path(), dir.path(), false).unwrap();
        assert!(!result.applied);
    }

    #[test]
    fn execute_plan_reports_progress_for_every_action() {
        let dir = tempfile::tempdir().unwrap();
        let source_root = dir.path().join("source");
        let dest_root = dir.path().join("dest");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(&dest_root).unwrap();
        fs::write(source_root.join("a.txt"), b"a").unwrap();
        fs::write(source_root.join("b.txt"), b"b").unwrap();

        let actions = vec![
            Action::CopySourceToDest(PathBuf::from("a.txt")),
            Action::CopySourceToDest(PathBuf::from("b.txt")),
        ];

        let mut progress_calls = Vec::new();
        let results = execute_plan(&actions, &source_root, &dest_root, false, |done, total| {
            progress_calls.push((done, total));
        })
        .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(progress_calls, vec![(1, 2), (2, 2)]);
    }
}
