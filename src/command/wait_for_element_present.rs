// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::time::Duration;

use crate::{
    error::RunnerErrorKind,
    webdriver::{Locator, Webdriver},
};

use super::Command;

pub struct WaitForElementPresent {
    target: Locator,
    timeout: Duration,
}

impl WaitForElementPresent {
    pub fn new(target: Locator, timeout: Duration) -> Self {
        Self { target, timeout }
    }
}

#[async_trait::async_trait]
impl Command for WaitForElementPresent {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner
            .get_webdriver()
            .wait_for_present(self.target.clone(), self.timeout)
            .await
            .map_err(|_| RunnerErrorKind::Timeout("WaitForElementPresent".to_owned()))?;

        Ok(())
    }
}

pub struct WaitForElementNotPresent {
    target: Locator,
    timeout: Duration,
}

impl WaitForElementNotPresent {
    pub fn new(target: Locator, timeout: Duration) -> Self {
        Self { target, timeout }
    }
}

#[async_trait::async_trait]
impl Command for WaitForElementNotPresent {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner
            .get_webdriver()
            .wait_for_not_present(self.target.clone(), self.timeout)
            .await
            .map_err(|_| RunnerErrorKind::Timeout("WaitForElementPresent".to_owned()))?;

        Ok(())
    }
}
