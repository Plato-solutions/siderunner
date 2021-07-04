use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct EditContent {
    target: Locator,
    text: String,
}

impl EditContent {
    pub fn new(target: Locator, text: String) -> Self {
        Self { target, text }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for EditContent {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let mut element = runner.get_webdriver().find(self.target.clone()).await?;
        element.send_keys(&self.text).await?;
        Ok(())
    }
}
