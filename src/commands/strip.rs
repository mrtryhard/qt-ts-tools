use clap::{ArgAction, Args};
use log::debug;

use crate::locale::tr;
use crate::parser::parse_error::ParseError;
use crate::parser::serializer::write_to_output;
use crate::parser::translation_type::TranslationType;
use crate::parser::ts_parser::{TsDocument, TsParser};

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

    let s: Vec<TranslationType> = args
        .translation_type
        .iter()
        .map(|arg| arg.clone().into())
        .collect();

    strip_nodes(&mut doc, &s);
    write_to_output(&args.output_path, &doc)
}

fn strip_nodes(doc: &mut TsDocument, translation_type_filter: &[TranslationType]) {
    let mut count = 0;
    doc.root.contexts.iter_mut().for_each(|context| {
        context.messages.iter_mut().for_each(|message| {
            if let Some(translation) = &mut message.translation.as_ref()
                && let Some(translation_type) = translation.translation_type.clone()
                && translation_type_filter.contains(&translation_type)
            {
                debug!(
                    "Stripping translation {:?} from message `{:?}`",
                    translation.translation_simple, message.source
                );
                message.translation = None;
                count += 1;
            }
        });
    });

    debug!("Stripped {count} translation tags");
}

#[cfg(test)]
mod strip_test {
    use super::*;
    use crate::commands::test_utils::{
        serialize_to_string, test_file_content_as_string, test_file_content_bytes,
    };

    #[test]
    fn test_strip() {
        let expected = test_file_content_as_string("example_strip_stripped.xml");
        let input = test_file_content_bytes("example_strip.xml");
        let mut doc = TsParser::from_buffer(input).expect("Parsable");

        let types = vec![TranslationType::Obsolete];
        strip_nodes(&mut doc, &types);

        assert_eq!(expected, serialize_to_string(&doc));
    }
}
