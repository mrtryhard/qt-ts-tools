mod sort;
mod ts_definition;

use crate::sort::{sort_main, SortArgs};
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Sort(SortArgs),
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Sort(args) => sort_main(&args),
    };

    println!("Hello, world!");
}
