//! Benchmarks covering the whole translation file pipeline: parsing a `.ts` file,
//! transforming it (sort, strip, extract, merge), reporting statistics and
//! compiling it down to a binary `.qm` file.
//!
//! Documents are generated in memory so that the benchmarks exercise realistic
//! sizes (hundreds of messages spread over dozens of contexts) instead of the
//! very small fixtures used by the unit tests. One benchmark also runs on a real
//! fixture from `test_data/` to keep an eye on the "typical small file" case.

use std::hint::black_box;
use std::io::Cursor;
use std::sync::OnceLock;

use divan::Bencher;
use qt_ts_tools::commands::extract::retain_ts_node;
use qt_ts_tools::commands::merge::merge_ts_nodes;
use qt_ts_tools::commands::release::compile_to_buffer;
use qt_ts_tools::commands::sort::sort_document;
use qt_ts_tools::commands::stat::stat_document;
use qt_ts_tools::commands::strip::strip_nodes;
use qt_ts_tools::parser::serializer::serialize;
use qt_ts_tools::parser::translation_type::TranslationType;
use qt_ts_tools::parser::ts_parser::{TsDocument, TsParser};

/// A real, small translation file shipped as test data.
const REAL_WORLD_FILE: &str = include_str!("../test_data/many_ctx_many_msgs_numerus.ts");

/// Small generated document: 5 contexts of 10 messages.
const SMALL: (usize, usize) = (5, 10);
/// Large generated document: 30 contexts of 20 messages.
const LARGE: (usize, usize) = (30, 20);

fn main() {
    divan::main();
}

/// Generates a translation document containing a mix of finished, unfinished,
/// obsolete, vanished and plural (numerus) messages, with several locations,
/// comments and identifiers, like a translation file produced by `lupdate`.
fn generate_ts(contexts: usize, messages_per_context: usize) -> String {
    // Contexts and messages are emitted in reverse order so that the sorting
    // benchmark has some actual work to do.
    generate_ts_ordered(contexts, messages_per_context, true)
}

fn generate_ts_ordered(contexts: usize, messages_per_context: usize, reversed: bool) -> String {
    let order = |count: usize| -> Vec<usize> {
        if reversed {
            (0..count).rev().collect()
        } else {
            (0..count).collect()
        }
    };

    let mut buf = String::with_capacity(contexts * messages_per_context * 256);
    buf.push_str("<!DOCTYPE TS>\n<TS version=\"2.1\" sourcelanguage=\"en\" language=\"fr_FR\">\n");

    for context in order(contexts) {
        buf.push_str("    <context>\n");
        buf.push_str(&format!(
            "        <name>widgets::Component{context:03}</name>\n"
        ));

        for message in order(messages_per_context) {
            let source = format!("Message {message} of component {context}");
            let file = format!("src/widgets/component_{context:03}.cpp");

            buf.push_str(&match message % 5 {
                0 => format!(
                    "        <message>\n\
                     \x20           <location filename=\"{file}\" line=\"{line}\"/>\n\
                     \x20           <location filename=\"src/common/shared.cpp\" line=\"{line}\"/>\n\
                     \x20           <source>{source}</source>\n\
                     \x20           <extracomment>Displayed in the main toolbar.</extracomment>\n\
                     \x20           <translation>Message {message} du composant {context}</translation>\n\
                     \x20       </message>\n",
                    line = message * 7 + 1
                ),
                1 => format!(
                    "        <message id=\"component_{context:03}_message_{message:03}\">\n\
                     \x20           <location filename=\"{file}\" line=\"{line}\"/>\n\
                     \x20           <source>{source}</source>\n\
                     \x20           <comment>Disambiguation for message {message}</comment>\n\
                     \x20           <translation type=\"unfinished\"></translation>\n\
                     \x20       </message>\n",
                    line = message * 7 + 2
                ),
                2 => format!(
                    "        <message>\n\
                     \x20           <location filename=\"{file}\" line=\"{line}\"/>\n\
                     \x20           <source>{source}</source>\n\
                     \x20           <translation type=\"vanished\">Ancien message {message}</translation>\n\
                     \x20       </message>\n",
                    line = message * 7 + 3
                ),
                3 => format!(
                    "        <message>\n\
                     \x20           <location filename=\"{file}\" line=\"{line}\"/>\n\
                     \x20           <source>{source}</source>\n\
                     \x20           <oldsource>Older message {message}</oldsource>\n\
                     \x20           <translation type=\"obsolete\">Message obsolete {message}</translation>\n\
                     \x20       </message>\n",
                    line = message * 7 + 4
                ),
                _ => format!(
                    "        <message numerus=\"yes\">\n\
                     \x20           <location filename=\"{file}\" line=\"{line}\"/>\n\
                     \x20           <source>{source}: %n file(s) selected</source>\n\
                     \x20           <translation>\n\
                     \x20               <numerusform>%n fichier selectionne</numerusform>\n\
                     \x20               <numerusform>%n fichiers selectionnes</numerusform>\n\
                     \x20           </translation>\n\
                     \x20       </message>\n",
                    line = message * 7 + 5
                ),
            });
        }

        buf.push_str("    </context>\n");
    }

    buf.push_str("</TS>");
    buf
}

