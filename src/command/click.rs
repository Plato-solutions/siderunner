// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct Click {
    target: Locator,
}

impl Click {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl Command for Click {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner
            .get_webdriver()
            .find(self.target.clone())
            .await?
            .click()
            .await?;

        Ok(())
    }
}

pub struct ClickAt {
    target: Locator,
    coord: (i32, i32),
}

impl ClickAt {
    pub fn new(target: Locator, coord: (i32, i32)) -> Self {
        Self { target, coord }
    }
}

#[async_trait::async_trait]
impl Command for ClickAt {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner
            .get_webdriver()
            .click_at(self.target.clone(), self.coord)
            .await?;

        Ok(())
    }
}
