use crate::parser::numerus_form_node::NumerusFormNode;
use crate::parser::parse_error::ParseError;
use crate::parser::translation_type::TranslationType;
use crate::parser::yesno::YesNo;
use log::{debug, info, warn};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

/// Translation node that indicates an actual translation for a message.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct TranslationNode {
    // Did not find a way to make it an enum
    // Therefore: either you have a `translation_simple` or a `numerus_forms`, but not both.
    /// Simple translation version, which do not take plural forms into account
    // TODO: From Qt docs: Should have either 1 translation string or multiple numerus or multiple length variants.
    pub translation_simple: Option<String>,
    /// Plural forms for the translation
    pub numerus_forms: Vec<NumerusFormNode>,
    pub length_variants: Vec<String>,
    /// Translation type (which represents the translation status)
    pub translation_type: Option<TranslationType>,
    pub variants: Option<YesNo>,
    /// Extra data
    pub userdata: Option<String>, // deprecated
}

impl TranslationNode {
    pub fn from_reader(
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<Self, ParseError> {
        #[derive(Debug, Eq, PartialEq)]
        enum Tag {
            None,
            Translation,
            LengthVariant,
            UserData,
        }
        let mut node = Self::default();
        let mut current_tag = Tag::None;
        info!("TranslationNode");

        loop {
            let event = reader.read_event()?;
            if let Event::Start(ref ev) = event {
                debug!("TranslationNode::{current_tag:?}: found {:#?}", ev.name());
            }

            match event {
                Event::Start(ref e) => {
                    debug!("TranslationNode::{current_tag:?}: Found element \"{e:#?}\"");

                    if Tag::None != current_tag {
                        return Err(ParseError::from(format!(
                            "TranslationNode::{current_tag:?}: Unexpected tag opening: ${e:?}"
                        )));
                    }

                    current_tag = match e.name().as_ref() {
                        "translation" => Tag::Translation,
                        "numerusform" => NumerusFormNode::from_reader(reader, e).map(|n| {
                            node.numerus_forms.push(n);
                            Tag::None
                        })?,
                        "lengthvariant" => Tag::LengthVariant,
                        "userdata" => Tag::UserData,
                        _ => {
                            warn!("TranslationNode::{current_tag:?}: Unknown field: {e:#?}");
                            Tag::None
                        }
                    };
                }
                Event::Text(e) => {
                    debug!("TranslationNode::{current_tag:?}: found text: {e:#?}");
                    let text = e.to_string();
                    match current_tag {
                        Tag::None => node.translation_simple = Some(text),
                        Tag::LengthVariant => node.length_variants.push(text),
                        Tag::UserData => node.userdata = Some(text),
                        _ => {} // TODO: error message?
                    }
                }

                Event::End(ref e) => {
                    debug!("TranslationNode::{current_tag:?}: Found END element \"{e:#?}\"");
                    match e.name().as_ref() {
                        "translation" => break,
                        "lengthvariant" | "userdata" => current_tag = Tag::None,
                        _ => {
                            debug!("TranslationNode::{current_tag:?}: ending unknown field: {e:#?}")
                        }
                    }
                }
                _ => debug!(
                    "TranslationNode::{current_tag:?}: unknown event: {:?}",
                    event
                ),
            }
        }

        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                "type" => node.translation_type = Some(TranslationType::from(a.value)),
                "variants" => node.variants = Some(YesNo::from(a.value)),
                _ => debug!("TranslationNode: unknown attribute: {:?}", a.key),
            });

        Ok(node)
    }
}
