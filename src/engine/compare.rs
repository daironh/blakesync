//! Comparison engine: classifies files between a source and destination
//! tree using a cheap metadata fast-path, falling back to BLAKE3 hashing
//! only when size and mtime cannot settle the question on their own.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::engine::hash::hash_file;
use crate::engine::walk::{walk_dir, FileEntry};

/// The relationship between a file in the source tree and the same
/// relative path in the destination tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareStatus {
    /// Exists only in the source tree.
    SourceOnly,
    /// Exists only in the destination tree.
    DestOnly,
    /// Exists in both trees and is considered identical.
    Same,
    /// Exists in both trees but differs.
    Modified,
}

/// A single classified entry, keyed by its path relative to both roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonEntry {
    pub rel_path: PathBuf,
    pub status: CompareStatus,
}

/// Walks `source_root` and `dest_root`, then classifies every file found
/// in either tree as [`CompareStatus::SourceOnly`], [`CompareStatus::DestOnly`],
/// [`CompareStatus::Same`], or [`CompareStatus::Modified`].
///
/// For files present in both trees, a fast metadata check (size + mtime)
/// is tried first; a BLAKE3 hash comparison is only performed when sizes
/// match but mtimes differ, since that's the only case metadata can't
/// settle on its own.
pub fn compare_trees(source_root: &Path, dest_root: &Path) -> anyhow::Result<Vec<ComparisonEntry>> {
    let source_map = to_map(walk_dir(source_root)?);
    let dest_map = to_map(walk_dir(dest_root)?);

    let all_paths: BTreeSet<PathBuf> = source_map.keys().chain(dest_map.keys()).cloned().collect();

    let mut results = Vec::with_capacity(all_paths.len());
    for rel_path in all_paths {
        let status = match (source_map.get(&rel_path), dest_map.get(&rel_path)) {
            (Some(_), None) => CompareStatus::SourceOnly,
            (None, Some(_)) => CompareStatus::DestOnly,
            (Some(src), Some(dst)) => classify(source_root, dest_root, &rel_path, src, dst)?,
            (None, None) => unreachable!("path was collected from one of the two maps"),
        };
        results.push(ComparisonEntry { rel_path, status });
    }

    Ok(results)
}

fn to_map(entries: Vec<FileEntry>) -> HashMap<PathBuf, FileEntry> {
    entries.into_iter().map(|e| (e.rel_path.clone(), e)).collect()
}

fn classify(
    source_root: &Path,
    dest_root: &Path,
    rel_path: &Path,
    src: &FileEntry,
    dst: &FileEntry,
) -> anyhow::Result<CompareStatus> {
    if src.size != dst.size {
        return Ok(CompareStatus::Modified);
    }
    if src.mtime == dst.mtime {
        return Ok(CompareStatus::Same);
    }

    // Same size but different mtime: metadata is ambiguous, fall back to hashing.
    let src_hash = hash_file(&source_root.join(rel_path))?;
    let dst_hash = hash_file(&dest_root.join(rel_path))?;
    Ok(if src_hash == dst_hash {
        CompareStatus::Same
    } else {
        CompareStatus::Modified
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::{Duration, SystemTime};

    fn set_mtime(path: &Path, time: SystemTime) {
        File::open(path).unwrap().set_modified(time).unwrap();
    }

    fn find<'a>(entries: &'a [ComparisonEntry], name: &str) -> &'a ComparisonEntry {
        entries
            .iter()
            .find(|e| e.rel_path == PathBuf::from(name))
            .unwrap_or_else(|| panic!("entry {name} not found in {entries:?}"))
    }

    #[test]
    fn classifies_source_only_and_dest_only() {
        let source = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        fs::write(source.path().join("only_in_source.txt"), b"a").unwrap();
        fs::write(dest.path().join("only_in_dest.txt"), b"b").unwrap();

        let results = compare_trees(source.path(), dest.path()).unwrap();

        assert_eq!(find(&results, "only_in_source.txt").status, CompareStatus::SourceOnly);
        assert_eq!(find(&results, "only_in_dest.txt").status, CompareStatus::DestOnly);
    }

    #[test]
    fn same_size_and_mtime_is_same_without_hashing() {
        let source = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        let common_time = SystemTime::now() - Duration::from_secs(60);

        fs::write(source.path().join("same.txt"), b"identical").unwrap();
        fs::write(dest.path().join("same.txt"), b"identical").unwrap();
        set_mtime(&source.path().join("same.txt"), common_time);
        set_mtime(&dest.path().join("same.txt"), common_time);

        let results = compare_trees(source.path(), dest.path()).unwrap();
        assert_eq!(find(&results, "same.txt").status, CompareStatus::Same);
    }

    #[test]
    fn different_size_is_modified() {
        let source = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        fs::write(source.path().join("changed.txt"), b"short").unwrap();
        fs::write(dest.path().join("changed.txt"), b"much longer content").unwrap();

        let results = compare_trees(source.path(), dest.path()).unwrap();
        assert_eq!(find(&results, "changed.txt").status, CompareStatus::Modified);
    }

    #[test]
    fn same_size_different_mtime_falls_back_to_hash_and_matches() {
        let source = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        fs::write(source.path().join("touched.txt"), b"same bytes").unwrap();
        fs::write(dest.path().join("touched.txt"), b"same bytes").unwrap();
        set_mtime(&source.path().join("touched.txt"), SystemTime::now() - Duration::from_secs(120));
        set_mtime(&dest.path().join("touched.txt"), SystemTime::now());

        let results = compare_trees(source.path(), dest.path()).unwrap();
        assert_eq!(find(&results, "touched.txt").status, CompareStatus::Same);
    }

    #[test]
    fn same_size_different_mtime_and_content_is_modified() {
        let source = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        fs::write(source.path().join("edited.txt"), b"aaaaaaaaaa").unwrap();
        fs::write(dest.path().join("edited.txt"), b"bbbbbbbbbb").unwrap();
        set_mtime(&source.path().join("edited.txt"), SystemTime::now() - Duration::from_secs(120));
        set_mtime(&dest.path().join("edited.txt"), SystemTime::now());

        let results = compare_trees(source.path(), dest.path()).unwrap();
        assert_eq!(find(&results, "edited.txt").status, CompareStatus::Modified);
    }
}
