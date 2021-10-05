// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct SendKeys {
    target: Locator,
    text: String,
}

impl SendKeys {
    pub fn new(target: Locator, text: String) -> Self {
        Self { target, text }
    }
}

#[async_trait::async_trait]
impl Command for SendKeys {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        // todo: add support for a KEY_STROKES like KEY_ENTER
        let element = runner.get_webdriver().find(self.target.clone()).await?;
        element.send_keys(&self.text).await?;
        Ok(())
    }
}
