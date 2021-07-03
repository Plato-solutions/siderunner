use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
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
impl<D: Webdriver> Command<D> for Click {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        runner
            .get_webdriver()
            .find(self.target.clone())
            .await?
            .click()
            .await?;

        Ok(())
    }
}
