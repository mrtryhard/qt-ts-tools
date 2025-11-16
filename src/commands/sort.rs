use clap::{ArgAction, Args};

use crate::ts::TSNode;
use crate::{tr, ts};

#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct SortArgs {
    /// File path to sort translations from.
    #[arg(help = tr!("cli-sort-input"), help_heading = tr!("cli-headers-arguments"))]
    pub input_path: String,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long, help = tr!("cli-sort-output"), help_heading = tr!("cli-headers-options"))]
    pub output_path: Option<String>,
    #[arg(short, long, action = ArgAction::Help, help = tr!("cli-help"), help_heading = tr!("cli-headers-options"))]
    pub help: Option<bool>,
}

/// Sorts an input TS file by context, then by messages.
/// It will output the result to the output file if specified.
/// Otherwise will output in stdout.
///
/// ## Windows notes
/// Writing non-UTF-8 characters or non-valid UTF-8 characters to `stdout` may result in an error.
pub fn sort_main(args: &SortArgs) -> Result<(), String> {
    match quick_xml::Reader::from_file(&args.input_path) {
        Ok(file) => {
            let nodes: Result<TSNode, _> = quick_xml::de::from_reader(file.into_inner());
            match nodes {
                Ok(mut ts_node) => {
                    sort_ts_node(&mut ts_node);
                    ts::write_to_output(&args.output_path, &ts_node)
                }
                Err(e) => Err(tr!(
                    "error-ts-file-parse",
                    input_path = args.input_path.as_str(),
                    error = e.to_string()
                )),
            }
        }
        Err(e) => Err(tr!(
            "error-open-or-parse",
            file = args.input_path.as_str(),
            error = e.to_string()
        )),
    }
}

/// Sorts the TS document with the following rules:
/// 1. Context comes before no-context messages.
/// 2. Context are ordered by name.
/// 3. Messages are ordered by filename then by line.
fn sort_ts_node(ts_node: &mut TSNode) {
    let contexts = &mut ts_node.contexts;
    contexts.sort();
    contexts.iter_mut().for_each(|context| {
        context.messages.sort();
        context
            .messages
            .iter_mut()
            .for_each(|message| message.locations.sort());
    });
}

#[cfg(test)]
mod sort_test {
    use crate::commands::test_utils::{node_to_formatted_string, read_test_file};

    use super::*;

    #[test]
    fn test_sort_ts_node() {
        let expected_sorted = read_test_file("example_sort_sorted.xml");

        let mut data_nosort: TSNode = {
            let reader_nosort = quick_xml::Reader::from_file("./test_data/example_sort.xml")
                .expect("Test file is readable");
            quick_xml::de::from_reader(reader_nosort.into_inner()).expect("Parsable")
        };

        sort_ts_node(&mut data_nosort);

        let sorted = node_to_formatted_string(&data_nosort);

        assert_eq!(expected_sorted, sorted);
    }
}
