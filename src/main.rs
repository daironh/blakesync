mod cli;
mod engine;

use clap::Parser;
use cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let results = engine::compare_trees(&cli.source, &cli.destination)?;
    for entry in &results {
        println!("{:?}\t{}", entry.status, entry.rel_path.display());
    }
    Ok(())
}
