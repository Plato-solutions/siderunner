// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct AssertSelectedValue {
    target: Locator,
    text: String,
}

impl AssertSelectedValue {
    pub fn new(target: Locator, text: String) -> Self {
        Self { target, text }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for AssertSelectedValue {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let mut el = runner.get_webdriver().find(self.target.clone()).await?;

        let value = el.prop("value").await?.unwrap_or_else(String::new);

        if value == self.text {
            Ok(())
        } else {
            Err(RunnerErrorKind::AssertFailed {
                lhs: value,
                rhs: self.text.clone(),
            })
        }
    }
}
