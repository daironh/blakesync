//! The user's resolution decision for a single file's conflict, shared
//! between the TUI (which sets it via keybindings) and the executor
//! (which applies it to the filesystem).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resolution {
    /// Keep the source tree's version; overwrite the destination.
    KeepSource,
    /// Keep the destination tree's version; overwrite the source.
    KeepDest,
    /// Leave both sides untouched.
    Ignore,
}
