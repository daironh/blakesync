//! Core synchronization engine: directory traversal, hashing, and comparison.

pub mod walk;

pub use walk::{walk_dir, FileEntry};
