use std::cmp::Ordering;
use std::io::{BufWriter, Write};

use serde::{Deserialize, Serialize};

use crate::locale::tr_args;

// This file defines the schema matching (or trying to match?) Qt's XSD
// Eventually when a proper Rust code generator exists it would be great to use that instead.
// For now they can't handle Qt's semi-weird XSD.
// https://doc.qt.io/qt-6/linguist-ts-file-format.html

/// TranslationType defines the status of a translation (aka the progress)
#[derive(Debug, Default, Clone, Eq, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TranslationType {
    /// Translation is completed
    #[default]
    #[serde(skip)]
    Finished,
    /// Translation is not finished
    Unfinished,
    /// Translation requires an update
    Obsolete,
    /// Translation is not used anymore
    Vanished,
}

#[derive(Debug, Eq, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum YesNo {
    Yes,
    No,
}

/// Root node of the translation file.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename = "TS")]
pub struct TSNode {
    /// Defines the version of the TS format, although unused by this tool.
    #[serde(rename = "@version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Source language on which this translation is based on.
    #[serde(rename = "@sourcelanguage", skip_serializing_if = "Option::is_none")]
    pub source_language: Option<String>,
    /// Language of this translation.
    #[serde(rename = "@language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Translations attached to a context
    #[serde(rename = "context", skip_serializing_if = "Vec::is_empty", default)]
    pub contexts: Vec<ContextNode>,
    /// Standalone translation messages.
    #[serde(rename = "message", skip_serializing_if = "Vec::is_empty", default)]
    pub messages: Vec<MessageNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<DependenciesNode>,
    /// Translation comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Previous translation comment.
    #[serde(rename = "oldcomment", skip_serializing_if = "Option::is_none")]
    pub old_comment: Option<String>,
    /// Other, extra comment
    #[serde(rename = "extracomment", skip_serializing_if = "Option::is_none")]
    pub extra_comment: Option<String>,
    /// Translator comment
    #[serde(rename = "translatorcomment", skip_serializing_if = "Option::is_none")]
    pub translator_comment: Option<String>,
    /*
       Following section corresponds to `extra-something` in Qt's XSD. From documentation:
       > extra elements may appear in TS and message elements. Each element may appear
       > only once within each scope. The contents are preserved verbatim; any
       > attributes are dropped.
    */
    #[serde(
        rename = "extra-po-msgid_plural",
        skip_serializing_if = "Option::is_none"
    )]
    pub po_msg_id_plural: Option<String>,
    #[serde(
        rename = "extra-po-old_msgid_plural",
        skip_serializing_if = "Option::is_none"
    )]
    pub po_old_msg_id_plural: Option<String>,
    /// Comma separated list
    #[serde(rename = "extra-po-flags", skip_serializing_if = "Option::is_none")]
    pub loc_flags: Option<String>,
    #[serde(
        rename = "extra-loc-layout_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub loc_layout_id: Option<String>,
    #[serde(rename = "extra-loc-feature", skip_serializing_if = "Option::is_none")]
    pub loc_feature: Option<String>,
    #[serde(rename = "extra-loc-blank", skip_serializing_if = "Option::is_none")]
    pub loc_blank: Option<String>,
}

