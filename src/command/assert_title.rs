// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct AssertTitle {
    text: String,
}

impl AssertTitle {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

#[async_trait::async_trait]
impl Command for AssertTitle {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        let title = runner.get_webdriver().title().await?;
        if title == self.text {
            Ok(())
        } else {
            Err(RunnerErrorKind::AssertFailed {
                lhs: title,
                rhs: self.text.clone(),
            })
        }
    }
}
