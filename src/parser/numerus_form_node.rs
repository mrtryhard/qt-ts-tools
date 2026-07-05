use crate::parser::yesno::YesNo;

/// Represents a translation plural form.
#[derive(Debug, Default, Eq, Clone, PartialEq)]
pub struct NumerusFormNode {
    // #[serde(default, rename = "$value", skip_serializing_if = "String::is_empty")]
    pub text: String,
    // #[serde(rename = "@variants", skip_serializing_if = "Option::is_none")]
    pub variants: Option<YesNo>,
}