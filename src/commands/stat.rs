use std::collections::HashMap;
use std::io::Write;
use std::ops::AddAssign;
use std::string::ToString;

use clap::{ArgAction, Args};
use log::debug;

use crate::tr;
use crate::ts::{MessageNode, TSNode, TranslationType};

#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct StatArgs {
    /// File path to sort translations from.
    #[arg(help = tr!("cli-stat-input"), help_heading = tr!("cli-headers-arguments"))]
    pub input_path: String,
    /// If set to true, will append a list of all unique file paths found.
    #[arg(short, long, help = tr!("cli-stats-files"), help_heading = tr!("cli-headers-options"))]
    pub files: Option<bool>,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long, help = tr!("cli-stat-output"), help_heading = tr!("cli-headers-options"))]
    pub output_path: Option<String>,
    #[arg(short, long, action = ArgAction::Help, help = tr!("cli-help"), help_heading = tr!("cli-headers-options"))]
    pub help: Option<bool>,
}

#[derive(Clone, Default, Eq, Ord, PartialEq, PartialOrd)]
struct FileStats {
    pub filepath: String,
    pub unfinished_translations: usize,
    pub vanished_translations: usize,
    pub obsolete_translations: usize,
    pub finished_translation: usize,
    /// For files, total_translations corresponds to number of time that file was
    /// mentioned as a location.
    pub total_translations: usize,
}

#[derive(Default)]
struct TotalStats {
    // Translation block
    pub total_missing_translations: usize,
    pub total_vanished_translations: usize,
    pub total_obsolete_translations: usize,
    /// Corresponds to the number of unique translation
    /// This is the sum of obsolete, vanished, missing and complete translations.
    pub total_unique_translations: usize,
    /// Corresponds to the number of references in files.
    /// For example, if a translation is the same for 3 files, it will return 3, not 1.
    /// Even if 2 locations is in the same file, it will count as 2.
    pub total_translations_references: usize,
    pub total_contexts: usize,
    pub total_messages: usize,
    pub total_context_less_messages: usize,

    /// Statistics by file
    pub files: Vec<FileStats>,
}

/// Aggregates the stats for provided file and arguments.
pub fn stat_main(args: &StatArgs) -> Result<(), String> {
    match quick_xml::Reader::from_file(&args.input_path) {
        Ok(file) => {
            let nodes: Result<TSNode, _> = quick_xml::de::from_reader(file.into_inner());
            match nodes {
                Ok(ts_node) => {
                    let total_stats = stats_ts_node(&ts_node);
                    let output = generate_message_for_stats(total_stats);

                    match &args.output_path {
                        None => {
                            println!("{}", output);
                            Ok(())
                        }
                        Some(output_path) => write_to_output(output_path, output),
                    }
                }
                Err(e) => Err(tr!(
                    "error-ts-file-parse",
                    ("file", args.input_path.as_str()),
                    ("error", e.to_string())
                )),
            }
        }
        Err(e) => Err(tr!(
            "error-open-or-parse",
            ("file", args.input_path.as_str()),
            ("error", e.to_string())
        )),
    }
}

