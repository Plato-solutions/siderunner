// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};
use serde_json::Value;

pub struct StoreAttribute {
    target: Locator,
    attribute: String,
    variable: String,
}

impl StoreAttribute {
    pub fn new(target: Locator, attribute: String, variable: String) -> Self {
        Self {
            target,
            attribute,
            variable,
        }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for StoreAttribute {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let value = runner
            .get_webdriver()
            .find(self.target.clone())
            .await?
            .attr(&self.attribute)
            .await?
            .unwrap_or_else(String::new);

        runner.save_value(self.variable.clone(), Value::String(value));

        Ok(())
    }
}
