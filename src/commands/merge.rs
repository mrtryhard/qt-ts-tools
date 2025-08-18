use std::hash::{Hash, Hasher};

use clap::{ArgAction, Args};
use log::debug;

use crate::locale::tr;
use crate::ts;
use crate::ts::{MessageNode, TSNode};

/// Merges two translation file contexts and messages into a single output.
#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct MergeArgs {
    /// File to receive the merge
    #[arg(help = tr!("cli-merge-input-left"), help_heading = tr!("cli-headers-arguments"))]
    pub input_left: String,
    /// File to include changes from
    #[arg(help = tr!("cli-merge-input-right"), help_heading = tr!("cli-headers-arguments"))]
    pub input_right: String,
    /// When true, do not update the translation value.
    #[arg(help = tr!("cli-merge-keep-translation"), help_heading = tr!("cli-headers-arguments"), action = ArgAction::SetTrue)]
    pub keep_translation: bool,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long, help = tr!("cli-merge-output"), help_heading = tr!("cli-headers-options"))]
    pub output_path: Option<String>,
    #[arg(short, long, action = ArgAction::Help, help = tr!("cli-help"), help_heading = tr!("cli-headers-options"))]
    pub help: Option<bool>,
}

// This works by depending on cmp looking up only source and location on messages nodes
// and on context by comparing the names only
pub fn merge_main(args: &MergeArgs) -> Result<(), String> {
    let left = load_file(&args.input_left);
    let right = load_file(&args.input_right);

    if let Err(e) = left {
        return Err(tr!(
            "error-open-or-parse",
            file = args.input_left.as_str(),
            error = e.to_string()
        ));
    }

    if let Err(e) = right {
        return Err(tr!(
            "error-open-or-parse",
            file = args.input_right.as_str(),
            error = e.to_string()
        ));
    }

    let result = merge_ts_nodes(left.unwrap(), right.unwrap(), args.keep_translation);

    ts::write_to_output(&args.output_path, &result)
}

/// MessageNode that can be `eq(...)`.
#[derive(Eq, PartialOrd, Clone)]
struct EquatableMessageNode {
    pub node: MessageNode,
}

/// The rule for equality is if the message id or source match.
impl PartialEq for EquatableMessageNode {
    fn eq(&self, other: &Self) -> bool {
        if let Some(this_id) = &self.node.id
            && let Some(other_id) = &other.node.id
        {
            return this_id == other_id;
        }

        self.node.source == other.node.source
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

fn merge_ts_nodes(mut left: TSNode, mut right: TSNode, keep_translation: bool) -> TSNode {
    if keep_translation {
        debug!(
            "--keep_translation flag is active, the following nodes will NOT be updated from the right-side file: translation, comment, oldcomment, oldsource, encoding"
        );
    }

    left.messages = merge_messages(&mut left.messages, &mut right.messages, keep_translation);
    merge_contexts(&mut left, right, keep_translation);
    left
}

fn merge_contexts(left: &mut TSNode, right: TSNode, keep_translation: bool) {
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

            if !keep_translation {
                left_context.comment = right_context.comment;
                left_context.encoding = right_context.encoding;
            }

            left_context.messages = merge_messages(
                &mut left_context.messages,
                &mut right_context.messages,
                keep_translation,
            );
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
    keep_translation: bool,
) -> Vec<MessageNode> {
    let mut unique_messages_left: Vec<_> = left_messages
        .drain(0..)
        .map(|node| EquatableMessageNode { node })
        .collect();

    let mut unique_messages_right: Vec<_> = right_messages
        .drain(0..)
        .map(|node| EquatableMessageNode { node })
        .collect();

    unique_messages_left.iter_mut().for_each(|left_message| {
        // Find matching right message and merge information
        let right_message = unique_messages_right
            .iter()
            .find(|&msg| msg == left_message);

        if let Some(right_message) = right_message {
            debug!(
                "Found matching message with source '{:?}' and id '{:?}' ",
                right_message.node.source, right_message.node.id
            );

            if right_message.node.source != left_message.node.source {
                debug!(
                    "Updating source '{:?}' to '{:?}'",
                    left_message.node.source, right_message.node.source
                );

                left_message
                    .node
                    .old_source
                    .clone_from(&left_message.node.source);
                left_message
                    .node
                    .source
                    .clone_from(&right_message.node.source);
            }

            if right_message.node.comment != left_message.node.comment {
                debug!(
                    "Updating comment '{:?}' to '{:?}'",
                    left_message.node.comment, right_message.node.comment
                );

                left_message
                    .node
                    .old_comment
                    .clone_from(&left_message.node.comment);
                left_message
                    .node
                    .comment
                    .clone_from(&right_message.node.comment);
            }

            left_message
                .node
                .locations
                .clone_from(&right_message.node.locations);

            if !keep_translation {
                debug!(
                    "Updating translation '{:?}' to '{:?}'",
                    left_message.node.translation, right_message.node.translation
                );
                debug!(
                    "Updating translator comment '{:?}' to '{:?}'",
                    left_message.node.translator_comment, right_message.node.translator_comment
                );

                left_message.node.translation = right_message.node.translation.clone();
                left_message.node.translator_comment =
                    right_message.node.translator_comment.clone();
            }
        }
    });

    let right_message_iter: Vec<EquatableMessageNode> = unique_messages_right
        .drain(0..)
        .filter(|right_message| !unique_messages_left.contains(right_message))
        .collect();

    debug!(
        "Expecting to add {} messages from 'right' file.",
        right_message_iter.len()
    );

    unique_messages_left.extend(right_message_iter);
    unique_messages_left
        .drain(0..)
        .map(|message| message.node)
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

        let result = merge_ts_nodes(left, right, false);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_merge_two_files_keep_translations() {
        let left = load_file(&"./test_data/example_merge_keep_translation_left.xml".to_string())
            .expect("Test data could not be loaded for left file.");
        let right = load_file(&"./test_data/example_merge_keep_translation_right.xml".to_string())
            .expect("Test data could not be loaded for right file.");
        let expected_result =
            load_file(&"./test_data/example_merge_keep_translation_result.xml".to_string())
                .expect("Test data could not be loaded for right file.");

        let result = merge_ts_nodes(left, right, true);

        assert_eq!(result, expected_result);
    }
}
