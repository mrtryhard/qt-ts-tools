use crate::parse_error::ParseError;
use log::{debug, info, warn};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::borrow::Cow;
use std::cmp::Ordering;

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

pub struct TsParser {
    buf: Vec<u8>,
}

impl TsParser {
    pub fn new(buf: Vec<u8>) -> Self {
        Self { buf }
    }

    // Zero-copy extraction linked directly to the original input buffer lifetime 'a
    pub fn parse(&mut self) -> TSNode<'_> {
        let mut reader = Reader::from_reader(self.buf.as_slice());
        let mut ts_node: TSNode<'_> = TSNode::default();
        let mut inner_buf = Vec::new();
        reader.config_mut().expand_empty_elements = true;

        loop {
            let event = reader
                .read_event_into(&mut inner_buf)
                .expect("Error reading event");
            if let Event::Start(ref ev) = event {
                debug!("Found {:#?}", ev.name());
            }

            match event {
                Event::Eof => {
                    debug!("EOF detected");
                    break;
                }
                Event::Start(ref e) if e.name().as_ref().eq_ignore_ascii_case(b"ts") => {
                    ts_node.parse(&mut reader, e);
                }
                _ => (),
            }
        }

        ts_node
    }
}

/// Root node of the translation file.
#[derive(Debug, Default, PartialEq)]
pub struct TSNode<'a> {
    /// Defines the version of the TS format, although unused by this tool.
    /// attribute -- do not serialize if missing
    pub version: Option<TsBytes<'a>>,
    /// Source language on which this translation is based on.
    pub source_language: Option<TsBytes<'a>>,
    /// Language of this translation.
    pub language: Option<TsBytes<'a>>,
    /// Translations attached to a context
    pub contexts: Vec<ContextNode<'a>>,
    //pub dependencies: Option<DependenciesNode>,
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

impl<'a> TSNode<'a> {
    fn parse(&mut self, reader: &mut Reader<&[u8]>, element: &BytesStart) {
        self.assign_attributes(element);

        let mut inner_buf = Vec::new();

        loop {
            let event = reader
                .read_event_into(&mut inner_buf)
                .expect("Error reading event");
            if let Event::Start(ref ev) = event {
                debug!("Found {:#?}", ev.name());
            }

            match event {
                Event::Start(ref e) if e.name().as_ref().eq_ignore_ascii_case(b"context") => {
                    debug!("Found context");
                    ContextNode::from_reader(reader, e)
                        .map(|n| self.contexts.push(n))
                        .map_err(|e| debug!("Error parsing context node: {:?}", e))
                        .expect("Expected to succeed"); // TODO: improve that.
                }
                Event::End(ref e) if e.name().as_ref().eq_ignore_ascii_case(b"ts") => break,
                _ => debug!("TsNode: unknown event: {:?}", event),
            }
        }
    }

    fn assign_attributes(&mut self, element: &BytesStart) {
        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                b"version" => self.version = Some(Cow::Owned(a.value.into_owned())),
                b"sourcelanguage" => self.source_language = Some(Cow::Owned(a.value.into_owned())),
                b"language" => self.language = Some(Cow::Owned(a.value.into_owned())),
                _ => debug!("Unknown attribute: {:?}", a.key),
            });
    }
}

