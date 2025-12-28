use std::{
    io::{BufReader, Write},
    path::Path,
};

use crate::{commands::hash::SysVHasher, ts::TSNode};
use clap::builder::TypedValueParser;

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

fn get_bcp47(input: &str) -> Result<String, String> {
    // TODO: better impl
    match input.to_lowercase().as_str() {
        "fr" => Ok("fr_FR".to_string()),
        _ => Err("BCP 47 not found (invalid target language code)".to_string()),
    }
}

fn compile_to_buffer<W: Write>(writer: &mut W, data: &TSNode) -> Result<(), String> {
    writer.write(&QM_HEADER);

    let lang = data.language.clone().expect("Have a language");
    let lang_data = get_bcp47(&lang).expect("Have a language");
    let lang_length = (lang_data.len() as u32).to_be_bytes();
    writer.write(&[BlockTag::Language as u8]);
    writer.write(&lang_length);
    writer.write(lang_data.as_bytes());

    // Not sure yet how to compute hash
    for context in &data.contexts {
        for message in &context.messages {
            let computed_hash = SysVHasher::new()
                .hash_with(message.source.as_ref().expect("Have source").as_bytes())
                .finish()
                .to_be_bytes();

            writer.write(&[BlockTag::Hashes as u8]);
            writer.write(&8u32.to_be_bytes());
            writer.write(&computed_hash);
            // TODO: determine why hash is written on 8 bytes ?
            // maybe because it's 2 hashes ? Maybe 1 hash per message, but the array is before messages.
            writer.write(&0u32.to_be_bytes());

            //
            // MESSAGE
            //
            writer.write(&[BlockTag::Messages as u8]);
            // TODO: write size, how ?
            writer.write(&49u32.to_be_bytes()); // TODO: place holder
            writer.write(&[MessageTag::Translation as u8]);

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

            writer.write(&(translation_utf16.len() as u32).to_be_bytes());
            writer.write(&translation_utf16.as_slice());

            //
            // COMMENT: TODO
            //
            writer.write(&[MessageTag::Comment as u8]);
            if let Some(comment) = &message.comment {
                writer.write(
                    &(message.comment.as_ref().unwrap_or(&String::new()).len() as u32)
                        .to_be_bytes(),
                );
                writer.write(comment.as_bytes());
            } else {
                writer.write(&0u32.to_be_bytes());
            }

            //
            // SOURCE
            //
            writer.write(&[MessageTag::Source as u8]);
            writer.write(
                &(message.source.as_ref().expect("TO HAVE A SOURCE").len() as u32).to_be_bytes(),
            );
            writer.write(&(message.source.as_ref().expect("have a source").as_bytes()));

            //
            // CONTEXT: TODO
            //
            writer.write(&[MessageTag::Context as u8]);
            if !context.name.is_empty() {
                writer.write(&(context.name.len() as u32).to_be_bytes());
                writer.write(context.name.as_bytes());
            } else {
                writer.write(&0u32.to_be_bytes());
            }

            //
            // END
            //
            writer.write(&[MessageTag::End as u8]);

            //
            // NUMERUS: TODO
            //
        }

        writer.write(&[BlockTag::NumerusRules as u8]);
        writer.write(&2u32.to_be_bytes());
        writer.write(&[3u8]); // Q_OP_LEQ not sure
        writer.write(&[1u8]);
    }

    Ok(())
}

#[cfg(test)]
mod release_tests {
    use crate::commands::release::{QM_HEADER, compile_to_buffer};

    #[test]
    fn compiling_empty_ts_file() {
        let expected_data = std::fs::read("./test_data/simple.qm").expect("File to exist");
        let base_ts_data =
            quick_xml::Reader::from_file("./test_data/simple.ts").expect("File to exist");
        let ts_node = quick_xml::de::from_reader(base_ts_data.into_inner()).expect("Parsable");

        let mut buf = Vec::<u8>::new();
        let mut writer = std::io::Cursor::new(&mut buf);

        compile_to_buffer(&mut writer, &ts_node);

        // TODO: Eventually compare the full file, but for now this
        // helps getting started.
        let lang_length = 1 + 4 + 5; // tag(1byte) + block_size(4bytes) + value.len()
        let hashes_length = 1 + 4 + 8;
        let messages_length = 1 + 4 + 13;
        assert_eq!(*buf.iter().as_slice(), expected_data);
    }
}
