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
impl<D: Webdriver> Command<D> for Pause {
    async fn run(&self, _: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        tokio::time::sleep(self.timeout).await;
        Ok(())
    }
}
