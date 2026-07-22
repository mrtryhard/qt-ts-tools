use crate::parser::parse_error::ParseError;
use crate::parser::ts_bytes::TsBytes;
use crate::parser::yesno::YesNo;
use log::{debug, info};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

/// Represents a translation plural form.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct NumerusFormNode<'a> {
    pub text: TsBytes<'a>,
    pub variants: Option<YesNo>,
}

impl<'a> NumerusFormNode<'a> {
    pub fn from_reader(
        reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<Self, ParseError> {
        let mut node = Self::default();
        let mut inner_buf = Vec::new();
        info!("NumerusFormNode");

        let g = reader.read_text_into(element.name(), &mut inner_buf)?;
        node.text = TsBytes::Owned(g.to_vec());
        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                b"variants" => node.variants = Some(YesNo::from(a.value)),
                _ => debug!("NumerusFormNode: unknown attribute: {:?}", a.key),
            });

        Ok(node)
    }
}
