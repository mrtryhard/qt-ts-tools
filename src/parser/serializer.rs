use crate::parser::context_node::ContextNode;
use crate::parser::location_node::LocationNode;
use crate::parser::message_node::MessageNode;
use crate::parser::numerus_form_node::NumerusFormNode;
use crate::parser::translation_node::TranslationNode;
use crate::parser::translation_type::TranslationType;
use crate::parser::ts_node::TsNode;
use crate::parser::ts_parser::TsDocument;
use crate::parser::yesno::YesNo;
use crate::tr;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::error::Error;
use std::io::Write;

pub fn write_to_output(output_path: &Option<String>, node: &TsDocument) -> Result<(), String> {
    let mut inner_writer: Box<dyn Write> = match &output_path {
        None => Box::new(std::io::stdout().lock()),
        Some(output_path) => match std::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(output_path)
        {
            Ok(file) => Box::new(file),
            Err(e) => {
                return Err(tr!(
                    "error-write-output-open",
                    output_path = output_path,
                    error = e.to_string()
                ));
            }
        },
    };

    serialize(&mut inner_writer, node).map_err(|e| e.to_string())
}

fn serialize(writer: &mut dyn Write, doc: &TsDocument) -> Result<(), Box<dyn Error>> {
    let mut xml_writer = Writer::new_with_indent(writer, b' ', 4);
    xml_writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;
    xml_writer.write_event(Event::DocType(BytesText::from_escaped("TS")))?;

    serialize_ts(&mut xml_writer, &doc.root)?;

    Ok(())
}

fn serialize_ts<W: Write>(writer: &mut Writer<W>, node: &TsNode) -> Result<(), Box<dyn Error>> {
    let mut ts = BytesStart::new("TS");

    if let Some(value) = &node.version {
        ts.push_attribute(("version".as_ref(), value.as_ref()));
    }

    if let Some(value) = &node.language {
        ts.push_attribute(("language".as_ref(), value.as_ref()));
    }

    if let Some(value) = &node.source_language {
        ts.push_attribute(("sourcelanguage".as_ref(), value.as_ref()));
    }

    writer.write_event(Event::Start(ts))?;

    for context in &node.contexts {
        serialize_context(writer, context)?;
    }

    // TODO: dependencies if needed

    writer.write_event(Event::End(BytesEnd::new("TS")))?;
    Ok(())
}

fn serialize_context<W: Write>(
    writer: &mut Writer<W>,
    node: &ContextNode,
) -> Result<(), Box<dyn Error>> {
    let mut context = BytesStart::new("context");
    if let Some(value) = &node.encoding {
        context.push_attribute(("encoding".as_ref(), value.as_ref()));
    }
    writer.write_event(Event::Start(context))?;

    write_string_event("name", writer, &node.name)?;
    write_string_event("comment", writer, &node.comment)?;

    for message in &node.messages {
        serialize_message(writer, message)?;
    }

    writer.write_event(Event::End(BytesEnd::new("context")))?;
    Ok(())
}

fn serialize_message<W: Write>(
    writer: &mut Writer<W>,
    node: &MessageNode,
) -> Result<(), Box<dyn Error>> {
    let mut message = BytesStart::new("message");
    if let Some(value) = &node.id {
        message.push_attribute(("id".as_ref(), value.as_ref()));
    }
    if let Some(YesNo::Yes) = &node.numerus {
        message.push_attribute(("numerus".as_ref(), "yes".as_ref()));
    }
    writer.write_event(Event::Start(message))?;

    for location in &node.locations {
        serialize_location(writer, location)?;
    }

    write_string_event("source", writer, &node.source)?;
    write_string_event("oldsource", writer, &node.old_source)?;
    write_string_event("comment", writer, &node.comment)?;
    write_string_event("oldcomment", writer, &node.old_comment)?;
    write_string_event("extracomment", writer, &node.extra_comment)?;
    write_string_event("translatorcomment", writer, &node.translator_comment)?;

    if let Some(translation) = &node.translation {
        serialize_translation(writer, translation)?;
    }

    write_string_event("userdata", writer, &node.userdata)?;
    write_string_event("extra-po-msgid-plural", writer, &node.po_msg_id_plural)?;
    write_string_event(
        "extra-po-oldmsgid-plural",
        writer,
        &node.po_old_msg_id_plural,
    )?;
    write_string_event("extra-loc-flags", writer, &node.loc_flags)?;
    write_string_event("extra-loc-layout_id", writer, &node.loc_layout_id)?;
    write_string_event("extra-loc-feature", writer, &node.loc_feature)?;
    write_string_event("extra-loc-blank", writer, &node.loc_blank)?;

    writer.write_event(Event::End(BytesEnd::new("message")))?;
    Ok(())
}