/// Context and its associated translated message.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ContextNode<'a> {
    /// Unique name of the context
    pub name: Option<TsBytes<'a>>,
    /// List of translation messages
    pub messages: Vec<MessageNode<'a>>,
    /// Comment describing information about the context
    pub comment: Option<TsBytes<'a>>,
    /// Encoding of the messages within that context.
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
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct MessageNode<'a> {
    /// Original string to translate
    pub source: Option<TsBytes<'a>>,
    /// Old source before a merge. Merging will set that field.
    pub old_source: Option<TsBytes<'a>>,
    /// Translation in the target language.
    pub translation: Option<TranslationNode>,
    /// Lines and files in which the translation message is used.
    pub locations: Vec<LocationNode>,
    /// This is "disambiguation" in the (new) API, or "msgctxt" in gettext speak
    pub comment: Option<TsBytes<'a>>,
    /// Previous content of comment (result of merge)
    pub old_comment: Option<TsBytes<'a>>,
    /// The real comment (added by developer/designer)
    pub extra_comment: Option<TsBytes<'a>>,
    /// Comment added by translator
    pub translator_comment: Option<TsBytes<'a>>,
    /// Support for the plural forms
    pub numerus: Option<YesNo>,
    /// Message unique id (not guaranteed to be existant)
    pub id: Option<TsBytes<'a>>,
    /// Extra information
    pub userdata: Option<TsBytes<'a>>,
    /*
       Following section corresponds to `extra-something` in Qt's XSD. From documentation:
       > extra elements may appear in TS and message elements. Each element may appear
       > only once within each scope. The contents are preserved verbatim; any
       > attributes are dropped.
    */
    pub po_msg_id_plural: Option<TsBytes<'a>>,
    pub po_old_msg_id_plural: Option<TsBytes<'a>>,
    /// Comma separated list
    pub loc_flags: Option<TsBytes<'a>>,
    pub loc_layout_id: Option<TsBytes<'a>>,
    pub loc_feature: Option<TsBytes<'a>>,
    pub loc_blank: Option<TsBytes<'a>>,
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

impl<'a> PartialOrd<Self> for MessageNode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for MessageNode<'a> {
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

impl<'a> PartialOrd<Self> for ContextNode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for ContextNode<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Contexts are generally module or classes names; let's assume they don't need any special collation treatment.
        self.name
            .as_ref()
            .unwrap()
            .to_ascii_lowercase()
            .cmp(&other.name.as_ref().unwrap().to_ascii_lowercase())
    }
}

impl<'a> ContextNode<'a> {
    fn from_reader(
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<ContextNode<'a>, ParseError> {
        #[derive(Debug, Eq, PartialEq)]
        enum Tag {
            None,
            Name,
            Comment,
            Message,
        }
        let mut context_node = ContextNode::default();
        let mut inner_buf = Vec::new();
        let mut current_tag = Tag::None;

        loop {
            let event = reader
                .read_event_into(&mut inner_buf)
                .expect("Error reading event");
            if let Event::Start(ref ev) = event {
                debug!("ContextNode: found {:#?}", ev.name());
            }
            match event {
                Event::Start(ref e) => {
                    debug!("ContextNode: Found element \"{e:#?}\"");

                    if Tag::None != current_tag {
                        // TODO: better logging or error message?
                        return Err(ParseError::from("Unexpected tag opening"));
                    }

                    current_tag = match e.name().as_ref() {
                        b"name" => Tag::Name,
                        b"comment" => Tag::Comment,
                        b"message" => {
                            // Note: Better to start parsing here to ensure access to element attributes.
                            info!("ContextNode: Parsing message node.");
                            MessageNode::from_reader(reader, e)
                                .map(|n| context_node.messages.push(n))
                                .expect("To be parsed");
                            Tag::Message
                        }
                        _ => {
                            warn!("ContextNode: Unknown field: {e:#?}");
                            Tag::None
                        }
                    };

                    debug!("ContextNode: found tag {current_tag:#?}");
                }
                Event::Text(ref e) => {
                    debug!("ContextNode: found text: {:#?}", e);
                    let text = Some(TsBytes::Owned(e.to_vec()));
                    match current_tag {
                        Tag::Name => context_node.name = text,
                        Tag::Comment => context_node.comment = text,
                        _ => {
                            warn!("ContextNode: what is going on")
                        } // TODO: better logging or error message?
                    }
                }

                Event::End(ref e) => {
                    debug!("ContextNode: Found END element \"{e:#?}\"");
                    match e.name().as_ref() {
                        b"context" => break,
                        b"comment" | b"message" | b"name" => current_tag = Tag::None,
                        _ => debug!("ContextNode: ending unknown field: {e:#?}"),
                    }
                }
                _ => debug!("Context node: unknown event: {:?}", event),
            }
        }

        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                b"encoding" => context_node.encoding = Some(Cow::Owned(a.value.into_owned())),
                _ => debug!("ContextNode: unknown attribute: {:?}", a.key),
            });

        Ok(context_node)
    }
}

