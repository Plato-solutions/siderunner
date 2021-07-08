// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct AssertValue {
    target: Locator,
    value: String,
}

impl AssertValue {
    pub fn new(target: Locator, value: String) -> Self {
        Self { target, value }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for AssertValue {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let mut element = runner.get_webdriver().find(self.target.clone()).await?;
        let value = element
            .prop("value")
            .await?
            .unwrap_or_else(|| "".to_owned());
        if value == self.value {
            Ok(())
        } else {
            Err(RunnerErrorKind::AssertFailed {
                lhs: value,
                rhs: self.value.clone(),
            })
        }
    }
}
