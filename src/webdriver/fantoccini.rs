// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::{Element as WebElement, Locator, Webdriver};
use fantoccini as fan;
use serde_json::Value as Json;
use std::time::Duration;

pub struct Client(pub fan::Client);

#[async_trait::async_trait]
impl Webdriver for Client {
    type Element = Element;
    type Error = crate::error::RunnerErrorKind;

    async fn goto(&mut self, url: &str) -> Result<(), Self::Error> {
        self.0.goto(url).await?;
        Ok(())
    }

    async fn find(&mut self, locator: Locator) -> Result<Self::Element, Self::Error> {
        let e = self.0.find((&locator).into()).await?;
        Ok(Element(e))
    }

    async fn find_all(&mut self, locator: Locator) -> Result<Vec<Self::Element>, Self::Error> {
        let elements = self
            .0
            .find_all((&locator).into())
            .await?
            .into_iter()
            .map(|e| Element(e))
            .collect();
        Ok(elements)
    }

    async fn wait_for_visible(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<Option<Duration>, Self::Error> {
        todo!()
    }

    async fn wait_for_not_present(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<Option<Duration>, Self::Error> {
        let locator = (&locator).into();

        let now = std::time::Instant::now();
        loop {
            match self.0.find(locator).await {
                Ok(..) => {} // TODO: sleep
                Err(fantoccini::error::CmdError::NoSuchElement(..)) => break Ok(None),
                Err(err) => Err(err)?,
            }

            if now.elapsed() > timeout {
                break Ok(Some(now.elapsed()));
            }
        }
        // std::thread::sleep_ms(4000);
    }

    async fn wait_for_present(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<Option<Duration>, Self::Error> {
        let locator = (&locator).into();

        let now = std::time::Instant::now();
        loop {
            match self.0.find(locator).await {
                Ok(..) => break Ok(None),
                Err(fantoccini::error::CmdError::NoSuchElement(..)) => (), // TODO: sleep
                Err(err) => Err(err)?,
            }

            if now.elapsed() > timeout {
                break Ok(Some(now.elapsed()));
            }
        }
    }

    async fn wait_for_editable(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<Option<Duration>, Self::Error> {
        // std::thread::sleep(*timeout);
        // std::thread::sleep_ms(4000);

        let locator = (&locator).into();
        let now = std::time::Instant::now();
        loop {
            match self.0.find(locator).await {
                Ok(mut element) => {
                    let is_displayed = match element.attr("style").await? {
                        Some(style) if !style.contains("display: none;") => true,
                        None => true,
                        _ => false,
                    };

                    let is_enabled = match element.attr("disabled").await? {
                        Some(..) => false,
                        _ => true,
                    };

                    if is_displayed && is_enabled {
                        break Ok(None);
                    }
                }
                Err(fantoccini::error::CmdError::NoSuchElement(..)) => {}
                Err(err) => Err(err)?,
            }

            if now.elapsed() > timeout {
                break Ok(Some(now.elapsed()));
            }
        }
        // TODO: ...
        // TODO: #issue https://github.com/jonhoo/fantoccini/issues/93
    }

    async fn current_url(&mut self) -> Result<url::Url, Self::Error> {
        let url = self.0.current_url().await?;
        Ok(url)
    }

    async fn set_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        self.0.set_window_size(width, height).await?;
        Ok(())
    }

    async fn execute(&mut self, script: &str, args: Vec<Json>) -> Result<Json, Self::Error> {
        let json = self.0.execute(script, args).await?;
        Ok(json)
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        self.0.close().await?;
        Ok(())
    }
}

pub struct Element(fan::Element);

#[async_trait::async_trait]
impl WebElement for Element {
    type Driver = Client;
    type Error = crate::error::RunnerErrorKind;

    async fn attr(&mut self, attribute: &str) -> Result<Option<String>, Self::Error> {
        let attr = self.0.attr(attribute).await?;
        Ok(attr)
    }

    async fn prop(&mut self, prop: &str) -> Result<Option<String>, Self::Error> {
        let prop = self.0.prop(prop).await?;
        Ok(prop)
    }

    async fn text(&mut self) -> Result<String, Self::Error> {
        let text = self.0.text().await?;
        Ok(text)
    }

    async fn html(&mut self, inner: bool) -> Result<String, Self::Error> {
        let html = self.0.html(inner).await?;
        Ok(html)
    }

    async fn find(&mut self, search: Locator) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let e = self.0.find((&search).into()).await?;
        Ok(Element(e))
    }

    async fn click(mut self) -> Result<Self::Driver, Self::Error> {
        let c = self.0.click().await?;
        Ok(Client(c))
    }

    async fn select_by_index(mut self, index: usize) -> Result<Self::Driver, Self::Error> {
        let c = self.0.select_by_index(index).await?;
        Ok(Client(c))
    }

    async fn select_by_value(mut self, value: &str) -> Result<Self::Driver, Self::Error> {
        let c = self.0.select_by_value(value).await?;
        Ok(Client(c))
    }
}

impl<'a> Into<fan::Locator<'a>> for &'a Locator {
    fn into(self) -> fan::Locator<'a> {
        match self {
            Locator::LinkText(s) => fan::Locator::LinkText(&s),
            Locator::Css(s) => fan::Locator::Css(&s),
            Locator::Id(s) => fan::Locator::Id(&s),
            Locator::XPath(s) => fan::Locator::XPath(&s),
        }
    }
}
