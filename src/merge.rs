use clap::Args;

/// Merges two translation file contexts and messages into a single output.
#[derive(Args)]
pub struct MergeArgs {
    /// File to receive the merge
    pub input_left: String,
    /// File to include changes from
    pub input_right: String,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long)]
    pub output_path: Option<String>,
}

pub fn merge_main(_args: &MergeArgs) -> Result<(), String> {
    Err("Merge is not yet implemented.".to_owned())
}
