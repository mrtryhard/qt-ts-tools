use crate::parser::parse_error::ParseError;
use crate::parser::yesno::YesNo;
use log::{debug, info};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

/// Represents a translation plural form.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct NumerusFormNode {
    pub text: String,
    pub variants: Option<YesNo>,
}

impl NumerusFormNode {
    pub fn from_reader(
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<Self, ParseError> {
        let mut node = Self::default();
        info!("NumerusFormNode");

        let text = reader.read_text(element.name())?;
        node.text = text.to_string();
        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                "variants" => node.variants = Some(YesNo::from(a.value)),
                _ => debug!("NumerusFormNode: unknown attribute: {:?}", a.key),
            });

        Ok(node)
    }
}
