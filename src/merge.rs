use crate::ts;
use crate::ts::TSNode;
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

pub fn merge_main(args: &MergeArgs) -> Result<(), String> {
    let left: Result<TSNode, _> = quick_xml::Reader::from_file(&args.input_left)
         .and_then(|file| quick_xml::de::from_reader(file.into_inner())?);
    let right: Result<TSNode, _> = quick_xml::Reader::from_file(&args.input_right)
         .and_then(|file| quick_xml::de::from_reader(file.into_inner())?);

    if let Err(e) = left {
         return Err(format!("Could not process left file '{}'. Error: {}", &args.input_left, e.to_string()));
    }

    if let Err(e) = right {
         return Err(format!("Could not process right file '{}'. Error: {}", &args.input_right, e.to_string()));
    }

    let mut left = left.unwrap();
    let right = right.unwrap();

    // Priority is always `right` wins.
    // Discriminant is `source`
    // Get messages that are completely different: different, non-matching source
    // Get messages that are common but has different properties
    // Add missing message in left from right
    // Update left messages from right, that matches on right (common message gets updated)

    Err("Merge is not yet implemented.".to_owned())
}
