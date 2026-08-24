mod cli;
mod engine;
mod executor;
mod tui;

use std::fs::File;
use std::path::Path;

use anyhow::{bail, Context};
use clap::Parser;
use cli::Cli;
use simplelog::{Config, LevelFilter, WriteLogger};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_logging(cli.log_file.as_deref())?;

    validate_input_root("source", &cli.source)?;
    validate_input_root("destination", &cli.destination)?;

    let results = engine::compare_trees(&cli.source, &cli.destination).with_context(|| {
        format!(
            "failed to compare source '{}' with destination '{}'",
            cli.source.display(),
            cli.destination.display()
        )
    })?;
    tui::run(cli.source, cli.destination, results, cli.dry_run)
}

fn validate_input_root(label: &str, path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        bail!("{label} path does not exist: {}", path.display());
    }
    if !path.is_dir() {
        bail!("{label} path is not a directory: {}", path.display());
    }
    Ok(())
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

    #[test]
    fn validate_input_root_rejects_missing_path() {
        let missing = Path::new("/definitely/missing/blakesync-test-path");
        assert!(validate_input_root("source", missing).is_err());
    }

    #[test]
    fn validate_input_root_rejects_files() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("not_a_directory");
        std::fs::write(&file_path, b"x").unwrap();

        assert!(validate_input_root("destination", &file_path).is_err());
    }
}
