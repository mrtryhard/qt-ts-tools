use std::hash::{Hash, Hasher};

use clap::Args;
use itertools::Itertools;

use crate::ts;
use crate::ts::{MessageNode, TSNode};

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

    let result = merge_ts_nodes(left.unwrap(), right.unwrap());

    ts::write_to_output(&args.output_path, &result)
}

fn merge_ts_nodes(mut left: TSNode, mut right: TSNode) -> TSNode {
    left.messages = merge_messages(&mut left.messages, &mut right.messages);
    merge_contexts(right, &mut left);
    left
}

fn merge_contexts(right: TSNode, left: &mut TSNode) {
    right.contexts.into_iter().for_each(|mut right_context| {
        let left_context_opt = left
            .contexts
            .iter_mut()
            .find(|left_context| left_context.name == right_context.name);

        if let Some(left_context) = left_context_opt {
            left_context.messages =
                merge_messages(&mut left_context.messages, &mut right_context.messages);
        } else {
            left.contexts.push(right_context);
        }
    });
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

#[cfg(test)]
mod merge_test {
    use super::*;

    #[test]
    fn test_merge_two_files() {
        let left = load_file(&"./test_data/example_merge_left.xml".to_string())
            .expect("Test data could not be loaded for left file.");
        let right = load_file(&"./test_data/example_merge_right.xml".to_string())
            .expect("Test data could not be loaded for right file.");
        let expected_result = load_file(&"./test_data/example_merge_result.xml".to_string())
            .expect("Test data could not be loaded for right file.");

        let result = merge_ts_nodes(left, right);

        assert_eq!(result, expected_result);
    }
}
