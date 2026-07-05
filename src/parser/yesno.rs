use std::borrow::Cow;

#[derive(Debug, Eq, Clone, PartialEq)]
pub enum YesNo {
    Yes,
    No,
}

impl<'a> From<Cow<'a, [u8]>> for YesNo {
    fn from(value: Cow<'a, [u8]>) -> Self {
        if value.trim_ascii().eq_ignore_ascii_case(b"yes") {
            YesNo::Yes
        } else {
            YesNo::No
        }
    }
}

#[cfg(test)]
mod test_yesno {
    use rstest::rstest;
    use crate::parser::yesno::YesNo;

    #[rstest]
    #[case(b"yes")]
    #[case(b"YES")]
    #[case(b"Yes")]
    #[case(b"yEs")]
    #[case(b"yeS")]
    #[case(b" yes ")]
    fn test_from_cow_should_match_yes(#[case] attr_value: &[u8]) {
        let actual = YesNo::from(std::borrow::Cow::Borrowed(attr_value));
        assert_eq!(YesNo::Yes, actual);
    }

    #[rstest]
    #[case(b"no")]
    #[case(b"NO")]
    #[case(b"No")]
    #[case(b"nO")]
    #[case(b" no ")]
    #[case(b"")]
    fn test_from_cow_should_match_no(#[case] attr_value: &[u8]) {
        let actual = YesNo::from(std::borrow::Cow::Borrowed(attr_value));
        assert_eq!(YesNo::No, actual);
    }
}