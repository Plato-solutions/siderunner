// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg(feature = "thirtyfour_backend")]

use super::{Element, Locator, Webdriver};
use crate::error::RunnerErrorKind;
use serde_json::Value as Json;
use std::time::Duration;
use thirtyfour::{
    components::select::SelectElement, prelude::ElementQueryable, By, OptionRect, ScriptArgs,
    WebDriverCommands,
};
use url::Url;

/// Thirtyfour Webdriver interface
pub struct Client<'a>(pub &'a thirtyfour::WebDriver);

#[async_trait::async_trait]
impl<'a> Webdriver for Client<'a> {
    type Element = WebElement<'a>;

    async fn goto(&mut self, url: &str) -> Result<(), RunnerErrorKind> {
        self.0.get(url).await?;
        Ok(())
    }

    async fn find(&mut self, locator: Locator) -> Result<Self::Element, RunnerErrorKind> {
        let e = self.0.find_element((&locator).into()).await?;
        Ok(WebElement(e, &self.0))
    }

    async fn find_all(&mut self, locator: Locator) -> Result<Vec<Self::Element>, RunnerErrorKind> {
        let elements = self
            .0
            .find_elements((&locator).into())
            .await?
            .into_iter()
            .map(move |e| WebElement(e, &self.0))
            .collect();
        Ok(elements)
    }

