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
impl<D: Webdriver> Command<D> for Echo {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let message = runner.emit(&self.message);
        runner.echo(&message);

        Ok(())
    }
}
