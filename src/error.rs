use fantoccini::error as ferorr;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, SideRunnerError>;

// TODO: IlligalSyntax
pub enum SideRunnerError {
    ParseError(ParseError),
    WebdriverError(ferorr::CmdError),
    MismatchedType(String),
    Timeout(String),
}

impl std::fmt::Debug for SideRunnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(err) => write!(f, "parse error: {:?}", err),
            Self::WebdriverError(err) => write!(f, "webdriver error {:?}", err),
            Self::MismatchedType(desc) => write!(f, "mismatched type {}", desc),
            Self::Timeout(desc) => write!(f, "timeout {}", desc),
        }
    }
}

impl From<ParseError> for SideRunnerError {
    fn from(err: ParseError) -> Self {
        SideRunnerError::ParseError(err)
    }
}

impl From<ferorr::CmdError> for SideRunnerError {
    fn from(err: ferorr::CmdError) -> Self {
        SideRunnerError::WebdriverError(err)
    }
}

pub enum ParseError {
    FormatError(serde_json::Error),
    LocatorFormatError(String),
    TypeError(String),
    ValidationError(String),
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FormatError(err) => write!(f, "parsing json error {:?}", err),
            Self::LocatorFormatError(err) => write!(f, "locator has wrong format {:?}", err),
            Self::TypeError(err) => write!(f, "unexpected type {:?}", err),
            Self::ValidationError(err) => write!(f, "validation error {:?}", err),
        }
    }
}
