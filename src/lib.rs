// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod error;
mod parser;
mod runner;
mod validation;
pub mod webdriver;

pub use error::{ParseError, RunnerError};
pub use parser::{parse, Command, File, Test};

#[cfg(feature = "fantoccini_backend")]
pub type Runner = runner::Runner<webdriver::fantoccini::Client>;

#[cfg(feature = "thirtyfour_backend")]
pub type Runner<'a> = runner::Runner<webdriver::thirtyfour::Client<'a>>;

#[cfg(feature = "thirtyfour_backend")]
impl<'a> Runner<'a> {
    /// Create a new runner
    pub fn new(client: &'a thirtyfour::WebDriver) -> Runner<'a> {
        Self::_new(webdriver::thirtyfour::Client(client))
    }
}

#[cfg(feature = "fantoccini_backend")]
impl Runner {
    /// Create a new runner
    pub fn new(client: fantoccini::Client) -> Runner {
        Self::_new(webdriver::fantoccini::Client(client))
    }
}
