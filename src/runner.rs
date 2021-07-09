// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::command::{
    AnswerOnNextPrompt, Assert, AssertAlert, AssertChecked, AssertConfirmation, AssertNotChecked,
    AssertNotSelectedValue, AssertNotText, AssertSelectedLabel, AssertSelectedValue, AssertText,
    AssertTitle, AssertValue, Check, ChooseCancelOnNextConfirmation, ChooseCancelOnNextPrompt,
    ChooseOkOnNextConfirmation, Click, Close, DoubleClick, Echo, EditContent, Execute,
    ExecuteAsync, MouseDown, MouseUp, Open, Pause, RunScript, Select, SendKeys, SetWindowSize,
    Store, StoreText, StoreTitle, StoreXpathCount, Type, UnCheck, WaitForElementEditable,
    WaitForElementNotPresent, WaitForElementPresent, WaitForElementVisible,
};
use crate::command::{AssertPrompt, Command as Cmd1};
use crate::parser::Target;
use crate::playground::Playground;
use crate::webdriver::{Locator, Webdriver};
use crate::File;
use crate::{
    error::{RunnerError, RunnerErrorKind},
    parser::{Cmd, Location},
};
use serde_json::Value;
use std::collections::HashMap;

/// A runtime for running test
///
/// It runs commands and controls program flow(manages conditions and loops).
/// It manages usage of variables.
pub struct Runner<D> {
    webdriver: D,
    data: HashMap<String, Value>,
    echo_hook: Box<dyn Fn(&str) + Send>,
}

impl<D> Runner<D> {
    /// Create a new Runner which uses a client as a Backend
    pub(crate) fn _new(client: D) -> Runner<D> {
        Self {
            webdriver: client,
            data: HashMap::new(),
            echo_hook: Box::new(|s| println!("{}", s)),
        }
    }

    /// Save a value in storage.
    ///
    /// All tests which will be run afterwards will able to work with the saved variable.
    pub fn save_value(&mut self, var: String, value: Value) {
        self.data.insert(var, value);
    }

    /// Get a value from a data storage.
    pub fn get_value(&mut self, var: &str) -> Option<&Value> {
        self.data.get(var)
    }

    /// Sets a callback which will be run on each Echo command.
    pub fn set_echo<F: Fn(&str) + Send + 'static>(&mut self, func: F) {
        self.echo_hook = Box::new(func);
    }

    /// Gets a list of variables which were collected over the runs.
    pub fn get_data(&self) -> &HashMap<String, Value> {
        &self.data
    }

    /// Gets a used webdriver backend
    pub fn get_webdriver(&mut self) -> &mut D {
        &mut self.webdriver
    }

    pub(crate) fn get_value_mut(&mut self, var: &str) -> Option<&mut Value> {
        self.data.get_mut(var)
    }

    pub(crate) fn echo(&self, message: &str) {
        self.echo_hook.as_ref()(message)
    }
}

