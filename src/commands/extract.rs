use crate::parser::translation_node::TranslationNode;
use crate::parser::translation_type::TranslationType;
use crate::parser::ts_parser::{TsDocument, TsParser};
use crate::tr;
use clap::{ArgAction, Args};
use log::debug;
use std::error::Error;
use std::path::PathBuf;

#[derive(clap::ValueEnum, PartialEq, Debug, Clone)]
pub enum TranslationTypeArg {
    Obsolete,
    Unfinished,
    Vanished,
}

/// Extracts messages and contexts from the file that satisfies the targeted translation type.
#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct ExtractArgs {
    /// File path to extract translations from.
    #[arg(help = tr!("cli-extract-input"), help_heading = tr!("cli-headers-arguments"))]
    pub input: PathBuf,
    /// Translation type list to extract into a single, valid translation output.
    #[arg(short('t'), long, value_enum, num_args = 1.., help = tr!("cli-extract-translation-type"), help_heading = tr!("cli-headers-arguments"))]
    pub translation_type: Vec<TranslationTypeArg>,
    /// If specified, will produce output in a file at a designated location instead of stdout.
    #[arg(short, long, help = tr!("cli-extract-output"), help_heading = tr!("cli-headers-options"))]
    pub output_path: Option<PathBuf>,
    #[arg(short, long, action = ArgAction::Help, help = tr!("cli-help"), help_heading = tr!("cli-headers-options"))]
    pub help: Option<bool>,
}

/// Filters the translation file to keep only the messages containing unfinished translations.
pub fn extract(extract_args: &ExtractArgs) -> Result<TsDocument, Box<dyn Error>> {
    TsParser::from_file(&extract_args.input)
        .map(|doc| {
            let types = extract_args
                .translation_type
                .iter()
                .map(to_translation_type)
                .collect();
            retain_ts_node(doc, types)
        })
        .map_err(|err| err.into())
}

fn to_translation_type(value: &TranslationTypeArg) -> TranslationType {
    match value {
        TranslationTypeArg::Obsolete => TranslationType::Obsolete,
        TranslationTypeArg::Unfinished => TranslationType::Unfinished,
        TranslationTypeArg::Vanished => TranslationType::Vanished,
    }
}

fn translation_is_wanted(
    translation_node: Option<&TranslationNode>,
    wanted_types: &[TranslationType],
) -> bool {
    translation_node.is_some_and(|translation| {
        debug!(
            "Translation node candidate for being retained: {:?} | {:?}",
            translation.translation_simple, translation.translation_type
        );

        translation
            .translation_type
            .as_ref()
            .is_some_and(|translation_type| wanted_types.contains(translation_type))
    })
}

/// Keep only the desired translation type from the node (if it matches one in `wanted_types`).
fn retain_ts_node(mut doc: TsDocument, wanted_types: Vec<TranslationType>) -> TsDocument {
    doc.root.contexts.retain_mut(|context| {
        context
            .messages
            .retain(|message| translation_is_wanted(message.translation.as_ref(), &wanted_types));
        !context.messages.is_empty()
    });
    doc
}

#[cfg(test)]
mod extract_test {
    use super::*;
    use crate::commands::test_utils::test_file_content_as_string;

    fn get_expected_extracted(filename: &str) -> TsDocument {
        TsParser::from_buffer(test_file_content_as_string(filename).into_bytes())
            .expect("Should be reading test file")
    }

    #[test]
    fn test_extract_ts_node() {
        let expected_extracted = get_expected_extracted("example_extract_extracted.xml");
        let args = ExtractArgs {
            input: "./test_data/example_extract.xml".into(),
            translation_type: vec![TranslationTypeArg::Obsolete],
            output_path: None, // ignore, we no longer write to file
            help: None,
        };
        let doc = extract(&args).expect("Retain node to be successful");

        assert_eq!(doc.root, expected_extracted.root);
    }
}
