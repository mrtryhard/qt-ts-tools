use crate::parser::parse_error::ParseError;
use log::{debug, info};
use quick_xml::Reader;
use quick_xml::events::BytesStart;
use std::cmp::Ordering;

/// Location of a translation
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocationNode {
    /// File from which the translation source originates from.
    pub filename: Option<String>,
    /// Line where the source of the translation message is located in the file.
    pub line: Option<i32>,
}

impl LocationNode {
    pub fn from_reader(
        _reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<Self, ParseError> {
        let mut location_node = LocationNode::default();
        info!("LocationNode");

        element
            .attributes()
            .flatten()
            .for_each(|a| match a.key.as_ref() {
                "filename" => location_node.filename = Some(a.value.to_string().to_string()),
                "line" => location_node.line = a.value.parse().ok(),
                _ => debug!("LocationNode: unknown attribute: {:?}", a.key),
            });

        Ok(location_node)
    }
}

impl PartialOrd<Self> for LocationNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LocationNode {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.filename.cmp(&other.filename) {
            Ordering::Equal => self.line.cmp(&other.line),
            ordering => ordering,
        }
    }
}

// TODO: Review location ordering
//       as location may have offset referencing previous
//       file. You cannot just arrange the file in disorder.

#[cfg(test)]
mod test_location_node_ord {
    use crate::parser::location_node::LocationNode;
    use rstest::rstest;
    use std::cmp::Ordering;

    #[rstest]
    #[case::none_none(None, None, Ordering::Equal)]
    #[case::none_some1(None, Some(1), Ordering::Less)]
    #[case::some0_none(Some(0), None, Ordering::Greater)]
    #[case::some1_some1(Some(1), Some(1), Ordering::Equal)]
    #[case::some0_some1(Some(0), Some(1), Ordering::Less)]
    #[case::some1_some0(Some(1), Some(0), Ordering::Greater)]
    #[case::some1_some0(Some(-1), Some(0), Ordering::Less)]
    fn test_ord_diff_by_line(
        #[case] left_line: Option<i32>,
        #[case] right_line: Option<i32>,
        #[case] ordering: Ordering,
    ) {
        let left = LocationNode {
            filename: Some("file".to_string()),
            line: left_line,
        };

        let right = LocationNode {
            filename: Some("file".to_string()),
            line: right_line,
        };
        assert_eq!(left.cmp(&right), ordering);
    }

    #[rstest]
    #[case::none_none(None, None, Ordering::Equal)]
    #[case::none_some1(None, Some("file"), Ordering::Less)]
    #[case::some_a_none(Some("a"), None, Ordering::Greater)]
    #[case::some_a_some_a(Some("a"), Some("a"), Ordering::Equal)]
    #[case::some_a_some_b(Some("a"), Some("b"), Ordering::Less)]
    #[case::some_a_some_a(Some("b"), Some("a"), Ordering::Greater)]
    #[case::some_a_some_a_capitalization(Some("a"), Some("A"), Ordering::Greater)]
    fn test_ord_diff_by_filename(
        #[case] left_name: Option<&str>,
        #[case] right_name: Option<&str>,
        #[case] ordering: Ordering,
    ) {
        let left = LocationNode {
            filename: left_name.map(|s| s.to_string()),
            line: None,
        };

        let right = LocationNode {
            filename: right_name.map(|s| s.to_string()),
            line: None,
        };
        assert_eq!(left.cmp(&right), ordering);
    }
}
