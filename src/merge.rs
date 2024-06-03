use std::hash::{Hash, Hasher};

use clap::{ArgAction, Args};
use itertools::Itertools;
use tracing::debug;

use crate::locale::{tr, tr_args};
use crate::ts;
use crate::ts::{MessageNode, TSNode};

/// Merges two translation file contexts and messages into a single output.
#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct MergeArgs {
    /// File to receive the merge
    #[arg(help = tr("cli-merge-input-left"), help_heading = tr("cli-headers-arguments"))]
    pub input_left: String,
    /// File to include changes from
    #[arg(help = tr("cli-merge-input-right"), help_heading = tr("cli-headers-arguments"))]
    pub input_right: String,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long, help = tr("cli-merge-output"), help_heading = tr("cli-headers-options"))]
    pub output_path: Option<String>,
    #[arg(short, long, action = ArgAction::Help, help = tr("cli-help"), help_heading = tr("cli-headers-options"))]
    pub help: bool,
}

/// MessageNode that can be `eq(...)`.
#[derive(Eq, PartialOrd, Clone)]
struct EquatableMessageNode {
    pub node: MessageNode,
}

/// The rule for equality is if the message id or source match.
impl PartialEq for EquatableMessageNode {
    fn eq(&self, other: &Self) -> bool {
        if let Some(this_id) = &self.node.id {
            if let Some(other_id) = &other.node.id {
                return this_id == other_id;
            }
        }

        self.node.source == other.node.source && self.node.locations == other.node.locations
    }
}

impl Hash for EquatableMessageNode {
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
        return Err(tr_args(
            "open-or-parse-error",
            [
                ("file", args.input_left.as_str().into()),
                ("error", e.to_string().as_str().into()),
            ]
            .into(),
        ));
    }

    if let Err(e) = right {
        return Err(tr_args(
            "open-or-parse-error",
            [
                ("file", args.input_right.as_str().into()),
                ("error", e.to_string().as_str().into()),
            ]
            .into(),
        ));
    }

    let result = merge_ts_nodes(left.unwrap(), right.unwrap());

    ts::write_to_output(&args.output_path, &result)
}

fn merge_ts_nodes(mut left: TSNode, mut right: TSNode) -> TSNode {
    left.messages = merge_messages(&mut left.messages, &mut right.messages);
    merge_contexts(&mut left, right);
    left
}

fn merge_contexts(left: &mut TSNode, right: TSNode) {
    right.contexts.into_iter().for_each(|mut right_context| {
        let left_context_opt = left
            .contexts
            .iter_mut()
            .find(|left_context| left_context.name == right_context.name);

        if let Some(left_context) = left_context_opt {
            debug!(
                "Found context '{}' matching in left and right files.",
                left_context.name
            );
            debug!(
                "Left context has {} messages, Right context has {} messages.",
                left_context.messages.len(),
                right_context.messages.len()
            );

            left_context.comment = right_context.comment;
            left_context.encoding = right_context.encoding;

            left_context.messages =
                merge_messages(&mut left_context.messages, &mut right_context.messages);
        } else {
            debug!(
                "No matching context with name '{}' in left file.",
                right_context.name
            );
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
        .map(|node| EquatableMessageNode { node })
        .collect();

    let mut unique_messages_right: Vec<_> = right_messages
        .drain(0..)
        .map(|node| EquatableMessageNode { node })
        .collect();

    // Update oldcomment, oldsource.
    unique_messages_right.iter_mut().for_each(|right_message| {
        let left_message = unique_messages_left
            .iter()
            .find(|&msg| msg == right_message);

        if let Some(left_message) = left_message {
            debug!(
                "Found matching message with source '{:?}' and id '{:?}' ",
                left_message.node.source, left_message.node.id
            );

            if right_message.node.source != left_message.node.source {
                debug!(
                    "Updating source '{:?}' to '{:?}'",
                    left_message.node.source, right_message.node.source
                );

                right_message
                    .node
                    .old_source
                    .clone_from(&left_message.node.source);
            }

            if right_message.node.comment != left_message.node.comment {
                debug!(
                    "Updating comment '{:?}' to '{:?}'",
                    left_message.node.comment, right_message.node.comment
                );

                right_message
                    .node
                    .old_comment
                    .clone_from(&left_message.node.comment);
            }
        }
    });

    unique_messages_left
        .drain(0..)
        .filter(|message| !unique_messages_right.contains(message))
        .merge(unique_messages_right.iter().cloned())
        .map(|node| node.node)
        .collect()
}

fn load_file(path: &String) -> Result<TSNode, String> {
    match quick_xml::Reader::from_file(path) {
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
