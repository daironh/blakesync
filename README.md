# BlakeSync ⚡

[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Built with Rust](https://img.shields.io/badge/Built_with-Rust-orange.svg)](https://www.rust-lang.org/)

A high-speed, local-first file synchronization and comparison tool. Written in Rust to leverage zero-cost abstractions, multithreaded directory traversal, and C-level I/O performance.

BlakeSync takes a Source and Destination directory, rapidly compares their contents using metadata and BLAKE3 hashing, and presents the differences in an interactive Terminal User Interface (TUI). You retain absolute control over the final state, manually approving exactly which files are copied, moved, or deleted.

## 🧠 The Agentic Workflow Experiment

This project serves as a live demonstration of **Spec-Driven Development (SDD)** and autonomous AI engineering. It was architected by a human and implemented entirely by an AI agent (Claude 3.5 Sonnet via GitHub Copilot Agent). 

By establishing strict specifications (`SPEC.md`), a decoupled architecture (`PLAN.md`), and token-optimized execution rules, the agent autonomously navigated Rust's strict borrow checker to build a memory-safe, multithreaded systems tool. 

## ✨ Key Features

*   **Multithreaded Traversal:** Capable of mapping directories with 100,000+ files in fractions of a second using the `ignore` crate.
*   **BLAKE3 Hashing Engine:** Uses a fast-path metadata check (size + modified time), falling back to cryptographic BLAKE3 hashing for ultra-fast content verification.
*   **Interactive TUI:** A responsive split-pane interface built with `ratatui` for tagging and approving file operations.
*   **Atomic Executor:** Safe execution engine with a real-time progress bar and comprehensive file logging.
*   **Dry-Run Mode:** Safely audit operations before committing any filesystem changes.

## 🚀 Getting Started

### Prerequisites
*   [Rust toolchain](https://rustup.rs/) (stable)
*   Windows (S Mode disabled) or standard Linux/macOS environments.

### Installation
Clone the repository and build from source:
```bash
git clone https://github.com/daironh/blakesync.git
cd blakesync
cargo build --release
```

The compiled binary will be at `target/release/blakesync`.

### Usage
BlakeSync takes a source and a destination directory and opens the TUI once the initial comparison finishes:

```bash
blakesync <SOURCE_DIR> <DEST_DIR>
```

Options:
| Flag | Description |
| --- | --- |
| `--dry-run` | Plan and log every action without touching the filesystem. |
| `--log-file <FILE>` | Write execution logs to `FILE`. Required if you want logging, since the TUI owns the terminal and can't print to stdout. |

Example:
```bash
cargo run --release -- ./source ./dest --dry-run --log-file sync.log
```

### TUI Keybindings
| Key | Action |
| --- | --- |
| `↑`/`k`, `↓`/`j` | Move selection in the file list |
| `s` | Tag selected entry: keep source |
| `d` | Tag selected entry: keep destination |
| `i` | Tag selected entry: ignore |
| `h` | Compute/cache a BLAKE3 hash for the selected entry |
| `Enter` | Execute the planned actions (respects `--dry-run`) with a live progress bar |
| `q` / `Esc` | Quit |

### Running Tests
```bash
cargo test
```