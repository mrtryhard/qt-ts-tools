use crate::parser::message_node::MessageNode;
use crate::parser::parse_error::ParseError;
use crate::parser::ts_bytes::TsBytes;
use log::{debug, warn};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::cmp::Ordering;

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

impl<'a> PartialOrd<Self> for ContextNode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for ContextNode<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Contexts are generally module or classes names; let's assume they don't need any special collation treatment.
        self.name.cmp(&other.name)
    }
}

impl<'a> ContextNode<'a> {
    pub fn from_reader(
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<ContextNode<'a>, ParseError> {
        #[derive(Debug, Eq, PartialEq)]
        enum Tag {
            None,
            Name,
            Comment,
        }
        let mut node = ContextNode::default();
        let mut buffer = Vec::new();
        let mut current_tag = Tag::None;
        debug!("ContextNode");
        
        loop {
            let event = reader.read_event_into(&mut buffer)?;

            if let Event::Start(ref ev) = event {
                debug!("ContextNode: found {:#?}", ev.name());
            }

            match event {
                Event::Start(ref e) => {
                    debug!("ContextNode::{current_tag:#?}: Found element \"{e:#?}\"");

                    if Tag::None != current_tag {
                        return Err(ParseError::from(format!(
                            "ContextNode::{current_tag:#?}: Unexpected tag opening: ${e:?}"
                        )));
                    }

                    current_tag = match e.name().as_ref() {
                        b"name" => Tag::Name,
                        b"comment" => Tag::Comment,
                        b"message" => {
                            MessageNode::from_reader(reader, e).map(|n| node.messages.push(n))?;
                            Tag::None
                        }
                        _ => {
                            warn!("ContextNode::{current_tag:#?}: Unknown field: {e:#?}");
                            Tag::None
                        }
                    };
                }
                Event::Text(ref e) => {
                    debug!("ContextNode::{current_tag:#?}: found text: {:#?}", e);
                    let text = Some(TsBytes::Owned(e.to_vec()));
                    match current_tag {
                        Tag::Name => node.name = text,
                        Tag::Comment => node.comment = text,
                        _ => {} // TODO: better logging or error message?
                    }
                }

                Event::End(ref e) => {
                    debug!("ContextNode::{current_tag:#?}: Found END element \"{e:#?}\"");
                    match e.name().as_ref() {
                        b"context" => break,
                        b"comment" | b"message" | b"name" => current_tag = Tag::None,
                        _ => debug!("ContextNode::{current_tag:#?}: unknown closing field: {e:#?}"),
                    }
                }
                _ => debug!("ContextNode::{current_tag:#?}: unknown event: {event:?}"),
            }
        }

        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                b"encoding" => node.encoding = Some(TsBytes::Owned(a.value.into_owned())),
                _ => debug!("ContextNode: unknown attribute: {:?}", a.key),
            });

        Ok(node)
    }
}
