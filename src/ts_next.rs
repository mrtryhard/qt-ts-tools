use log::debug;
use quick_xml::events::Event;
use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;

use crate::parse_error::ParseError;

type TsBytes<'a> = Cow<'a, [u8]>;
// This file defines the schema matching (or trying to match?) Qt's XSD
// Eventually when a proper Rust code generator exists it would be great to use that instead.
// For now they can't handle Qt's semi-weird XSD.
// https://doc.qt.io/qt-6/linguist-ts-file-format.html

/// TranslationType defines the status of a translation (aka the progress)
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum TranslationType {
    /// Translation is completed
    #[default]
    Finished,
    /// Translation is not finished
    Unfinished,
    /// Translation requires an update
    Obsolete,
    /// Translation is not used anymore
    Vanished,
}

impl<'a> From<Cow<'a, [u8]>> for TranslationType {
    fn from(value: Cow<'a, [u8]>) -> Self {
        let trimmed = value.trim_ascii();
        if trimmed.eq_ignore_ascii_case(b"unfinished") {
            TranslationType::Unfinished
        } else if trimmed.eq_ignore_ascii_case(b"obsolete") {
            TranslationType::Obsolete
        } else if trimmed.eq_ignore_ascii_case(b"vanished") {
            TranslationType::Vanished
        } else {
            TranslationType::Finished
        }
    }
}

#[derive(Debug, Eq, Clone, PartialEq)]
pub enum YesNo {
    Yes,
    No,
}

impl<'a> From<Cow<'a, [u8]>> for YesNo {
    fn from(value: Cow<'a, [u8]>) -> Self {
        if value.trim_ascii().eq_ignore_ascii_case(b"yes") {
            YesNo::Yes
        } else {
            YesNo::No
        }
    }
}

/// Root node of the translation file.
#[derive(Debug, Default, PartialEq)]
pub struct TSNode<'a> {
    buf: Rc<Vec<u8>>,
    /// Defines the version of the TS format, although unused by this tool.
    /// attribute -- do not serialize if missing
    pub version: Option<TsBytes<'a>>,
    /// Source language on which this translation is based on.
    /// #[serde(rename = "@sourcelanguage", skip_serializing_if = "Option::is_none")]
    pub source_language: Option<TsBytes<'a>>,
    /// Language of this translation.
    pub language: Option<TsBytes<'a>>,
    //     /// Translations attached to a context
    pub contexts: Vec<ContextNode>,
    //     /// #[serde(skip_serializing_if = "Option::is_none")]
    //     pub dependencies: Option<DependenciesNode>,
    //     /// Translation comment.
    //     /// #[serde(skip_serializing_if = "Option::is_none")]
    //     pub comment: Option<String>,
    //     /// Previous translation comment.
    //     /// #[serde(rename = "oldcomment", skip_serializing_if = "Option::is_none")]
    //     pub old_comment: Option<String>,
    //     /// Other, extra comment
    //     /// #[serde(rename = "extracomment", skip_serializing_if = "Option::is_none")]
    //     pub extra_comment: Option<String>,
    //     /// Translator comment
    //     /// #[serde(rename = "translatorcomment", skip_serializing_if = "Option::is_none")]
    //     pub translator_comment: Option<String>,
    //     /*
    //        Following section corresponds to `extra-something` in Qt's XSD. From documentation:
    //        > extra elements may appear in TS and message elements. Each element may appear
    //        > only once within each scope. The contents are preserved verbatim; any
    //        > attributes are dropped.
    //     */
    //     /// #[serde(
    //     ///    rename = "extra-po-msgid_plural",
    //     ///    skip_serializing_if = "Option::is_none"
    //     ///)]
    //     pub po_msg_id_plural: Option<String>,
    //     ///#[serde(
    //     ///    rename = "extra-po-old_msgid_plural",
    //     ///    skip_serializing_if = "Option::is_none"
    //     ///)]
    //     pub po_old_msg_id_plural: Option<String>,
    //     /// Comma separated list
    //     ///#[serde(rename = "extra-po-flags", skip_serializing_if = "Option::is_none")]
    //     pub loc_flags: Option<String>,
    //     ///#[serde(
    //     ///  rename = "extra-loc-layout_id",
    //     ///skip_serializing_if = "Option::is_none"
    //     ///)]
    //     pub loc_layout_id: Option<String>,
    //     ///#[serde(rename = "extra-loc-feature", skip_serializing_if = "Option::is_none")]
    //     pub loc_feature: Option<String>,
    //     ///#[serde(rename = "extra-loc-blank", skip_serializing_if = "Option::is_none")]
    //     pub loc_blank: Option<String>,
}

