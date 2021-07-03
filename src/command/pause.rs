use std::time::Duration;

use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct Pause {
    timeout: Duration,
}

impl Pause {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

#[async_trait::async_trait]
impl Command for Pause {
    async fn run<D>(&self, _: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        tokio::time::sleep(self.timeout).await;
        Ok(())
    }
}
