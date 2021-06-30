use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct Close;

#[async_trait::async_trait]
impl Command for Close {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        runner.get_webdriver().close().await?;
        Ok(())
    }
}