/// Context and its associated translated message.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ContextNode<'a> {
    /// Unique name of the context
    pub name: Option<TsBytes<'a>>,
    /// List of translation messages
    ///#[serde(rename = "message")]
    pub messages: Vec<MessageNode>,
    /// Comment describing information about the context
    ///#[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Encoding of the messages within that context.
    ///#[serde(rename = "@encoding", skip_serializing_if = "Option::is_none")]
    pub encoding: Option<TsBytes<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct DependenciesNode {
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Dependency {
    pub catalog: String,
}

/// Translation message node.
#[derive(Debug, Eq, Clone, PartialEq)]
pub struct MessageNode {
    /// Original string to translate
    ///    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Old source before a merge. Merging will set that field.
    ///  #[serde(rename = "oldsource", skip_serializing_if = "Option::is_none")]
    pub old_source: Option<String>,
    /// Translation in the target language.
    ///#[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<TranslationNode>,
    /// Lines and files in which the translation message is used.
    ///#[serde(skip_serializing_if = "Vec::is_empty", rename = "location", default)]
    pub locations: Vec<LocationNode>,
    /// This is "disambiguation" in the (new) API, or "msgctxt" in gettext speak
    ///#[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Previous content of comment (result of merge)
    ///#[serde(rename = "oldcomment", skip_serializing_if = "Option::is_none")]
    pub old_comment: Option<String>,
    /// The real comment (added by developer/designer)
    ///#[serde(rename = "extracomment", skip_serializing_if = "Option::is_none")]
    pub extra_comment: Option<String>,
    /// Comment added by translator
    ///#[serde(rename = "translatorcomment", skip_serializing_if = "Option::is_none")]
    pub translator_comment: Option<String>,
    /// Support for the plural forms
    ///#[serde(rename = "@numerus", skip_serializing_if = "Option::is_none")]
    pub numerus: Option<YesNo>,
    /// Message unique id (not guaranteed to be existant)
    ///#[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Extra information
    ///#[serde(skip_serializing_if = "Option::is_none")]
    pub userdata: Option<String>,
    /*
       Following section corresponds to `extra-something` in Qt's XSD. From documentation:
       > extra elements may appear in TS and message elements. Each element may appear
       > only once within each scope. The contents are preserved verbatim; any
       > attributes are dropped.
    */
    ///#[serde(
    ///  rename = "extra-po-msgid_plural",
    ///skip_serializing_if = "Option::is_none"
    ///)]
    pub po_msg_id_plural: Option<String>,
    ///#[serde(
    ///  rename = "extra-po-old_msgid_plural",
    ///skip_serializing_if = "Option::is_none"
    ///)]
    pub po_old_msg_id_plural: Option<String>,
    /// Comma separated list
    ///#[serde(rename = "extra-po-flags", skip_serializing_if = "Option::is_none")]
    pub loc_flags: Option<String>,
    ///#[serde(
    ///  rename = "extra-loc-layout_id",
    ///skip_serializing_if = "Option::is_none"
    ///)]
    pub loc_layout_id: Option<String>,
    ///#[serde(rename = "extra-loc-feature", skip_serializing_if = "Option::is_none")]
    pub loc_feature: Option<String>,
    ///#[serde(rename = "extra-loc-blank", skip_serializing_if = "Option::is_none")]
    pub loc_blank: Option<String>,
}

/// Translation node that indicates an actual translation for a message.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct TranslationNode {
    // Did not find a way to make it an enum
    // Therefore: either you have a `translation_simple` or a `numerus_forms`, but not both.
    /// Simple translation version, which do not take plural forms into account
    // #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub translation_simple: Option<String>,
    /// Plural forms for the translation
    // #[serde(rename = "numerusform", skip_serializing_if = "Vec::is_empty", default)]
    pub numerus_forms: Vec<NumerusFormNode>,
    /// Translation type (which represents the translation status)
    // #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub translation_type: Option<TranslationType>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<YesNo>,
    /// Extra data
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub userdata: Option<String>, // deprecated
}

/// Location of a translation
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocationNode {
    /// File from which the translation source originates from.
    // #[serde(rename = "@filename", skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Line where the source of the translation message is located in the file.
    // #[serde(rename = "@line", skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// Represents a translation plural form.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct NumerusFormNode {
    // #[serde(default, rename = "$value", skip_serializing_if = "String::is_empty")]
    pub text: String,
    // #[serde(rename = "@variants", skip_serializing_if = "Option::is_none")]
    pub variants: Option<YesNo>,
}

