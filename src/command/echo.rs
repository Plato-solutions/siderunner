use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

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
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        let message = runner.emit(&self.message);
        runner.echo(&message);

        Ok(())
    }
}
