// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Locator, Webdriver},
};

pub struct MouseUp {
    target: Locator,
}

impl MouseUp {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for MouseUp {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        runner.get_webdriver().mouse_up(self.target.clone()).await?;
        Ok(())
    }
}

pub struct MouseDown {
    target: Locator,
}

impl MouseDown {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for MouseDown {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        runner
            .get_webdriver()
            .mouse_down(self.target.clone())
            .await?;
        Ok(())
    }
}
