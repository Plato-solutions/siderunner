// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::ParseError;
use std::result::Result;
use std::time::Duration;

/// Parse [.side format] into rust representation
///
/// [.side format]: https://github.com/SeleniumHQ/selenium-ide/issues/77
pub fn parse<R: std::io::Read>(side_file: R) -> Result<File, ParseError> {
    let side: format::SideFile =
        serde_json::from_reader(side_file).map_err(ParseError::FormatError)?;
    let mut tests = Vec::new();
    for test in side.tests {
        let mut commands = Vec::with_capacity(test.commands.len());
        for command in test.commands {
            let cmd = parse_cmd(&command)?;
            commands.push(Command {
                comment: command.comment,
                id: command.id,
                cmd,
            });
        }

        tests.push(Test {
            id: test.id,
            name: test.name,
            commands,
        });
    }

    Ok(File {
        id: side.id,
        name: side.name,
        url: side.url,
        version: side.version,
        tests,
    })
}

fn parse_cmd(command: &format::Command) -> Result<Cmd, ParseError> {
    let parse_fn = match command.cmd.as_str() {
        "open" => Cmd::parse_open,
        "store" => Cmd::parse_store,
        "storeText" => Cmd::parse_store_text,
        "executeScript" => Cmd::parse_execute_script,
        "executeScriptAsync" => Cmd::parse_execute_async_script,
        "waitForElementVisible" => Cmd::parse_wait_for_visible,
        "waitForElementEditable" => Cmd::parse_wait_for_editable,
        "waitForElementNotPresent" => Cmd::parse_wait_for_not_present,
        "waitForElementPresent" => Cmd::parse_wait_for_present,
        "select" => Cmd::parse_select,
        "addSelection" => Cmd::parse_add_selection,
        "echo" => Cmd::parse_echo,
        "pause" => Cmd::parse_pause,
        "click" => Cmd::parse_click,
        "while" => Cmd::parse_while,
        "if" => Cmd::parse_if,
        "elseIf" => Cmd::parse_else_if,
        "else" => Cmd::parse_else,
        "end" => Cmd::parse_end,
        "setWindowSize" => Cmd::parse_set_window_size,
        "do" => Cmd::parse_do,
        "repeatIf" => Cmd::parse_repeat_if,
        "forEach" => Cmd::parse_for_each,
        "close" => Cmd::parse_close,
        "storeXpathCount" => Cmd::parse_store_xpath_count,
        "assert" => Cmd::parse_assert,
        "runScript" => Cmd::parse_run_script,
        "answerOnNextPrompt" => Cmd::parse_answer_on_next_prompt,
        "assertAlert" => Cmd::parse_assert_alert,
        "assertChecked" => Cmd::parse_assert_checked,
        "assertNotChecked" => Cmd::parse_assert_not_checked,
        "assertPrompt" => Cmd::parse_assert_prompt,
        "assertSelectedValue" => Cmd::parse_assert_selected_value,
        "assertNotSelectedValue" => Cmd::parse_assert_not_selected_value,
        "assertText" => Cmd::parse_assert_text,
        "assertNotText" => Cmd::parse_assert_not_text,
        "doubleClick" => Cmd::parse_double_click,
        "editContent" => Cmd::parse_edit_content,
        "sendKeys" => Cmd::parse_send_keys,
        "type" => Cmd::parse_type,
        "check" => Cmd::parse_check,
        "uncheck" => Cmd::parse_uncheck,
        "mouseUp" => Cmd::parse_mouse_up,
        "mouseDown" => Cmd::parse_mouse_down,
        "chooseCancelOnNextConfirmation" => Cmd::parse_choose_cancel_on_next_confirmation,
        "chooseOkOnNextConfirmation" => Cmd::parse_choose_ok_on_next_confirmation,
        "chooseCancelOnNextPrompt" => Cmd::parse_choose_cancel_on_next_prompt,
        "assertTitle" => Cmd::parse_assert_title,
        "storeTitle" => Cmd::parse_store_title,
        "assertValue" => Cmd::parse_assert_value,
        "assertConfirmation" => Cmd::parse_assert_confirmation,
        "assertSelectedLabel" => Cmd::parse_assert_selected_label,
        "times" => Cmd::parse_times,
        "run" => Cmd::parse_run,
        cmd if cmd.is_empty() || cmd.starts_with("//") => {
            // We create an empty command to not lose an order of commands.
            // It's usefull for error messages to not break the indexes of commands from a file.
            //
            // Having an empty command could add a subtle overhead on runtime as it add an additional iteration to the running loop.
            // Creating a bool flag for each command to check if it's commented seems also inefition because we don't need a information
            // about the commented commands at least now.
            //
            // The overhead is removed on a stage of creationn of running list.
            Cmd::parse_custom_cmd
        }
        cmd => {
            return Err(ParseError::ValidationError(format!(
                "Command {:?} is not implemented",
                cmd
            )))
        }
    };

    parse_fn(command)
}

