use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
    File, Runner,
};
use serde_json::Value;

pub struct Echo {
    message: String,
}

impl Echo {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

#[async_trait::async_trait]
impl Command for Echo {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        let message = runner.emit(&self.message);
        runner.echo(&message);

        Ok(())
    }
}
