use super::Command;
use crate::{
    error::RunnerErrorKind,
    parser::SelectLocator,
    webdriver::{Element, Locator, Webdriver},
    File, Runner,
};

pub struct Select {
    target: Locator,
    select_target: SelectLocator,
}

impl Select {
    pub fn new(target: Locator, select_target: SelectLocator) -> Self {
        Self {
            target,
            select_target,
        }
    }
}

#[async_trait::async_trait]
impl Command for Select {
    async fn run<D, E>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: crate::webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
    {
        let mut select = runner.get_webdriver().find(self.target.clone()).await?;
        match &self.select_target {
            SelectLocator::Index(index) => {
                // todo: DO emit of locators before calling Command

                let index = runner.emit(index);
                match index.parse() {
                    Ok(index) => {
                        select.select_by_index(index).await?;
                    }
                    // TODO: IlligalSyntax  Failed: Illegal Index: {version_counter}
                    Err(..) => {
                        return Err(RunnerErrorKind::MismatchedType(format!(
                            "expected to get int type but got {:?}",
                            index
                        )));
                    }
                }
            }
            SelectLocator::Value(value) => {
                let value = runner.emit(value);
                select.select_by_value(&value).await?;
            }
            SelectLocator::Id(id) => {
                let id = runner.emit(id);
                let locator = format!(r#"option[id='{}']"#, id);
                select.find(Locator::Css(locator)).await?.click().await?;
            }
            SelectLocator::Label(label) => {
                let label = runner.emit(label);
                // somehow .//option[normalize-space(.)='{}'] doesn work...
                let locator = format!(".//*[normalize-space(.)='{}']", label);
                select.find(Locator::XPath(locator)).await?.click().await?;
            }
        };
        Ok(())
    }
}