/// File represent a [`Side` file] information
///
/// [`Side` file]: https://github.com/SeleniumHQ/selenium-ide/issues/77
#[derive(Debug)]
pub struct File {
    /// Id of a file.
    /// It is generated by Selenium IDE automatically.
    pub id: String,
    /// Version of a scheme
    pub version: String,
    /// Name of a project
    pub name: String,
    /// Address of a parsed site
    pub url: String,
    /// A list of [`Test`]s
    ///
    /// [`Test`]: struct.Test.html
    pub tests: Vec<Test>,
}

impl File {
    pub fn new(id: String, name: String, url: String, version: String, tests: Vec<Test>) -> Self {
        Self {
            id,
            version,
            name,
            url,
            tests,
        }
    }
}

/// The structure represent a selenium test
#[derive(Debug)]
pub struct Test {
    /// Id of a test.
    /// Generated automatically by Selenium IDE.
    pub id: String,
    /// Name of the test
    pub name: String,
    /// A list of commands
    pub commands: Vec<Command>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Command {
    /// Id of a command.
    /// It's generated automatically by Selenium IDE.
    pub id: String,
    /// A comment for a given command.
    pub comment: String,
    /// Particualar command for run.
    pub cmd: Cmd,
}

impl Command {
    pub fn new<Id: AsRef<str>, Comment: AsRef<str>>(id: Id, comment: Comment, cmd: Cmd) -> Self {
        Self {
            id: id.as_ref().to_owned(),
            comment: comment.as_ref().to_owned(),
            cmd,
        }
    }
}

/// Command corresponds a selenium command
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Cmd {
    Open(String),
    Echo(String),
    Click(Target),
    Pause(Duration),
    SetWindowSize(u32, u32),
    // todo: targets?
    Select {
        target: Target,
        locator: SelectLocator,
    },
    WaitForElementVisible {
        target: Target,
        timeout: Duration,
    },
    WaitForElementEditable {
        target: Target,
        timeout: Duration,
    },
    WaitForElementNotPresent {
        target: Target,
        timeout: Duration,
    },
    WaitForElementPresent {
        target: Target,
        timeout: Duration,
    },
    StoreText {
        var: String,
        target: Target,
        targets: Vec<Target>,
    },
    Store {
        var: String,
        value: String,
    },
    Execute {
        script: String,
        var: Option<String>,
    },
    ExecuteAsync {
        script: String,
        var: Option<String>,
    },
    While(String),
    If(String),
    ElseIf(String),
    Else,
    Do,
    RepeatIf(String),
    ForEach {
        iterator: String,
        var: String,
    },
    End,
    StoreXpathCount {
        var: Option<String>,
        xpath: String,
    },
    Close,
    Custom {
        cmd: String,
        target: String,
        targets: Vec<Target>,
        value: String,
    },
    Assert {
        var: String,
        value: String,
    },
    RunScript {
        script: String,
    },
    AnswerOnNextPrompt(String),
    AssertAlert(String),
    AssertPrompt(String),
    AssertChecked(Target),
    AssertNotChecked(Target),
    AssertSelectedValue(Target, String),
    AssertNotSelectedValue(Target, String),
    AssertText(Target, String),
    AssertNotText(Target, String),
    DoubleClick(Target),
    EditContent(Target, String),
    SendKeys(Target, String),
    Type(Target, String),
    Check(Target),
    UnCheck(Target),
    MouseUp(Target),
    MouseDown(Target),
    ChooseCancelOnNextConfirmation,
    ChooseOkOnNextConfirmation,
    ChooseCancelOnNextPrompt,
    AssertTitle(String),
    StoreTitle(String),
    AssertValue(Target, String),
    AssertConfirmation(String),
    AssertSelectedLabel(Target, String),
    Times(String),
    RunTest(String),
}

impl Cmd {
    fn parse_open(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::Open(c.target.clone()))
    }