    async fn wait_for_visible(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind> {
        let locator: By = (&locator).into();
        self.0
            .query(locator)
            .and_displayed()
            .wait(timeout, timeout / 3)
            .first()
            .await?;

        Ok(())
    }

    async fn wait_for_not_visible(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind> {
        let locator: By = (&locator).into();
        self.0
            .query(locator)
            .and_not_displayed()
            .wait(timeout, timeout / 3)
            .first()
            .await?;

        Ok(())
    }

    async fn wait_for_not_present(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind> {
        let locator: By = (&locator).into();
        self.0
            .query(locator)
            .wait(timeout, timeout / 3)
            .not_exists()
            .await?;

        Ok(())
    }

    async fn wait_for_present(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind> {
        let locator: By = (&locator).into();
        self.0
            .query(locator)
            .wait(timeout, timeout / 3)
            .exists()
            .await?;

        Ok(())
    }

    async fn wait_for_editable(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind> {
        let locator: By = (&locator).into();
        self.0
            .query(locator)
            .wait(timeout, timeout / 3)
            .and_enabled()
            .with_attribute("readonly", "false") // todo: change to "" || "false" after PR
            .first()
            .await?;

        Ok(())
    }

    async fn wait_for_not_editable(
        &mut self,
        locator: Locator,
        timeout: Duration,
    ) -> Result<(), RunnerErrorKind> {
        self.0
            .query((&locator).into())
            .wait(timeout, timeout / 3)
            .and_not_enabled()
            .with_attribute("readonly", "true")
            .first()
            .await?;

        Ok(())
    }

    async fn current_url(&mut self) -> Result<Url, RunnerErrorKind> {
        let url = self.0.current_url().await?;
        Ok(Url::parse(&url).unwrap())
    }

    async fn set_window_size(&mut self, width: u32, height: u32) -> Result<(), RunnerErrorKind> {
        self.0
            .set_window_rect(OptionRect::new().with_size(width as i32, height as i32))
            .await?;
        Ok(())
    }

    async fn execute(&mut self, script: &str, a: Vec<Json>) -> Result<Json, RunnerErrorKind> {
        let mut args = ScriptArgs::new();
        for v in a {
            args.push_value(v);
        }

        let ret = self.0.execute_script_with_args(script, &args).await?;
        let json = ret.value().clone();

        Ok(json)
    }

    async fn execute_async(&mut self, script: &str, a: Vec<Json>) -> Result<Json, RunnerErrorKind> {
        let mut args = ScriptArgs::new();
        for v in a {
            args.push_value(v);
        }

        let ret = self.0.execute_async_script_with_args(script, &args).await?;
        let json = ret.value().clone();

        Ok(json)
    }

    async fn close(&mut self) -> Result<(), RunnerErrorKind> {
        self.0.close().await?;
        Ok(())
    }

    async fn alert_text(&mut self) -> Result<String, RunnerErrorKind> {
        let text = self.0.switch_to().alert().text().await?;
        Ok(text)
    }

    async fn alert_accept(&mut self) -> Result<(), RunnerErrorKind> {
        self.0.switch_to().alert().accept().await?;
        Ok(())
    }

    async fn alert_dissmis(&mut self) -> Result<(), RunnerErrorKind> {
        self.0.switch_to().alert().dismiss().await?;
        Ok(())
    }

    async fn double_click(&mut self, locator: Locator) -> Result<(), RunnerErrorKind> {
        let by: By = (&locator).into();
        let el = self.0.find_element(by).await?;
        self.0
            .action_chain()
            .move_to_element_center(&el)
            .double_click()
            .perform()
            .await?;

        Ok(())
    }

    async fn mouse_down(&mut self, locator: Locator) -> Result<(), RunnerErrorKind> {
        let by: By = (&locator).into();
        let el = self.0.find_element(by).await?;
        self.0
            .action_chain()
            .move_to_element_center(&el)
            .click_and_hold()
            .perform()
            .await?;

        Ok(())
    }

    async fn mouse_up(&mut self, locator: Locator) -> Result<(), RunnerErrorKind> {
        let by: By = (&locator).into();
        let el = self.0.find_element(by).await?;
        self.0
            .action_chain()
            .move_to_element_center(&el)
            .release()
            .perform()
            .await?;

        Ok(())
    }

    async fn title(&mut self) -> Result<String, RunnerErrorKind> {
        let title = self.0.title().await?;
        Ok(title)
    }

    async fn click_at(
        &mut self,
        locator: Locator,
        coord: (i32, i32),
    ) -> Result<(), RunnerErrorKind> {
        let by: By = (&locator).into();
        let el = self.0.find_element(by).await?;
        self.0
            .action_chain()
            .move_to_element_center(&el)
            .move_by_offset(coord.0, coord.1)
            .click()
            .perform()
            .await?;

        Ok(())
    }

    async fn double_click_at(
        &mut self,
        locator: Locator,
        coord: (i32, i32),
    ) -> Result<(), RunnerErrorKind> {
        let by: By = (&locator).into();
        let el = self.0.find_element(by).await?;
        self.0
            .action_chain()
            .move_to_element_center(&el)
            .move_by_offset(coord.0, coord.1)
            .double_click()
            .perform()
            .await?;

        Ok(())
    }
}

pub struct WebElement<'a>(thirtyfour::WebElement<'a>, &'a thirtyfour::WebDriver);

#[async_trait::async_trait]
impl<'a> Element for WebElement<'a> {
    type Driver = Client<'a>;

    async fn attr(&mut self, attribute: &str) -> Result<Option<String>, RunnerErrorKind> {
        let attr = self.0.get_attribute(attribute).await?;
        Ok(attr)
    }

    async fn prop(&mut self, prop: &str) -> Result<Option<String>, RunnerErrorKind> {
        let prop = self.0.get_property(prop).await?;
        Ok(prop)
    }

    async fn text(&mut self) -> Result<String, RunnerErrorKind> {
        let text = self.0.text().await?;
        Ok(text)
    }

    async fn html(&mut self, inner: bool) -> Result<String, RunnerErrorKind> {
        let html = if inner {
            self.0.inner_html().await?
        } else {
            self.0.outer_html().await?
        };

        Ok(html)
    }

    async fn find(&mut self, search: Locator) -> Result<Self, RunnerErrorKind>
    where
        Self: Sized,
    {
        let search: By = (&search).into();
        let e = self.0.find_element(search).await?;

        Ok(Self(e, self.1))
    }

    async fn click(mut self) -> Result<Self::Driver, RunnerErrorKind> {
        self.0.click().await?;
        Ok(Client(self.1))
    }

    async fn select_by_index(mut self, index: usize) -> Result<Self::Driver, RunnerErrorKind> {
        SelectElement::new(&self.0)
            .await?
            .select_by_index(index as u32)
            .await?;

        Ok(Client(self.1))
    }

    async fn select_by_value(mut self, value: &str) -> Result<Self::Driver, RunnerErrorKind> {
        SelectElement::new(&self.0)
            .await?
            .select_by_value(value)
            .await?;

        Ok(Client(self.1))
    }

    async fn send_keys(mut self, value: &str) -> Result<(), RunnerErrorKind> {
        self.0.send_keys(value).await?;
        Ok(())
    }

    async fn select_by_label(mut self, value: &str) -> Result<Self::Driver, RunnerErrorKind> {
        SelectElement::new(&self.0)
            .await?
            .select_by_visible_text(value)
            .await?;

        Ok(Client(self.1))
    }

    async fn is_selected(&mut self) -> Result<bool, RunnerErrorKind> {
        let r = self.0.is_selected().await?;
        Ok(r)
    }

    async fn is_present(&mut self) -> Result<bool, RunnerErrorKind> {
        let r = self.0.is_present().await?;
        Ok(r)
    }

    async fn is_enabled(&mut self) -> Result<bool, RunnerErrorKind> {
        let r = self.0.is_enabled().await?;
        Ok(r)
    }
}

impl<'a> From<&'a Locator> for By<'a> {
    fn from(locator: &'a Locator) -> By<'a> {
        match locator {
            Locator::LinkText(s) => By::LinkText(&s),
            Locator::Css(s) => By::Css(&s),
            Locator::Id(s) => By::Id(&s),
            Locator::XPath(s) => By::XPath(&s),
        }
    }
}
