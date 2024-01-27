mod sort;
mod ts_definition;

use crate::sort::{sort_main, SortArgs};
use clap::{Parser, Subcommand};

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

    if let Err(e) = result {
        println!("{e}");
        std::process::exit(1);
    }
}
