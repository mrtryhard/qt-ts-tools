mod extract;
mod sort;
mod ts;

use crate::extract::{extract_main, ExtractArgs};
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
    Extract(ExtractArgs),
}

fn get_cli_result(cli: Cli) -> Result<(), String> {
    match &cli.command {
        Commands::Sort(args) => sort_main(&args),
        Commands::Extract(args) => extract_main(&args),
    }
}

fn main() {
    if let Err(e) = get_cli_result(Cli::parse()) {
        println!("{e}");
        std::process::exit(1);
    }
}
