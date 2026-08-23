mod cli;
mod engine;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    println!("{:#?}", cli);
}
