use clap::{ArgAction, Args};

use crate::parser::parse_error::ParseError;
use crate::parser::serializer::write_to_output;
use crate::parser::ts_parser::{TsDocument, TsParser};
use crate::tr;

#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct SortArgs {
    /// File path to sort translations from.
    #[arg(help = tr!("cli-sort-input"), help_heading = tr!("cli-headers-arguments"))]
    pub input_path: String,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long, help = tr!("cli-sort-output"), help_heading = tr!("cli-headers-options"))]
    pub output_path: Option<String>,
    #[arg(short, long, action = ArgAction::Help, help = tr!("cli-help"), help_heading = tr!("cli-headers-options")
    )]
    pub help: Option<bool>,
}

/// Sorts an input TS file by context, then by messages.
/// It will output the result to the output file if specified.
/// Otherwise will output in stdout.
///
/// ## Windows notes
/// Writing non-UTF-8 characters or non-valid UTF-8 characters to `stdout` may result in an error.
pub fn sort_main(args: &SortArgs) -> Result<(), String> {
    let mut doc = std::fs::read(&args.input_path)
        .map_err(|err| {
            ParseError::from(tr!(
                "error-open-or-parse",
                file = args.input_path.as_str(),
                error = err.to_string()
            ))
        })
        .and_then(TsParser::from_buffer)
        .map_err(|err| err.to_string())?;
    sort_document(&mut doc);
    write_to_output(&args.output_path, &doc)
}

/// Sorts the TS document with the following rules:
/// 1. Context comes before no-context messages.
/// 2. Context are ordered by name.
/// 3. Messages are ordered by filename then by line.
fn sort_document(ts_node: &mut TsDocument) {
    let contexts = &mut ts_node.root.contexts;
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
    use crate::parser::serializer::serialize;
    use std::io::BufWriter;
    use crate::logging::initialize_logging;

    #[test]
    fn test_sort_ts_node() {
        initialize_logging();
        let expected_sorted =
            std::fs::read_to_string("./test_data/example_sort_sorted.xml").expect("File to exist");
        let base_ts_data = std::fs::read("./test_data/example_sort.xml").expect("File to exist");
        let mut sorted = TsParser::from_buffer(base_ts_data).expect("Parsable");

        sort_document(&mut sorted);

        let mut buf = BufWriter::new(Vec::<u8>::new());
        serialize(&mut buf, &sorted).expect("Sorted data can be serialized");
        let sorted_string = String::from_utf8(buf.into_inner().expect("Sorted data is utf-8"))
            .expect("Sorted data is utf-8");

        println!("{}", sorted_string);
        println!("{}", expected_sorted);
        assert_eq!(
            "",
            sorted.root.contexts[1].messages[2]
                .translation
                .as_ref()
                .unwrap()
                .translation_simple
                .as_ref()
                .unwrap()
        );
        assert_eq!(expected_sorted, sorted_string);
    }
}
