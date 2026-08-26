use std::borrow::Cow;

/// TranslationType defines the status of a translation (aka the progress)
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum TranslationType {
    /// Translation is completed
    #[default]
    Finished,
    /// Translation is not finished
    Unfinished,
    /// Translation requires an update
    Obsolete,
    /// Translation is not used anymore
    Vanished,
}

impl<'a> From<Cow<'a, str>> for TranslationType {
    fn from(value: Cow<'a, str>) -> Self {
        let trimmed = value.trim_ascii();
        if trimmed.eq_ignore_ascii_case("unfinished") {
            TranslationType::Unfinished
        } else if trimmed.eq_ignore_ascii_case("obsolete") {
            TranslationType::Obsolete
        } else if trimmed.eq_ignore_ascii_case("vanished") {
            TranslationType::Vanished
        } else {
            TranslationType::Finished
        }
    }
}

#[cfg(test)]
mod test_translation_type {
    use crate::parser::translation_type::TranslationType;
    use rstest::rstest;

    #[rstest]
    #[case("unFinIshed", TranslationType::Unfinished)]
    #[case("fiNIshed", TranslationType::Finished)]
    #[case("VAnishEd", TranslationType::Vanished)]
    #[case("obsoLETE", TranslationType::Obsolete)]
    fn from_cow_should_return_correct_value(
        #[case] attr_value: &str,
        #[case] expected: TranslationType,
    ) {
        let actual = TranslationType::from(std::borrow::Cow::Borrowed(attr_value));
        assert_eq!(expected, actual);
    }
}
