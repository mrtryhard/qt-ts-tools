use clap::Args;

#[derive(Args)]
pub struct MergeArgs {
    pub input_left: String,
    pub input_right: String,
    #[arg(short, long)]
    pub output_path: Option<String>,
}

pub fn merge_main(_args: &MergeArgs) -> Result<(), String> {
    Err("Merge is not yet implemented.".to_owned())
}
