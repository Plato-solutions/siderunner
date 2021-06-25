use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
    File, Runner,
};
use serde_json::Value;

pub struct Store {
    variable: String,
    value: String,
}

impl Store {
    pub fn new(variable: String, value: String) -> Self {
        Self { variable, value }
    }
}

#[async_trait::async_trait]
impl Command for Store {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        runner.save_value(self.variable.clone(), self.value.clone().into());
        Ok(())
    }
}
