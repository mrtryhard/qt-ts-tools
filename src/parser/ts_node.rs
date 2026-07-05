use crate::parser::context_node::ContextNode;
use crate::parser::ts_bytes::TsBytes;
use log::debug;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::borrow::Cow;

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
    pub fn from_reader(&mut self, reader: &mut Reader<&[u8]>, element: &BytesStart) {
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
