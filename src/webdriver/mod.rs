// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde_json::Value as Json;
use std::time::Duration;

use crate::error::RunnerErrorKind;

pub(crate) mod fantoccini;
pub(crate) mod thirtyfour;

/// Webdriver an interface over a ebdriver functionality.
///
/// Mainly created for test purpouses and to be able to support 2 backends.
#[async_trait::async_trait]
pub trait Webdriver: Send {
    type Element: Element<Driver = Self>;

    async fn goto(&mut self, url: &str) -> Result<(), RunnerErrorKind>;
    async fn find(&mut self, locator: Locator) -> Result<Self::Element, RunnerErrorKind>;
    async fn find_all(&mut self, locator: Locator) -> Result<Vec<Self::Element>, RunnerErrorKind>;
    async fn current_url(&mut self) -> Result<url::Url, RunnerErrorKind>;
    async fn wait_for_visible(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind>;
    async fn wait_for_not_visible(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind>;
    async fn wait_for_present(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind>;
    async fn wait_for_not_present(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind>;
    async fn wait_for_editable(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind>;
    async fn wait_for_not_editable(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind>;
    async fn set_window_size(&mut self, width: u32, height: u32) -> Result<(), RunnerErrorKind>;
    async fn execute(&mut self, script: &str, mut args: Vec<Json>)
        -> Result<Json, RunnerErrorKind>;
    async fn execute_async(
        &mut self,
        script: &str,
        mut args: Vec<Json>,
    ) -> Result<Json, RunnerErrorKind>;
    async fn close(&mut self) -> Result<(), RunnerErrorKind>;
    async fn alert_text(&mut self) -> Result<String, RunnerErrorKind>;
    async fn alert_accept(&mut self) -> Result<(), RunnerErrorKind>;
    async fn alert_dissmis(&mut self) -> Result<(), RunnerErrorKind>;
    async fn double_click(&mut self, locator: Locator) -> Result<(), RunnerErrorKind>;
    async fn mouse_down(&mut self, locator: Locator) -> Result<(), RunnerErrorKind>;
    async fn mouse_up(&mut self, locator: Locator) -> Result<(), RunnerErrorKind>;
    async fn title(&mut self) -> Result<String, RunnerErrorKind>;
    async fn click_at(
        &mut self,
        locator: Locator,
        coord: (i32, i32),
    ) -> Result<(), RunnerErrorKind>;
    async fn double_click_at(
        &mut self,
        locator: Locator,
        coord: (i32, i32),
    ) -> Result<(), RunnerErrorKind>;
}

/// Element represents functionality which may be taken agains a WebElement by means of Webdriver.
#[async_trait::async_trait]
pub trait Element: Send {
    type Driver;

    async fn attr(&mut self, attribute: &str) -> Result<Option<String>, RunnerErrorKind>;
    async fn prop(&mut self, prop: &str) -> Result<Option<String>, RunnerErrorKind>;
    async fn text(&mut self) -> Result<String, RunnerErrorKind>;
    async fn html(&mut self, inner: bool) -> Result<String, RunnerErrorKind>;
    async fn find(&mut self, search: Locator) -> Result<Self, RunnerErrorKind>
    where
        Self: Sized;
    async fn click(mut self) -> Result<Self::Driver, RunnerErrorKind>;
    async fn select_by_index(mut self, index: usize) -> Result<Self::Driver, RunnerErrorKind>;
    async fn select_by_value(mut self, value: &str) -> Result<Self::Driver, RunnerErrorKind>;
    async fn select_by_label(mut self, value: &str) -> Result<Self::Driver, RunnerErrorKind>;
    async fn send_keys(mut self, value: &str) -> Result<(), RunnerErrorKind>;
    async fn is_selected(&mut self) -> Result<bool, RunnerErrorKind>;
    async fn is_present(&mut self) -> Result<bool, RunnerErrorKind>;
    async fn is_enabled(&mut self) -> Result<bool, RunnerErrorKind>;
}

/// Locator represents a way how to find a particular web element.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Locator {
    Css(String),
    Id(String),
    LinkText(String),
    XPath(String),
}
