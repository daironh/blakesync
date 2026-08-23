# PLAN.md: Technical Architecture
* **cli.rs**: `clap` for args (--dry-run, --log-file).
* **engine.rs**: `ignore` crate for multithreaded traversal. `blake3` for hashing.
* **tui/**: `ratatui` + `crossterm`. Manages `App` state, lists, and keystrokes.
* **executor.rs**: Atomic fs operations, retains mtime, writes to log via `simplelog`.