impl<'a> MessageNode<'a> {
    fn from_reader(reader: &mut Reader<&[u8]>, element: &BytesStart) -> Result<Self, ParseError> {
        #[derive(Debug, Eq, PartialEq)]
        enum Tag {
            None,
            Id,
            Comment,
            ExtraComment,
            LocBlank,
            Locations,
            LocFeature,
            LocFlags,
            LocLayoutId,
            Numerus,
            OldComment,
            OldSource,
            PoMsgIdPlural,
            PoOldMsgIdPlural,
            Source,
            Translation,
            TranslatorComment,
            UserData,
        }
        let mut message_node = MessageNode::default();
        let mut inner_buf = Vec::new();
        let mut current_tag = Tag::None;
        info!("Message node");

        loop {
            let event = reader
                .read_event_into(&mut inner_buf)
                .expect("Error reading event");
            if let Event::Start(ref ev) = event {
                debug!("MessageNode: found {:#?}", ev.name());
            }

            match event {
                Event::Start(ref e) => {
                    debug!("MessageNode: Found element \"{e:#?}\"");

                    if Tag::None != current_tag {
                        // TODO: better logging or error message?
                        return Err(ParseError::from("Unexpected tag opening"));
                    }

                    current_tag = match e.name().as_ref() {
                        b"comment" => Tag::Comment,
                        b"location" => Tag::Locations,
                        b"oldsource" => Tag::OldSource, // TODO: check string if not old_source
                        b"source" => Tag::Source,

                        b"translation" => Tag::Translation,

                        // None,
                        // Id,
                        // Comment,
                        // ExtraComment,
                        // LocBlank,
                        // Locations,
                        // LocFeature,
                        // LocFlags,
                        // LocLayoutId,
                        // Numerus,
                        // OldComment,
                        // OldSource,
                        // PoMsgIdPlural,
                        // PoOldMsgIdPlural,
                        // Source,
                        // Translation,
                        // TranslatorComment,
                        // UserData
                        _ => {
                            warn!("ContextNode: Unknown field: {e:#?}");
                            Tag::None
                        }
                    };

                    debug!("Found tag {current_tag:#?}");
                }
                Event::Text(ref e) => {
                    debug!("MessageNode: found text: {:#?}", e);
                    let text = Some(TsBytes::Owned(e.to_vec()));
                    match current_tag {
                        Tag::None => {}
                        Tag::Id => {}
                        Tag::Comment => message_node.comment = text,
                        Tag::ExtraComment => {}
                        Tag::LocBlank => {}
                        Tag::Locations => {}
                        Tag::LocFeature => {}
                        Tag::LocFlags => {}
                        Tag::LocLayoutId => {}
                        Tag::OldComment => {}
                        Tag::OldSource => {}
                        Tag::PoMsgIdPlural => {}
                        Tag::PoOldMsgIdPlural => {}
                        Tag::Source => message_node.source = text,
                        Tag::Translation => {}
                        Tag::TranslatorComment => {}
                        Tag::UserData => {}
                        _ => {} // TODO: error message?
                    }
                }

                Event::End(ref e) => {
                    debug!("MessageNode: Found END element \"{e:#?}\"");
                    match e.name().as_ref() {
                        b"message" => break,
                        b"comment" | b"translation" | b"oldsource" | b"source" => {
                            current_tag = Tag::None
                        }
                        _ => debug!("MessageNode: ending unknown field: {e:#?}"),
                    }
                }
                _ => debug!("MessageNode: unknown event: {:?}", event),
            }
        }

        element.attributes().flatten().for_each(|a| {
            match a.key.as_ref() {
                b"numerus" => message_node.numerus = Some(YesNo::from(a.value)),
                _ => debug!("MessageNode: unknown attribute: {:?}", a.key),
            }
        });

        println!("{message_node:#?}");
        Ok(message_node)
    }
}

