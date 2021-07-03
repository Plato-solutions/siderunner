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
impl<D: Webdriver> Command<D> for ExecuteAsync {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let res = runner.exec_async(&self.script).await?;
        if let Some(var) = self.variable.as_ref() {
            runner.save_value(var.clone(), res);
        }

        Ok(())
    }
}
