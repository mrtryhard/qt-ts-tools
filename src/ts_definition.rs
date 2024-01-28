// https://doc.qt.io/qt-6/linguist-ts-file-format.html
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename = "TS")]
pub struct TSNode {
    #[serde(rename = "@version", skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(rename = "@sourcelanguage", skip_serializing_if = "Option::is_none")]
    source_language: Option<String>,
    #[serde(rename = "@language", skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(rename = "context", skip_serializing_if = "Option::is_none")]
    contexts: Option<Vec<ContextNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    messages: Option<Vec<MessageNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<DependenciesNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oldcomment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extracomment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translatorcomment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ContextNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "message")]
    messages: Vec<MessageNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "@encoding", skip_serializing_if = "Option::is_none")]
    encoding: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DependenciesNode {
    #[serde(rename = "dependency")]
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Dependency {
    catalog: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct MessageNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oldsource: Option<String>, // Result of merge
    #[serde(skip_serializing_if = "Option::is_none")]
    translation: Option<TranslationNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<Vec<LocationNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oldcomment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extracomment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    translatorcomment: Option<String>,
    #[serde(rename = "@numerus", skip_serializing_if = "Option::is_none")]
    numerus: Option<String>, // todo: boolean/enum? ("yes", "no", None/Default)
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    userdata: Option<String>,
    // todo: extra-something
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct TranslationNode {
    // Did not find a way to make it an enum
    // Therefore: either you have a `translation_simple` or a `numerus_forms`, but not both.
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    translation_simple: Option<String>,
    #[serde(rename = "numerusform", skip_serializing_if = "Option::is_none")]
    numerus_forms: Option<Vec<NumerusFormNode>>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    translation_type: Option<String>, // e.g. "unfinished", "obsolete", "vanished"
    #[serde(skip_serializing_if = "Option::is_none")]
    variants: Option<String>, // "yes", "no"
    #[serde(skip_serializing_if = "Option::is_none")]
    userdata: Option<String>, // deprecated
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LocationNode {
    #[serde(rename = "@line", skip_serializing_if = "Option::is_none")]
    line: Option<u32>,
    #[serde(rename = "@filename", skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct NumerusFormNode {
    #[serde(default, rename = "$value", skip_serializing_if = "String::is_empty")]
    text: String,
    #[serde(rename = "@variants", skip_serializing_if = "Option::is_none")]
    filename: Option<String>, // "yes", "no"
}

#[cfg(test)]
mod test {
    use super::*;
    use quick_xml;

    #[test]
    fn parse_with_numerus_forms() {
        let f =
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
            numerus_forms[0].text,
            "%1 prend au maximum %n argument. %2 est donc invalide."
        );
        assert_eq!(
            numerus_forms[1].text,
            "%1 prend au maximum %n arguments. %2 est donc invalide."
        );
    }

    #[test]
    fn parse_with_locations() {
        let f = quick_xml::Reader::from_file("example_key_de.xml")
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
