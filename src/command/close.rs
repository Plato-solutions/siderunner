use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct Close;

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for Close {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        runner.get_webdriver().close().await?;
        Ok(())
    }
}
