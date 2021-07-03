use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

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
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner.save_value(self.variable.clone(), self.value.clone().into());
        Ok(())
    }
}
