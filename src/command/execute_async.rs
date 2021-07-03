use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct ExecuteAsync {
    script: String,
    variable: Option<String>,
}

impl ExecuteAsync {
    pub fn new(script: String, variable: Option<String>) -> Self {
        Self { script, variable }
    }
}

#[async_trait::async_trait]
impl Command for ExecuteAsync {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        let res = runner.exec_async(&self.script).await?;
        if let Some(var) = self.variable.as_ref() {
            runner.save_value(var.clone(), res);
        }

        Ok(())
    }
}
