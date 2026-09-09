use crate::parser::location_node::LocationNode;
use crate::parser::parse_error::ParseError;
use crate::parser::text_node::text_node_from_reader;
use crate::parser::translation_node::TranslationNode;
use crate::parser::yesno::YesNo;
use log::{debug, info, warn};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::cmp::Ordering;

/// Translation message node.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct MessageNode {
    /// This is "disambiguation" in the (new) API, or "msgctxt" in gettext speak
    pub comment: Option<String>,
    /// Message unique id (not guaranteed to be existant)
    pub id: Option<String>,
    /// The real comment (added by developer/designer)
    pub extra_comment: Option<String>,
    /// Lines and files in which the translation message is used.
    pub locations: Vec<LocationNode>,
    /// Support for the plural forms
    pub numerus: Option<YesNo>,
    /// Previous content of comment (result of merge)
    pub old_comment: Option<String>,
    /// Old source before a merge. Merging will set that field.
    /// TODO: merging should set that field.
    pub old_source: Option<String>,
    /// Original string to translate
    pub source: Option<String>,
    /// Translation in the target language.
    pub translation: Option<TranslationNode>,
    /// Comment added by translator
    pub translator_comment: Option<String>,
    /// Extra information
    pub userdata: Option<String>,
    /*
       Following section corresponds to `extra-something` in Qt's XSD. From documentation:
       > extra elements may appear in TS and message elements. Each element may appear
       > only once within each scope. The contents are preserved verbatim; any
       > attributes are dropped.
    */
    pub po_msg_id_plural: Option<String>,
    pub po_old_msg_id_plural: Option<String>,
    /// Comma separated list
    pub loc_flags: Option<String>,
    pub loc_layout_id: Option<String>,
    pub loc_feature: Option<String>,
    pub loc_blank: Option<String>,
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

impl MessageNode {
    pub fn from_reader(
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<Self, ParseError> {
        #[derive(Debug, Eq, PartialEq)]
        enum Tag {
            None,
            Comment,
            ExtraComment,
            Location,
            // LocBlank,
            // LocFeature,
            // LocFlags,
            // LocLayoutId,
            OldComment,
            OldSource,
            // PoMsgIdPlural,
            // PoOldMsgIdPlural,
            TranslatorComment,
            UserData,
        }
        let mut node = MessageNode::default();
        let mut current_tag = Tag::None;
        info!("MessageNode");

        loop {
            let event = reader.read_event()?;
            if let Event::Start(ref ev) = event {
                debug!("MessageNode::{current_tag:?}: found {:#?}", ev.name());
            }

            match event {
                Event::Start(ref e) => {
                    debug!(
                        "MessageNode::{current_tag:?}: Found element \"{:#?}\"",
                        e.name()
                    );

                    if Tag::None != current_tag {
                        return Err(ParseError::from(format!(
                            "MessageNode::{current_tag:?}: Unexpected tag opening: ${e:?}"
                        )));
                    }

                    current_tag = match e.name().as_ref() {
                        "comment" => Tag::Comment,
                        "extracomment" => Tag::ExtraComment,
                        "location" => {
                            LocationNode::from_reader(reader, e).map(|l| node.locations.push(l))?;
                            Tag::Location
                        }
                        "oldcomment" => Tag::OldComment,
                        "oldsource" => Tag::OldSource,
                        "source" => {
                            node.source = text_node_from_reader(reader, e)?;
                            Tag::None
                        }
                        "translatorcomment" => Tag::TranslatorComment,
                        "translation" => {
                            node.translation = Some(TranslationNode::from_reader(reader, e)?);
                            Tag::None
                        }
                        "userdata" => Tag::UserData,
                        _ => {
                            warn!("MessageNode::{current_tag:?}: Unknown field: {e:#?}");
                            Tag::None
                        }
                    };
                }
                Event::Text(ref e) => {
                    debug!("MessageNode::{current_tag:?}: found text: {e:#?}");
                    let text = Some(e.to_string());
                    match current_tag {
                        Tag::None => {}
                        Tag::Comment => node.comment = text,
                        Tag::ExtraComment => node.extra_comment = text,
                        // Tag::LocBlank => node.loc_blank = text,
                        // Tag::LocFeature => node.loc_feature = text,
                        // Tag::LocFlags => node.loc_flags = text,
                        // Tag::LocLayoutId => node.loc_layout_id = text,
                        Tag::OldComment => node.old_comment = text,
                        Tag::OldSource => node.old_source = text,
                        // Tag::PoMsgIdPlural => node.po_msg_id_plural = text,
                        // Tag::PoOldMsgIdPlural => node.po_old_msg_id_plural = text,
                        Tag::TranslatorComment => node.translator_comment = text,
                        Tag::UserData => node.userdata = text,
                        _ => {} // TODO: error message?
                    }
                }

                Event::End(ref e) => {
                    debug!("MessageNode::{current_tag:?}: Found END element \"{e:#?}\"");
                    match e.name().as_ref() {
                        "message" => break,
                        "comment" | "location" | "translation" | "oldsource" | "extracomment"
                        | "oldcomment" | "source" | "translatorcomment" | "userdata" => {
                            current_tag = Tag::None
                        }
                        _ => debug!("MessageNode::{current_tag:?}: ending unknown field: {e:#?}"),
                    }
                }
                Event::Eof => break,
                _ => debug!("MessageNode::{current_tag:?}: unknown event: {event:?}"),
            }
        }

        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                "id" => node.id = Some(a.value.to_string()),
                "numerus" => node.numerus = Some(YesNo::from(a.value)),
                _ => debug!("MessageNode: unknown attribute: {:?}", a.key),
            });

        Ok(node)
    }
}
