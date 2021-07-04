use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct AssertText {
    target: Locator,
    text: String,
}

impl AssertText {
    pub fn new(target: Locator, text: String) -> Self {
        Self { target, text }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for AssertText {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let mut element = runner.get_webdriver().find(self.target.clone()).await?;
        let element_text = element.text().await?;
        if element_text == self.text {
            Ok(())
        } else {
            Err(RunnerErrorKind::AssertFailed {
                lhs: element_text,
                rhs: self.text.clone(),
            })
        }
    }
}

pub struct AssertNotText {
    target: Locator,
    text: String,
}

impl AssertNotText {
    pub fn new(target: Locator, text: String) -> Self {
        Self { target, text }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for AssertNotText {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let mut element = runner.get_webdriver().find(self.target.clone()).await?;
        let element_text = element.text().await?;
        if element_text != self.text {
            Ok(())
        } else {
            Err(RunnerErrorKind::AssertFailed {
                lhs: element_text,
                rhs: self.text.clone(),
            })
        }
    }
}
