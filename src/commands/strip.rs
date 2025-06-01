use clap::{ArgAction, Args};
use log::debug;

use crate::locale::tr;
use crate::ts;
use crate::ts::{TSNode, TranslationType};

#[derive(clap::ValueEnum, PartialEq, Debug, Clone)]
pub enum TranslationTypeArg {
    Obsolete,
    Unfinished,
    Vanished,
}

impl From<TranslationTypeArg> for TranslationType {
    fn from(value: TranslationTypeArg) -> Self {
        match value {
            TranslationTypeArg::Obsolete => TranslationType::Obsolete,
            TranslationTypeArg::Unfinished => TranslationType::Unfinished,
            TranslationTypeArg::Vanished => TranslationType::Vanished,
        }
    }
}

#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct StripArgs {
    /// File path to sort translations from.
    #[arg(help = tr!("cli-strip-input"), help_heading = tr!("cli-headers-arguments"))]
    pub input_path: String,
    /// Translation type list to strip from input.
    #[arg(short('t'), long, value_enum, num_args = 1.., help = tr!("cli-strip-translation-type"), help_heading = tr!("cli-headers-arguments"))]
    pub translation_type: Vec<TranslationTypeArg>,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long, help = tr!("cli-strip-output"), help_heading = tr!("cli-headers-options"))]
    pub output_path: Option<String>,
    #[arg(short, long, action = ArgAction::Help, help = tr!("cli-help"), help_heading = tr!("cli-headers-options"))]
    pub help: Option<bool>,
}

pub fn strip_main(args: &StripArgs) -> Result<(), String> {
    match quick_xml::Reader::from_file(&args.input_path) {
        Ok(file) => {
            let nodes: Result<TSNode, _> = quick_xml::de::from_reader(file.into_inner());
            match nodes {
                Ok(mut ts_node) => {
                    let s: Vec<TranslationType> = args
                        .translation_type
                        .iter()
                        .map(|arg| arg.clone().into())
                        .collect();

                    strip_nodes(&mut ts_node, &s);
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

fn strip_nodes(nodes: &mut TSNode, translation_type_filter: &[TranslationType]) {
    let mut count = 0;
    nodes.contexts.iter_mut().for_each(|context| {
        context.messages.iter_mut().for_each(|message| {
            if let Some(translation) = &mut message.translation.as_ref() {
                if let Some(translation_type) = translation.translation_type.clone() {
                    if translation_type_filter.contains(&translation_type) {
                        debug!(
                            "Stripping translation {:?} from message `{}`",
                            &translation.translation_simple,
                            &message
                                .source
                                .as_ref()
                                .unwrap_or(&"Unknown source text".to_owned())
                        );
                        message.translation = None;
                        count += 1;
                    }
                }
            }
        });
    });

    nodes.messages.iter_mut().for_each(|message| {
        if let Some(translation) = &mut message.translation.as_ref() {
            if let Some(translation_type) = translation.translation_type.clone() {
                if translation_type_filter.contains(&translation_type) {
                    debug!(
                        "Stripping translation {:?} from message `{}`",
                        &translation.translation_simple,
                        &message
                            .source
                            .as_ref()
                            .unwrap_or(&"Unknown source text".to_owned())
                    );
                    message.translation = None;
                    count += 1;
                }
            }
        }
    });

    debug!("Stripped {count} translation tags");
}

#[cfg(test)]
mod strip_test {
    use super::*;

    #[test]
    fn test_strip() {
        let reader_unstripped = quick_xml::Reader::from_file("./test_data/example_strip.xml")
            .expect("Couldn't open example_strip test file");
        let reader_stripped =
            quick_xml::Reader::from_file("./test_data/example_strip_stripped.xml")
                .expect("Couldn't open example_strip_stripped test file");
        let mut data: TSNode =
            quick_xml::de::from_reader(reader_unstripped.into_inner()).expect("Parsable");
        let data_stripped: TSNode =
            quick_xml::de::from_reader(reader_stripped.into_inner()).expect("Parsable");

        let types = vec![TranslationType::Obsolete];
        strip_nodes(&mut data, &types);

        assert_eq!(data, data_stripped);
    }
}
