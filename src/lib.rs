// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A crate provides interface for parsing `side` files and running them.
//!
//! # Example
//! ```
//! use siderunner::{parse, Runner};
//! use thirtyfour::{DesiredCapabilities, WebDriver};
//!
//! #[tokio::main]
//! async fn main() {
//!     let wiki = std::fs::File::open("examples/wiki.side").expect("Can't open a side file");
//!     let file = parse(wiki).expect("Failed parsing a file");
//!
//!     let client = WebDriver::new("http://localhost:4444", DesiredCapabilities::chrome())
//!         .await
//!         .expect("can't connect to webdriver");
//!     let mut runner = Runner::new(&client);
//!     runner.run_test(&file, 0).await.expect("Fail in running first test");
//!
//!     assert_eq!(
//!         runner.get_data().get("slogan"),
//!         Some(&serde_json::json!("The Free Encyclopedia")),
//!     );
//!
//!     runner.close().await.expect("Error occured while closing webdriver");
//! }
//! ```

mod command;
mod error;
mod parser;
mod runner;
#[cfg(test)]
mod playground_test;
mod validation;
mod webdriver;
mod playground;

pub use error::{ParseError, RunnerError};
pub use parser::{parse, Command, File, Test};

/// Runner responsible for running a [`Test`](./struct.Test.html)
/// and collecting data.
#[cfg(feature = "fantoccini_backend")]
pub type Runner = runner::Runner<webdriver::fantoccini::Client>;

/// Runner responsible for running a [`Test`](./struct.Test.html)
/// and collecting data.
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
