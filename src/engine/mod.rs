//! Core synchronization engine: directory traversal, hashing, and comparison.

pub mod hash;
pub mod walk;

pub use hash::hash_file;
pub use walk::{walk_dir, FileEntry};
