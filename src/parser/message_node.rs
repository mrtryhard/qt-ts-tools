use crate::parser::location_node::LocationNode;
use crate::parser::parse_error::ParseError;
use crate::parser::translation_node::TranslationNode;
use crate::parser::ts_bytes::TsBytes;
use crate::parser::yesno::YesNo;
use log::{debug, info, warn};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::cmp::Ordering;

/// Translation message node.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct MessageNode<'a> {
    /// This is "disambiguation" in the (new) API, or "msgctxt" in gettext speak
    pub comment: Option<TsBytes<'a>>,
    /// Message unique id (not guaranteed to be existant)
    pub id: Option<TsBytes<'a>>,
    /// The real comment (added by developer/designer)
    pub extra_comment: Option<TsBytes<'a>>,
    /// Lines and files in which the translation message is used.
    pub locations: Vec<LocationNode<'a>>,
    /// Support for the plural forms
    pub numerus: Option<YesNo>,
    /// Previous content of comment (result of merge)
    pub old_comment: Option<TsBytes<'a>>,
    /// Old source before a merge. Merging will set that field.
    /// TODO: merging should set that field.
    pub old_source: Option<TsBytes<'a>>,
    /// Original string to translate
    pub source: Option<TsBytes<'a>>,
    /// Translation in the target language.
    pub translation: Option<TranslationNode<'a>>,
    /// Comment added by translator
    pub translator_comment: Option<TsBytes<'a>>,
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

impl<'a> MessageNode<'a> {
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
            LocBlank,
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
        let mut node = MessageNode::default();
        let mut buffer = Vec::new();
        let mut current_tag = Tag::None;
        info!("MessageNode");

        loop {
            let event = reader.read_event_into(&mut buffer)?;
            if let Event::Start(ref ev) = event {
                debug!("MessageNode::{current_tag:?}: found {:#?}", ev.name());
            }

            match event {
                Event::Start(ref e) => {
                    debug!("MessageNode::{current_tag:?}: Found element \"{:#?}\"", e.name());

                    if Tag::None != current_tag {
                        return Err(ParseError::from(format!("MessageNode::{current_tag:?}: Unexpected tag opening: ${e:?}")));
                    }

                    current_tag = match e.name().as_ref() {
                        b"comment" => Tag::Comment,
                        b"extracomment" => Tag::ExtraComment,
                        b"location" => {
                            LocationNode::from_reader(reader, e)
                                .map(|l| node.locations.push(l))?;
                            Tag::Location
                        }
                        b"oldcomment" => Tag::OldComment,
                        b"oldsource" => Tag::OldSource,
                        b"source" => Tag::Source,
                        b"translatorcomment" => Tag::TranslatorComment,
                        b"translation" => {
                            node.translation =
                                Some(TranslationNode::from_reader(reader, e)?);
                            Tag::None
                        }
                        b"userdata" => Tag::UserData,
                        _ => {
                            warn!("MessageNode::{current_tag:?}: Unknown field: {e:#?}");
                            Tag::None
                        }
                    };
                }
                Event::Text(ref e) => {
                    debug!("MessageNode::{current_tag:?}: found text: {e:#?}");
                    let text = Some(TsBytes::Owned(e.to_vec()));
                    match current_tag {
                        Tag::None => {}
                        Tag::Comment => node.comment = text,
                        Tag::ExtraComment => node.extra_comment = text,
                        Tag::LocBlank => node.loc_blank = text,
                        Tag::LocFeature => node.loc_feature = text,
                        Tag::LocFlags => node.loc_flags = text,
                        Tag::LocLayoutId => node.loc_layout_id = text,
                        Tag::OldComment => node.old_comment = text,
                        Tag::OldSource => node.old_source = text,
                        Tag::PoMsgIdPlural => node.po_msg_id_plural = text,
                        Tag::PoOldMsgIdPlural => node.po_old_msg_id_plural = text,
                        Tag::Source => node.source = text,
                        Tag::Translation => {}
                        Tag::TranslatorComment => node.translator_comment = text,
                        Tag::UserData => node.userdata = text,
                        _ => {} // TODO: error message?
                    }
                }

                Event::End(ref e) => {
                    debug!("MessageNode::{current_tag:?}: Found END element \"{e:#?}\"");
                    match e.name().as_ref() {
                        b"message" => break,
                        b"comment" | b"location" | b"translation" | b"oldsource"
                        | b"extracomment" | b"oldcomment" | b"source" | b"translatorcomment"
                        | b"userdata" => current_tag = Tag::None,
                        _ => debug!("MessageNode::{current_tag:?}: ending unknown field: {e:#?}"),
                    }
                }
                _ => debug!("MessageNode::{current_tag:?}: unknown event: {event:?}"),
            }
        }

        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                b"id" => node.id = Some(TsBytes::Owned(a.value.into_owned())),
                b"numerus" => node.numerus = Some(YesNo::from(a.value)),
                _ => debug!("MessageNode: unknown attribute: {:?}", a.key),
            });

        Ok(node)
    }
}
