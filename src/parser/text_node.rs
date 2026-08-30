use crate::parser::parse_error::ParseError;
use log::{debug, info, warn};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

pub fn text_node_from_reader(
    reader: &mut Reader<&[u8]>,
    element: &BytesStart,
) -> Result<Option<String>, ParseError> {
    let event = reader.read_event()?;
    text_node_from_reader_with_event(reader, element, &event)
}

pub fn text_node_from_reader_with_event(
    reader: &mut Reader<&[u8]>,
    element: &BytesStart,
    event: &Event,
) -> Result<Option<String>, ParseError> {
    let mut text = vec![];
    info!("TextNode with event: {event:?}");

    let mut inner_event = event.clone();
    loop {
        match inner_event {
            Event::Start(e) => {
                debug!("TextNode: Found element \"{e:#?}\"");

                match e.name().as_ref() {
                    "byte" => {
                        if let Ok(attr) = e.try_get_attribute("value") {
                            attr.map(|kv| {
                                text.push(format!("<byte value=\"{}\"", kv.value));
                            });
                        };
                    }
                    _ => {
                        warn!("TextNode: Unknown field: {e:#?}");
                    }
                };
            }
            Event::Text(e) => {
                text.push(e.to_string());
            }

            Event::End(e) => {
                debug!("TextNode: Found END element \"{e:#?}\"");
                if element.name() == e.name() {
                    break;
                } else if e.name().as_ref() == "byte" {
                    text.push("/>".to_string());
                }
            }
            Event::Eof => break,
            _ => debug!("TextNode: unknown event: {:?}", inner_event),
        }
        inner_event = reader.read_event()?;
    }

    let value = text.join("");
    info!("TextNode: computed value: {value}");
    Ok(match value.len() {
        0 => None,
        _ => Some(value),
    })
}
