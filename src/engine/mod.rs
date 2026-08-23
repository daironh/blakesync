//! Core synchronization engine: directory traversal, hashing, comparison,
//! and the shared `Resolution` type consumed by both the TUI and executor.

pub mod compare;
pub mod hash;
pub mod resolution;
pub mod walk;

pub use compare::{compare_trees, CompareStatus, ComparisonEntry};
pub use hash::hash_file;
pub use resolution::Resolution;
// walk_dir/FileEntry are only referenced by name in test code (via this
// flattened path); production code goes through compare_trees instead.
#[allow(unused_imports)]
pub use walk::{walk_dir, FileEntry};
