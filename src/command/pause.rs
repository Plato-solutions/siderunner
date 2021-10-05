// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::time::Duration;

use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct Pause {
    timeout: Duration,
}

impl Pause {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

#[async_trait::async_trait]
impl Command for Pause {
    async fn run<D>(&self, _: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        futures_timer::Delay::new(self.timeout).await;
        Ok(())
    }
}
