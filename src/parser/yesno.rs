use std::borrow::Cow;

#[derive(Debug, Eq, Clone, PartialEq)]
pub enum YesNo {
    Yes,
    No,
}

impl<'a> From<Cow<'a, str>> for YesNo {
    fn from(value: Cow<'a, str>) -> Self {
        if value.trim_ascii().eq_ignore_ascii_case("yes") {
            YesNo::Yes
        } else {
            YesNo::No
        }
    }
}

#[cfg(test)]
mod test_yesno {
    use crate::parser::yesno::YesNo;
    use rstest::rstest;

    #[rstest]
    #[case("yes")]
    #[case("YES")]
    #[case("Yes")]
    #[case("yEs")]
    #[case("yeS")]
    #[case(" yes ")]
    fn test_from_cow_should_match_yes(#[case] attr_value: &str) {
        let actual = YesNo::from(std::borrow::Cow::Borrowed(attr_value));
        assert_eq!(YesNo::Yes, actual);
    }

    #[rstest]
    #[case("no")]
    #[case("NO")]
    #[case("No")]
    #[case("nO")]
    #[case(" no ")]
    #[case("")]
    fn test_from_cow_should_match_no(#[case] attr_value: &str) {
        let actual = YesNo::from(std::borrow::Cow::Borrowed(attr_value));
        assert_eq!(YesNo::No, actual);
    }
}
