use std::{
    cmp::Ordering,
    io::{BufWriter, Cursor, Write},
};

use clap::{ArgAction, Args};
use log::debug;

use crate::{
    commands::hash::ElfHasher,
    tr,
    ts::{ContextNode, MessageNode, TSNode, TranslationType, YesNo},
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

trait ToUtf16BytesBE {
    fn to_utf16_be_u8(&self) -> Vec<u8>;
}

impl ToUtf16BytesBE for String {
    fn to_utf16_be_u8(&self) -> Vec<u8> {
        self.encode_utf16()
            .flat_map(|u16char| u16char.to_be_bytes())
            .collect::<Vec<u8>>()
    }
}

impl From<BlockTag> for u8 {
    fn from(val: BlockTag) -> u8 {
        val as u8
    }
}

impl From<MessageTag> for u8 {
    fn from(val: MessageTag) -> u8 {
        val as u8
    }
}

fn write_block<W, T>(writer: &mut W, tag: T, data: &[u8]) -> Result<usize, std::io::Error>
where
    W: Write,
    T: Into<u8>,
{
    writer
        .write(&[tag.into()])
        .and_then(|_| writer.write(&(data.len() as u32).to_be_bytes()))
        .and_then(|_| writer.write(data))
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

    write_block(writer, BlockTag::Hashes, &buffer)
}

fn write_lang<W: Write>(writer: &mut W, data: &TSNode) -> Result<usize, std::io::Error> {
    debug!("Writing QM file language");

    match data.language.as_ref() {
        Some(value) => {
            debug!("Found language '{value}'");
            write_block(writer, BlockTag::Language, value.as_bytes())
        }

        None => Err(std::io::Error::other("No language set for TS file.")),
    }
}

///
/// Determines what message should be skipped or kept.
/// Interestingly, QtLinguist keeps unfinished translations.
///
fn keep_message(message: &&MessageNode) -> bool {
    match &message.translation {
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

fn cmp_numerus(msg_left: &&MessageNode, msg_right: &&MessageNode) -> Ordering {
    match (&msg_left.numerus, &msg_right.numerus) {
        (Some(YesNo::Yes), Some(YesNo::No)) => Ordering::Less,
        (Some(YesNo::Yes), None) => Ordering::Less,
        (Some(YesNo::No), Some(YesNo::Yes)) => Ordering::Greater,
        (None, Some(YesNo::Yes)) => Ordering::Greater,
        _ => Ordering::Equal,
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
        context
            .messages
            .iter()
            .filter(keep_message)
            .for_each(|c| ordered_msg.push(c));
        ordered_msg.sort_by(cmp_numerus);

        for message in &ordered_msg {
            let mut buffer = Cursor::new(Vec::<u8>::new());

            match &message.translation.as_ref() {
                Some(node) => {
                    // A valid TS file should not mhave a
                    node.translation_simple
                        .iter()
                        .chain(node.numerus_forms.iter().map(|c| &c.text))
                        .map(|c| {
                            write_block(&mut buffer, MessageTag::Translation, &c.to_utf16_be_u8())
                        })
                        .find(|c| c.is_err())
                        .unwrap_or(Ok(0))
                }
                // No translation, Qt Linguists puts 0xffffff
                None => buffer.write(&[MessageTag::Translation as u8, 0xff, 0xff, 0xff, 0xff]),
            }
            .and_then(|_| {
                // QtLinguist does not seem to keep comments, so out of scope for first implementation
                write_block(&mut buffer, MessageTag::Comment, &[])
            })
            .and_then(|_| {
                // Original string is utf8 (or ascii?) probably to match C++ source files encoding
                match message.source.as_ref() {
                    Some(source) => write_block(&mut buffer, MessageTag::Source, source.as_bytes()),
                    None => Err(std::io::Error::other("Could not find source for message !")),
                }
            })
            .and_then(|_| write_block(&mut buffer, MessageTag::Context, context.name.as_bytes()))
            .and_then(|_| buffer.write(&[MessageTag::End as u8]))
            .map_or_else(|e| Err(e.to_string()), |_| Ok(Vec::<HashAndMessage>::new()))?;

            serialized.push(HashAndMessage {
                hash: ElfHasher::new()
                    .hash(message.source.as_ref().unwrap_or(&"".to_owned()).as_bytes())
                    .compute(),
                msg: buffer.into_inner(),
            });
        }
    }

    Ok(serialized)
}

fn numerus_rule(lang: &str) -> Vec<u8> {
    // First 4 bytes are the block size. Then the rule, then end of rule = 1
    // These are the rules generated with lupdate -list-languages and lrelease
    match lang {
        "fr_CA" => vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x01],
        "aa_ET" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "af_ZA" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "sq_AL" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "am_ET" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ar_EG" => vec![
            0x00, 0x00, 0x00, 0x0f, 0x01, 0x00, 0xff, 0x01, 0x01, 0xff, 0x01, 0x02, 0xff, 0x24,
            0x03, 0x0a, 0xff, 0x2a, 0x0b,
        ],
        "hy_AM" => vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x01],
        "as_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "az_AZ" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ba_RU" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "eu_ES" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "be_BY" => vec![
            0x00, 0x00, 0x00, 0x0d, 0x11, 0x01, 0xfd, 0x29, 0x0b, 0xff, 0x14, 0x02, 0x04, 0xfd,
            0x2c, 0x0a, 0x13,
        ],
        "bn_BD" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "bs_BA" => vec![
            0x00, 0x00, 0x00, 0x0d, 0x11, 0x01, 0xfd, 0x29, 0x0b, 0xff, 0x14, 0x02, 0x04, 0xfd,
            0x2c, 0x0a, 0x13,
        ],
        "br_FR" => vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x01],
        "bg_BG" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "my_MM" => vec![],
        "ca_ES" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "zh_CN" => vec![],
        "kw_GB" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "co_FR" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "hr_HR" => vec![
            0x00, 0x00, 0x00, 0x0d, 0x11, 0x01, 0xfd, 0x29, 0x0b, 0xff, 0x14, 0x02, 0x04, 0xfd,
            0x2c, 0x0a, 0x13,
        ],
        "cs_CZ" => vec![0x00, 0x00, 0x00, 0x06, 0x01, 0x01, 0xff, 0x04, 0x02, 0x04],
        "da_DK" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "dv_MV" => vec![0x00, 0x00, 0x00, 0x05, 0x01, 0x01, 0xff, 0x01, 0x02],
        "nl_NL" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "dz_BT" => vec![],
        "en_US" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "eo_00" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "et_EE" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "fo_FO" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "fil_P" => vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x01],
        "fi_FI" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "fr_FR" => vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x01],
        "fur_I" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "gd_GB" => vec![
            0x00, 0x00, 0x00, 0x0f, 0x01, 0x01, 0xfe, 0x01, 0x0b, 0xff, 0x01, 0x02, 0xfe, 0x01,
            0x0c, 0xff, 0x04, 0x03, 0x13,
        ],
        "gl_ES" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "lg_UG" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ka_GE" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "de_DE" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "el_GR" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "kl_GL" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "gn_PY" => vec![],
        "gu_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ha_NG" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "he_IL" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "hi_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "hu_HU" => vec![],
        "is_IS" => vec![0x00, 0x00, 0x00, 0x05, 0x11, 0x01, 0xfd, 0x29, 0x0b],
        "id_ID" => vec![],
        "ia_00" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "iu_CA" => vec![0x00, 0x00, 0x00, 0x05, 0x01, 0x01, 0xff, 0x01, 0x02],
        "ga_IE" => vec![0x00, 0x00, 0x00, 0x05, 0x01, 0x01, 0xff, 0x01, 0x02],
        "it_IT" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ja_JP" => vec![],
        "jv_ID" => vec![],
        "kn_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ks_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "kk_KZ" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "km_KH" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "rw_RW" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ky_KG" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ko_KR" => vec![],
        "ku_TR" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "lo_LA" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "la_VA" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "lv_LV" => vec![
            0x00, 0x00, 0x00, 0x08, 0x11, 0x01, 0xfd, 0x29, 0x0b, 0xff, 0x09, 0x00,
        ],
        "ln_CD" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "lt_LT" => vec![
            0x00, 0x00, 0x00, 0x0c, 0x11, 0x01, 0xfd, 0x29, 0x0b, 0xff, 0x19, 0x00, 0xfd, 0x2c,
            0x0a, 0x13,
        ],
        "lb_LU" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "mk_MK" => vec![0x00, 0x00, 0x00, 0x05, 0x11, 0x01, 0xff, 0x11, 0x02],
        "mg_MG" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ms_MY" => vec![],
        "ml_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "mt_MT" => vec![
            0x00, 0x00, 0x00, 0x0d, 0x01, 0x01, 0xff, 0x01, 0x00, 0xfe, 0x24, 0x01, 0x0a, 0xff,
            0x24, 0x0b, 0x13,
        ],
        "gv_IM" => vec![0x00, 0x00, 0x00, 0x05, 0x01, 0x01, 0xff, 0x01, 0x02],
        "mi_NZ" => vec![0x00, 0x00, 0x00, 0x05, 0x01, 0x01, 0xff, 0x01, 0x02],
        "mr_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "mn_MN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ne_NP" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "se_NO" => vec![0x00, 0x00, 0x00, 0x05, 0x01, 0x01, 0xff, 0x01, 0x02],
        "nso_Z" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "nb_NO" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "nn_NO" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "oc_FR" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "or_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "om_ET" => vec![],
        "ps_AF" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "fa_IR" => vec![],
        "pl_PL" => vec![
            0x00, 0x00, 0x00, 0x0a, 0x01, 0x01, 0xff, 0x14, 0x02, 0x04, 0xfd, 0x2c, 0x0a, 0x13,
        ],
        "pt_BR" => vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x01],
        "pa_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "qu_PE" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ro_RO" => vec![
            0x00, 0x00, 0x00, 0x09, 0x01, 0x01, 0xff, 0x01, 0x00, 0xfe, 0x24, 0x01, 0x13,
        ],
        "rm_CH" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "rn_BI" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ru_RU" => vec![
            0x00, 0x00, 0x00, 0x0d, 0x11, 0x01, 0xfd, 0x29, 0x0b, 0xff, 0x14, 0x02, 0x04, 0xfd,
            0x2c, 0x0a, 0x13,
        ],
        "sa_IN" => vec![0x00, 0x00, 0x00, 0x05, 0x01, 0x01, 0xff, 0x01, 0x02],
        "sr_RS" => vec![
            0x00, 0x00, 0x00, 0x0d, 0x11, 0x01, 0xfd, 0x29, 0x0b, 0xff, 0x14, 0x02, 0x04, 0xfd,
            0x2c, 0x0a, 0x13,
        ],
        "sn_ZW" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "sd_PK" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "si_LK" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "sk_SK" => vec![0x00, 0x00, 0x00, 0x06, 0x01, 0x01, 0xff, 0x04, 0x02, 0x04],
        "sl_SI" => vec![
            0x00, 0x00, 0x00, 0x09, 0x21, 0x01, 0xff, 0x21, 0x02, 0xff, 0x24, 0x03, 0x04,
        ],
        "so_SO" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "st_ZA" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "es_ES" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "su_ID" => vec![],
        "sw_TZ" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ss_ZA" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "sv_SE" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "tg_TJ" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ta_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "tt_RU" => vec![],
        "te_IN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "th_TH" => vec![],
        "bo_CN" => vec![],
        "ti_ET" => vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x01],
        "to_TO" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ts_ZA" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "tn_ZA" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "tr_TR" => vec![],
        "tk_TM" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "ug_CN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "uk_UA" => vec![
            0x00, 0x00, 0x00, 0x0d, 0x11, 0x01, 0xfd, 0x29, 0x0b, 0xff, 0x14, 0x02, 0x04, 0xfd,
            0x2c, 0x0a, 0x13,
        ],
        "ur_PK" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "uz_UZ" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "vi_VN" => vec![],
        "vo_00" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "wa_BE" => vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x01],
        "cy_GB" => vec![
            0x00, 0x00, 0x00, 0x0c, 0x01, 0x00, 0xff, 0x01, 0x01, 0xff, 0x04, 0x02, 0x05, 0xff,
            0x01, 0x06,
        ],
        "fy_NL" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "wo_SN" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "xh_ZA" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "yi_00" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],
        "yo_NG" => vec![],
        "zu_ZA" => vec![0x00, 0x00, 0x00, 0x02, 0x01, 0x01],

        _ => vec![],
    }
}

fn compile_to_buffer<W: Write>(writer: &mut W, data: &TSNode) -> Result<(), String> {
    let msgs = produce_messages(data)?;
    let msg_block: Vec<u8> = msgs.iter().flat_map(|hm| &hm.msg).copied().collect();
    let lang = numerus_rule(&data.language.as_ref().unwrap_or(&String::new()));

    writer
        .write(&QM_HEADER)
        .and_then(|_| write_lang(writer, data))
        .and_then(|_| write_hashes(writer, &msgs))
        .and_then(|_| writer.write(&[BlockTag::Messages as u8]))
        .and_then(|_| writer.write(&(msg_block.len() as u32).to_be_bytes()))
        .and_then(|_| writer.write(&msg_block))
        .and_then(|_| writer.write(&[BlockTag::NumerusRules as u8]))
        .and_then(|_| writer.write(&lang))
        .map_err(|e| e.to_string())
        .map(|_| ())
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
