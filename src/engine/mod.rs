//! Core synchronization engine: directory traversal, hashing, and comparison.
//!
//! Re-exports form the public API consumed by the TUI (Tasks 5-7) and the
//! executor (Task 8); some are not yet referenced by name until those
//! modules land.
#![allow(unused_imports)]

pub mod compare;
pub mod hash;
pub mod walk;

pub use compare::{compare_trees, CompareStatus, ComparisonEntry};
pub use hash::hash_file;
pub use walk::{walk_dir, FileEntry};
