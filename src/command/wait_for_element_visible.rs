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
impl<D: Webdriver> Command<D> for WaitForElementVisible {
    async fn run(&self, _: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        tokio::time::sleep(self.timeout).await;
        Ok(())
    }
}