/// Same document as [`generate_ts`], with updated translations, a different
/// message ordering and extra contexts. Used as the right hand side of the merge
/// benchmark so that matching messages are not found immediately.
fn generate_ts_update(contexts: usize, messages_per_context: usize) -> String {
    let mut content = generate_ts_ordered(contexts, messages_per_context, false)
        .replace("Message ", "Message v2 ");

    // A few contexts are brand new on the right hand side and have to be appended.
    for context in 0..(contexts / 10).max(1) {
        content = content.replace(
            &format!("widgets::Component{context:03}<"),
            &format!("widgets::Added{context:03}<"),
        );
    }

    content
}

fn small_content() -> &'static str {
    static CONTENT: OnceLock<String> = OnceLock::new();
    CONTENT.get_or_init(|| generate_ts(SMALL.0, SMALL.1))
}

fn large_content() -> &'static str {
    static CONTENT: OnceLock<String> = OnceLock::new();
    CONTENT.get_or_init(|| generate_ts(LARGE.0, LARGE.1))
}

fn large_update_content() -> &'static str {
    static CONTENT: OnceLock<String> = OnceLock::new();
    CONTENT.get_or_init(|| generate_ts_update(LARGE.0, LARGE.1))
}

fn parse(content: &str) -> TsDocument {
    TsParser::from_buffer(content.as_bytes().to_vec()).expect("Generated document is parsable")
}

mod parse_ts {
    use super::*;

    /// Parsing a small real translation file from `test_data`.
    #[divan::bench]
    fn real_world_file(bencher: Bencher) {
        bencher
            .with_inputs(|| REAL_WORLD_FILE.as_bytes().to_vec())
            .bench_values(|buffer| black_box(TsParser::from_buffer(buffer)).is_ok());
    }

    /// Parsing a 50 messages document.
    #[divan::bench]
    fn small(bencher: Bencher) {
        bencher
            .with_inputs(|| small_content().as_bytes().to_vec())
            .bench_values(|buffer| black_box(TsParser::from_buffer(buffer)).is_ok());
    }

    /// Parsing a 600 messages document.
    #[divan::bench]
    fn large(bencher: Bencher) {
        bencher
            .with_inputs(|| large_content().as_bytes().to_vec())
            .bench_values(|buffer| black_box(TsParser::from_buffer(buffer)).is_ok());
    }
}

mod serialize_ts {
    use super::*;

    /// Writing a 600 messages document back as XML.
    #[divan::bench]
    fn large(bencher: Bencher) {
        let doc = parse(large_content());

        bencher
            .with_inputs(|| Vec::<u8>::with_capacity(large_content().len()))
            .bench_values(|mut buffer| {
                serialize(&mut buffer, black_box(&doc)).expect("Document is serializable");
                buffer
            });
    }

    /// Full read/write cycle, as performed by most of the sub-commands.
    #[divan::bench]
    fn round_trip(bencher: Bencher) {
        bencher
            .with_inputs(|| large_content().as_bytes().to_vec())
            .bench_values(|buffer| {
                let doc = TsParser::from_buffer(buffer).expect("Document is parsable");
                let mut out = Vec::<u8>::new();
                serialize(&mut out, black_box(&doc)).expect("Document is serializable");
                out
            });
    }
}

mod commands {
    use super::*;

    /// `qt-ts-tools sort`: orders contexts, messages and locations.
    #[divan::bench]
    fn sort(bencher: Bencher) {
        bencher
            .with_inputs(|| parse(large_content()))
            .bench_values(|mut doc| {
                sort_document(black_box(&mut doc));
                doc
            });
    }

    /// `qt-ts-tools strip -t vanished -t obsolete`: drops translation nodes.
    #[divan::bench]
    fn strip(bencher: Bencher) {
        let filter = [TranslationType::Vanished, TranslationType::Obsolete];

        bencher
            .with_inputs(|| parse(large_content()))
            .bench_values(|mut doc| {
                strip_nodes(black_box(&mut doc), &filter);
                doc
            });
    }

    /// `qt-ts-tools extract -t unfinished`: keeps only the matching messages.
    #[divan::bench]
    fn extract(bencher: Bencher) {
        bencher
            .with_inputs(|| parse(large_content()))
            .bench_values(|doc| retain_ts_node(black_box(doc), vec![TranslationType::Unfinished]));
    }

    /// `qt-ts-tools merge`: merges an updated document into a base document.
    #[divan::bench]
    fn merge(bencher: Bencher) {
        bencher
            .with_inputs(|| (parse(large_content()), parse(large_update_content())))
            .bench_values(|(left, right)| merge_ts_nodes(black_box(left), black_box(right), false));
    }

    /// `qt-ts-tools stat`: aggregates per-file and global translation counters.
    #[divan::bench]
    fn stat(bencher: Bencher) {
        let doc = parse(large_content());

        bencher.bench(|| stat_document(black_box(&doc), false));
    }

    /// `qt-ts-tools stat --verbose`: same, with the detailed per-file report.
    #[divan::bench]
    fn stat_verbose(bencher: Bencher) {
        let doc = parse(large_content());

        bencher.bench(|| stat_document(black_box(&doc), true));
    }

    /// `qt-ts-tools release`: compiles the document into the binary QM format.
    #[divan::bench]
    fn release(bencher: Bencher) {
        let doc = parse(large_content());

        bencher
            .with_inputs(|| Cursor::new(Vec::<u8>::with_capacity(64 * 1024)))
            .bench_values(|mut writer| {
                compile_to_buffer(&mut writer, black_box(&doc)).expect("Document is compilable");
                writer
            });
    }
}
