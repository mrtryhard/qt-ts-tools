use crate::ts;
use crate::ts::TSNode;
use clap::Args;

/// Sorts the input translation file by context, then by messages.
#[derive(Args)]
pub struct SortArgs {
    /// File path to sort translations from.
    pub input_path: String,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long)]
    pub output_path: Option<String>,
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
                Err(e) => Err(format!(
                    "Could not parse input file \"{}\". Error: {e:?}.",
                    args.input_path
                )),
            }
        }
        Err(e) => Err(format!(
            "Could not open or parse input file \"{}\". Error: {e:?}",
            args.input_path
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
    use super::*;
    use quick_xml;

    #[test]
    fn test_sort_ts_node() {
        let reader_nosort = quick_xml::Reader::from_file("./test_data/example_unfinished.xml")
            .expect("Couldn't open example_unfinished test file");
        let mut data_nosort: TSNode =
            quick_xml::de::from_reader(reader_nosort.into_inner()).expect("Parsable");

        sort_ts_node(&mut data_nosort);

        // Validate context ordering
        assert_eq!(data_nosort.contexts[0].name, "CodeContext".to_owned());
        assert_eq!(data_nosort.contexts[1].name, "UiContext".to_owned());

        // Validate message ordering
        let messages = &data_nosort.contexts[1].messages;
        assert_eq!(messages[0].source, Some("This is just a Sample".to_owned()));
        assert_eq!(messages[1].source, Some("Name".to_owned()));
        assert_eq!(messages[2].source, Some("Practice more".to_owned()));

        // Validate locations ordering
        assert_eq!(
            messages[0].locations[0].filename,
            Some("ui_main.cpp".to_owned())
        );
        assert_eq!(messages[0].locations[0].line, Some(144));
        assert_eq!(
            messages[0].locations[1].filename,
            Some("ui_potato_viewer.cpp".to_owned())
        );
        assert_eq!(messages[0].locations[1].line, Some(10));

        assert_eq!(
            messages[1].locations[0].filename,
            Some("ui_main.cpp".to_owned())
        );
        assert_eq!(messages[1].locations[0].line, Some(321));
        assert_eq!(
            messages[1].locations[1].filename,
            Some("ui_main.cpp".to_owned())
        );
        assert_eq!(messages[1].locations[1].line, Some(456));
        assert_eq!(
            messages[1].locations[2].filename,
            Some("ui_potato_viewer.cpp".to_owned())
        );
        assert_eq!(messages[1].locations[2].line, Some(10));
        assert_eq!(
            messages[1].locations[3].filename,
            Some("ui_potato_viewer.cpp".to_owned())
        );
        assert_eq!(messages[1].locations[3].line, Some(11));
    }
}
