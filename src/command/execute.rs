use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
    File, Runner,
};
use serde_json::Value;

pub struct Execute {
    variable: Option<String>,
    script: String,
}

impl Execute {
    pub fn new(script: String, var: Option<String>) -> Self {
        Self {
            script,
            variable: var,
        }
    }
}

#[async_trait::async_trait]
impl Command for Execute {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        let res = runner.exec(&self.script).await?;
        if let Some(var) = self.variable.as_ref() {
            runner.save_value(var.clone(), res);
        }

        Ok(())
    }
}
