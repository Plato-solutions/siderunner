use crate::{error::RunnerErrorKind, webdriver::Webdriver};
use url::Url;

use super::Command;

pub struct Open {
    url: String,
    file_url: String,
}

impl Open {
    pub fn new(url: String, file_url: String) -> Self {
        Self { url, file_url }
    }
}

#[async_trait::async_trait]
impl Command for Open {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E> + Send,
        E: crate::webdriver::Element<Driver = D> + Send,
    {
        let url = runner.emit(&self.url);
        let url = build_url(&self.file_url, &url)?;
        let url = url.as_str();

        runner.get_webdriver().goto(url).await?;

        let url = runner.get_webdriver().current_url().await?;
        assert_eq!(url.as_ref(), url.as_ref());

        Ok(())
    }
}

fn build_url(base: &str, url: &str) -> Result<Url, url::ParseError> {
    match Url::parse(url) {
        Ok(url) => Ok(url),
        Err(url::ParseError::RelativeUrlWithoutBase) => Url::parse(base)?.join(&url),
        e => e,
    }
}
