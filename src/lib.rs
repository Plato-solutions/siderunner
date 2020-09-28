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
