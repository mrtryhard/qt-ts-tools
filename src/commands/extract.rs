use clap::{ArgAction, Args};
use tracing::debug;

use crate::locale::{tr, tr_args};
use crate::ts;
use crate::ts::{TSNode, TranslationType};

/// Extracts a translation type messages and contexts from the input translation file.
#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct ExtractArgs {
    /// File path to exthelpract translations from.
    #[arg(help = tr("cli-extract-input"), help_heading = tr("cli-headers-arguments"))]
    pub input_path: String,
    /// Translation type list to extract into a single, valid translation output.
    #[arg(short('t'), long, value_enum, num_args = 1.., help = tr("cli-extract-translation-type"), help_heading = tr("cli-headers-arguments"))]
    pub translation_type: Vec<TranslationTypeArg>,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long, help = tr("cli-extract-output"), help_heading = tr("cli-headers-options"))]
    pub output_path: Option<String>,
    #[arg(short, long, action = ArgAction::Help, help = tr("cli-help"), help_heading = tr("cli-headers-options"))]
    pub help: Option<bool>,
}

#[derive(clap::ValueEnum, PartialEq, Debug, Clone)]
pub enum TranslationTypeArg {
    Obsolete,
    Unfinished,
    Vanished,
}

/// Filters the translation file to keep only the messages containing unfinished translations.
pub fn extract_main(args: &ExtractArgs) -> Result<(), String> {
    match quick_xml::Reader::from_file(&args.input_path) {
        Ok(file) => {
            let nodes: Result<TSNode, _> = quick_xml::de::from_reader(file.into_inner());
            match nodes {
                Ok(mut ts_node) => {
                    let wanted_types = args
                        .translation_type
                        .iter()
                        .map(to_translation_type)
                        .collect::<Vec<TranslationType>>();
                    retain_ts_node(&mut ts_node, &wanted_types);
                    ts::write_to_output(&args.output_path, &ts_node)
                }
                Err(e) => Err(tr_args(
                    "error-open-or-parse",
                    [
                        ("file", args.input_path.as_str().into()),
                        ("error", e.to_string().into()),
                    ]
                    .into(),
                )),
            }
        }
        Err(e) => Err(tr_args(
            "error-open-or-parse",
            [
                ("file", args.input_path.as_str().into()),
                ("error", e.to_string().into()),
            ]
            .into(),
        )),
    }
}

fn to_translation_type(value: &TranslationTypeArg) -> TranslationType {
    match value {
        TranslationTypeArg::Obsolete => TranslationType::Obsolete,
        TranslationTypeArg::Unfinished => TranslationType::Unfinished,
        TranslationTypeArg::Vanished => TranslationType::Vanished,
    }
}

/// Keep only the desired translation type from the node (if it matches one in `wanted_types`).
fn retain_ts_node(ts_node: &mut TSNode, wanted_types: &[TranslationType]) {
    ts_node.contexts.retain_mut(|context| {
        context.messages.retain(|message| {
            message.translation.as_ref().is_some_and(|translation| {
                debug!(
                    "Translation node candidate for being retained: {:?} | {:?}",
                    translation.translation_simple, translation.translation_type
                );

                translation
                    .translation_type
                    .as_ref()
                    .is_some_and(|translation_type| wanted_types.contains(translation_type))
            })
        });

        !context.messages.is_empty()
    });
}

#[cfg(test)]
mod extract_test {
    use super::*;

    #[test]
    fn test_extract_ts_node() {
        let reader_nosort = quick_xml::Reader::from_file("./test_data/example_key_de.xml")
            .expect("Couldn't open example_unfinished test file");
        let mut data: TSNode =
            quick_xml::de::from_reader(reader_nosort.into_inner()).expect("Parsable");

        let types = vec![TranslationType::Obsolete];
        retain_ts_node(&mut data, &types);

        assert_eq!(data.contexts[0].messages.len(), 3);
        assert_eq!(
            data.contexts[0].messages[0].source.as_ref().unwrap(),
            "Shift+K"
        );
        assert_eq!(
            data.contexts[0].messages[1].source.as_ref().unwrap(),
            "Ctrl+K"
        );
        assert_eq!(
            data.contexts[0].messages[2].source.as_ref().unwrap(),
            "Alt+K"
        );
    }
}
