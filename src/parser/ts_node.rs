use crate::parser::context_node::ContextNode;
use crate::parser::dependency_node::DependenciesNode;
use crate::parser::parse_error::ParseError;
use log::{debug, warn};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

/// Root node of the translation file.
#[derive(Debug, Default, PartialEq)]
pub struct TsNode {
    /// Defines the version of the TS format, although unused by this tool.
    pub version: Option<String>,
    /// Source language on which this translation is based on.
    pub source_language: Option<String>,
    /// Language of this translation.
    pub language: Option<String>,
    /// Translations attached to a context
    pub contexts: Vec<ContextNode>, // Context[0..*] or message[0..*] TODO: support that.
    /// Catalogs dependencies
    pub dependencies: Option<DependenciesNode>, // TODO: parse them
                                                // TODO: support extra-something
}

impl TsNode {
    pub fn from_reader(
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<Self, ParseError> {
        let mut contexts = vec![];
        let mut buffer = Vec::new();

        loop {
            let event = reader.read_event_into(&mut buffer)?;
            if let Event::Start(ref ev) = event {
                debug!("TsNode: Found {:#?}", ev.name());
            }

            match event {
                Event::Start(ref e) => match e.name().as_ref() {
                    "context" => ContextNode::from_reader(reader, e).map(|n| contexts.push(n))?,
                    _ => {
                        warn!("TsNode: unknown start event: {:?}", event);
                    }
                },
                Event::End(ref e) if e.name().as_ref().eq_ignore_ascii_case("ts") => break,
                _ => debug!("TsNode: unknown event: {:?}", event),
            }
        }

        let mut node = TsNode {
            contexts,
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
                "version" => self.version = Some(a.value.to_string().to_string()),
                "sourcelanguage" => self.source_language = Some(a.value.to_string()),
                "language" => self.language = Some(a.value.to_string()),
                _ => debug!("Unknown attribute: {:?}", a.key),
            });
    }
}
