use std::path::PathBuf;

use clap::Parser;

/// High-performance file synchronizer with a TUI for reviewing changes.
#[derive(Parser, Debug)]
#[command(name = "blakesync", version, about, long_about = None)]
pub struct Cli {
    /// Source directory
    pub source: PathBuf,

    /// Destination directory
    pub destination: PathBuf,

    /// Perform a trial run with no changes made
    #[arg(long)]
    pub dry_run: bool,

    /// Write a log of operations to the given file
    #[arg(long, value_name = "FILE")]
    pub log_file: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_source_and_destination() {
        let cli = Cli::try_parse_from(["blakesync", "src_dir", "dst_dir"]).unwrap();
        assert_eq!(cli.source, PathBuf::from("src_dir"));
        assert_eq!(cli.destination, PathBuf::from("dst_dir"));
        assert!(!cli.dry_run);
        assert_eq!(cli.log_file, None);
    }

    #[test]
    fn parses_dry_run_flag() {
        let cli = Cli::try_parse_from(["blakesync", "src_dir", "dst_dir", "--dry-run"]).unwrap();
        assert!(cli.dry_run);
    }

    #[test]
    fn parses_log_file_option() {
        let cli =
            Cli::try_parse_from(["blakesync", "src_dir", "dst_dir", "--log-file", "sync.log"])
                .unwrap();
        assert_eq!(cli.log_file, Some(PathBuf::from("sync.log")));
    }

    #[test]
    fn missing_required_args_errors() {
        let result = Cli::try_parse_from(["blakesync"]);
        assert!(result.is_err());
    }
}
