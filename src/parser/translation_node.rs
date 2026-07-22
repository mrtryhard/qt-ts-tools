use crate::parser::numerus_form_node::NumerusFormNode;
use crate::parser::parse_error::ParseError;
use crate::parser::translation_type::TranslationType;
use crate::parser::ts_bytes::TsBytes;
use crate::parser::yesno::YesNo;
use log::{debug, info, warn};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

/// Translation node that indicates an actual translation for a message.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct TranslationNode<'a> {
    // Did not find a way to make it an enum
    // Therefore: either you have a `translation_simple` or a `numerus_forms`, but not both.
    /// Simple translation version, which do not take plural forms into account
    // TODO: From Qt docs: Should have either 1 translation string or multiple numerus or multiple length variants.
    pub translation_simple: Option<TsBytes<'a>>,
    /// Plural forms for the translation
    pub numerus_forms: Vec<NumerusFormNode<'a>>,
    pub length_variants: Vec<TsBytes<'a>>,
    /// Translation type (which represents the translation status)
    pub translation_type: Option<TranslationType>,
    pub variants: Option<YesNo>,
    /// Extra data
    pub userdata: Option<TsBytes<'a>>, // deprecated
}

impl<'a> TranslationNode<'a> {
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
        let mut buffer = Vec::new();
        let mut current_tag = Tag::None;
        info!("Translation node");

        loop {
            let event = reader.read_event_into(&mut buffer)?;
            if let Event::Start(ref ev) = event {
                debug!("TranslationNode: found {:#?}", ev.name());
            }

            match event {
                Event::Start(ref e) => {
                    debug!("TranslationNode: Found element \"{e:#?}\"");

                    if Tag::None != current_tag {
                        // TODO: better logging or error message?
                        return Err(ParseError::from("Unexpected tag opening"));
                    }

                    current_tag = match e.name().as_ref() {
                        b"translation" => Tag::Translation,
                        b"numerusform" => NumerusFormNode::from_reader(reader, e).map(|n| {
                            node.numerus_forms.push(n);
                            Tag::None
                        })?,
                        b"lengthvariant" => Tag::LengthVariant,
                        b"userdata" => Tag::UserData,
                        _ => {
                            warn!("TranslationNode: Unknown field: {e:#?}");
                            Tag::None
                        }
                    };

                    debug!("Found tag {current_tag:#?}");
                }
                Event::Text(ref e) => {
                    debug!("TranslationNode: found text: {:#?}", e);
                    let text = TsBytes::Owned(e.to_vec());
                    match current_tag {
                        Tag::None => node.translation_simple = Some(text),
                        Tag::LengthVariant => node.length_variants.push(text),
                        Tag::UserData => node.userdata = Some(text),
                        _ => {} // TODO: error message?
                    }
                }

                Event::End(ref e) => {
                    debug!("TranslationNode: Found END element \"{e:#?}\"");
                    match e.name().as_ref() {
                        b"translation" => break,
                        b"lengthvariant" | b"userdata" => current_tag = Tag::None,
                        _ => debug!("TranslationNode: ending unknown field: {e:#?}"),
                    }
                }
                _ => debug!("TranslationNode: unknown event: {:?}", event),
            }
        }

        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                b"type" => node.translation_type = Some(TranslationType::from(a.value)),
                b"variants" => node.variants = Some(YesNo::from(a.value)),
                _ => debug!("TranslationNode: unknown attribute: {:?}", a.key),
            });

        Ok(node)
    }
}