impl PartialOrd<Self> for MessageNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MessageNode {
    fn cmp(&self, other: &Self) -> Ordering {
        let id_cmp = other.id.cmp(&self.id);

        if id_cmp != Ordering::Equal {
            return id_cmp;
        }

        let (filename, line) = self
            .locations
            .iter()
            .min_by_key(|location| (location.filename.as_ref(), location.line))
            .map(|location| (location.filename.as_ref(), location.line.as_ref()))
            .unwrap_or_default();

        let (other_filename, other_line) = other
            .locations
            .iter()
            .min_by_key(|location| (location.filename.as_ref(), location.line))
            .map(|location| (location.filename.as_ref(), location.line.as_ref()))
            .unwrap_or_default();

        // Counterintuitive, but we want to have locationless message at the end:
        // handle `None` differently from default.
        if filename.is_none() && other_filename.is_some() {
            Ordering::Greater
        } else {
            (filename, line).cmp(&(other_filename, other_line))
        }
    }
}

impl Ord for LocationNode {
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .filename
            .as_ref()
            .unwrap_or(&"".to_owned())
            .to_lowercase()
            .cmp(
                &other
                    .filename
                    .as_ref()
                    .unwrap_or(&"".to_owned())
                    .to_lowercase(),
            ) {
            Ordering::Equal => self.line.cmp(&other.line),
            ordering => ordering,
        }
    }
}

impl PartialOrd<Self> for LocationNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<Self> for ContextNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContextNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Contexts are generally module or classes names; let's assume they don't need any special collation treatment.
        self.name.to_lowercase().cmp(&other.name.to_lowercase())
    }
}

impl<'a> FromStr for TSNode<'a> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cursor = Cursor::new(s);
        let reader = quick_xml::Reader::from_reader(cursor);
        TSNode::from_reader(reader)
    }
}

impl<'a> TSNode<'a> {
    fn from_file(path: &Path) -> Result<Self, ParseError> {
        let reader = quick_xml::Reader::from_file(path)?;
        TSNode::from_reader(reader)
    }

    fn from_reader(reader: quick_xml::Reader<impl BufRead>) -> Result<Self, ParseError> {
        let mut reader = quick_xml::Reader::from_reader(reader).into_inner();
        let mut ts_node: TSNode<'a> = TSNode::default();
        let buf: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
        // TODO: does not work for ownership
        {
            match reader.read_event_into(buf.borrow_mut()) {
                Ok(Event::Eof) => (),
                Ok(Event::Start(e)) if e.name().as_ref().eq_ignore_ascii_case(b"ts") => {
                    println!("Found {:#?}", e.name());

                    e.attributes()
                        .flatten()
                        .for_each(|attr| match attr.key.as_ref() {
                            b"version" => ts_node.version = Some(attr.value),
                            b"sourcelanguage" => ts_node.source_language = Some(attr.value),
                            b"language" => ts_node.language = Some(attr.value),
                            _ => debug!("Unknown attribute: {:?}", attr.key),
                        });
                }
                Ok(Event::Start(e)) if e.name().as_ref().eq_ignore_ascii_case(b"context") => {
                    // todo: check we are in a TS.
                    println!("Context");
                    let mut ctx = ContextNode::default();
                    e.attributes().flatten().for_each(|a| match a.key.as_ref() {
                        b"name" => ctx.name = Some(a.value),
                        b"encoding" => ctx.encoding = Some(a.value),
                    });
                }
                Err(_todo) => (),
                _ => (),
            }
        }

        Ok(ts_node)
    }
}

fn parse_dependency_node(
    element: &quick_xml::events::BytesStart,
) -> Result<Dependency, ParseError> {
    let mut node = Dependency::default();

    for a in element.attributes() {
        if let Ok(attr) = a {
            match attr.key.as_ref() {
                b"catalog" => node.catalog = String::from_utf8(attr.value.into_owned())?,
                _ => (),
            }
        }
    }

    Ok(node)
}

/// OK
fn parse_location_node(
    element: &quick_xml::events::BytesStart,
) -> Result<LocationNode, ParseError> {
    let mut node = LocationNode::default();

    for a in element.attributes() {
        if let Ok(attr) = a {
            match attr.key.as_ref() {
                b"filename" => node.filename = Some(String::from_utf8(attr.value.into_owned())?),
                b"line" => {
                    node.line = Some(std::str::from_utf8(&attr.value.into_owned())?.parse::<u32>()?)
                }

                _ => (),
            }
        }
    }

    Ok(node)
}

fn parse_numerus_form_node(
    reader: &mut quick_xml::reader::Reader<BufReader<File>>,
    element: &quick_xml::events::BytesStart,
) -> Result<NumerusFormNode, ParseError> {
    let mut node = NumerusFormNode::default();

    for a in element.attributes() {
        if let Ok(attr) = a {
            match attr.key.as_ref() {
                b"variants" => match attr.value.to_ascii_lowercase().as_slice() {
                    b"yes" => node.variants = Some(YesNo::Yes),
                    _ => node.variants = Some(YesNo::No),
                },
                _ => (),
            }
        }
    }

    let mut buf = vec![];
    node.text = read_raw_string(reader, element, &mut buf)?;

    Ok(node)
}