    fn parse_store_text(c: &format::Command) -> Result<Self, ParseError> {
        let targets = parse_targets(&c.targets)?;
        let var = c.value.clone();
        let location = parse_location(&c.target)?;
        let target = Target::new(location);

        Ok(Self::StoreText {
            var,
            target,
            targets,
        })
    }

    fn parse_execute_script(c: &format::Command) -> Result<Self, ParseError> {
        let var = if c.value.is_empty() {
            None
        } else {
            Some(c.value.clone())
        };
        let script = c.target.clone();

        Ok(Self::Execute { script, var })
    }

    fn parse_execute_async_script(c: &format::Command) -> Result<Self, ParseError> {
        let var = if c.value.is_empty() {
            None
        } else {
            Some(c.value.clone())
        };
        let script = c.target.clone();

        Ok(Self::ExecuteAsync { script, var })
    }

    fn parse_wait_for_visible(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Self::WaitForElementVisible { target, timeout })
    }

    fn parse_wait_for_editable(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Self::WaitForElementEditable { target, timeout })
    }

    fn parse_wait_for_not_present(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Self::WaitForElementNotPresent { target, timeout })
    }

    fn parse_wait_for_present(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Self::WaitForElementPresent { target, timeout })
    }

    fn parse_select(c: &format::Command) -> Result<Self, ParseError> {
        let locator = parse_select_locator(&c.value)?;
        let location = parse_location(&c.target)?;
        let target = Target::new(location);

        Ok(Self::Select { target, locator })
    }

    fn parse_add_selection(c: &format::Command) -> Result<Self, ParseError> {
        let locator = SelectLocator::Label(c.value.clone());
        let location = parse_location(&c.target)?;
        let target = Target::new(location);

        Ok(Self::Select { target, locator })
    }

    fn parse_echo(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::Echo(c.target.clone()))
    }

    fn parse_while(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::While(c.target.clone()))
    }

    fn parse_if(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::If(c.target.clone()))
    }

    fn parse_else(_: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::Else)
    }

    fn parse_else_if(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::ElseIf(c.target.clone()))
    }

    fn parse_end(_: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::End)
    }

    fn parse_pause(c: &format::Command) -> Result<Self, ParseError> {
        let timeout = cast_timeout(&c.target)?;
        Ok(Self::Pause(timeout))
    }

    fn parse_click(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::Click(target))
    }

    fn parse_store(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::Store {
            value: c.target.to_owned(),
            var: c.value.to_owned(),
        })
    }

    fn parse_set_window_size(c: &format::Command) -> Result<Self, ParseError> {
        let settings = c.target.split('x').map(|n| n.parse()).collect::<Vec<_>>();
        if settings.len() != 2 {
            return Err(ParseError::TypeError(
                "window size expected to get in a form like this 1916x1034 (Width x Height)"
                    .to_owned(),
            ));
        }

        let w = settings[0]
            .clone()
            .map_err(|_| ParseError::TypeError("expected to get int".to_owned()))?;
        let h = settings[1]
            .clone()
            .map_err(|_| ParseError::TypeError("expected to get int".to_owned()))?;

        Ok(Self::SetWindowSize(w, h))
    }

    fn parse_do(_: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::Do)
    }

    fn parse_repeat_if(cmd: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::RepeatIf(cmd.target.clone()))
    }

    fn parse_for_each(cmd: &format::Command) -> Result<Self, ParseError> {
        let iterator = cmd.target.clone();
        let var = cmd.value.clone();
        Ok(Self::ForEach { iterator, var })
    }

    fn parse_close(_: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::Close)
    }

    fn parse_custom_cmd(cmd: &format::Command) -> Result<Self, ParseError> {
        let targets = parse_targets(&cmd.targets)?;
        Ok(Self::Custom {
            cmd: cmd.cmd.clone(),
            target: cmd.target.clone(),
            value: cmd.value.clone(),
            targets,
        })
    }

    pub fn empty_custom() -> Self {
        Self::Custom {
            cmd: String::default(),
            target: String::default(),
            value: String::default(),
            targets: Vec::default(),
        }
    }

    fn parse_store_xpath_count(c: &format::Command) -> Result<Self, ParseError> {
        let var = if c.value.is_empty() {
            None
        } else {
            Some(c.value.clone())
        };
        let location = parse_location(&c.target)?;
        match location {
            Location::XPath(xpath) => Ok(Self::StoreXpathCount { var, xpath }),
            _ => Err(ParseError::LocatorFormatError(
                "expected to get an xpath locator".to_owned(),
            )),
        }
    }

    fn parse_assert(c: &format::Command) -> Result<Self, ParseError> {
        let var = c.target.clone();
        let value = c.value.clone();
        Ok(Self::Assert { value, var })
    }

    fn parse_run_script(c: &format::Command) -> Result<Self, ParseError> {
        let script = c.target.clone();

        Ok(Self::RunScript { script })
    }

    fn parse_answer_on_next_prompt(c: &format::Command) -> Result<Self, ParseError> {
        let message = c.target.clone();

        Ok(Self::AnswerOnNextPrompt(message))
    }

    fn parse_assert_alert(c: &format::Command) -> Result<Self, ParseError> {
        let expected = c.target.clone();
        Ok(Self::AssertAlert(expected))
    }

    fn parse_assert_prompt(c: &format::Command) -> Result<Self, ParseError> {
        let expected = c.target.clone();
        Ok(Self::AssertPrompt(expected))
    }

    fn parse_assert_checked(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::AssertChecked(target))
    }

    fn parse_assert_not_checked(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::AssertNotChecked(target))
    }

    fn parse_assert_selected_value(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::AssertSelectedValue(target, c.value.clone()))
    }

    fn parse_assert_not_selected_value(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::AssertNotSelectedValue(target, c.value.clone()))
    }

    fn parse_assert_text(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::AssertText(target, c.value.clone()))
    }

    fn parse_assert_not_text(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::AssertNotText(target, c.value.clone()))
    }

    fn parse_double_click(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::DoubleClick(target))
    }

    fn parse_edit_content(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::EditContent(target, c.value.clone()))
    }

    fn parse_send_keys(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::SendKeys(target, c.value.clone()))
    }

    fn parse_type(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::Type(target, c.value.clone()))
    }

    fn parse_check(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::Check(target))
    }

    fn parse_uncheck(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::UnCheck(target))
    }

    fn parse_mouse_down(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::MouseDown(target))
    }

    fn parse_mouse_up(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::MouseUp(target))
    }

    fn parse_choose_cancel_on_next_confirmation(_: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::ChooseCancelOnNextConfirmation)
    }

    fn parse_choose_ok_on_next_confirmation(_: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::ChooseOkOnNextConfirmation)
    }

    fn parse_choose_cancel_on_next_prompt(_: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::ChooseCancelOnNextPrompt)
    }

    fn parse_assert_title(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::AssertTitle(c.target.clone()))
    }

    fn parse_store_title(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::StoreTitle(c.value.clone()))
    }

    fn parse_assert_value(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::AssertValue(target, c.value.clone()))
    }

    fn parse_assert_confirmation(c: &format::Command) -> Result<Self, ParseError> {
        let expected = c.target.clone();
        Ok(Self::AssertConfirmation(expected))
    }

    fn parse_assert_selected_label(c: &format::Command) -> Result<Self, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Self::AssertSelectedLabel(target, c.value.clone()))
    }

    fn parse_times(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::Times(c.target.clone()))
    }

    fn parse_run(c: &format::Command) -> Result<Self, ParseError> {
        Ok(Self::RunTest(c.target.clone()))
    }
}

