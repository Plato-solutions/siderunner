// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::Command;
use crate::{
    error::RunnerErrorKind,
    webdriver::{Element, Locator, Webdriver},
};

pub struct AssertSelectedLabel {
    target: Locator,
    text: String,
}

impl AssertSelectedLabel {
    pub fn new(target: Locator, text: String) -> Self {
        Self { target, text }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for AssertSelectedLabel {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let mut el = runner.get_webdriver().find(self.target.clone()).await?;
        match el.prop("selectedIndex").await? {
            Some(index) => {
                let index: usize = index.parse().map_err(|_| {
                    RunnerErrorKind::MismatchedType("Unexpected type of selectedIndex".to_owned())
                })?;
                let option_label = el
                    .find(Locator::Css(format!("option:nth-child({})", index + 1)))
                    .await?
                    .text()
                    .await?;

                if option_label != self.text {
                    Err(RunnerErrorKind::AssertFailed {
                        lhs: option_label,
                        rhs: self.text.clone(),
                    })
                } else {
                    Ok(())
                }
            }
            None => Err(RunnerErrorKind::AssertFailed {
                lhs: "".to_owned(),
                rhs: self.text.clone(),
            }),
        }
    }
}