fn parse_translation_node(
    reader: &mut quick_xml::reader::Reader<BufReader<File>>,
    element: &quick_xml::events::BytesStart,
    shared_buf: &mut Vec<u8>,
) -> Result<TranslationNode, ParseError> {
    let mut node = TranslationNode::default();

    if let Some(attr) = element.try_get_attribute("type")? {
        node.translation_type = Some(TranslationType::from(attr.value));
    }

    match reader.read_event_into(shared_buf) {
        Ok(Event::Text(text_element)) => {
            node.translation_simple = Some(text_element.xml10_content()?.into_owned())
        }
        Ok(Event::Empty(start)) => {}
        _ => {}
    }
    // TODO: handle complex
    Ok(node)
}

fn read_raw_string(
    reader: &mut quick_xml::reader::Reader<BufReader<File>>,
    element: &quick_xml::events::BytesStart,
    shared_buf: &mut Vec<u8>,
) -> Result<String, ParseError> {
    let previous_name = reader.config_mut().check_end_names;
    reader.config_mut().check_end_names = false;

    // TODO: proper restauration of configuration
    let content = reader
        .read_text_into(element.to_end().name(), shared_buf)?
        .decode()?
        .into_owned();

    reader.config_mut().check_end_names = previous_name;

    Ok(content)
}

// #[cfg(test)]
// mod write_file_test {
//     use super::*;

//     #[test]
//     fn test_write_to_output_file() {
//         const OUTPUT_TEST_FILE: &str = "./test_data/test_result_write_to_ts.xml";

//         let reader = quick_xml::Reader::from_file("./test_data/example1.xml")
//             .expect("Couldn't open example1 test file");

//         let data: TSNode = quick_xml::de::from_reader(reader.into_inner()).expect("Parsable");

//         write_to_output(&Some(OUTPUT_TEST_FILE.to_owned()), &data).expect("Output");

//         let f =
//             quick_xml::Reader::from_file(OUTPUT_TEST_FILE).expect("Couldn't open output test file");

//         let output_data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
//         std::fs::remove_file(OUTPUT_TEST_FILE).expect("Test should clean test file.");
//         assert_eq!(data, output_data);
//     }
// }

#[cfg(test)]
mod test_tsnode {
    use rstest::rstest;
    use std::str::FromStr;

    use crate::ts_next::TSNode;

    #[rstest]
    #[case("version=\"\"", Some(""))]
    #[case("version=\"2.1\"", Some("2.1"))]
    #[case("", None)]
    fn test_parses_version(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
        let raw = format!(r#"<!DOCTYPE TS><TS {} ></TS>"#, raw);
        let node = TSNode::from_str(&raw).expect("Parses.");

        assert_eq!(
            node.version
                .as_ref()
                .map(|s| str::from_utf8(&s).expect("valid utf8")),
            expected_parsed
        );
    }
}

#[cfg(test)]
mod test_yesno {
    use rstest::rstest;

    use crate::ts_next::YesNo;

    #[rstest]
    #[case(b"yes")]
    #[case(b"YES")]
    #[case(b"Yes")]
    #[case(b"yEs")]
    #[case(b"yeS")]
    #[case(b" yes ")]
    fn test_from_cow_should_match_yes(#[case] attr_value: &[u8]) {
        let actual = YesNo::from(std::borrow::Cow::Borrowed(attr_value));
        assert_eq!(YesNo::Yes, actual);
    }

    #[rstest]
    #[case(b"no")]
    #[case(b"NO")]
    #[case(b"No")]
    #[case(b"nO")]
    #[case(b" no ")]
    #[case(b"")]
    fn test_from_cow_should_match_no(#[case] attr_value: &[u8]) {
        let actual = YesNo::from(std::borrow::Cow::Borrowed(attr_value));
        assert_eq!(YesNo::No, actual);
    }
}

#[cfg(test)]
mod test_translation_type {
    use rstest::rstest;

    use crate::ts_next::TranslationType;

    #[rstest]
    #[case(b"unFinIshed", TranslationType::Unfinished)]
    #[case(b"fiNIshed", TranslationType::Finished)]
    #[case(b"VAnishEd", TranslationType::Vanished)]
    #[case(b"obsoLETE", TranslationType::Obsolete)]
    fn from_cow_should_return_correct_value(
        #[case] attr_value: &[u8],
        #[case] expected: TranslationType,
    ) {
        let actual = TranslationType::from(std::borrow::Cow::Borrowed(attr_value));
        assert_eq!(expected, actual);
    }
}
