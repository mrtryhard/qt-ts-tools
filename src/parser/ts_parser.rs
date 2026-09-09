use crate::parser::parse_error::ParseError;
use crate::parser::ts_node::TsNode;
use log::debug;
use quick_xml::Reader;
use quick_xml::events::Event;

pub struct TsParser {}
pub struct TsDocument {
    pub root: TsNode,
}

impl TsParser {
    pub fn from_buffer(buf: Vec<u8>) -> Result<TsDocument, ParseError> {
        let mut reader = Reader::from_reader(buf.as_slice());
        let mut ts_node: Result<TsNode, ParseError> = Err(ParseError::from("Not parsed."));
        reader.config_mut().expand_empty_elements = true;

        loop {
            let event = reader.read_event()?;

            if let Event::Start(ref ev) = event {
                debug!("TsParser: Found {:#?}", ev.name());
            }

            match event {
                Event::Eof => {
                    debug!("TsParser: EOF detected");
                    break;
                }
                Event::Start(ref e) if e.name().as_ref().eq_ignore_ascii_case("ts") => {
                    ts_node = TsNode::from_reader(&mut reader, e);
                }
                _ => (), // TODO: Better handling here
            }
        }

        Ok(TsDocument { root: ts_node? })
    }
}

#[cfg(test)]
mod test_tsnode {
    use crate::parser::ts_parser::TsParser;
    use rstest::rstest;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    mod test_location_node {
        use crate::parser::ts_parser::TsParser;
        use crate::parser::ts_parser::test_tsnode::init;
        use rstest::rstest;

        #[rstest]
        #[case("", None)]
        #[case("filename=\"root/file/path\"", Some("root/file/path"))]
        #[case("filename=\"\"", Some(""))]
        fn test_location_node_filename(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        <source>This is a test</source>
                        <location {raw} line="2" />
                        <location {raw} line="10" />
                    </message>
                </context>
            </ts>"#
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;

            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(node.contexts[0].messages[0].locations.len(), 2);
            assert_eq!(
                node.contexts[0].messages[0].locations[0]
                    .filename
                    .as_ref()
                    .map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("line=\"\"", None)]
        #[case("line=\"2\"", Some(2))]
        #[case("line=\"-3\"", Some(-3))]
        fn test_location_node_line(#[case] raw: &str, #[case] expected_parsed: Option<i32>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        <source>This is a test</source>
                        <location {raw} />
                    </message>
                </context>
            </ts>"#
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;

            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert!(!node.contexts[0].messages[0].locations.is_empty());
            assert_eq!(
                node.contexts[0].messages[0].locations[0].line,
                expected_parsed
            );
        }
    }

    mod test_translation_node {
        use crate::parser::numerus_form_node::NumerusFormNode;
        use crate::parser::translation_type::TranslationType;
        use crate::parser::ts_parser::TsParser;
        use crate::parser::ts_parser::test_tsnode::init;
        use rstest::rstest;

        #[rstest]
        #[case::no_attribute("", None)]
        // TODO: determine if we should throw error instead.
        #[case::empty("type=\"\"", Some(TranslationType::Finished))]
        #[case::finished("type=\"finished\"", Some(TranslationType::Finished))]
        #[case::unfinished("type=\"unfinished\"", Some(TranslationType::Unfinished))]
        #[case::obsolete("type=\"obsolete\"", Some(TranslationType::Obsolete))]
        #[case::vanished("type=\"vanished\"", Some(TranslationType::Vanished))]
        fn test_translation_node_type(
            #[case] raw: &str,
            #[case] expected_parsed: Option<TranslationType>,
        ) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        <source>This is a test</source>
                        <translation {raw}></translation>
                    </message>
                </context>
            </ts>"#
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert!(node.contexts[0].messages[0].translation.is_some());
            assert_eq!(
                node.contexts[0].messages[0]
                    .translation
                    .as_ref()
                    .expect("to have translation")
                    .translation_type,
                expected_parsed
            );
        }

        #[rstest]
        #[case::nothing("", vec![])]
        #[case::empty("<numerusform></numerusform>", vec![NumerusFormNode::default()])] // TODO: confirm behaviour
        #[case::with_text("<numerusform>%n text</numerusform>", vec![NumerusFormNode { text: "%n text".to_string(), ..Default::default() }])]
        #[case::with_text_with_bytes("<numerusform>text <byte value=\"xD\"/> test</numerusform>", vec![NumerusFormNode { text: "text <byte value=\"xD\"/> test".to_string(), ..Default::default()}])]
        #[case::with_many("<numerusform>text: %n</numerusform><numerusform>text single</numerusform>", vec![NumerusFormNode { text: "text: %n".to_string(), ..Default::default() }, NumerusFormNode { text: "text single".to_string(), ..Default::default() }])]
        fn test_translation_node_numerusform(
            #[case] raw: &str,
            #[case] expected_parsed: Vec<NumerusFormNode>,
        ) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        <source>This is a test</source>
                        <translation>
                            {raw}
                        </translation>
                    </message>
                </context>
            </ts>"#
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert!(node.contexts[0].messages[0].translation.is_some());
            assert_eq!(
                node.contexts[0].messages[0]
                    .translation
                    .as_ref()
                    .expect("to have translation")
                    .numerus_forms,
                expected_parsed
            );
        }

