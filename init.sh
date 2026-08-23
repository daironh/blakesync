#!/bin/bash
set -e

echo "🚀 Initializing BlakeSync project structure..."

# 1. Initialize Rust and Git (cargo init creates Cargo.toml)
cargo init --name blakesync
echo "/target" > .gitignore
echo "**/*.rs.bk" >> .gitignore
echo "*.log" >> .gitignore

# 2. Inject required dependencies into Cargo.toml
cat << 'EOF' > Cargo.toml
[package]
name = "blakesync"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.6", features = ["derive"] }
ignore = "0.4"
blake3 = "1.5"
ratatui = "0.30"
crossterm = "0.28"
anyhow = "1.0"
log = "0.4"
simplelog = "0.12"
EOF

# 3. Setup Copilot Agent Rules (Optimized for Tokens)
mkdir -p .github
cat << 'EOF' > .github/copilot-instructions.md
# Rust Agent Token & Performance Rules
1. Never read files in the `/target` directory.
2. Keep Rust files under 300 lines. If a file grows larger, refactor into modules to save context window reading time.
3. Only use `cargo check` for syntax verification. Do not run heavy builds during standard agent loops unless testing final compilation.
4. When fixing a compiler error, only read the specific file mentioned in the stack trace. Do not re-read the entire workspace.
EOF

# 4. Create SPEC.md
cat << 'EOF' > SPEC.md
# SPEC.md: High-Performance File Synchronizer
1. **Core:** Compare Source and Destination dirs, categorize by Source Only, Dest Only, Modified.
2. **TUI:** Split-pane interface to tag files (Keep Source, Keep Dest, Ignore). Attribute inspector for size, mtime, BLAKE3 hash.
3. **Execution:** Multithreaded traversal, fast-path metadata check -> BLAKE3 fallback. `--dry-run` and `--log-file` support.
4. **Constraints:** Rust (Stable), Cross-Platform Portable Executable, no daemons, no background watching.
EOF

# 5. Create PLAN.md
cat << 'EOF' > PLAN.md
# PLAN.md: Technical Architecture
* **cli.rs**: `clap` for args (--dry-run, --log-file).
* **engine.rs**: `ignore` crate for multithreaded traversal. `blake3` for hashing.
* **tui/**: `ratatui` + `crossterm`. Manages `App` state, lists, and keystrokes.
* **executor.rs**: Atomic fs operations, retains mtime, writes to log via `simplelog`.
EOF

# 6. Create TASKS.md
cat << 'EOF' > TASKS.md
# TASKS.md: Execution Plan
- [ ] Task 1: Scaffolding & CLI (`clap` args).
- [ ] Task 2: Multithreaded Traversal (`ignore` crate).
- [ ] Task 3: BLAKE3 Hashing utility.
- [ ] Task 4: Comparison Engine (Metadata fast-path -> Hash).
- [ ] Task 5: TUI Scaffolding (`ratatui` init).
- [ ] Task 6: TUI Layout (Split panes, Attribute inspector).
- [ ] Task 7: TUI Keybindings & State tagging.
- [ ] Task 8: Executor Engine (fs operations & progress bar).
- [ ] Task 9: `--dry-run` & Logging hookup.
EOF

echo "✅ Workspace generated successfully with pre-configured Cargo.toml!"