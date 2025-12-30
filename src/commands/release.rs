use std::{io::Write, path::Path};

use log::debug;

use crate::{
    commands::hash::SysVHasher,
    ts::{ContextNode, TSNode},
};

///
/// This is a fixed identifier for Qt's QM files.
///
const QM_HEADER: [u8; 16] = [
    0x3c, 0xb8, 0x64, 0x18, 0xca, 0xef, 0x9c, 0x95, 0xcd, 0x21, 0x1c, 0xbf, 0x60, 0xa1, 0xbd, 0xdd,
];

#[repr(u8)]
enum BlockTag {
    Language = 0xa7,
    Hashes = 0x42,
    Messages = 0x69,
    NumerusRules = 0x88,
    Contexts = 0x2f,
    Dependencies = 0x96,
}

#[repr(u8)]
enum MessageTag {
    Translation = 0x03,
    Source = 0x06,
    End = 0x01,
    Context = 0x07,
    Comment = 0x08,
}

struct HashAndOffset {
    hash: u32,
    offset: u32,
}

struct HashAndMessage {
    hash: u32,
    msg: Vec<u8>,
}

pub fn compile_file(file: &Path, target: &Path) -> Result<(), String> {
    let f = quick_xml::Reader::from_file(file).expect("Couldn't open source file");

    let data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");

    Ok(())
}

///
/// Resolves the BCP47 code for the provided shorthand code
/// e.g. "fr" -> "fr_FR"
///
/// TODO: This function is brittle and should be replaced by a crate.
fn get_bcp47(input: &String) -> Option<String> {
    debug!("Resolved language: {input}");
    match input.to_lowercase().as_str() {
        "fr" => Some("fr_FR".to_string()),
        "sv" => Some("sv_SE".to_string()),
        _ => {
            debug!("No BCP47 correspondance found!");
            None
        }
    }
}

fn write_hashes<W: Write>(
    writer: &mut W,
    hashed_messages: &Vec<HashAndMessage>,
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
    let bcp47 = data.language.as_ref().and_then(|v| get_bcp47(v));

    match bcp47 {
        Some(value) => writer
            .write(&[BlockTag::Language as u8])
            .and_then(|_| writer.write(&(value.len() as u32).to_be_bytes()))
            .and_then(|_| writer.write(value.as_bytes())),

        None => Err(std::io::Error::other("Invalid language set for TS file.")),
    }
}

fn produce_messages(data: &TSNode) -> Result<Vec<HashAndMessage>, String> {
    let mut serialized: Vec<HashAndMessage> = vec![];

    // View on context nodes in order to sort them without affecting the original collection
    let mut ordered_ctx: Vec<&ContextNode> = vec![];
    data.contexts.iter().for_each(|c| ordered_ctx.push(c));
    ordered_ctx.sort_by_key(|l| &l.name);

    for context in &ordered_ctx {
        for message in &context.messages {
            let mut buffer: Vec<u8> = vec![];
            buffer.extend(&[MessageTag::Translation as u8]);

            let translation_utf16 = &message
                .translation
                .as_ref()
                .expect("Supposed to have translation")
                .translation_simple
                .as_ref()
                .expect("TODO: Supposed to have translation")
                .encode_utf16()
                .map(|value| value.to_be_bytes())
                .flatten()
                .collect::<Vec<u8>>();

            buffer.extend(&(translation_utf16.len() as u32).to_be_bytes());
            buffer.extend(translation_utf16.as_slice());

            //
            // COMMENT: TODO -- do not include comments until activated by flag ?
            //
            buffer.extend(&[MessageTag::Comment as u8]);
            if let Some(comment) = &message.comment
                && false
            {
                buffer.extend(
                    &(message.comment.as_ref().unwrap_or(&String::new()).len() as u32)
                        .to_be_bytes(),
                );
                buffer.extend(comment.as_bytes());
            } else {
                buffer.extend(&0u32.to_be_bytes());
            }

            //
            // SOURCE
            //

            buffer.extend(&[MessageTag::Source as u8]);
            buffer.extend(
                &(message.source.as_ref().expect("TO HAVE A SOURCE").len() as u32).to_be_bytes(),
            );
            buffer.extend(message.source.as_ref().expect("have a source").as_bytes());

            //
            // CONTEXT: TODO
            //
            buffer.extend(&[MessageTag::Context as u8]);
            if !context.name.is_empty() {
                buffer.extend(&(context.name.len() as u32).to_be_bytes());
                buffer.extend(context.name.as_bytes());
            } else {
                buffer.extend(&0u32.to_be_bytes());
            }

            //
            // END
            //
            buffer.extend(&[MessageTag::End as u8]);
            serialized.push(HashAndMessage {
                hash: SysVHasher::new()
                    .hash(message.source.as_ref().unwrap_or(&"".to_owned()).as_bytes())
                    .compute(),
                msg: buffer,
            });
        }
    }

    Ok(serialized)
}

fn compile_to_buffer<W: Write>(writer: &mut W, data: &TSNode) -> Result<(), String> {
    let msgs = produce_messages(&data)?;
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
    use rstest::rstest;

    use crate::commands::release::compile_to_buffer;

    #[rstest]
    #[case::one_ctx_one_msg("simple")]
    #[case::one_ctx_many_msgs("one_ctx_many_msgs")]
    #[case::many_ctx_many_msgs("many_ctx_many_msgs")]
    fn compile_ts_to_qm(#[case] case: &str) {
        let expected_data = std::fs::read(format!("./test_data/{case}.qm")).expect("File to exist");
        let base_ts_data =
            quick_xml::Reader::from_file(format!("./test_data/{case}.ts")).expect("File to exist");
        let ts_node = quick_xml::de::from_reader(base_ts_data.into_inner()).expect("Parsable");

        let mut buf = Vec::<u8>::new();
        let mut writer = std::io::Cursor::new(&mut buf);

        let result = compile_to_buffer(&mut writer, &ts_node);

        //        println!("{:02X?}", &buf.as_slice());
        //        std::fs::write(format!("./test_data/output_{case}"), &buf);
        assert_eq!(result, Ok(()));
        assert_eq!(buf.iter().as_slice(), &expected_data);
    }
}
