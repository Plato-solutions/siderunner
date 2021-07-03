use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct Close;

#[async_trait::async_trait]
impl Command for Close {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner.get_webdriver().close().await?;
        Ok(())
    }
}
