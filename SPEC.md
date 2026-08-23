# SPEC.md: High-Performance File Synchronizer
1. **Core:** Compare Source and Destination dirs, categorize by Source Only, Dest Only, Modified.
2. **TUI:** Split-pane interface to tag files (Keep Source, Keep Dest, Ignore). Attribute inspector for size, mtime, BLAKE3 hash.
3. **Execution:** Multithreaded traversal, fast-path metadata check -> BLAKE3 fallback. `--dry-run` and `--log-file` support.
4. **Constraints:** Rust (Stable), Cross-Platform Portable Executable, no daemons, no background watching.
