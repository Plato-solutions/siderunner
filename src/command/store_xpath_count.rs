use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Locator, Webdriver},
};

pub struct StoreXpathCount {
    xpath: String,
    variable: Option<String>,
}

impl StoreXpathCount {
    pub fn new(xpath: String, variable: Option<String>) -> Self {
        Self { xpath, variable }
    }
}

#[async_trait::async_trait]
impl Command for StoreXpathCount {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E> + Send,
        E: crate::webdriver::Element<Driver = D> + Send,
    {
        let locator = Locator::XPath(self.xpath.clone());
        let elements = runner.get_webdriver().find_all(locator).await?;
        if let Some(var) = &self.variable {
            runner.save_value(var.clone(), elements.len().into());
        }

        Ok(())
    }
}
