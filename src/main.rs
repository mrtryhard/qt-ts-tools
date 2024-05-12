use crate::cli::get_cli_result;

mod cli;
mod extract;
mod locale;
mod merge;
mod sort;
mod ts;

fn main() {
    if let Err(e) = get_cli_result() {
        println!("{e}");
        std::process::exit(1);
    }
}
