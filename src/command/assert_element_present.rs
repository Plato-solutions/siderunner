// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct AssertElementPresent {
    target: Locator,
}

impl AssertElementPresent {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for AssertElementPresent {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let is_present = runner
            .get_webdriver()
            .find(self.target.clone())
            .await?
            .is_present()
            .await?;

        if !is_present {
            return Err(RunnerErrorKind::AssertFailed {
                lhs: "false".to_string(),
                rhs: "true".to_string(),
            });
        }

        Ok(())
    }
}

pub struct AssertElementNotPresent {
    target: Locator,
}

impl AssertElementNotPresent {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for AssertElementNotPresent {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let is_present = runner
            .get_webdriver()
            .find(self.target.clone())
            .await?
            .is_present()
            .await?;

        if is_present {
            return Err(RunnerErrorKind::AssertFailed {
                lhs: "true".to_string(),
                rhs: "false".to_string(),
            });
        }

        Ok(())
    }
}
