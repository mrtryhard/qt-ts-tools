use crate::parser::parse_error::ParseError;
use crate::parser::ts_bytes::TsBytes;
use crate::parser::yesno::YesNo;
use log::{debug, info};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

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
        info!("NumerusFormNode  ");

        loop {
            let event = reader
                .read_event_into(&mut inner_buf)
                .expect("Error reading event");
            if let Event::Start(ref ev) = event {
                debug!("NumerusFormNode: found {:#?}", ev.name());
            }

            match event {
                Event::Text(ref e) => {
                    debug!("NumerusFormNode: found text: {:#?}", e);
                    node.text = TsBytes::Owned(e.to_vec());
                }

                Event::End(ref e) => {
                    debug!("NumerusFormNode: Found END element \"{e:#?}\"");
                    match e.name().as_ref() {
                        b"numerusform" => break,
                        _ => debug!("NumerusFormNode: ending unknown field: {e:#?}"),
                    }
                }
                _ => debug!("NumerusFormNode: unknown event: {:?}", event),
            }
        }

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
