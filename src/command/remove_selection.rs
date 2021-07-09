// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct RemoveSelection {
    target: Locator,
    label: String,
}

impl RemoveSelection {
    pub fn new(target: Locator, label: String) -> Self {
        Self { target, label }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for RemoveSelection {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let mut select = runner.get_webdriver().find(self.target.clone()).await?;

        let label = runner.emit(&self.label);
        // somehow .//option[normalize-space(.)='{}'] doesn work...
        let locator = format!(".//*[normalize-space(.)='{}']", label);

        let mut opt = select.find(Locator::XPath(locator)).await?;
        if opt.is_selected().await? {
            opt.click().await?;
        }

        Ok(())
    }
}
