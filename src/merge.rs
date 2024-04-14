use crate::ts;
use crate::ts::{MessageNode, TSNode};
use clap::Args;
use itertools::Itertools;
use std::hash::{Hash, Hasher};

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

#[derive(Eq, PartialOrd, Clone)]
struct NodeGroup {
    pub node: MessageNode,
}

impl PartialEq for NodeGroup {
    fn eq(&self, other: &Self) -> bool {
        self.node.source == other.node.source && self.node.locations == other.node.locations
    }
}

impl Hash for NodeGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.source.hash(state);
        self.node.locations.iter().for_each(|loc| {
            loc.line.hash(state);
            loc.filename.hash(state);
        });
    }
}

// This works by depending on cmp looking up only source and location on messages nodes
// and on context by comparing the names only
pub fn merge_main(args: &MergeArgs) -> Result<(), String> {
    let left = load_file(&args.input_left);
    let right = load_file(&args.input_right);

    if let Err(e) = left {
        return Err(format!(
            "Could not process left file '{}'. Error: {}",
            &args.input_left,
            e.to_string()
        ));
    }

    if let Err(e) = right {
        return Err(format!(
            "Could not process right file '{}'. Error: {}",
            &args.input_right,
            e.to_string()
        ));
    }

    let mut right = right.unwrap();
    let mut left = left.unwrap();

    left.messages = merge_messages(&mut left.messages, &mut right.messages);

    ts::write_to_output(&args.output_path, &left)
}

/// Merges two messages collections
fn merge_messages(
    left_messages: &mut Vec<MessageNode>,
    right_messages: &mut Vec<MessageNode>,
) -> Vec<MessageNode> {
    let mut unique_messages_left: Vec<_> = left_messages
        .drain(0..)
        .map(|node| NodeGroup { node })
        .collect();

    let unique_messages_right: Vec<_> = right_messages
        .drain(0..)
        .map(|node| NodeGroup { node })
        .collect();

    unique_messages_left
        .drain(0..)
        .filter(|a| !unique_messages_right.contains(&a))
        .merge(unique_messages_right.iter().cloned())
        .map(|node| node.node)
        .collect()
}

fn load_file(path: &String) -> Result<TSNode, String> {
    match quick_xml::Reader::from_file(&path) {
        Ok(reader) => {
            let nodes: Result<TSNode, _> = quick_xml::de::from_reader(reader.into_inner());
            match nodes {
                Ok(nodes) => Ok(nodes),
                Err(err) => Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}
