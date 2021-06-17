// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;

/// RunnerError represents a Error which may occure while running
/// running [`Command`].
///
/// [`Command`]: enum.Command.html
pub struct RunnerError {
    pub kind: RunnerErrorKind,
    pub index: usize,
    pub test: Option<String>,
}

impl RunnerError {
    /// New creates a new instance of RunnerError
    pub fn new(kind: RunnerErrorKind, index: usize) -> Self {
        Self {
            kind,
            index,
            test: None,
        }
    }
}

impl fmt::Debug for RunnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} on the command with index={}{}",
            self.kind,
            self.index,
            self.test
                .as_ref()
                .map_or("".into(), |test| format!(" {}", test))
        )
    }
}

/// RunnerErrorKind represents errors which may occure while
/// running a particular [`Test`].
///
/// [`Test`]: struct.Test.html
pub enum RunnerErrorKind {
    #[cfg(feature = "fantoccini_backend")]
    WebdriverError(fantoccini::error::CmdError),
    #[cfg(feature = "thirtyfour_backend")]
    WebdriverError(thirtyfour::error::WebDriverError),
    BranchValidationError(String),
    MismatchedType(String),
    Url(url::ParseError),
    Timeout(String),
    AssertFailed {
        lhs: String,
        rhs: String,
    },
}

impl std::fmt::Debug for RunnerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BranchValidationError(err) => write!(f, "branching error {:?}", err),
            Self::WebdriverError(err) => write!(f, "webdriver error {:?}", err),
            Self::Url(e) => write!(f, "url construction error {}", e),
            Self::MismatchedType(desc) => write!(f, "mismatched type {}", desc),
            Self::Timeout(desc) => write!(f, "timeout {}", desc),
            Self::AssertFailed { lhs, rhs } => write!(f, "assert failed {} == {}", lhs, rhs),
        }
    }
}

#[cfg(feature = "fantoccini_backend")]
impl From<fantoccini::error::CmdError> for RunnerErrorKind {
    fn from(err: fantoccini::error::CmdError) -> Self {
        RunnerErrorKind::WebdriverError(err)
    }
}

#[cfg(feature = "thirtyfour_backend")]
impl From<thirtyfour::error::WebDriverError> for RunnerErrorKind {
    fn from(err: thirtyfour::error::WebDriverError) -> Self {
        RunnerErrorKind::WebdriverError(err)
    }
}

impl From<url::ParseError> for RunnerErrorKind {
    fn from(err: url::ParseError) -> Self {
        RunnerErrorKind::Url(err)
    }
}

/// ParseError represents errors which may occure while
/// parsing a `Side` file.
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
