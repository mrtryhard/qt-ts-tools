use crate::parser::parse_error::ParseError;
use crate::parser::ts_node::TsNode;
use log::debug;
use quick_xml::Reader;
use quick_xml::events::Event;

pub struct TsParser {
    buf: Vec<u8>,
}

impl TsParser {
    pub fn new(buf: Vec<u8>) -> Self {
        Self { buf }
    }

    pub fn parse(&mut self) -> Result<TsNode<'_>, ParseError> {
        let mut reader = Reader::from_reader(self.buf.as_slice());
        let mut ts_node: Result<TsNode<'_>, ParseError> = Err(ParseError::from("Not parsed."));
        let mut inner_buf = Vec::new();
        reader.config_mut().expand_empty_elements = true;

        loop {
            let event = reader.read_event_into(&mut inner_buf)?;

            if let Event::Start(ref ev) = event {
                debug!("Found {:#?}", ev.name());
            }

            match event {
                Event::Eof => {
                    debug!("EOF detected");
                    break;
                }
                Event::Start(ref e) if e.name().as_ref().eq_ignore_ascii_case(b"ts") => {
                    ts_node = TsNode::from_reader(&mut reader, e);
                }
                _ => (), // TODO: Better handling here
            }
        }

        ts_node
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));

            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(node.contexts[0].messages[0].locations.len(), 2);
            assert_eq!(
                node.contexts[0].messages[0].locations[0]
                    .filename
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("to parse")),
                expected_parsed
            );
        }
    }

    mod test_translation_node {
        use crate::parser::translation_type::TranslationType;
        use crate::parser::ts_parser::TsParser;
        use crate::parser::ts_parser::test_tsnode::init;
        use rstest::rstest;

        #[rstest]
        #[case("", None)]
        #[case("type=\"\"", None)]
        #[case("type=\"finished\"", Some(TranslationType::Finished))]
        #[case("type=\"unfinished\"", Some(TranslationType::Unfinished))]
        #[case("type=\"obsolete\"", Some(TranslationType::Obsolete))]
        #[case("type=\"vanished\"", Some(TranslationType::Vanished))]
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
                        <translation type="{}"></translation>
                    </message>
                </context>
            </ts>"#,
                raw
            );

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .id
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("to parse")),
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .source
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("to parse")),
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .extra_comment
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("to parse")),
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .old_source
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("to parse")),
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .old_comment
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("to parse")),
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .translator_comment
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("to parse")),
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .userdata
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("to parse")),
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

            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(
                node.contexts[0].messages[0]
                    .comment
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("to parse")),
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
            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0]
                    .encoding
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("valid utf8")),
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
            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0]
                    .name
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("valid utf8")),
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
            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0]
                    .comment
                    .as_ref()
                    .map(|s| str::from_utf8(s).expect("valid utf8")),
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
        let mut parser = TsParser::new(raw.as_bytes().into());
        let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));

        assert_eq!(
            node.version
                .as_ref()
                .map(|s| str::from_utf8(s).expect("valid utf8")),
            expected_parsed
        );
    }

    #[rstest]
    #[case("sourcelanguage=\"\"", Some(""))]
    #[case("sourcelanguage=\"en\"", Some("en"))]
    #[case("sourcelanguage=\"en_US\"", Some("en_US"))]
    #[case("", None)]
    fn test_parses_sourcelanguage(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
        let raw = format!(r#"<!DOCTYPE TS><TS {} ></TS>"#, raw);
        let mut parser = TsParser::new(raw.as_bytes().into());
        let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));

        assert_eq!(
            node.source_language
                .as_ref()
                .map(|s| str::from_utf8(s).expect("valid utf8")),
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
        let mut parser = TsParser::new(raw.as_bytes().into());
        let node = parser.parse().unwrap_or_else(|_| panic!("{raw:?}"));

        assert_eq!(
            node.language
                .as_ref()
                .map(|s| str::from_utf8(s).expect("valid utf8")),
            expected_parsed
        );
    }
}
