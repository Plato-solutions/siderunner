// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::time::Duration;

use crate::{
    error::RunnerErrorKind,
    webdriver::{Locator, Webdriver},
};

use super::Command;

#[allow(dead_code)]
pub struct WaitForElementVisible {
    target: Locator,
    timeout: Duration,
}

impl WaitForElementVisible {
    pub fn new(target: Locator, timeout: Duration) -> Self {
        Self { target, timeout }
    }
}

#[async_trait::async_trait]
impl Command for WaitForElementVisible {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner
            .get_webdriver()
            .wait_for_visible(self.target.clone(), self.timeout)
            .await?;
        Ok(())
    }
}
pub struct WaitForElementNotVisible {
    target: Locator,
    timeout: Duration,
}

impl WaitForElementNotVisible {
    pub fn new(target: Locator, timeout: Duration) -> Self {
        Self { target, timeout }
    }
}

#[async_trait::async_trait]
impl Command for WaitForElementNotVisible {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner
            .get_webdriver()
            .wait_for_not_visible(self.target.clone(), self.timeout)
            .await?;
        Ok(())
    }
}
