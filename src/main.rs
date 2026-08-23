mod cli;
mod engine;
mod tui;

use clap::Parser;
use cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let results = engine::compare_trees(&cli.source, &cli.destination)?;
    tui::run(cli.source, cli.destination, results)
}
