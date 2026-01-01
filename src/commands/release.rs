use std::{
    cmp::Ordering,
    io::{BufWriter, Cursor, Write},
};

use clap::{ArgAction, Args};
use log::{debug, error};

use crate::{
    commands::hash::ElfHasher,
    tr,
    ts::{ContextNode, MessageNode, TSNode, TranslationNode, TranslationType, YesNo},
};

#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct ReleaseArgs {
    /// File to release
    #[arg(help = tr!("cli-release-input"), help_heading = tr!("cli-headers-arguments"))]
    pub input: String,
    /// If specified, will produce output in a file at designated location instead of stdout.
    #[arg(short, long, help = tr!("cli-release-output"), help_heading = tr!("cli-headers-options"))]
    pub output_path: Option<String>,
    #[arg(short, long, action = ArgAction::Help, help = tr!("cli-help"), help_heading = tr!("cli-headers-options"))]
    pub help: Option<bool>,
}

pub fn release_main(args: &ReleaseArgs) -> Result<(), String> {
    let f = quick_xml::Reader::from_file(&args.input).expect("Couldn't open source file");
    let data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
    let mut writer = Cursor::new(Vec::<u8>::new());

    compile_to_buffer(&mut writer, &data)
        .and_then(|_| write_output(&args.output_path, &writer.into_inner()))
}