        #[rstest]
        #[case::nothing("", vec![])]
        #[case::empty("<lengthvariant></lengthvariant>", vec![])] // TODO: confirm behaviour
        #[case::with_text("<lengthvariant>text</lengthvariant>", vec!["text"])]
        //TODO: #[case::with_text_with_bytes("<lengthvariant>text <byte value=\"xD\"/> test</lengthvariant>", vec![TsBytes::from("text test")])]
        #[case::with_many("<lengthvariant>text</lengthvariant><lengthvariant>text second</lengthvariant>", vec!["text", "text second"])]
        fn test_translation_node_lengthvariants(
            #[case] raw: &str,
            #[case] expected_parsed: Vec<&str>,
        ) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        <source>This is a test</source>
                        <translation>
                            {raw}
                        </translation>
                    </message>
                </context>
            </ts>"#
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert!(node.contexts[0].messages[0].translation.is_some());
            assert_eq!(
                node.contexts[0].messages[0]
                    .translation
                    .as_ref()
                    .expect("to have translation")
                    .length_variants,
                expected_parsed
            );
        }
    }

    mod test_message_node {
        use crate::parser::ts_parser::TsParser;
        use crate::parser::ts_parser::test_tsnode::init;
        use crate::parser::yesno::YesNo;
        use rstest::rstest;

        #[rstest]
        #[case("", None)]
        #[case("numerus=\"\"", Some(YesNo::No))]
        #[case("numerus=\"yes\"", Some(YesNo::Yes))]
        #[case("numerus=\"no\"", Some(YesNo::No))]
        fn test_message_node_numerus(#[case] raw: &str, #[case] expected_parsed: Option<YesNo>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message {}>
                        <source>This is a test</source>
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(node.contexts[0].messages[0].numerus, expected_parsed);
        }

        #[rstest]
        #[case("", None)]
        #[case("id=\"\"", Some(""))]
        #[case("id=\"test\"", Some("test"))]
        fn test_message_node_id(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message {}>
                        <source>This is a test</source>
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0].id.as_ref().map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<source></source>", None)]
        #[case("<source>a/b/c.cpp</source>", Some("a/b/c.cpp"))]
        fn test_message_node_source(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        {}
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .source
                    .as_ref()
                    .map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<extracomment></extracomment>", None)]
        #[case("<extracomment>test</extracomment>", Some("test"))]
        fn test_message_node_extracomment(
            #[case] raw: &str,
            #[case] expected_parsed: Option<&str>,
        ) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        {}
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .extra_comment
                    .as_ref()
                    .map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<oldsource></oldsource>", None)]
        #[case("<oldsource>a/b/c.cpp</oldsource>", Some("a/b/c.cpp"))]
        fn test_message_node_oldsource(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        {}
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .old_source
                    .as_ref()
                    .map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<oldcomment></oldcomment>", None)]
        #[case("<oldcomment>a/b/c.cpp</oldcomment>", Some("a/b/c.cpp"))]
        fn test_message_node_oldcomment(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        {}
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .old_comment
                    .as_ref()
                    .map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<translatorcomment></translatorcomment>", None)]
        #[case("<translatorcomment>a/b/c.cpp</translatorcomment>", Some("a/b/c.cpp"))]
        fn test_message_node_translatorcomment(
            #[case] raw: &str,
            #[case] expected_parsed: Option<&str>,
        ) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        {}
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .translator_comment
                    .as_ref()
                    .map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<userdata></userdata>", None)]
        #[case("<userdata>a/b/c.cpp</userdata>", Some("a/b/c.cpp"))]
        fn test_message_node_userdata(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        {}
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .userdata
                    .as_ref()
                    .map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<comment></comment>", None)]
        #[case("<comment>a/b/c.cpp</comment>", Some("a/b/c.cpp"))]
        fn test_message_node_comment(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(
                r#"<!DOCTYPE TS>
            <ts>
                <context>
                    <message>
                        {}
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .comment
                    .as_ref()
                    .map(|s| s.as_ref()),
                expected_parsed
            );
        }
    }

    mod test_context_node {
        use crate::parser::ts_parser::TsParser;
        use crate::parser::ts_parser::test_tsnode::init;
        use rstest::rstest;

        #[rstest]
        #[case("", None)]
        #[case("encoding=\"\"", Some(""))]
        #[case("encoding=\"utf-8\"", Some("utf-8"))]
        fn test_context_node_encoding(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(r#"<!DOCTYPE TS><ts><context {}></context></ts>"#, raw);
            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0].encoding.as_ref().map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<name></name>", None)]
        #[case("<name>some_name</name>", Some("some_name"))]
        fn test_context_node_name(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(r#"<!DOCTYPE TS><ts><context>{}</context></ts>"#, raw);
            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0].name.as_ref().map(|s| s.as_ref()),
                expected_parsed
            );
        }

        #[rstest]
        #[case("", None)]
        #[case("<comment></comment>", None)]
        #[case("<comment>commentaire</comment>", Some("commentaire"))]
        fn test_context_node_comment(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(r#"<!DOCTYPE TS><ts><context>{}</context></ts>"#, raw);
            let parser =
                TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
            let node = parser.root;
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0].comment.as_ref().map(|s| s.as_ref()),
                expected_parsed
            );
        }
    }

    #[rstest]
    #[case("version=\"\"", Some(""))]
    #[case("version=\"2.1\"", Some("2.1"))]
    #[case("", None)]
    fn test_parses_version(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
        let raw = format!(r#"<!DOCTYPE TS><TS {} ></TS>"#, raw);
        let parser =
            TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
        let node = parser.root;

        assert_eq!(node.version.as_ref().map(|s| s.as_ref()), expected_parsed);
    }

    #[rstest]
    #[case("sourcelanguage=\"\"", Some(""))]
    #[case("sourcelanguage=\"en\"", Some("en"))]
    #[case("sourcelanguage=\"en_US\"", Some("en_US"))]
    #[case("", None)]
    fn test_parses_sourcelanguage(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
        let raw = format!(r#"<!DOCTYPE TS><TS {} ></TS>"#, raw);
        let parser =
            TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
        let node = parser.root;

        assert_eq!(
            node.source_language.as_ref().map(|s| s.as_ref()),
            expected_parsed
        );
    }

    #[rstest]
    #[case("language=\"\"", Some(""))]
    #[case("language=\"en\"", Some("en"))]
    #[case("language=\"en_US\"", Some("en_US"))]
    #[case("", None)]
    fn test_parses_language(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
        let raw = format!(r#"<!DOCTYPE TS><TS {} ></TS>"#, raw);
        let parser =
            TsParser::from_buffer(raw.as_bytes().into()).unwrap_or_else(|_| panic!("{raw:?}"));
        let node = parser.root;

        assert_eq!(node.language.as_ref().map(|s| s.as_ref()), expected_parsed);
    }
}
