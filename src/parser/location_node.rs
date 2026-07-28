use crate::parser::parse_error::ParseError;
use crate::parser::ts_bytes::TsBytes;
use log::{debug, info};
use quick_xml::Reader;
use quick_xml::events::BytesStart;
use std::cmp::Ordering;

/// Location of a translation
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocationNode<'a> {
    /// File from which the translation source originates from.
    pub filename: Option<TsBytes<'a>>,
    /// Line where the source of the translation message is located in the file.
    pub line: Option<i32>,
}

impl<'a> LocationNode<'a> {
    pub fn from_reader(
        _reader: &mut Reader<&[u8]>,
        element: &BytesStart,
    ) -> Result<Self, ParseError> {
        let mut location_node = LocationNode::default();
        info!("LocationNode");

        element.attributes().flatten().for_each(|a| {
            match a.key.as_ref() {
                b"filename" => location_node.filename = Some(TsBytes::Owned(a.value.into_owned())),
                b"line" => {
                    location_node.line = str::from_utf8(&a.value).ok().and_then(|s| s.parse().ok())
                }
                _ => debug!("LocationNode: unknown attribute: {:?}", a.key),
            }
        });

        Ok(location_node)
    }
}

impl<'a> PartialOrd<Self> for LocationNode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for LocationNode<'a> {
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
    use crate::parser::ts_bytes::TsBytes;
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
            filename: Some(TsBytes::from(b"file")),
            line: left_line,
        };

        let right = LocationNode {
            filename: Some(TsBytes::from(b"file")),
            line: right_line,
        };
        assert_eq!(left.cmp(&right), ordering);
    }

    #[rstest]
    #[case::none_none(None, None, Ordering::Equal)]
    #[case::none_some1(None, Some(TsBytes::from(b"file")), Ordering::Less)]
    #[case::some_a_none(Some(TsBytes::from(b"a")), None, Ordering::Greater)]
    #[case::some_a_some_a(Some(TsBytes::from(b"a")), Some(TsBytes::from(b"a")), Ordering::Equal)]
    #[case::some_a_some_b(Some(TsBytes::from(b"a")), Some(TsBytes::from(b"b")), Ordering::Less)]
    #[case::some_a_some_a(
        Some(TsBytes::from(b"b")),
        Some(TsBytes::from(b"a")),
        Ordering::Greater
    )]
    #[case::some_a_some_a_capitalization(
        Some(TsBytes::from(b"a")),
        Some(TsBytes::from(b"A")),
        Ordering::Greater
    )]
    fn test_ord_diff_by_filename(
        #[case] left_name: Option<TsBytes<'_>>,
        #[case] right_name: Option<TsBytes<'_>>,
        #[case] ordering: Ordering,
    ) {
        let left = LocationNode {
            filename: left_name,
            line: None,
        };

        let right = LocationNode {
            filename: right_name,
            line: None,
        };
        assert_eq!(left.cmp(&right), ordering);
    }
}
