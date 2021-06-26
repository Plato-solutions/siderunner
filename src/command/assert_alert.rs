use super::Command;
use crate::{
    error::RunnerErrorKind,
    parser::SelectLocator,
    webdriver::{Element, Locator, Webdriver},
    File, Runner,
};
use serde_json::Value;
use std::time::Duration;

pub struct AssertAlert {
    text: String,
}

impl AssertAlert {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

#[async_trait::async_trait]
impl Command for AssertAlert {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        let alert = runner.get_webdriver().alert_text().await?;

        if alert == self.text {
            Ok(())
        } else {
            Err(RunnerErrorKind::AssertFailed {
                lhs: alert,
                rhs: self.text.clone(),
            })
        }
    }
}