/// Target represents a locator of html element
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    /// Location is a way to find an element
    pub location: Location,
    /// Tag is an additional information of location type e.g.
    /// location = xpath, tag = Some(relative) | Some(positional) | None
    pub tag: Option<String>,
}

impl Target {
    pub fn new(location: Location) -> Self {
        Target {
            location,
            tag: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectLocator {
    Index(String),
    Label(String),
    Id(String),
    Value(String),
}

/// Location is a locator of a HTML element
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Location {
    XPath(String),
    Css(String),
    Id(String),
}

impl Location {
    fn new(tp: &str, path: &str) -> Result<Self, ParseError> {
        let path = path.to_owned();
        match tp {
            "xpath" => Ok(Location::XPath(path)),
            "css" => Ok(Location::Css(path)),
            "id" => Ok(Location::Id(path)),
            _ => Err(ParseError::LocatorFormatError(format!(
                "unexpected locator type {}, supported xpath|css|id",
                tp
            ))),
        }
    }
}

fn parse_location(text: &str) -> Result<Location, ParseError> {
    let mut target_location = text.splitn(2, '=');
    let location_type = target_location.next().ok_or_else(|| {
        ParseError::LocatorFormatError(
            "target should contain a type of selector and a selector splited by '='".to_owned(),
        )
    })?;
    let location = target_location.next().ok_or_else(|| {
        ParseError::LocatorFormatError(
            "target should contain a type of selector and a selector splited by '='".to_owned(),
        )
    })?;

    Location::new(location_type, location)
}

fn parse_select_locator(text: &str) -> Result<SelectLocator, ParseError> {
    const ERROR_TEXT: &str = "unexpected type of selector";

    let mut locator = text.splitn(2, '=');
    let locator_type = locator
        .next()
        .ok_or_else(|| ParseError::LocatorFormatError(ERROR_TEXT.to_owned()))?;
    let locator = locator
        .next()
        .ok_or_else(|| ParseError::LocatorFormatError(ERROR_TEXT.to_owned()))?;

    match locator_type {
        "index" => Ok(SelectLocator::Index(locator.to_owned())),
        "label" => Ok(SelectLocator::Label(locator.to_owned())),
        "id" => Ok(SelectLocator::Id(locator.to_owned())),
        "value" => Ok(SelectLocator::Value(locator.to_owned())),
        _ => Err(ParseError::LocatorFormatError(ERROR_TEXT.to_owned())),
    }
}

fn parse_targets(targets: &[(String, String)]) -> Result<Vec<Target>, ParseError> {
    let mut out = Vec::new();
    for target in targets {
        let location = parse_location(&target.0)?;
        let tag = parse_target_tag(&target.1)?.to_owned();

        let target = Target {
            location,
            tag: Some(tag),
        };
        out.push(target);
    }

    Ok(out)
}

fn parse_target_tag(tag: &str) -> Result<&'_ str, ParseError> {
    Ok(tag.splitn(2, ':').nth(1).unwrap_or(tag))
}

fn cast_timeout(s: &str) -> Result<Duration, ParseError> {
    // TODO: posibly there may be a variable not only a number
    // so we would need to create a enum like
    // enum Value<T> {
    //    Completed(T),
    //    Incomplete(String),
    // }
    s.parse()
        .map_err(|_| ParseError::TypeError("expected to get int".to_owned()))
        .map(Duration::from_millis)
}

mod format {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SideFile {
        pub id: String,
        pub version: String,
        pub name: String,
        pub url: String,
        pub tests: Vec<Test>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Test {
        pub id: String,
        pub name: String,
        pub commands: Vec<Command>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Command {
        pub id: String,
        pub comment: String,
        #[serde(rename = "command")]
        pub cmd: String,
        pub target: String,
        pub targets: Vec<(String, String)>,
        pub value: String,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn _parse() {
        let file = side_file();
        let reader = file.as_slice();
        let file = parse(reader);
        assert!(file.is_ok());
        let file = file.unwrap();
        assert_eq!(file.tests.len(), 1);
        assert_eq!(file.tests[0].commands.len(), 3);
        assert!(matches!(
            file.tests[0].commands[0].cmd,
            Cmd::Open(ref url) if url == "RELATIVE_URL/"
        ));
        assert!(matches!(
            file.tests[0].commands[1].cmd,
            Cmd::Pause(ref timeout) if *timeout == Duration::from_secs(5)
        ));
        assert!(matches!(
            file.tests[0].commands[2].cmd,
            Cmd::Execute {ref script, ref var} if script == "return \"hello world\"" && var == &Some("variable_name".to_string())
        ));
    }

    #[test]
    fn _parse_empty_commands() {
        let file: Vec<u8> = r#"{
            "id": "bfc1bd56-39bd-4a0d-be2b-583ad75ac104",
            "version": "2.0",
            "name": "",
            "url": "",
            "tests": [{
              "id": "5d61ce01-d373-4b14-a1a1-7474a4e192e5",
              "name": "basic",
              "commands": [{
                "id": "5de247d8-432a-4c79-a44c-2e1fdc0c0ff9",
                "comment": "",
                "command": "",
                "target": "",
                "targets": [],
                "value": ""
              }, {
                "id": "5de247d8-432a-4c79-a44c-2e1fdc0c0ff9",
                "comment": "",
                "command": "//open",
                "target": "http://google.com",
                "targets": [
                    ["id=content", "id"],
                    ["css=#content", "css:finder"],
                    ["xpath=//div[@id='content']", "xpath:attributes"],
                    ["xpath=//div[4]/div[2]", "xpath:position"]
                ],
                "value": ""
              }, {
                "id": "3c00633c-4237-4c1b-b1e5-b8c2fc05e57d",
                "comment": "",
                "command": "open",
                "target": "http://google.com",
                "targets": [],
                "value": ""
              }]
            }],
            "suites": [{
              "id": "925de5ce-03ae-4dcb-9146-c956ff3f090d",
              "name": "Default Suite",
              "persistSession": false,
              "parallel": false,
              "timeout": 300,
              "tests": ["5d61ce01-d373-4b14-a1a1-7474a4e192e5"]
            }],
            "urls": ["https://bsscommerce.com/magento-2-one-step-checkout-extension.html"],
            "plugins": []
          }"#
        .as_bytes()
        .to_vec();

        let reader = file.as_slice();
        let file = parse(reader).unwrap();
        assert_eq!(file.tests.len(), 1);
        let test = &file.tests[0];
        let commands = &test.commands;
        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[0].cmd, Cmd::Custom { .. }));
        assert!(matches!(commands[1].cmd, Cmd::Custom { .. }));
        assert!(matches!(commands[2].cmd, Cmd::Open(..)));

        if let Cmd::Custom { targets, .. } = &commands[1].cmd {
            assert_eq!(targets.len(), 4);
            assert_eq!(
                targets[0],
                Target {
                    location: Location::Id("content".to_string()),
                    tag: Some("id".to_string())
                }
            );
            assert_eq!(
                targets[1],
                Target {
                    location: Location::Css("#content".to_string()),
                    tag: Some("finder".to_string())
                }
            );
            assert_eq!(
                targets[2],
                Target {
                    location: Location::XPath("//div[@id='content']".to_string()),
                    tag: Some("attributes".to_string())
                }
            );
            assert_eq!(
                targets[3],
                Target {
                    location: Location::XPath("//div[4]/div[2]".to_string()),
                    tag: Some("position".to_string())
                }
            );
        }
    }

    fn side_file() -> Vec<u8> {
        r#"{
            "id": "bfc1bd56-39bd-4a0d-be2b-583ad75ac104",
            "version": "2.0",
            "name": "EXAMPLE_TEST",
            "url": "https://BASE_URL.com/",
            "tests": [{
              "id": "5d61ce01-d373-4b14-a1a1-7474a4e192e5",
              "name": "basic",
              "commands": [{
                "id": "5de247d8-432a-4c79-a44c-2e1fdc0c0ff9",
                "comment": "",
                "command": "open",
                "target": "RELATIVE_URL/",
                "targets": [],
                "value": ""
              }, {
                "id": "3c00633c-4237-4c1b-b1e5-b8c2fc05e57d",
                "comment": "",
                "command": "pause",
                "target": "5000",
                "targets": [],
                "value": ""
              }, {
                "id": "b59f9f80-aef4-44cc-a7f2-41d34d7ad482",
                "comment": "",
                "command": "executeScript",
                "target": "return \"hello world\"",
                "targets": [],
                "value": "variable_name"
              }]
            }],
            "suites": [{
              "id": "925de5ce-03ae-4dcb-9146-c956ff3f090d",
              "name": "Default Suite",
              "persistSession": false,
              "parallel": false,
              "timeout": 300,
              "tests": ["5d61ce01-d373-4b14-a1a1-7474a4e192e5"]
            }],
            "urls": ["https://bsscommerce.com/magento-2-one-step-checkout-extension.html"],
            "plugins": []
          }"#
        .as_bytes()
        .to_vec()
    }
}
