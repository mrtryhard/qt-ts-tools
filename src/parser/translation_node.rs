use crate::parser::numerus_form_node::NumerusFormNode;
use crate::parser::translation_type::TranslationType;
use crate::parser::yesno::YesNo;

/// Translation node that indicates an actual translation for a message.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct TranslationNode {
    // Did not find a way to make it an enum
    // Therefore: either you have a `translation_simple` or a `numerus_forms`, but not both.
    /// Simple translation version, which do not take plural forms into account
    // #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub translation_simple: Option<String>,
    /// Plural forms for the translation
    // #[serde(rename = "numerusform", skip_serializing_if = "Vec::is_empty", default)]
    pub numerus_forms: Vec<NumerusFormNode>,
    /// Translation type (which represents the translation status)
    // #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub translation_type: Option<TranslationType>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<YesNo>,
    /// Extra data
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub userdata: Option<String>, // deprecated
}
