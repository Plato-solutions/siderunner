// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct AssertElementEditable {
    target: Locator,
}

impl AssertElementEditable {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl Command for AssertElementEditable {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        let mut el = runner.get_webdriver().find(self.target.clone()).await?;

        let err = Err(RunnerErrorKind::AssertFailed {
            lhs: "false".to_string(),
            rhs: "true".to_string(),
        });
        if !el.is_enabled().await? {
            return err;
        }

        let is_readonly = el.attr("readonly").await?;
        match is_readonly.as_deref() {
            Some("false") | None => err,
            _ => Ok(()),
        }
    }
}

pub struct AssertElementNotEditable {
    target: Locator,
}

impl AssertElementNotEditable {
    pub fn new(target: Locator) -> Self {
        Self { target }
    }
}

#[async_trait::async_trait]
impl Command for AssertElementNotEditable {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        match AssertElementEditable::new(self.target.clone())
            .run(runner)
            .await
        {
            Ok(()) => Err(RunnerErrorKind::AssertFailed {
                lhs: "true".to_string(),
                rhs: "false".to_string(),
            }),
            Err(RunnerErrorKind::AssertFailed { .. }) => Ok(()),
            result => result,
        }
    }
}
