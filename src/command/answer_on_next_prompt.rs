// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{error::RunnerErrorKind, js_lib, webdriver::Webdriver};

pub struct AnswerOnNextPrompt {
    answer: String,
}

impl AnswerOnNextPrompt {
    pub fn new(answer: String) -> Self {
        Self { answer }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for AnswerOnNextPrompt {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        js_lib::answer_on_next_prompt(runner, &self.answer).await
    }
}
