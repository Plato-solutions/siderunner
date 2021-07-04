// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct Execute {
    script: String,
    variable: Option<String>,
}

impl Execute {
    pub fn new(script: String, var: Option<String>) -> Self {
        Self {
            script,
            variable: var,
        }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for Execute {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let res = runner.exec(&self.script).await?;
        if let Some(var) = self.variable.as_ref() {
            runner.save_value(var.clone(), res);
        }

        Ok(())
    }
}