impl<D> Runner<D>
where
    D: Webdriver,
{
    /// Close underlying webdriver client.
    ///
    /// It must be run as some backends require it's call to release a Webdriver session.
    pub async fn close(mut self) -> Result<(), RunnerErrorKind> {
        self.webdriver.close().await
    }

    /// Run all tests in a side file starting from first test.
    pub async fn run(&mut self, file: &File) -> Result<(), RunnerError> {
        for test in 0..file.tests.len() {
            self.run_test(file, test).await?;
        }

        Ok(())
    }

    /// Run a particular test in a file.
    /// Test represented by an index of it.
    pub async fn run_test(&mut self, file: &File, test: usize) -> Result<(), RunnerError> {
        Playground::run_test(self, file, test).await
    }

    pub(crate) async fn run_command(
        &mut self,
        file_url: &str,
        cmd: &Cmd,
    ) -> Result<(), RunnerErrorKind> {
        // TODO: emit variables in value field too
        println!("CMD {:?}", cmd);
        match cmd {
            Cmd::Open(url) => Open::new(url.clone(), file_url.to_owned()).run(self).await,
            Cmd::StoreText { var, target, .. } => {
                StoreText::new(target.clone().into(), var.to_owned())
                    .run(self)
                    .await
            }
            Cmd::Store { var, value } => Store::new(var.clone(), value.clone()).run(self).await,
            Cmd::Execute { script, var } => {
                Execute::new(script.clone(), var.clone()).run(self).await
            }
            Cmd::ExecuteAsync { script, var } => {
                ExecuteAsync::new(script.clone(), var.clone())
                    .run(self)
                    .await
            }
            Cmd::Echo(text) => Echo::new(text.clone()).run(self).await,
            Cmd::WaitForElementVisible { timeout, target } => {
                WaitForElementVisible::new(target.clone().into(), *timeout)
                    .run(self)
                    .await
            }
            Cmd::WaitForElementPresent { timeout, target } => {
                WaitForElementPresent::new(target.clone().into(), *timeout)
                    .run(self)
                    .await
            }
            Cmd::WaitForElementNotPresent { timeout, target } => {
                WaitForElementNotPresent::new(target.clone().into(), *timeout)
                    .run(self)
                    .await
            }
            Cmd::WaitForElementEditable { timeout, target } => {
                WaitForElementEditable::new(target.clone().into(), *timeout)
                    .run(self)
                    .await
            }
            Cmd::Select { locator, target } => {
                Select::new(target.clone().into(), locator.clone())
                    .run(self)
                    .await
            }
            Cmd::Click(target) => Click::new(target.clone().into()).run(self).await,
            Cmd::Pause(timeout) => Pause::new(*timeout).run(self).await,
            Cmd::SetWindowSize(w, h) => SetWindowSize::new(*w, *h).run(self).await,
            Cmd::StoreXpathCount { var, xpath } => {
                StoreXpathCount::new(xpath.clone(), var.clone())
                    .run(self)
                    .await
            }
            Cmd::Close => Close.run(self).await,
            Cmd::Assert { var, value } => Assert::new(var.clone(), value.clone()).run(self).await,
            Cmd::RunScript { script } => RunScript::new(script.clone()).run(self).await,
            Cmd::AnswerOnNextPrompt(message) => {
                AnswerOnNextPrompt::new(message.clone()).run(self).await
            }
            Cmd::AssertAlert(expect) => AssertAlert::new(expect.clone()).run(self).await,
            Cmd::AssertChecked(target) => AssertChecked::new(target.clone().into()).run(self).await,
            Cmd::AssertNotChecked(target) => {
                AssertNotChecked::new(target.clone().into()).run(self).await
            }
            Cmd::AssertPrompt(expect) => AssertPrompt::new(expect.clone()).run(self).await,
            Cmd::AssertSelectedValue(target, value) => {
                AssertSelectedValue::new(target.clone().into(), value.clone())
                    .run(self)
                    .await
            }
            Cmd::AssertNotSelectedValue(target, value) => {
                AssertNotSelectedValue::new(target.clone().into(), value.clone())
                    .run(self)
                    .await
            }
            Cmd::AssertText(target, value) => {
                AssertText::new(target.clone().into(), value.clone())
                    .run(self)
                    .await
            }
            Cmd::AssertNotText(target, value) => {
                AssertNotText::new(target.clone().into(), value.clone())
                    .run(self)
                    .await
            }
            Cmd::DoubleClick(target) => DoubleClick::new(target.clone().into()).run(self).await,
            Cmd::EditContent(target, value) => {
                EditContent::new(target.clone().into(), value.clone())
                    .run(self)
                    .await
            }
            Cmd::SendKeys(target, value) => {
                SendKeys::new(target.clone().into(), value.clone())
                    .run(self)
                    .await
            }
            Cmd::Type(target, value) => {
                Type::new(target.clone().into(), value.clone())
                    .run(self)
                    .await
            }
            Cmd::Check(target) => Check::new(target.clone().into()).run(self).await,
            Cmd::UnCheck(target) => UnCheck::new(target.clone().into()).run(self).await,
            Cmd::MouseDown(target) => MouseDown::new(target.clone().into()).run(self).await,
            Cmd::MouseUp(target) => MouseUp::new(target.clone().into()).run(self).await,
            Cmd::ChooseCancelOnNextConfirmation => ChooseCancelOnNextConfirmation.run(self).await,
            Cmd::ChooseOkOnNextConfirmation => ChooseOkOnNextConfirmation.run(self).await,
            Cmd::ChooseCancelOnNextPrompt => ChooseCancelOnNextPrompt.run(self).await,
            Cmd::AssertTitle(t) => AssertTitle::new(t.clone()).run(self).await,
            Cmd::StoreTitle(t) => StoreTitle::new(t.clone()).run(self).await,
            Cmd::AssertValue(target, value) => {
                AssertValue::new(target.clone().into(), value.clone())
                    .run(self)
                    .await
            }
            Cmd::AssertConfirmation(target) => {
                AssertConfirmation::new(target.clone()).run(self).await
            }
            Cmd::AssertSelectedLabel(target, value) => {
                AssertSelectedLabel::new(target.clone().into(), value.clone())
                    .run(self)
                    .await
            }
            Cmd::While(..)
            | Cmd::Else
            | Cmd::If(..)
            | Cmd::ElseIf(..)
            | Cmd::ForEach { .. }
            | Cmd::RepeatIf(..)
            | Cmd::Times(..)
            | Cmd::Do
            | Cmd::End
            | Cmd::Custom { .. } => unreachable!("All flow commands are handled at this point"),
        }
    }

    pub(crate) async fn exec(
        &mut self,
        script: &str,
    ) -> std::result::Result<serde_json::Value, RunnerErrorKind> {
        let (script, used_vars) = emit_variables_custom(script);
        let args = used_vars.iter().map(|var| self.data[var].clone()).collect();
        let prepared_script = format!("return (function(arguments) {{ {} }})(arguments)", script);

        let value = self.webdriver.execute(&prepared_script, args).await?;

        Ok(value)
    }

    pub(crate) async fn exec_async(
        &mut self,
        script: &str,
    ) -> std::result::Result<serde_json::Value, RunnerErrorKind> {
        let (script, used_vars) = emit_variables_custom(script);
        let args = used_vars.iter().map(|var| self.data[var].clone()).collect();
        let value = self
            .webdriver
            .execute(
                &format!("return (function(arguments) {{ {} }})(arguments)", script),
                args,
            )
            .await?;

        Ok(value)
    }

    pub(crate) fn emit(&self, s: &str) -> String {
        emit_variables(s, &self.data)
    }
}

