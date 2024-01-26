// https://doc.qt.io/qt-6/linguist-ts-file-format.html
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
struct TSNode {
    #[serde(rename = "@version")]
    version: Option<String>,
    #[serde(rename = "@sourcelanguage")]
    source_language: Option<String>,
    #[serde(rename = "@language")]
    language: Option<String>,
    #[serde(rename = "context")]
    contexts: Option<Vec<ContextNode>>,
    messages: Option<Vec<MessageNode>>,
    dependencies: Option<DependenciesNode>,
    comment: Option<String>,
    oldcomment: Option<String>,
    extracomment: Option<String>,
    translatorcomment: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ContextNode {
    name: Option<String>,
    #[serde(rename = "message")]
    messages: Vec<MessageNode>,
    comment: Option<String>,
    #[serde(rename = "@encoding")]
    encoding: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct DependenciesNode {
    #[serde(rename = "dependency")]
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Dependency {
    catalog: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct MessageNode {
    source: Option<String>,
    oldsource: Option<String>, // Result of merge
    translation: Option<TranslationNode>,
    location: Option<Vec<LocationNode>>,
    comment: Option<String>,
    oldcomment: Option<String>,
    extracomment: Option<String>,
    translatorcomment: Option<String>,
    #[serde(rename = "@numerus")]
    numerus: Option<String>, // todo: boolean/enum? ("yes", "no", None/Default)
    id: Option<String>,
    userdata: Option<String>,
    // todo: extra-something
}

#[derive(Debug, Deserialize, PartialEq)]
struct TranslationNode {
    // Did not find a way to make it an enum
    // Therefore: either you have a `translation_simple` or a `numerus_forms`, but not both.
    #[serde(rename = "$text")]
    translation_simple: Option<String>,
    #[serde(rename = "numerusform")]
    numerus_forms: Option<Vec<NumerusFormNode>>,
    // TODO: lengthvariants ?
    #[serde(rename = "@type")]
    translation_type: Option<String>, // e.g. "unfinished", "obsolete", "vanished"
    variants: Option<String>, // "yes", "no"
    userdata: Option<String>, // deprecated
}

#[derive(Debug, Deserialize, PartialEq)]
struct LocationNode {
    #[serde(rename = "@line")]
    line: Option<u32>,
    #[serde(rename = "@filename")]
    filename: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct NumerusFormNode {
    #[serde(rename = "$value")]
    text: Option<String>,
    #[serde(rename = "@variants")]
    filename: Option<String>, // "yes", "no"
}

#[cfg(test)]
mod test {
    use super::*;
    use quick_xml;

    #[test]
    fn parse_with_numerus_forms() {
        let mut f =
            quick_xml::Reader::from_file("example1.xml").expect("Couldn't open example1 test file");

        let data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
        assert_eq!(data.contexts.as_ref().unwrap().len(), 2);
        assert_eq!(data.version.unwrap(), "2.1");
        assert_eq!(data.source_language.unwrap(), "en");
        assert_eq!(data.language.unwrap(), "sv");

        let context1 = &data.contexts.as_ref().unwrap()[0];
        assert_eq!(context1.name.as_ref().unwrap(), "kernel/navigationpart");
        assert_eq!(context1.messages.len(), 3);

        let message_c1_2 = &context1.messages[1];
        assert_eq!(message_c1_2.comment.as_ref().unwrap(), "Navigation part");
        assert_eq!(message_c1_2.source.as_ref().unwrap(), "vztnewsletter");
        assert_eq!(
            message_c1_2
                .translation
                .as_ref()
                .unwrap()
                .translation_simple
                .as_ref()
                .unwrap(),
            "vztnewsletter2"
        );

        let message_c1_3 = &context1.messages[2];
        assert_eq!(message_c1_3.comment, None);
        assert_eq!(
            message_c1_3.source.as_ref().unwrap(),
            "%1 takes at most %n argument(s). %2 is therefore invalid."
        );
        assert_eq!(
            message_c1_3
                .translation
                .as_ref()
                .unwrap()
                .translation_simple,
            None
        );
        let numerus_forms = message_c1_3
            .translation
            .as_ref()
            .unwrap()
            .numerus_forms
            .as_ref()
            .unwrap();
        assert_eq!(numerus_forms.len(), 2);
        assert_eq!(
            numerus_forms[0].text.as_ref().unwrap(),
            "%1 prend au maximum %n argument. %2 est donc invalide."
        );
        assert_eq!(
            numerus_forms[1].text.as_ref().unwrap(),
            "%1 prend au maximum %n arguments. %2 est donc invalide."
        );
    }

    #[test]
    fn parse_with_locations() {
        let mut f = quick_xml::Reader::from_file("example_key_de.xml")
            .expect("Couldn't open example1 test file");

        let data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
        assert_eq!(data.contexts.as_ref().unwrap().len(), 1);
        assert_eq!(data.version.unwrap(), "1.1");
        assert_eq!(data.source_language, None);
        assert_eq!(data.language.unwrap(), "de");

        let context1 = &data.contexts.as_ref().unwrap()[0];
        assert_eq!(context1.name.as_ref().unwrap(), "tst_QKeySequence");
        assert_eq!(context1.messages.len(), 11);
        let message_c1_2 = &context1.messages[2];
        let locations = message_c1_2.location.as_ref().unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(
            locations[0].filename.as_ref().unwrap(),
            "tst_qkeysequence.cpp"
        );
        assert_eq!(locations[0].line.as_ref().unwrap(), &150u32);
        assert_eq!(
            locations[1].filename.as_ref().unwrap(),
            "tst_qkeysequence.cpp"
        );
        assert_eq!(locations[1].line.as_ref().unwrap(), &371u32);
        let translation = &message_c1_2.translation.as_ref().unwrap();
        assert_eq!(translation.translation_simple.as_ref().unwrap(), "Alt+K");
        assert_eq!(translation.translation_type.as_ref().unwrap(), "obsolete");
    }
}
