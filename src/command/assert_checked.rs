use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Locator, Webdriver},
};

pub struct AssertChecked {
    target: Locator,
}

impl AssertChecked {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl Command for AssertChecked {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        let mut element = runner.get_webdriver().find(self.target.clone()).await?;
        let checked = element.prop("checked").await?;
        match checked {
            Some(s) if s == "true" => Ok(()),
            _ => Err(RunnerErrorKind::AssertFailed {
                lhs: "Checked".to_owned(),
                rhs: "Not checked".to_owned(),
            }),
        }
    }
}