fn emit_variables(s: &str, vars: &HashMap<String, Value>) -> String {
    emit_vars(s, |var| match vars.get(var) {
        Some(value) => print_plain_value(value),
        None => format!("${{{}}}", var),
    })
}

fn emit_variables_custom(text: &str) -> (String, Vec<String>) {
    let mut emited_vars = Vec::new();

    let new_text = emit_vars(text, |var| {
        let arg_pos = match emited_vars.iter().position(|arg| arg == var) {
            Some(pos) => pos,
            None => {
                emited_vars.push(var.to_owned());
                emited_vars.len() - 1
            }
        };

        format!("arguments[{}]", arg_pos)
    });

    (new_text, emited_vars)
}

fn emit_vars<P: FnMut(&str) -> String>(s: &str, mut printer: P) -> String {
    // todo: use lazystatic for regex
    // TODO: check how to emit string in quotes or not
    //
    // regex look up for variable name in brackets #{var}
    // it exclude " sign to manage cases like ${var} }
    // it's important in emiting vars in JSON
    //
    // https://github.com/SeleniumHQ/selenium-ide/blob/dd0c8ce313171672d2f0670cfb05786611f85b73/packages/side-runtime/src/preprocessors.js#L119
    // let re = regex::Regex::new(r#"\$\{(.*?)\}"#).unwrap();
    let re = regex::Regex::new(r#"\$\{(.*?)\}"#).unwrap();
    let new_s = re.replace_all(s, |caps: &regex::Captures| printer(&caps[1]));
    new_s.to_string()
}

fn print_plain_value(val: &Value) -> String {
    match val {
        Value::String(val) => val.clone(),
        Value::Null => "".to_string(),
        Value::Number(val) => val.to_string(),
        Value::Object(..) => "[object Object]".to_string(), // is it ok behaviour?
        Value::Array(values) => values
            .iter()
            .map(|v| print_plain_value(v))
            .collect::<Vec<_>>()
            .join(","),
        Value::Bool(val) => val.to_string(),
    }
}

impl From<Target> for Locator {
    fn from(target: Target) -> Self {
        match target.location {
            Location::Css(css) => Locator::Css(css),
            Location::Id(id) => Locator::Id(id),
            Location::XPath(path) => Locator::XPath(path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_emit_variables() {
        let mut vars = HashMap::new();
        vars.insert("hello".to_string(), json!("Hello"));
        vars.insert("world".to_string(), json!("World"));
        vars.insert("something".to_string(), json!("XXX"));
        vars.insert("hello_world".to_string(), json!("Hello World"));

        assert_eq!("Hello", emit_variables("${hello}", &vars));
        assert_eq!("Hello World!", emit_variables("${hello} ${world}!", &vars));
        assert_eq!(
            "There are no vars here",
            emit_variables("There are no vars here", &vars)
        );

        assert_eq!("\"World\"", emit_variables("\"${world}\"", &vars));
        assert_eq!("World\" }", emit_variables("${world}\" }", &vars));
        assert_eq!("World\"}", emit_variables("${world}\"}", &vars));
        assert_eq!("World }", emit_variables("${world} }", &vars));
        assert_eq!("World}", emit_variables("${world}}", &vars));

        assert_eq!("Hello World", emit_variables("${hello_world}", &vars));
    }

    #[test]
    fn test_emit_variables_in_template_strings() {
        let mut vars = HashMap::new();

        assert_eq!(
            "let bar = \"123\"; return `${bar} hello`",
            emit_variables("let bar = \"123\"; return `${bar} hello`", &vars)
        );

        vars.insert("bar".to_string(), json!("Hello"));

        assert_eq!(
            "let bar = \"123\"; return `Hello hello`",
            emit_variables("let bar = \"123\"; return `${bar} hello`", &vars)
        );
    }

    // there could be added a support for internal variables by
    // r#"\$\{(.*?)\}+"# and recursive calling + handling spaces
    // but is there any use case for it?
    //
    // Selenium seemingly doesn't handle this.
    #[test]
    fn test_emit_internal_variables_doesn_work() {
        let mut vars = HashMap::new();
        vars.insert("something".to_string(), json!("XXX"));

        assert_eq!("${${something}}", emit_variables("${${something}}", &vars));
    }

    #[test]
    fn test_emit_variables_types() {
        let mut vars = HashMap::new();

        vars.insert("test".to_string(), json!("string"));
        assert_eq!("string", emit_variables("${test}", &vars));

        vars.insert("test".to_string(), json!(2));
        assert_eq!("2", emit_variables("${test}", &vars));

        vars.insert("test".to_string(), json!({"h3": 3}));
        assert_eq!("[object Object]", emit_variables("${test}", &vars));

        vars.insert("test".to_string(), json!(["h4", 4]));
        assert_eq!("h4,4", emit_variables("${test}", &vars));
    }
}
