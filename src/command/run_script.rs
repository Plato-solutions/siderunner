// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::{execute::Execute, Command};
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct RunScript {
    script: String,
}

impl RunScript {
    pub fn new(script: String) -> Self {
        Self { script }
    }
}

#[async_trait::async_trait]
impl Command for RunScript {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        // Acording to Selenium specification we would have to instrument a script block in HTML,
        // but from what I see in there code base they don't follow there own spec?
        // https://github.com/SeleniumHQ/selenium/issues/9583
        Execute::new(self.script.clone(), None).run(runner).await
    }
}
