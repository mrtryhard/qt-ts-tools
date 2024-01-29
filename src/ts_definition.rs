use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

// This file defines the schema matching (or trying to match?) Qt's XSD
// Eventually when a proper Rust code generator exists it would be great to use that instead.
// For now they can't handle Qt's semi-weird XSD.
// https://doc.qt.io/qt-6/linguist-ts-file-format.html

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename = "TS")]
pub struct TSNode {
    #[serde(rename = "@version", skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(rename = "@sourcelanguage", skip_serializing_if = "Option::is_none")]
    source_language: Option<String>,
    #[serde(rename = "@language", skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(rename = "context", skip_serializing_if = "Vec::is_empty", default)]
    pub contexts: Vec<ContextNode>,
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

#[derive(Debug, Eq, Deserialize, Serialize, PartialEq)]
pub struct ContextNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "message")]
    pub messages: Vec<MessageNode>,
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

#[derive(Debug, Eq, Deserialize, Serialize, PartialEq)]
pub struct MessageNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oldsource: Option<String>, // Result of merge
    #[serde(skip_serializing_if = "Option::is_none")]
    translation: Option<TranslationNode>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "location", default)]
    pub locations: Vec<LocationNode>,
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

#[derive(Debug, Eq, Deserialize, Serialize, PartialEq)]
pub struct TranslationNode {
    // Did not find a way to make it an enum
    // Therefore: either you have a `translation_simple` or a `numerus_forms`, but not both.
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    translation_simple: Option<String>,
    #[serde(rename = "numerusform", skip_serializing_if = "Vec::is_empty", default)]
    numerus_forms: Vec<NumerusFormNode>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    translation_type: Option<String>, // e.g. "unfinished", "obsolete", "vanished"
    #[serde(skip_serializing_if = "Option::is_none")]
    variants: Option<String>, // "yes", "no"
    #[serde(skip_serializing_if = "Option::is_none")]
    userdata: Option<String>, // deprecated
}

#[derive(Debug, Eq, Deserialize, Serialize, PartialEq)]
pub struct LocationNode {
    #[serde(rename = "@filename", skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(rename = "@line", skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug, Eq, Deserialize, Serialize, PartialEq)]
pub struct NumerusFormNode {
    #[serde(default, rename = "$value", skip_serializing_if = "String::is_empty")]
    text: String,
    #[serde(rename = "@variants", skip_serializing_if = "Option::is_none")]
    filename: Option<String>, // "yes", "no"
}

impl PartialOrd<Self> for MessageNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let min_self = self
            .locations
            .iter()
            .min_by_key(|location| (location.filename.as_ref(), location.line))
            .map(|location| (location.filename.as_ref(), location.line.as_ref()))
            .unwrap_or_default();

        let min_other = other
            .locations
            .iter()
            .min_by_key(|location| (location.filename.as_ref(), location.line))
            .map(|location| (location.filename.as_ref(), location.line.as_ref()))
            .unwrap_or_default();

        // Counterintuitive, but we want to have locationless message at the end:
        // handle `None` differently from default.
        if min_self.0 == None && min_other.0 != None {
            Some(Ordering::Greater)
        } else if min_self.0 == min_other.0 && min_self.1 == None && min_other.1 != None {
            Some(Ordering::Greater)
        } else {
            min_self.partial_cmp(&min_other)
        }
    }
}

impl Ord for MessageNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other)
            .expect("PartialOrd should always return a value for MessageNode")
    }
}

impl Ord for LocationNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other)
            .expect("PartialOrd should always return a value for LocationNode")
    }
}

impl PartialOrd<Self> for LocationNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self
            .filename
            .as_ref()
            .unwrap_or(&"".to_owned())
            .to_lowercase()
            .partial_cmp(
                &other
                    .filename
                    .as_ref()
                    .unwrap_or(&"".to_owned())
                    .to_lowercase(),
            )
            .expect("LocationNode::filename should have an ordering")
        {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => self.line.partial_cmp(&other.line),
        }
    }
}

impl PartialOrd<Self> for ContextNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Contexts are generally module or classes names; let's assume they don't need any special collation treatment.
        self.name
            .as_ref()
            .unwrap_or(&"".to_owned())
            .to_lowercase()
            .partial_cmp(&other.name.as_ref().unwrap_or(&"".to_owned()).to_lowercase())
    }
}

impl Ord for ContextNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Contexts are generally module or classes names; let's assume they don't need any special collation treatment.
        self.name.cmp(&other.name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quick_xml;
    // TODO: Data set. https://github.com/qt/qttranslations/
    #[test]
    fn parse_with_numerus_forms() {
        let f = quick_xml::Reader::from_file("./test_data/example1.xml")
            .expect("Couldn't open example1 test file");

        let data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
        assert_eq!(data.contexts.len(), 2);
        assert_eq!(data.version.unwrap(), "2.1");
        assert_eq!(data.source_language.unwrap(), "en");
        assert_eq!(data.language.unwrap(), "sv");

        let context1 = &data.contexts[0];
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
        let numerus_forms = &message_c1_3.translation.as_ref().unwrap().numerus_forms;
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
        let f = quick_xml::Reader::from_file("./test_data/example_key_de.xml")
            .expect("Couldn't open example1 test file");

        let data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
        assert_eq!(data.contexts.len(), 1);
        assert_eq!(data.version.unwrap(), "1.1");
        assert_eq!(data.source_language, None);
        assert_eq!(data.language.unwrap(), "de");

        let context1 = &data.contexts[0];
        assert_eq!(context1.name.as_ref().unwrap(), "tst_QKeySequence");
        assert_eq!(context1.messages.len(), 11);
        let message_c1_2 = &context1.messages[2];
        let locations = &message_c1_2.locations;
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
