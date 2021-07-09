// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct StoreJson {
    value: String,
    variable: String,
}

impl StoreJson {
    pub fn new(value: String, variable: String) -> Self {
        Self { value, variable }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for StoreJson {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let value = runner.emit(&self.value);
        let value = serde_json::from_str(&value).map_err(|_| {
            RunnerErrorKind::MismatchedType("Unexpected type of json object".to_string())
        })?;
        runner.save_value(self.variable.clone(), value);
        Ok(())
    }
}