/// Writes the output TS file to the specified output (file or stdout).
/// This writer will auto indent/pretty print. It will always expand empty nodes, e.g.
/// `<name></name>` instead of `<name/>`.
pub fn write_to_output(output_path: &String, output: String) -> Result<(), String> {
    debug!("Writing {} characters to '{output_path}'", output.len());

    match std::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(output_path)
    {
        Ok(mut file) => match file.write(output.as_bytes()) {
            Ok(bytes) => {
                debug!("Successfully wrote {}", bytes);
                Ok(())
            }
            Err(err) => {
                debug!("Failed to write to output_path: {err:?}");
                Err(tr!(
                    "error-write-output",
                    ("output_path", output_path),
                    ("error", err.to_string())
                ))
            }
        },
        Err(e) => Err(tr!(
            "error-write-output-open",
            ("output_path", output_path),
            ("error", e.to_string())
        )),
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
enum FileKey<'a> {
    Invalid,
    Valid(&'a String),
}

fn stats_ts_node(ts_node: &TSNode) -> TotalStats {
    let mut stats = TotalStats {
        total_contexts: ts_node.contexts.len(),
        total_messages: ts_node.messages.len(),
        total_context_less_messages: ts_node.messages.len(),
        ..TotalStats::default()
    };

    let mut files_stats = HashMap::<FileKey, FileStats>::new();

    stats_for_messages(&ts_node.messages, &mut stats, &mut files_stats);

    for context in &ts_node.contexts {
        stats.total_messages.add_assign(context.messages.len());
        stats_for_messages(&context.messages, &mut stats, &mut files_stats);
    }

    stats.files = files_stats.values().cloned().collect();
    stats.files.sort();

    stats
}

fn stats_for_messages<'a>(
    messages: &'a [MessageNode],
    stats: &mut TotalStats,
    files_stats: &mut HashMap<FileKey<'a>, FileStats>,
) {
    for message in messages {
        if message.translation.is_some() {
            stats.total_unique_translations.add_assign(1);
        }

        stats
            .total_translations_references
            .add_assign(message.locations.len());

        for location in &message.locations {
            // Filename _may_ be empty, although it should not really happen.
            // We can report this as an error ?
            let file_key = match location.filename.as_ref() {
                None => FileKey::Invalid,
                Some(path) => FileKey::Valid(path),
            };

            // Note: there could be duplicate in locations, right ?
            let file = &mut files_stats.entry(file_key.clone()).or_default();
            file.total_translations.add_assign(1);
            file.filepath = match file_key {
                FileKey::Invalid => "invalid".to_owned(),
                FileKey::Valid(path) => path.clone(),
            };

            match &message.translation {
                None => file.unfinished_translations.add_assign(1),
                Some(node) => match node.translation_type {
                    None => {}
                    Some(TranslationType::Finished) => file.finished_translation.add_assign(1),
                    Some(TranslationType::Obsolete) => file.obsolete_translations.add_assign(1),
                    Some(TranslationType::Vanished) => file.vanished_translations.add_assign(1),
                    Some(TranslationType::Unfinished) => file.unfinished_translations.add_assign(1),
                },
            }
        }

        match &message.translation {
            None => {}
            Some(node) => match node.translation_type {
                None => {}
                Some(TranslationType::Finished) => {}
                Some(TranslationType::Obsolete) => stats.total_obsolete_translations.add_assign(1),
                Some(TranslationType::Vanished) => stats.total_vanished_translations.add_assign(1),
                Some(TranslationType::Unfinished) => stats.total_missing_translations.add_assign(1),
            },
        }
    }
}

fn generate_message_for_stats(stats: TotalStats) -> String {
    let mut buf = String::new();

    if !stats.files.is_empty() {
        buf.push_str("------------------------------------------------------------------------------------------------------\n");
        buf.push_str(&format!(
            "{: <30} | {: <22} | {: <10} | {: <8} | {: <8} | {: <8}\n",
            tr!("cli-stat-filepath-header"),
            tr!("cli-stat-total-trans-ref"),
            // These are literals in the xml file, let's not translate.
            "Unfinished",
            "Complete",
            "Obsolete",
            "Vanished"
        ));
        buf.push_str("------------------------------------------------------------------------------------------------------\n");
        for file in &stats.files {
            buf.push_str(&format!(
                "{: <30} | {: <22} | {: <10} | {: <8} | {: <8} | {: <8}\n",
                file.filepath,
                file.total_translations,
                file.unfinished_translations,
                file.finished_translation,
                file.obsolete_translations,
                file.vanished_translations
            ));
        }
    }

    buf.push_str("------------------------------------------------------------------------------------------------------\n");
    buf.push_str(&format!("{}\n", tr!("cli-stat-file-summary")));
    buf.push_str("------------------------------------------------------------------------------------------------------\n");
    buf.push_str(&format!(
        "{: <24} : {}\n",
        tr!("cli-stat-files"),
        stats.files.len()
    ));
    buf.push_str(&format!("{: <24} : {}\n", "Contexts", stats.total_contexts));
    buf.push_str(&format!("{: <24} : {}\n", "Messages", stats.total_messages));
    buf.push_str(&format!(
        "{: <24} : {}\n",
        tr!("cli-stat-messages-without-context"),
        stats.total_context_less_messages
    ));
    buf.push_str(&format!(
        "{: <24} : {}\n",
        tr!("cli-stat-unique-translations"),
        stats.total_unique_translations
    ));
    buf.push_str(&format!(
        "{: <24} : {}\n",
        tr!("cli-stat-translations-refs"),
        stats.total_translations_references
    ));
    buf.push_str(&format!(
        "{: <24} : {}\n",
        tr!("cli-stat-type-translations", ("type", "Missing")),
        stats.total_missing_translations
    ));
    buf.push_str(&format!(
        "{: <24} : {}\n",
        tr!("cli-stat-type-translations", ("type", "Obsolete")),
        stats.total_obsolete_translations
    ));
    buf.push_str(&format!(
        "{: <24} : {}\n",
        tr!("cli-stat-type-translations", ("type", "Vanished")),
        stats.total_vanished_translations
    ));

    buf
}

#[cfg(test)]
mod stats_tests {
    use super::*;

    #[test]
    fn test_stats_aggregate() {
        let data_nostats: TSNode = {
            let reader_stats = quick_xml::Reader::from_file("./test_data/example_strip.xml")
                .expect("Test file is readable");
            quick_xml::de::from_reader(reader_stats.into_inner()).expect("Parsable")
        };

        let stats = stats_ts_node(&data_nostats);

        // Per file: a translation count is the number of location for a translation for a file.
        // Total: simply the number of time a translation appear in a message
        assert_eq!(stats.total_context_less_messages, 3);
        assert_eq!(stats.total_contexts, 1);
        assert_eq!(stats.total_messages, 14);
        assert_eq!(stats.total_obsolete_translations, 4);
        assert_eq!(stats.total_vanished_translations, 1);
        assert_eq!(stats.total_missing_translations, 0);
        assert_eq!(stats.total_unique_translations, 14);
        // One node has 2 locations
        assert_eq!(stats.total_translations_references, 15);

        let files = &stats.files;

        assert_eq!(
            files
                .iter()
                .filter(|entry| entry.filepath == "tst_qkeysequence.cpp")
                .count(),
            1
        );
        assert_eq!(
            files
                .iter()
                .filter(|entry| entry.filepath == "tst_nostrip.cpp")
                .count(),
            1
        );
        assert_eq!(
            files
                .iter()
                .filter(|entry| entry.filepath == "tst_nostrip2.cpp")
                .count(),
            1
        );

        let nostrip = files.first().expect("Has index 0 entry");
        assert_eq!(nostrip.filepath, "tst_nostrip.cpp");
        assert_eq!(nostrip.total_translations, 1);
        assert_eq!(nostrip.vanished_translations, 0);
        assert_eq!(nostrip.unfinished_translations, 0);
        assert_eq!(nostrip.obsolete_translations, 0);

        let nostrip2 = files.get(1).expect("Has index 1 entry");
        assert_eq!(nostrip2.filepath, "tst_nostrip2.cpp");
        assert_eq!(nostrip2.total_translations, 1);
        assert_eq!(nostrip2.vanished_translations, 1);
        assert_eq!(nostrip2.unfinished_translations, 0);
        assert_eq!(nostrip2.obsolete_translations, 0);

        let qkey = files.get(2).expect("Has index 2 entry");
        assert_eq!(qkey.filepath, "tst_qkeysequence.cpp");
        assert_eq!(qkey.total_translations, 13);
        assert_eq!(qkey.vanished_translations, 0);
        assert_eq!(qkey.unfinished_translations, 0);
        // Might expect 4, but there's a message node with 2 locations pointing to the same file.
        // We count them as 2 references obsolete.
        assert_eq!(qkey.obsolete_translations, 5);
    }
}
