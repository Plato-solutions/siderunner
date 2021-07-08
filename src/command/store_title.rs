// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};
use serde_json::Value;

pub struct StoreTitle {
    variable: String,
}

impl StoreTitle {
    pub fn new(variable: String) -> Self {
        Self { variable }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for StoreTitle {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let value = runner.get_webdriver().title().await?;

        runner.save_value(self.variable.clone(), Value::String(value));

        Ok(())
    }
}
