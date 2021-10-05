// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct SetWindowSize {
    width: u32,
    heigth: u32,
}

impl SetWindowSize {
    pub fn new(width: u32, heigth: u32) -> Self {
        Self { width, heigth }
    }
}

#[async_trait::async_trait]
impl Command for SetWindowSize {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner
            .get_webdriver()
            .set_window_size(self.width, self.heigth)
            .await?;

        Ok(())
    }
}
