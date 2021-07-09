// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct Check {
    target: Locator,
}

impl Check {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for Check {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let mut e = runner.get_webdriver().find(self.target.clone()).await?;
        let selected = e.prop("selected").await?;

        if selected.is_none() {
            e.click().await?;
        }

        Ok(())
    }
}

pub struct UnCheck {
    target: Locator,
}

impl UnCheck {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for UnCheck {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let mut e = runner.get_webdriver().find(self.target.clone()).await?;
        let selected = e.prop("selected").await?;

        if selected.is_some() {
            e.click().await?;
        }

        Ok(())
    }
}