/// Context and its associated translated message.
#[derive(Debug, Eq, Deserialize, Serialize, PartialEq)]
pub struct ContextNode {
    /// Unique name of the context
    pub name: String,
    /// List of translation messages
    #[serde(rename = "message")]
    pub messages: Vec<MessageNode>,
    /// Comment describing information about the context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Encoding of the messages within that context.
    #[serde(rename = "@encoding", skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DependenciesNode {
    #[serde(rename = "dependency")]
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Dependency {
    pub catalog: String,
}

/// Translation message node.
#[derive(Debug, Eq, Clone, Deserialize, Serialize, PartialEq)]
pub struct MessageNode {
    /// Original string to translate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Old source before a merge. Merging will set that field.
    #[serde(rename = "oldsource", skip_serializing_if = "Option::is_none")]
    pub old_source: Option<String>,
    /// Translation in the target language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<TranslationNode>,
    /// Lines and files in which the translation message is used.
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "location", default)]
    pub locations: Vec<LocationNode>,
    /// This is "disambiguation" in the (new) API, or "msgctxt" in gettext speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Previous content of comment (result of merge)
    #[serde(rename = "oldcomment", skip_serializing_if = "Option::is_none")]
    pub old_comment: Option<String>,
    /// The real comment (added by developer/designer)
    #[serde(rename = "extracomment", skip_serializing_if = "Option::is_none")]
    pub extra_comment: Option<String>,
    /// Comment added by translator
    #[serde(rename = "translatorcomment", skip_serializing_if = "Option::is_none")]
    pub translator_comment: Option<String>,
    /// Support for the plural forms
    #[serde(rename = "@numerus", skip_serializing_if = "Option::is_none")]
    pub numerus: Option<YesNo>,
    /// Message unique id (not guaranteed to be existant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Extra information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userdata: Option<String>,
    /*
       Following section corresponds to `extra-something` in Qt's XSD. From documentation:
       > extra elements may appear in TS and message elements. Each element may appear
       > only once within each scope. The contents are preserved verbatim; any
       > attributes are dropped.
    */
    #[serde(
        rename = "extra-po-msgid_plural",
        skip_serializing_if = "Option::is_none"
    )]
    pub po_msg_id_plural: Option<String>,
    #[serde(
        rename = "extra-po-old_msgid_plural",
        skip_serializing_if = "Option::is_none"
    )]
    pub po_old_msg_id_plural: Option<String>,
    /// Comma separated list
    #[serde(rename = "extra-po-flags", skip_serializing_if = "Option::is_none")]
    pub loc_flags: Option<String>,
    #[serde(
        rename = "extra-loc-layout_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub loc_layout_id: Option<String>,
    #[serde(rename = "extra-loc-feature", skip_serializing_if = "Option::is_none")]
    pub loc_feature: Option<String>,
    #[serde(rename = "extra-loc-blank", skip_serializing_if = "Option::is_none")]
    pub loc_blank: Option<String>,
}

/// Translation node that indicates an actual translation for a message.
#[derive(Debug, Eq, Clone, Deserialize, Serialize, PartialEq)]
pub struct TranslationNode {
    // Did not find a way to make it an enum
    // Therefore: either you have a `translation_simple` or a `numerus_forms`, but not both.
    /// Simple translation version, which do not take plural forms into account
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub translation_simple: Option<String>,
    /// Plural forms for the translation
    #[serde(rename = "numerusform", skip_serializing_if = "Vec::is_empty", default)]
    pub numerus_forms: Vec<NumerusFormNode>,
    /// Translation type (which represents the translation status)
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub translation_type: Option<TranslationType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<YesNo>,
    /// Extra data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userdata: Option<String>, // deprecated
}

/// Location of a translation
#[derive(Debug, Eq, Clone, Deserialize, Serialize, PartialEq)]
pub struct LocationNode {
    /// File from which the translation source originates from.
    #[serde(rename = "@filename", skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Line where the source of the translation message is located in the file.
    #[serde(rename = "@line", skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// Represents a translation plural form.
#[derive(Debug, Eq, Clone, Deserialize, Serialize, PartialEq)]
pub struct NumerusFormNode {
    #[serde(default, rename = "$value", skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(rename = "@variants", skip_serializing_if = "Option::is_none")]
    pub variants: Option<YesNo>,
}

impl PartialOrd<Self> for MessageNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MessageNode {
    fn cmp(&self, other: &Self) -> Ordering {
        let id_cmp = other.id.cmp(&self.id);

        if id_cmp != Ordering::Equal {
            return id_cmp;
        }

        let (filename, line) = self
            .locations
            .iter()
            .min_by_key(|location| (location.filename.as_ref(), location.line))
            .map(|location| (location.filename.as_ref(), location.line.as_ref()))
            .unwrap_or_default();

        let (other_filename, other_line) = other
            .locations
            .iter()
            .min_by_key(|location| (location.filename.as_ref(), location.line))
            .map(|location| (location.filename.as_ref(), location.line.as_ref()))
            .unwrap_or_default();

        // Counterintuitive, but we want to have locationless message at the end:
        // handle `None` differently from default.
        if filename.is_none() && other_filename.is_some() {
            Ordering::Greater
        } else {
            (filename, line).cmp(&(other_filename, other_line))
        }
    }
}

impl Ord for LocationNode {
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .filename
            .as_ref()
            .unwrap_or(&"".to_owned())
            .to_lowercase()
            .cmp(
                &other
                    .filename
                    .as_ref()
                    .unwrap_or(&"".to_owned())
                    .to_lowercase(),
            ) {
            Ordering::Equal => self.line.cmp(&other.line),
            ordering => ordering,
        }
    }
}

