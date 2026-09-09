use crate::parser::message_node::MessageNode;
use crate::parser::parse_error::ParseError;
use log::{debug, warn};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::cmp::Ordering;

/// Context and its associated translated message.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ContextNode {
    /// Unique name of the context
    pub name: Option<String>,
    /// List of translation messages
    pub messages: Vec<MessageNode>,
    /// Comment describing information about the context
    pub comment: Option<String>,
    /// Encoding of the messages within that context.
    pub encoding: Option<String>,
}

impl PartialOrd<Self> for ContextNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContextNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Contexts are generally module or classes names; let's assume they don't need any special collation treatment.
        self.name.cmp(&other.name)
    }
}

impl ContextNode {
    pub fn from_reader(
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<Self, ParseError> {
        #[derive(Debug, Eq, PartialEq)]
        enum Tag {
            None,
            Name,
            Comment,
        }
        let mut node = ContextNode::default();
        let mut current_tag = Tag::None;
        debug!("ContextNode");

        loop {
            let event = reader.read_event()?;

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
                        "name" => Tag::Name,
                        "comment" => Tag::Comment,
                        "message" => {
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
                    let text = Some(e.to_string());
                    match current_tag {
                        Tag::Name => node.name = text,
                        Tag::Comment => node.comment = text,
                        _ => {} // TODO: better logging or error message?
                    }
                }

                Event::End(ref e) => {
                    debug!("ContextNode::{current_tag:#?}: Found END element \"{e:#?}\"");
                    match e.name().as_ref() {
                        "context" => break,
                        "comment" | "message" | "name" => current_tag = Tag::None,
                        _ => debug!("ContextNode::{current_tag:#?}: unknown closing field: {e:#?}"),
                    }
                }
                Event::Eof => break,
                _ => debug!("ContextNode::{current_tag:#?}: unknown event: {event:?}"),
            }
        }

        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                "encoding" => node.encoding = Some(a.value.to_string()),
                _ => debug!("ContextNode: unknown attribute: {:?}", a.key),
            });

        Ok(node)
    }
}
