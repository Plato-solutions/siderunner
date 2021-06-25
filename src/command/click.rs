use super::Command;
use crate::{
    error::RunnerErrorKind,
    parser::SelectLocator,
    webdriver::{Element, Locator, Webdriver},
    File, Runner,
};

pub struct Click {
    target: Locator,
}

impl Click {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl Command for Click {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        runner
            .get_webdriver()
            .find(self.target.clone())
            .await?
            .click()
            .await?;

        Ok(())
    }
}
