use std::time::Duration;

use crate::{
    error::RunnerErrorKind,
    webdriver::{Locator, Webdriver},
};

use super::Command;

#[allow(dead_code)]
pub struct WaitForElementVisible {
    target: Locator,
    timeout: Duration,
}

impl WaitForElementVisible {
    pub fn new(target: Locator, timeout: Duration) -> Self {
        Self { target, timeout }
    }
}

#[async_trait::async_trait]
impl Command for WaitForElementVisible {
    async fn run<D, E>(&self, _: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        tokio::time::sleep(self.timeout).await;
        Ok(())
    }
}
