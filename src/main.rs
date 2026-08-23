mod cli;
mod engine;
mod executor;
mod tui;

use std::fs::File;
use std::path::Path;

use clap::Parser;
use cli::Cli;
use simplelog::{Config, LevelFilter, WriteLogger};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_logging(cli.log_file.as_deref())?;

    let results = engine::compare_trees(&cli.source, &cli.destination)?;
    tui::run(cli.source, cli.destination, results, cli.dry_run)
}

/// Sets up file-based logging when `--log-file` is provided. Deliberately
/// never logs to stdout/stderr, since the TUI takes over the terminal via
/// an alternate screen and interleaved log output would corrupt it. With
/// no log file, log::* calls are silently no-ops (no logger installed).
fn init_logging(log_file: Option<&Path>) -> anyhow::Result<()> {
    let Some(path) = log_file else {
        return Ok(());
    };
    let file = File::create(path)?;
    WriteLogger::init(LevelFilter::Info, Config::default(), file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_log_file_is_a_no_op() {
        assert!(init_logging(None).is_ok());
    }

    #[test]
    fn log_file_path_creates_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sync.log");

        init_logging(Some(&path)).unwrap();

        assert!(path.exists());
    }
}