fn write_string_event<W: Write>(
    field_name: &str,
    writer: &mut Writer<W>,
    opt_value: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    if let Some(value) = &opt_value {
        writer.write_event(Event::Start(BytesStart::new(field_name)))?;
        writer.write_event(Event::Text(BytesText::from_escaped(value)))?;
        writer.write_event(Event::End(BytesEnd::new(field_name)))?;
    }
    Ok(())
}

fn serialize_location<W: Write>(
    writer: &mut Writer<W>,
    node: &LocationNode,
) -> Result<(), Box<dyn Error>> {
    let mut location = BytesStart::new("location");
    if let Some(value) = &node.filename {
        location.push_attribute(("filename".as_ref(), value.as_ref()));
    }
    if let Some(value) = node.line {
        location.push_attribute(("line".as_ref(), value.to_string().as_ref()));
    }
    writer.write_event(Event::Empty(location))?;
    Ok(())
}

fn serialize_translation<W: Write>(
    writer: &mut Writer<W>,
    node: &TranslationNode,
) -> Result<(), Box<dyn Error>> {
    let mut translation = BytesStart::new("translation");
    if let Some(t_type) = &node.translation_type {
        let type_str = match t_type {
            TranslationType::Finished => "finished",
            TranslationType::Unfinished => "unfinished",
            TranslationType::Obsolete => "obsolete",
            TranslationType::Vanished => "vanished",
        };
        translation.push_attribute(("type".as_ref(), type_str));
    }
    if let Some(YesNo::Yes) = &node.variants {
        translation.push_attribute(("variants".as_ref(), "yes".as_ref()));
    }
    writer.write_event(Event::Start(translation))?;

    if let Some(value) = &node.translation_simple {
        writer.write_event(Event::Text(BytesText::from_escaped(value)))?;
    }

    for form in &node.numerus_forms {
        serialize_numerus_form(writer, form)?;
    }

    for variant in &node.length_variants {
        writer.write_event(Event::Start(BytesStart::new("lengthvariant")))?;
        writer.write_event(Event::Text(BytesText::from_escaped(variant)))?;
        writer.write_event(Event::End(BytesEnd::new("lengthvariant")))?;
    }

    write_string_event("userdata", writer, &node.userdata)?;
    writer.write_event(Event::End(BytesEnd::new("translation")))?;
    Ok(())
}

fn serialize_numerus_form<W: Write>(
    writer: &mut Writer<W>,
    node: &NumerusFormNode,
) -> Result<(), Box<dyn Error>> {
    let mut form = BytesStart::new("numerusform");

    if let Some(YesNo::Yes) = &node.variants {
        form.push_attribute(("variants".as_ref(), "yes".as_ref()));
    }

    writer.write_event(Event::Start(form))?;
    writer.write_event(Event::Text(BytesText::from_escaped(node.text.clone())))?;
    writer.write_event(Event::End(BytesEnd::new("numerusform")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::initialize_logging;
    use crate::parser::ts_parser::TsParser;
    use rstest::{fixture, rstest};

    #[rstest]
    #[case::basic("basic.ts.xml")]
    #[case::embedded_tag("issue304_embedded_tag.ts.xml")]
    #[case::one_ctx_many_msg("one_ctx_many_msg.ts.xml")]
    #[case::many_ctx_many_msg("many_ctx_many_msgs_numerus.ts.xml")]
    fn test_serialization_should_be_symmetrical(#[case] basic: &str) {
        let expected_xml = std::fs::read_to_string(format!("test_data/serializer/{}", basic))
            .expect("Test file not found");
        let doc = TsParser::from_buffer(expected_xml.as_bytes().to_vec()).expect("Parsing failed");
        let mut buffer = Vec::new();
        serialize(&mut buffer, &doc).expect("Serialization failed");
        let serialized_xml = String::from_utf8(buffer).expect("Invalid utf8");

        assert_eq!(serialized_xml, expected_xml);
    }
}
