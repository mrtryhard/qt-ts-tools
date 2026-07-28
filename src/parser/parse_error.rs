use std::error::Error;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use quick_xml::encoding::EncodingError;
use quick_xml::events::attributes::AttrError;

#[derive(Debug, Eq, PartialEq)]
pub struct ParseError {
    err: String,
}

impl From<&str> for ParseError {
    fn from(value: &str) -> Self {
        ParseError {
            err: value.to_string(),
        }
    }
}

impl From<String> for ParseError {
    fn from(value: String) -> Self {
        ParseError {
            err: value,
        }
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(value: FromUtf8Error) -> Self {
        ParseError {
            err: value.to_string(),
        }
    }
}

impl From<Utf8Error> for ParseError {
    fn from(value: Utf8Error) -> Self {
        ParseError {
            err: value.to_string(),
        }
    }
}

impl From<ParseIntError> for ParseError {
    fn from(value: ParseIntError) -> Self {
        ParseError {
            err: value.to_string(),
        }
    }
}

impl From<quick_xml::Error> for ParseError {
    fn from(value: quick_xml::Error) -> Self {
        ParseError {
            err: value.to_string(),
        }
    }
}

impl From<EncodingError> for ParseError {
    fn from(value: EncodingError) -> Self {
        ParseError {
            err: value.to_string(),
        }
    }
}

impl From<AttrError> for ParseError {
    fn from(value: AttrError) -> Self {
        ParseError {
            err: value.to_string(),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.err)
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
