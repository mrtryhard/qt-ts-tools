use crate::ts_definition::*;
use clap::Args;
use serde::Serialize;
use std::io::{BufWriter, Write};

#[derive(Args)]
pub struct SortArgs {
    pub input_path: String,
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
                    Ok(())
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

fn sort_ts_node(_ts_node: &mut TSNode) {
    todo!();
}

fn write_ts_file(args: &SortArgs, node: &TSNode) -> Result<(), String> {
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

    let mut output_buffer = String::from("<!DOCTYPE TS>\n");
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
mod write_file_test {
    use super::*;
    use quick_xml;

    #[test]
    fn write_ts_file_test() {
        let reader =
            quick_xml::Reader::from_file("example1.xml").expect("Couldn't open example1 test file");

        let data: TSNode = quick_xml::de::from_reader(reader.into_inner()).expect("Parsable");
        let args = SortArgs {
            input_path: "whatever".to_owned(),
            output_path: Some("test_result_write_to_ts.xml".to_owned()),
        };
        write_ts_file(&args, &data).expect("Output");

        let f = quick_xml::Reader::from_file("test_result_write_to_ts.xml")
            .expect("Couldn't open output test file");

        let output_data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
        assert_eq!(data, output_data);
    }
}
