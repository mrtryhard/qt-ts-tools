use std::cmp::Ordering;

/// Location of a translation
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocationNode {
    /// File from which the translation source originates from.
    // #[serde(rename = "@filename", skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Line where the source of the translation message is located in the file.
    // #[serde(rename = "@line", skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

impl PartialOrd<Self> for LocationNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
