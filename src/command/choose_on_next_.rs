// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct ChooseCancelOnNextConfirmation;

#[async_trait::async_trait]
impl Command for ChooseCancelOnNextConfirmation {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        crate::js_lib::choose_cancel_on_next_confirmation(runner).await
    }
}

pub struct ChooseOkOnNextConfirmation;

#[async_trait::async_trait]
impl Command for ChooseOkOnNextConfirmation {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        crate::js_lib::choose_ok_on_next_confirmation(runner).await
    }
}

pub struct ChooseCancelOnNextPrompt;

#[async_trait::async_trait]
impl Command for ChooseCancelOnNextPrompt {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        crate::js_lib::choose_cancel_on_next_prompt(runner).await
    }
}
