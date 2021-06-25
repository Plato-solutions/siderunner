use super::Command;
use crate::{
    error::RunnerErrorKind,
    parser::SelectLocator,
    webdriver::{Element, Locator, Webdriver},
    File, Runner,
};
use serde_json::Value;
use std::time::Duration;

pub struct Assert {
    var: String,
    value: String,
}

impl Assert {
    pub fn new(variable: String, value: String) -> Self {
        Self {
            var: variable,
            value,
        }
    }
}

#[async_trait::async_trait]
impl Command for Assert {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        // NOTION: intentially don't use a print_plain_value even though SELENIUM IDE uses this approach
        let var = runner.get_value(&self.var).map_or_else(
            || "undefined".to_string(),
            |v| v.to_string().trim_matches('\"').to_string(),
        );

        if var != self.value {
            return Err(RunnerErrorKind::AssertFailed {
                lhs: var,
                rhs: self.value.clone(),
            });
        }

        Ok(())
    }
}
