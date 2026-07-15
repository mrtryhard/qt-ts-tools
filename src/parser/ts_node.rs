use crate::parser::context_node::ContextNode;
use crate::parser::dependency_node::DependenciesNode;
use crate::parser::parse_error::ParseError;
use crate::parser::ts_bytes::TsBytes;
use log::debug;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::borrow::Cow;

/// Root node of the translation file.
#[derive(Debug, Default, PartialEq)]
pub struct TsNode<'a> {
    /// Defines the version of the TS format, although unused by this tool.
    /// attribute -- do not serialize if missing
    pub version: Option<TsBytes<'a>>,
    /// Source language on which this translation is based on.
    pub source_language: Option<TsBytes<'a>>,
    /// Language of this translation.
    pub language: Option<TsBytes<'a>>,
    /// Translations attached to a context
    pub contexts: Vec<ContextNode<'a>>, // Context[0..*] or message[0..*] TODO: support that.
    /// Catalogs dependencies
    pub dependencies: Option<DependenciesNode>, // TODO: parse them
                                                // TODO: support extra-something
}

impl<'a> TsNode<'a> {
    pub fn from_reader(
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<Self, ParseError> {
        let mut contexts = vec![];
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
                        .map(|n| contexts.push(n))
                        .map_err(|e| debug!("Error parsing context node: {:?}", e))
                        .expect("Expected to succeed"); // TODO: improve that.
                }
                Event::End(ref e) if e.name().as_ref().eq_ignore_ascii_case(b"ts") => break,
                _ => debug!("TsNode: unknown event: {:?}", event),
            }
        }

        let mut node = TsNode::<'_> {
            contexts: contexts,
            ..Default::default()
        };
        node.assign_attributes(element);

        Ok(node)
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