fn write_output(output: &Option<String>, data: &[u8]) -> Result<(), String> {
    let mut buf: BufWriter<Box<dyn Write>> = match output {
        None => BufWriter::new(Box::new(std::io::stdout().lock())),
        Some(path) => match std::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
        {
            Ok(file) => BufWriter::new(Box::new(file)),
            Err(e) => {
                return Err(tr!(
                    "error-write-output-open",
                    output_path = path,
                    error = e.to_string()
                ));
            }
        },
    };

    match buf.write(data) {
        Ok(written) => {
            if written == data.len() {
                Ok(())
            } else {
                Err(format!(
                    "Could not write to the file completely! {written} bytes written, {} bytes expected",
                    data.len()
                ))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

///
/// This is a fixed identifier for Qt's QM files, probably serving as a file identifier.
///
const QM_HEADER: [u8; 16] = [
    0x3c, 0xb8, 0x64, 0x18, 0xca, 0xef, 0x9c, 0x95, 0xcd, 0x21, 0x1c, 0xbf, 0x60, 0xa1, 0xbd, 0xdd,
];

///
/// The QM top level structure blocks.
/// See [docs/qm_file.md] for details.
///
#[repr(u8)]
enum BlockTag {
    /// Block for language encoding data (bcp47)
    Language = 0xa7,
    /// Block for the hashes table.
    /// The hashes are messages' hash pointing to the actual message in the message block.
    /// This is used for quick lookup when loading the QM in the Qt application.
    Hashes = 0x42,
    /// Messages table block.
    /// This contain the original string (utf-8), translation (utf-16), context name.
    Messages = 0x69,
    /// Numerus rule block
    /// This is expressed as an encoded formula depending on the language.
    NumerusRules = 0x88,

    // Below is unsupported.
    _Contexts = 0x2f,
    _Dependencies = 0x96,
}

/// A message entry in the messages table is split by its component.
/// This structure expresses the tags to identify said components.
#[repr(u8)]
enum MessageTag {
    /// Translated, utf-16
    Translation = 0x03,

    /// Original, untranslated string, utf-8
    Source = 0x06,

    // End of message
    End = 0x01,

    // Context name to which this message is associated
    Context = 0x07,

    // Comment associated with the message. Unsupported for now.
    Comment = 0x08,
}

/// This structure represents an hash table entry.
/// The hash itself is the message original text hashed,
/// the offset is a pointer to the full message and its translation.
struct HashAndOffset {
    hash: u32,
    offset: u32,
}

struct HashAndMessage {
    hash: u32,
    msg: Vec<u8>,
}

fn write_hashes<W: Write>(
    writer: &mut W,
    hashed_messages: &[HashAndMessage],
) -> Result<usize, std::io::Error> {
    let mut buffer: Vec<u8> = vec![];
    let mut hashes: Vec<HashAndOffset> = vec![];

    debug!(
        "QM Hashes Table: {} entries (hash, offset).",
        hashed_messages.len()
    );

    hashed_messages.iter().fold(0u32, |distance, hm| {
        debug!(
            "\t{:02X?} => {:02X?}",
            hm.hash.to_be_bytes(),
            distance.to_be_bytes()
        );

        hashes.push(HashAndOffset {
            hash: hm.hash,
            offset: distance,
        });
        distance + hm.msg.len() as u32
    });

    // While the messages offsets are calculated while not sorted, the hash tables generated
    // by Qt Linguist appear to be sorted. We try to not deviate from that to simplify testing and matching the file.
    hashes.sort_by_key(|d| d.hash);

    for ho in hashes {
        buffer.extend(&ho.hash.to_be_bytes());
        buffer.extend(&ho.offset.to_be_bytes());
    }

    writer
        .write(&[BlockTag::Hashes as u8])
        .and_then(|_| writer.write(&(buffer.len() as u32).to_be_bytes()))
        .and_then(|_| writer.write(&buffer))
}

fn write_lang<W: Write>(writer: &mut W, data: &TSNode) -> Result<usize, std::io::Error> {
    debug!("Writing QM file language");

    match data.language.as_ref() {
        Some(value) => {
            debug!("Found language '{value}'");
            writer
                .write(&[BlockTag::Language as u8])
                .and_then(|_| writer.write(&(value.len() as u32).to_be_bytes()))
                .and_then(|_| writer.write(value.as_bytes()))
        }

        None => Err(std::io::Error::other("No language set for TS file.")),
    }
}

///
/// Determines what message should be skipped or kept.
/// Interestingly, QtLinguist keeps unfinished translations.
///
fn keep_message(translation: &Option<TranslationNode>) -> bool {
    match translation {
        None => true, // Qt Linguist keeps missing nodes too -- empty translation
        Some(t) => match &t.translation_type {
            Some(tt) => match tt {
                TranslationType::Finished => true,
                TranslationType::Unfinished => true,
                TranslationType::Obsolete => false,
                TranslationType::Vanished => false,
            },
            None => true,
        },
    }
}

fn produce_messages(data: &TSNode) -> Result<Vec<HashAndMessage>, String> {
    let mut serialized: Vec<HashAndMessage> = vec![];

    // View on context nodes in order to sort them without affecting the original collection
    let mut ordered_ctx: Vec<&ContextNode> = vec![];
    data.contexts.iter().for_each(|c| ordered_ctx.push(c));
    ordered_ctx.sort_by_key(|l| &l.name);

    for context in &ordered_ctx {
        // Numerus messages are put first by Qt Linguist
        let mut ordered_msg: Vec<&MessageNode> = vec![];
        context.messages.iter().for_each(|c| ordered_msg.push(c));
        ordered_msg.sort_by(|m, m2| match (&m.numerus, &m2.numerus) {
            (Some(YesNo::Yes), Some(YesNo::No)) => Ordering::Less,
            (Some(YesNo::Yes), None) => Ordering::Less,
            (Some(YesNo::No), Some(YesNo::Yes)) => Ordering::Greater,
            (None, Some(YesNo::Yes)) => Ordering::Greater,
            _ => Ordering::Equal,
        });

        for message in &ordered_msg {
            if !keep_message(&message.translation) {
                continue;
            }

            let mut buffer: Vec<u8> = vec![];

            match &message.translation.as_ref() {
                Some(node) => {
                    match message.numerus {
                        Some(YesNo::Yes) => {
                            debug!("Processing {} numerus node", node.numerus_forms.len());
                            let tdata = node
                                .numerus_forms
                                .iter()
                                .map(|data| {
                                    data.text
                                        .encode_utf16()
                                        .flat_map(|value| value.to_be_bytes())
                                        .collect::<Vec<u8>>()
                                })
                                .flat_map(|d| {
                                    let mut a = vec![MessageTag::Translation as u8];
                                    a.extend((d.len() as u32).to_be_bytes());
                                    a.extend(d);
                                    a
                                })
                                .collect::<Vec<u8>>();

                            buffer.extend(tdata.as_slice());
                        }
                        _ => {
                            let tdata = if let Some(translation) = node.translation_simple.as_ref()
                            {
                                translation
                                    .encode_utf16()
                                    .flat_map(|value| value.to_be_bytes())
                                    .collect::<Vec<u8>>()
                            } else {
                                // Invalid state.
                                error!(
                                    "Reached an invalid scenario: numerus is either absent or set to \"no\" and there is no translation node"
                                );
                                todo!("Handle error case")
                            };

                            buffer.extend(&[MessageTag::Translation as u8]);
                            buffer.extend(&(tdata.len() as u32).to_be_bytes());
                            buffer.extend(tdata.as_slice());
                        }
                    }
                }
                // No translation, Qt Linguists puts 0xffffff
                None => buffer.extend(&[MessageTag::Translation as u8, 0xff, 0xff, 0xff, 0xff]),
            }

            //
            // COMMENT: QtLinguist does not seem to keep comments, so out of scope for first
            // implementation
            //
            buffer.extend(&[MessageTag::Comment as u8]);
            buffer.extend(&0u32.to_be_bytes());

            //
            // SOURCE: Probably to match C++ source code encoding, it appears that default utf-8 strings
            // works fine.
            //
            buffer.extend(&[MessageTag::Source as u8]);
            buffer.extend(
                &(message.source.as_ref().expect("TO HAVE A SOURCE").len() as u32).to_be_bytes(),
            );
            buffer.extend(message.source.as_ref().expect("have a source").as_bytes());

            //
            // CONTEXT
            //
            buffer.extend(&[MessageTag::Context as u8]);
            buffer.extend(&(context.name.len() as u32).to_be_bytes());
            buffer.extend(context.name.as_bytes());

            //
            // END
            //
            buffer.extend(&[MessageTag::End as u8]);
            serialized.push(HashAndMessage {
                hash: ElfHasher::new()
                    .hash(message.source.as_ref().unwrap_or(&"".to_owned()).as_bytes())
                    .compute(),
                msg: buffer,
            });
        }
    }

    Ok(serialized)
}

fn compile_to_buffer<W: Write>(writer: &mut W, data: &TSNode) -> Result<(), String> {
    let msgs = produce_messages(data)?;
    let msg_block: Vec<u8> = msgs.iter().flat_map(|hm| &hm.msg).copied().collect();

    writer
        .write(&QM_HEADER)
        .and_then(|_| write_lang(writer, data))
        .and_then(|_| write_hashes(writer, &msgs))
        .and_then(|_| writer.write(&[BlockTag::Messages as u8]))
        .and_then(|_| writer.write(&(msg_block.len() as u32).to_be_bytes()))
        .and_then(|_| writer.write(&msg_block))
        .map_err(|e| e.to_string())
        .map(|_| ())?;

    //
    // NUMERUS: TODO
    // These are rules computed according to the target locale, giving information about what form
    // to use etc etc. This is a bit more advanced to reverse engineer. So for now let's pretend there's none.
    //
    writer
        .write(&[BlockTag::NumerusRules as u8])
        .and_then(|_| writer.write(&2u32.to_be_bytes()))
        .and_then(|_| writer.write(&[0u8])) // Operation mask ?
        .and_then(|_| writer.write(&[0u8])) // Operation operand ?
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod release_tests {
    use rstest::{fixture, rstest};

    use crate::{commands::release::compile_to_buffer, logging::initialize_logging};

    #[fixture]
    #[once]
    fn logs() {
        // Recommended to run test as `RUST_LOG=debug cargo test release -- --test-threads=1`
        initialize_logging();
    }

    #[rstest]
    #[case::one_ctx_one_msg("simple")]
    #[case::one_ctx_many_msgs("one_ctx_many_msgs")]
    #[case::many_ctx_many_msgs("many_ctx_many_msgs")]
    #[case::unfinished_translation("many_ctx_many_msgs_non_finished")]
    #[case::missing_translation_tag("many_ctx_many_msgs_notranslation_tag")]
    #[case::with_numerus("many_ctx_many_msgs_numerus")]
    fn compile_ts_to_qm(#[case] case: &str, #[allow(unused)] logs: ()) {
        let expected_data = std::fs::read(format!("./test_data/{case}.qm")).expect("File to exist");
        let base_ts_data =
            quick_xml::Reader::from_file(format!("./test_data/{case}.ts")).expect("File to exist");
        let ts_node = quick_xml::de::from_reader(base_ts_data.into_inner()).expect("Parsable");

        let mut writer = std::io::Cursor::new(Vec::<u8>::new());

        let result = compile_to_buffer(&mut writer, &ts_node);

        assert_eq!(result, Ok(()));
        assert_eq!(writer.into_inner(), expected_data);
    }
}