impl PartialOrd<Self> for LocationNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<Self> for ContextNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContextNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Contexts are generally module or classes names; let's assume they don't need any special collation treatment.
        self.name.to_lowercase().cmp(&other.name.to_lowercase())
    }
}

/// Writes the output TS file to the specified output (file or stdout).
/// This writer will auto indent/pretty print. It will always expand empty nodes, e.g.
/// `<name></name>` instead of `<name/>`.
pub fn write_to_output(output_path: &Option<String>, node: &TSNode) -> Result<(), String> {
    let mut inner_writer: BufWriter<Box<dyn Write>> = match &output_path {
        None => BufWriter::new(Box::new(std::io::stdout().lock())),
        Some(output_path) => match std::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(output_path)
        {
            Ok(file) => BufWriter::new(Box::new(file)),
            Err(e) => {
                return Err(tr_args(
                    "ts-error-write-output-open",
                    [
                        ("output_path", output_path.into()),
                        ("error", e.to_string().into()),
                    ]
                    .into(),
                ))
            }
        },
    };

    let mut output_buffer =
        String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE TS>\n");
    let mut ser = quick_xml::se::Serializer::new(&mut output_buffer);
    ser.indent(' ', 2).expand_empty_elements(true);

    match node.serialize(ser) {
        Ok(_) => {
            let res = inner_writer.write_all(output_buffer.as_bytes());
            match res {
                Ok(_) => Ok(()),
                Err(e) => Err(tr_args(
                    "ts-error-write-serialize",
                    [("error", e.to_string().into())].into(),
                )),
            }
        }
        Err(e) => Err(tr_args(
            "ts-error-write-serialize",
            [("error", e.to_string().into())].into(),
        )),
    }
}

#[cfg(test)]
mod write_file_test {
    use super::*;

    #[test]
    fn test_write_to_output_file() {
        const OUTPUT_TEST_FILE: &str = "./test_data/test_result_write_to_ts.xml";

        let reader = quick_xml::Reader::from_file("./test_data/example1.xml")
            .expect("Couldn't open example1 test file");

        let data: TSNode = quick_xml::de::from_reader(reader.into_inner()).expect("Parsable");

        write_to_output(&Some(OUTPUT_TEST_FILE.to_owned()), &data).expect("Output");

        let f =
            quick_xml::Reader::from_file(OUTPUT_TEST_FILE).expect("Couldn't open output test file");

        let output_data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
        std::fs::remove_file(OUTPUT_TEST_FILE).expect("Test should clean test file.");
        assert_eq!(data, output_data);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO: Data set. https://github.com/qt/qttranslations/
    #[test]
    fn test_parse_with_numerus_forms() {
        let f = quick_xml::Reader::from_file("./test_data/example1.xml")
            .expect("Couldn't open example1 test file");

        let data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
        assert_eq!(data.contexts.len(), 2);
        assert_eq!(data.version.unwrap(), "2.1");
        assert_eq!(data.source_language.unwrap(), "en");
        assert_eq!(data.language.unwrap(), "sv");

        let context1 = &data.contexts[0];
        assert_eq!(context1.name, "kernel/navigationpart");
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
    fn test_parse_with_locations() {
        let f = quick_xml::Reader::from_file("./test_data/example_key_de.xml")
            .expect("Couldn't open example1 test file");

        let data: TSNode = quick_xml::de::from_reader(f.into_inner()).expect("Parsable");
        assert_eq!(data.contexts.len(), 1);
        assert_eq!(data.version.unwrap(), "1.1");
        assert_eq!(data.source_language, None);
        assert_eq!(data.language.unwrap(), "de");

        let context1 = &data.contexts[0];
        assert_eq!(context1.name, "tst_QKeySequence");
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
        assert_eq!(
            translation.translation_type,
            Some(TranslationType::Obsolete)
        );
    }
}
