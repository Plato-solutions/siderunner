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

pub type Runner = runner::Runner<webdriver::fantoccini::Client>;

impl Runner {
    /// Create a new runner
    pub fn new(client: &fantoccini::Client) -> Runner {
        Self::_new(webdriver::fantoccini::Client(client.clone()))
    }
}
