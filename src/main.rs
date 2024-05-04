mod cli;
mod extract;
mod merge;
mod sort;
mod ts;

use crate::cli::get_cli_result;

fn main() {
    if let Err(e) = get_cli_result() {
        println!("{e}");
        std::process::exit(1);
    }
}
