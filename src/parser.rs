// TODO: parse basic information
// TODO: custom errors with test name + command + field information where the error occuried
// TODO: do we need Target tag?
// TODO: create a default errors?

use crate::error::ParseError;
use std::result::Result;
use std::time::Duration;

/// Parse [.side format] into rust representation
///
/// [.side format]: https://github.com/SeleniumHQ/selenium-ide/issues/77
pub fn parse<R: std::io::Read>(side_file: R) -> Result<Vec<Test>, ParseError> {
    let side: format::SideFile =
        serde_json::from_reader(side_file).map_err(|err| ParseError::FormatError(err))?;
    let mut tests = Vec::new();
    for test in &side.tests {
        let mut commands = Vec::with_capacity(test.commands.len());
        for command in &test.commands {
            let cmd = parse_cmd(command)?;
            commands.push(cmd);
        }

        tests.push(Test {
            name: test.name.clone(),
            commands,
        });
    }

    Ok(tests)
}

fn parse_cmd(command: &format::Command) -> Result<Command, ParseError> {
    let parse_fn = match command.cmd.as_str() {
        "open" => Command::parse_open,
        "store" => Command::parse_store,
        "storeText" => Command::parse_store_text,
        "executeScript" => Command::parse_execute_script,
        "waitForElementVisible" => Command::parse_wait_for_visible,
        "waitForElementEditable" => Command::parse_wait_for_editable,
        "waitForElementNotPresent" => Command::parse_wait_for_not_present,
        "select" => Command::parse_select,
        "echo" => Command::parse_echo,
        "pause" => Command::parse_pause,
        "click" => Command::parse_click,
        "while" => Command::parse_while,
        "if" => Command::parse_if,
        "else if" => Command::parse_else_if,
        "else" => Command::parse_else,
        "end" => Command::parse_end,
        "setWindowSize" => Command::parse_set_window_size,
        _ => unimplemented!(),
    };

    parse_fn(command)
}

/// The structure represent a selenium test
pub struct Test {
    pub name: String,
    pub commands: Vec<Command>,
}

/// Command corresponds a selenium command
///
/// The list of commands still not completed
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
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
    While(String),
    If(String),
    ElseIf(String),
    Else,
    End,
}

impl Command {
    fn parse_open(c: &format::Command) -> Result<Command, ParseError> {
        Ok(Command::Open(c.target.clone()))
    }

    fn parse_store_text(c: &format::Command) -> Result<Command, ParseError> {
        let targets = parse_targets(&c.targets)?;
        let var = c.value.clone();
        let location = parse_location(&c.target)?;
        let target = Target::new(location);

        Ok(Command::StoreText {
            var,
            target,
            targets,
        })
    }

    fn parse_execute_script(c: &format::Command) -> Result<Command, ParseError> {
        let var = if c.value.is_empty() {
            None
        } else {
            Some(c.value.clone())
        };
        let script = c.target.clone();

        Ok(Command::Execute { script, var })
    }

    fn parse_wait_for_visible(c: &format::Command) -> Result<Command, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Command::WaitForElementVisible { target, timeout })
    }

    fn parse_wait_for_editable(c: &format::Command) -> Result<Command, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Command::WaitForElementEditable { target, timeout })
    }

    fn parse_wait_for_not_present(c: &format::Command) -> Result<Command, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Command::WaitForElementNotPresent { target, timeout })
    }

    fn parse_select(c: &format::Command) -> Result<Command, ParseError> {
        let locator = parse_select_locator(&c.value)?;
        let location = parse_location(&c.target)?;
        let target = Target::new(location);

        Ok(Command::Select { target, locator })
    }

    fn parse_echo(c: &format::Command) -> Result<Command, ParseError> {
        Ok(Command::Echo(c.target.clone()))
    }

    fn parse_while(c: &format::Command) -> Result<Command, ParseError> {
        Ok(Command::While(c.target.clone()))
    }

    fn parse_if(c: &format::Command) -> Result<Command, ParseError> {
        Ok(Command::If(c.target.clone()))
    }

    fn parse_else(_: &format::Command) -> Result<Command, ParseError> {
        Ok(Command::Else)
    }

    fn parse_else_if(c: &format::Command) -> Result<Command, ParseError> {
        Ok(Command::ElseIf(c.target.clone()))
    }

    fn parse_end(_: &format::Command) -> Result<Command, ParseError> {
        Ok(Command::End)
    }

    fn parse_pause(c: &format::Command) -> Result<Command, ParseError> {
        let timeout = cast_timeout(&c.target)?;
        Ok(Command::Pause(timeout))
    }

    fn parse_click(c: &format::Command) -> Result<Command, ParseError> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Command::Click(target))
    }

    fn parse_store(c: &format::Command) -> Result<Command, ParseError> {
        Ok(Command::Store {
            value: c.target.to_owned(),
            var: c.value.to_owned(),
        })
    }

    fn parse_set_window_size(c: &format::Command) -> Result<Command, ParseError> {
        let settings = c.target.split("x").map(|n| n.parse()).collect::<Vec<_>>();
        if settings.len() != 2 {
            Err(ParseError::TypeError(
                "window size expected to get in a form like this 1916x1034 (Width x Height)"
                    .to_owned(),
            ))?
        }

        let w = settings[0]
            .clone()
            .map_err(|_| ParseError::TypeError("expected to get int".to_owned()))?;
        let h = settings[1]
            .clone()
            .map_err(|_| ParseError::TypeError("expected to get int".to_owned()))?;

        Ok(Command::SetWindowSize(w, h))
    }
}

