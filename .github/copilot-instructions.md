# Rust Agent Token & Performance Rules
1. Never read files in the `/target` directory.
2. Keep Rust files under 300 lines. If a file grows larger, refactor into modules to save context window reading time.
3. Only use `cargo check` for syntax verification. Do not run heavy builds during standard agent loops unless testing final compilation.
4. When fixing a compiler error, only read the specific file mentioned in the stack trace. Do not re-read the entire workspace.
