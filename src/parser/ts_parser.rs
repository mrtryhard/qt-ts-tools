use crate::parser::parse_error::ParseError;
use crate::parser::ts_node::TSNode;
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

    pub fn parse(&mut self) -> Result<TSNode<'_>, ParseError> {
        let mut reader = Reader::from_reader(self.buf.as_slice());
        let mut ts_node: TSNode<'_> = TSNode::default();
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
                    ts_node.from_reader(&mut reader, e);
                }
                _ => (),
            }
        }

        Ok(ts_node)
    }
}


#[cfg(test)]
mod test_tsnode {
    use rstest::rstest;
    use crate::parser::ts_parser::TsParser;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    mod test_message_node {
        use rstest::rstest;
        use crate::parser::ts_parser::test_tsnode::init;
        use crate::parser::ts_parser::TsParser;
        use crate::parser::yesno::YesNo;

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
            let node = parser.parse().expect(&format!("{:?}", raw));
            assert!(!node.contexts.is_empty());
            assert!(!node.contexts[0].messages.is_empty());
            assert_eq!(node.contexts[0].messages[0].numerus, expected_parsed);
        }
    }

    mod test_context_node {
        use rstest::rstest;
        use crate::parser::ts_parser::test_tsnode::init;
        use crate::parser::ts_parser::TsParser;

        #[rstest]
        #[case("", None)]
        #[case("encoding=\"\"", Some(""))]
        #[case("encoding=\"utf-8\"", Some("utf-8"))]
        fn test_context_node_encoding(#[case] raw: &str, #[case] expected_parsed: Option<&str>) {
            init();
            let raw = format!(r#"<!DOCTYPE TS><ts><context {}></context></ts>"#, raw);
            let mut parser = TsParser::new(raw.as_bytes().into());
            let node = parser.parse().expect(&format!("{:?}", raw));
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0]
                    .encoding
                    .as_ref()
                    .map(|s| str::from_utf8(&s).expect("valid utf8")),
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
            let node = parser.parse().expect(&format!("{:?}", raw));
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0]
                    .name
                    .as_ref()
                    .map(|s| str::from_utf8(&s).expect("valid utf8")),
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
            let node = parser.parse().expect(&format!("{:?}", raw));
            assert!(!node.contexts.is_empty());
            assert_eq!(
                node.contexts[0]
                    .comment
                    .as_ref()
                    .map(|s| str::from_utf8(&s).expect("valid utf8")),
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
        let node = parser.parse().expect(&format!("{:?}", raw));

        assert_eq!(
            node.version
                .as_ref()
                .map(|s| str::from_utf8(&s).expect("valid utf8")),
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
        let node = parser.parse().expect(&format!("{:?}", raw));

        assert_eq!(
            node.source_language
                .as_ref()
                .map(|s| str::from_utf8(&s).expect("valid utf8")),
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
        let node = parser.parse().expect(&format!("{:?}", raw));

        assert_eq!(
            node.language
                .as_ref()
                .map(|s| str::from_utf8(&s).expect("valid utf8")),
            expected_parsed
        );
    }
}