/// Target represents a locator of html element 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    pub location: Location,
    /// tag is an additional information of location type e.g.
    /// location = xpath, tag = Some(relative) | Some(positional) | None
    pub tag: Option<String>,
}

impl Target {
    fn new(location: Location) -> Self {
        Target {
            tag: None,
            location,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectLocator {
    // todo: Looks like we should handle ${} stored values right in parsing stage too?
    Index(String),
}

trait IncompleteStr<T> {
    fn eval(vars: Vec<usize>) -> T;
}

/// Locator of a HTML element
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
            )))?,
        }
    }
}

fn parse_location(text: &str) -> Result<Location, ParseError> {
    let mut target_location = text.splitn(2, '=');
    let location_type = target_location
        .next()
        .ok_or(ParseError::LocatorFormatError(
            "target should contain a type of selector and a selector splited by '='".to_owned(),
        ))?;
    let location = target_location
        .next()
        .ok_or(ParseError::LocatorFormatError(
            "target should contain a type of selector and a selector splited by '='".to_owned(),
        ))?;

    Location::new(location_type, location)
}

fn parse_select_locator(text: &str) -> Result<SelectLocator, ParseError> {
    const ERROR_TEXT: &str = "unexpected type of selector";

    let mut locator = text.splitn(2, '=');
    let locator_type = locator
        .next()
        .ok_or(ParseError::LocatorFormatError(ERROR_TEXT.to_owned()))?;
    let locator = locator
        .next()
        .ok_or(ParseError::LocatorFormatError(ERROR_TEXT.to_owned()))?;

    match locator_type {
        "index" => Ok(SelectLocator::Index(locator.to_owned())),
        _ => Err(ParseError::LocatorFormatError(ERROR_TEXT.to_owned()))?,
    }
}

fn parse_targets(targets: &[Vec<String>]) -> Result<Vec<Target>, ParseError> {
    let mut out = Vec::new();
    for target in targets {
        if target.len() != 2 {
            Err(ParseError::LocatorFormatError(
                "targets wrong format".to_owned(),
            ))?
        }

        let location = parse_location(&target[0])?;
        let tag = parse_target_tag(&target[1])?.to_owned();

        let target = Target {
            location,
            tag: Some(tag),
        };
        out.push(target);
    }

    Ok(out)
}

fn parse_target_tag<'a>(tag: &'a str) -> Result<&'a str, ParseError> {
    tag.splitn(2, ':')
        .nth(1)
        .ok_or(ParseError::LocatorFormatError(
            "type of selector is unknown".to_owned(),
        ))
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
        .map(|timeout| Duration::from_millis(timeout))
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
        pub targets: Vec<Vec<String>>,
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
        let tests = parse(reader);
        assert!(tests.is_ok());
        let tests = tests.unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].commands.len(), 3);
        assert!(matches!(
            tests[0].commands[0],
            Command::Open(ref url) if url == "RELATIVE_URL/"
        ));
        assert!(matches!(
            tests[0].commands[1],
            Command::Pause(ref timeout) if *timeout == Duration::from_secs(5)
        ));
        assert!(matches!(
            tests[0].commands[2],
            Command::Execute {ref script, ref var} if script == "return \"hello world\"" && var == &Some("variable_name".to_string())
        ));
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
        .iter()
        .cloned()
        .collect()
    }
}