#[cfg(test)]
mod test_tsnode {
    use rstest::rstest;

    use crate::ts_next::TsParser;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    mod test_message_node {
        use crate::ts_next::TsParser;
        use crate::ts_next::YesNo;
        use crate::ts_next::test_tsnode::init;
        use rstest::rstest;

        #[rstest]
        #[case("", None)]
        #[case("numerus=\"\"", Some(YesNo::No))]
        #[case("numerus=\"yes\"", Some(YesNo::Yes))]
        #[case("numerus=\"no\"", Some(YesNo::No))]
        fn test_message_node_numerus(#[case] raw: &str, #[case] expected_parsed: Option<YesNo>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message {}>
                        <source>This is a test</source>
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse();
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(node.contexts[0].messages[0].numerus, expected_parsed);
        }
    }

    mod test_context_node {
        use crate::ts_next::TsParser;
        use crate::ts_next::test_tsnode::init;
        use rstest::rstest;

        #[rstest]
        #[case("", None)]
        #[case("encoding=\"\"", Some(""))]
        #[case("encoding=\"utf-8\"", Some("utf-8"))]
        fn test_context_node_encoding(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(r#"<!DOCTYPE TS><ts><context {}></context></ts>"#, raw);
            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse();
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0]
                    .encoding
                    .as_ref()
                    .map(|s| str::from_utf8(&s).expect("valid utf8")),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<name></name>", None)]
        #[case("<name>some_name</name>", Some("some_name"))]
        fn test_context_node_name(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(r#"<!DOCTYPE TS><ts><context>{}</context></ts>"#, raw);
            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse();
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0]
                    .name
                    .as_ref()
                    .map(|s| str::from_utf8(&s).expect("valid utf8")),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<comment></comment>", None)]
        #[case("<comment>commentaire</comment>", Some("commentaire"))]
        fn test_context_node_comment(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(r#"<!DOCTYPE TS><ts><context>{}</context></ts>"#, raw);
            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse();
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0]
                    .comment
                    .as_ref()
                    .map(|s| str::from_utf8(&s).expect("valid utf8")),
                expected_parsed
            );
        }
    }

    #[rstest]
    #[case("version=\"\"", Some(""))]
    #[case("version=\"2.1\"", Some("2.1"))]
    #[case("", None)]
    fn test_parses_version(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
        let raw = format!(r#"<!DOCTYPE TS><TS {} ></TS>"#, raw);
        let mut parser = TsParser::new(raw.as_bytes().into());
        let node = parser.parse();

        assert_eq!(
            node.version
                .as_ref()
                .map(|s| str::from_utf8(&s).expect("valid utf8")),
            expected_parsed
        );
    }

    #[rstest]
    #[case("sourcelanguage=\"\"", Some(""))]
    #[case("sourcelanguage=\"en\"", Some("en"))]
    #[case("sourcelanguage=\"en_US\"", Some("en_US"))]
    #[case("", None)]
    fn test_parses_sourcelanguage(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
        let raw = format!(r#"<!DOCTYPE TS><TS {} ></TS>"#, raw);
        let mut parser = TsParser::new(raw.as_bytes().into());
        let node = parser.parse();

        assert_eq!(
            node.source_language
                .as_ref()
                .map(|s| str::from_utf8(&s).expect("valid utf8")),
            expected_parsed
        );
    }

    #[rstest]
    #[case("language=\"\"", Some(""))]
    #[case("language=\"en\"", Some("en"))]
    #[case("language=\"en_US\"", Some("en_US"))]
    #[case("", None)]
    fn test_parses_language(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
        let raw = format!(r#"<!DOCTYPE TS><TS {} ></TS>"#, raw);
        let mut parser = TsParser::new(raw.as_bytes().into());
        let node = parser.parse();

        assert_eq!(
            node.language
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
