use std::{
    io::{BufReader, Write},
    path::Path,
};

use crate::{
    commands::hash::SysVHasher,
    ts::{ContextNode, TSNode},
};

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

fn compile_file(file: &Path, target: &Path) -> Result<(), String> {
    let f = quick_xml::Reader::from_file(file).expect("Couldn't open source file");

    let data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");

    Ok(())
}

fn get_bcp47(input: &String) -> Result<String, String> {
    // TODO: better impl
    match input.to_lowercase().as_str() {
        "fr" => Ok("fr_FR".to_string()),
        "sv" => Ok("sv_SE".to_string()),
        _ => Err("BCP 47 not found (invalid target language code)".to_string()),
    }
}

struct HashAndOffset {
    hash: u32,
    offset: u32,
}

/// TODO: Hashes require to know the distance to where the message and contexts
/// are written to. Navigating per tables.
fn write_hashes<W: Write>(
    writer: &mut W,
    data: &[ContextNode],
    msgs: &Vec<HashAndMessage>,
) -> Result<(), String> {
    let mut hashes_buffer: Vec<u8> = vec![];
    let mut hashes: Vec<HashAndOffset> = vec![];

    for context in data {
        for message in &context.messages {
            let hash = SysVHasher::new()
                .hash(message.source.as_ref().expect("Have source").as_bytes())
                .compute();
            let distance = msgs
                .iter()
                .take_while(|hm| hm.hash != hash)
                .map(|hm| hm.msg.len() as u32)
                .sum::<u32>();

            hashes.push(HashAndOffset {
                hash: hash,
                offset: distance,
            });
        }
    }
    hashes.sort_by_key(|d| d.hash);
    for ho in hashes {
        hashes_buffer.extend(&ho.hash.to_be_bytes());
        // TODO: Expected position or offset:
        hashes_buffer.extend(&ho.offset.to_be_bytes());
    }
    writer.write(&[BlockTag::Hashes as u8]);
    writer.write(&(hashes_buffer.len() as u32).to_be_bytes());
    writer.write(&hashes_buffer);

    Ok(())
}

fn write_lang<W: Write>(writer: &mut W, data: &TSNode) -> Result<(), String> {
    data.language
        .as_ref()
        .map_or_else(|| Err("No language set".to_owned()), get_bcp47)
        .and_then(|value| {
            let lang_length = (value.len() as u32).to_be_bytes();

            let mut buffer = Vec::from(&[BlockTag::Language as u8]);
            buffer.extend(lang_length);
            buffer.extend(value.as_bytes());

            writer.write(&buffer).map(|_| ()).map_err(|e| e.to_string())
        })
}

struct HashAndMessage {
    hash: u32,
    msg: Vec<u8>,
}

fn produce_messages(data: &TSNode) -> Result<Vec<HashAndMessage>, String> {
    let mut serialized: Vec<HashAndMessage> = vec![];

    for context in &data.contexts {
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
            // COMMENT: TODO
            //
            buffer.extend(&[MessageTag::Comment as u8]);
            if let Some(comment) = &message.comment {
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
    writer.write(&QM_HEADER);

    write_lang(writer, data)?;

    let msgs = produce_messages(&data)?;
    let hashes = write_hashes(writer, &data.contexts, &msgs)?;

    //    let mut result: Vec<u8> = vec![];
    writer.write(&[BlockTag::Messages as u8]);
    writer.write(&(msgs.iter().map(|hm| hm.msg.len() as u32).sum::<u32>() as u32).to_be_bytes());
    for message in msgs {
        writer.write(&message.msg);
    }
    //
    // NUMERUS: TODO
    //
    writer.write(&[BlockTag::NumerusRules as u8]);
    writer.write(&2u32.to_be_bytes()); // length of the numerus buffer
    writer.write(&[3u8]); // Q_OP_LEQ not sure
    writer.write(&[1u8]);

    Ok(())
}

#[cfg(test)]
mod release_tests {
    use assert_hex::assert_eq_hex;

    use crate::commands::release::{QM_HEADER, compile_to_buffer};

    #[test]
    fn compiling_single_context_single_message_file() {
        let expected_data = std::fs::read("./test_data/simple.qm").expect("File to exist");
        let base_ts_data =
            quick_xml::Reader::from_file("./test_data/simple.ts").expect("File to exist");
        let ts_node = quick_xml::de::from_reader(base_ts_data.into_inner()).expect("Parsable");

        let mut buf = Vec::<u8>::new();
        let mut writer = std::io::Cursor::new(&mut buf);

        let result = compile_to_buffer(&mut writer, &ts_node);

        assert_eq!(result, Ok(()));
        assert_eq!(*buf.iter().as_slice(), expected_data);
    }

    #[test]
    fn compiling_single_context_multi_message_file() {
        let expected_data =
            std::fs::read("./test_data/example_multimessage.qm").expect("File to exist");
        let base_ts_data = quick_xml::Reader::from_file("./test_data/example_multimessage.ts")
            .expect("File to exist");
        let ts_node = quick_xml::de::from_reader(base_ts_data.into_inner()).expect("Parsable");

        let mut buf = Vec::<u8>::new();
        let mut writer = std::io::Cursor::new(&mut buf);

        let result = compile_to_buffer(&mut writer, &ts_node);
        println!("{buf:02X?}");
        assert_eq!(result, Ok(()));
        assert_eq!(*buf.iter().as_slice(), expected_data);
        println!("{buf:X?}");
    }
}
