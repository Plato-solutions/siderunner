// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct AssertChecked {
    target: Locator,
}

impl AssertChecked {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl Command for AssertChecked {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        let mut element = runner.get_webdriver().find(self.target.clone()).await?;
        let checked = element.prop("checked").await?;
        match checked {
            Some(s) if s == "true" => Ok(()),
            _ => Err(RunnerErrorKind::AssertFailed {
                lhs: "Checked".to_owned(),
                rhs: "Not checked".to_owned(),
            }),
        }
    }
}

pub struct AssertNotChecked {
    target: Locator,
}

impl AssertNotChecked {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl Command for AssertNotChecked {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        let mut element = runner.get_webdriver().find(self.target.clone()).await?;
        let checked = element.prop("checked").await?;
        match checked {
            Some(s) if s == "true" => Err(RunnerErrorKind::AssertFailed {
                lhs: "Not checked".to_owned(),
                rhs: "Checked".to_owned(),
            }),
            _ => Ok(()),
        }
    }
}
