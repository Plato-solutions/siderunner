// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde_json::Value as Json;
use std::time::Duration;

pub(crate) mod fantoccini;
pub(crate) mod thirtyfour;

/// Webdriver an interface over a ebdriver functionality.
///
/// Mainly created for test purpouses and to be able to support 2 backends.
#[async_trait::async_trait]
pub trait Webdriver {
    type Element;
    type Error;

    async fn goto(&mut self, url: &str) -> Result<(), Self::Error>;
    async fn find(&mut self, locator: Locator) -> Result<Self::Element, Self::Error>;
    async fn find_all(&mut self, locator: Locator) -> Result<Vec<Self::Element>, Self::Error>;
    async fn current_url(&mut self) -> Result<url::Url, Self::Error>;
    async fn wait_for_visible(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), Self::Error>;
    async fn wait_for_not_present(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), Self::Error>;
    async fn wait_for_present(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), Self::Error>;
    async fn wait_for_editable(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), Self::Error>;
    async fn set_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error>;
    async fn execute(&mut self, script: &str, mut args: Vec<Json>) -> Result<Json, Self::Error>;
    async fn execute_async(&mut self, script: &str, mut args: Vec<Json>) -> Result<Json, Self::Error>;
    async fn close(&mut self) -> Result<(), Self::Error>;
}

/// Element represents functionality which may be taken agains a WebElement by means of Webdriver.
#[async_trait::async_trait]
pub trait Element {
    type Driver;
    type Error;

    async fn attr(&mut self, attribute: &str) -> Result<Option<String>, Self::Error>;
    async fn prop(&mut self, prop: &str) -> Result<Option<String>, Self::Error>;
    async fn text(&mut self) -> Result<String, Self::Error>;
    async fn html(&mut self, inner: bool) -> Result<String, Self::Error>;
    async fn find(&mut self, search: Locator) -> Result<Self, Self::Error>
    where
        Self: Sized;
    async fn click(mut self) -> Result<Self::Driver, Self::Error>;
    async fn select_by_index(mut self, index: usize) -> Result<Self::Driver, Self::Error>;
    async fn select_by_value(mut self, value: &str) -> Result<Self::Driver, Self::Error>;
}

/// Locator represents a way how to find a particular web element.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Locator {
    Css(String),
    Id(String),
    LinkText(String),
    XPath(String),
}
