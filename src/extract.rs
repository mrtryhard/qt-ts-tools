use clap::Args;

#[derive(Args)]
pub struct ExtractArgs {
    pub input_path: String,
    #[arg(short('t'), long, value_enum, num_args = 1..)]
    pub translation_type: Vec<TranslationType>,
    #[arg(short, long)]
    pub output_path: Option<String>,
}

#[derive(clap::ValueEnum, PartialEq, Debug, Clone)]
pub enum TranslationType {
    Obsolete,
    Unfinished,
    Vanished,
}
