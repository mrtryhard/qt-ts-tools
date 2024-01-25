// https://sts10.github.io/2023/01/29/sorting-words-alphabetically-rust.html
use clap::Args;

#[derive(Args)]
pub struct SortArgs {
    pub input_path: String,
    pub output_path: Option<String>,
}

pub fn sort_main(args: &SortArgs) -> Result<(), String> {
    Ok(())
}
