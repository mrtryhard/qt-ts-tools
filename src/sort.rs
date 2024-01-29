use crate::ts_definition::*;
use clap::Args;
use serde::Serialize;
use std::io::{BufWriter, Write};

#[derive(Args)]
pub struct SortArgs {
    pub input_path: String,
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
                    write_ts_to_output(&args, &ts_node)
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

/// Writes the output TS file to the specified output (file or stdout).
/// This writer will auto indent/pretty print. It will always expand empty nodes, e.g.
/// `<name></name>` instead of `<name/>`.
fn write_ts_to_output(args: &SortArgs, node: &TSNode) -> Result<(), String> {
    let mut inner_writer: BufWriter<Box<dyn Write>> = match &args.output_path {
        None => BufWriter::new(Box::new(std::io::stdout().lock())),
        Some(output_path) => match std::fs::File::options()
            .create(true)
            .write(true)
            .open(output_path)
        {
            Ok(file) => BufWriter::new(Box::new(file)),
            Err(e) => {
                return Err(format!(
                    "Error occured while opening output file \"{output_path}\": {e:?}"
                ))
            }
        },
    };

    let mut output_buffer =
        String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE TS>\n");
    let mut ser = quick_xml::se::Serializer::new(&mut output_buffer);
    ser.indent(' ', 2).expand_empty_elements(true);

    match node.serialize(ser) {
        Ok(_) => {
            let res = inner_writer.write_all(output_buffer.as_bytes());
            match res {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Problem occured while serializing output: {e:?}")),
            }
        }
        Err(e) => Err(format!("Problem occured while serializing output: {e:?}")),
    }
}

#[cfg(test)]
mod sort_test {
    use super::*;
    use quick_xml;

    #[test]
    fn sort_ts_node_ts() {
        let reader_nosort = quick_xml::Reader::from_file("./test_data/example_unfinished.xml")
            .expect("Couldn't open example_unfinished test file");
        let mut data_nosort: TSNode =
            quick_xml::de::from_reader(reader_nosort.into_inner()).expect("Parsable");

        sort_ts_node(&mut data_nosort);

        // Validate context ordering
        assert_eq!(data_nosort.contexts[0].name, Some("CodeContext".to_owned()));
        assert_eq!(data_nosort.contexts[1].name, Some("UiContext".to_owned()));

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

#[cfg(test)]
mod write_file_test {
    use super::*;
    use quick_xml;

    #[test]
    fn write_ts_file_test() {
        const OUTPUT_TEST_FILE: &str = "./test_data/test_result_write_to_ts.xml";

        let reader = quick_xml::Reader::from_file("./test_data/example1.xml")
            .expect("Couldn't open example1 test file");

        let data: TSNode = quick_xml::de::from_reader(reader.into_inner()).expect("Parsable");
        let args = SortArgs {
            input_path: "whatever".to_owned(),
            output_path: Some(OUTPUT_TEST_FILE.to_owned()),
        };
        write_ts_to_output(&args, &data).expect("Output");

        let f =
            quick_xml::Reader::from_file(OUTPUT_TEST_FILE).expect("Couldn't open output test file");

        let output_data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
        std::fs::remove_file(OUTPUT_TEST_FILE).expect("Test should clean test file.");
        assert_eq!(data, output_data);
    }
}
