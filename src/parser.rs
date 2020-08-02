// TODO: parse basic information
// TODO: custom errors with test name + command + field information where the error occuried
// TODO: do we need Target tag?
// TODO: create a default errors?

use crate::{error::ParseError, Result};
use std::result;
use std::time::Duration;

pub fn parse<R: std::io::Read>(side_file: R) -> Result<Vec<Test>> {
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

fn parse_cmd(command: &format::Command) -> Result<Command> {
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

pub struct Test {
    pub name: String,
    pub commands: Vec<Command>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Open(String),
    Echo(String),
    Click(Target),
    Pause(Duration),
    SetWindowSize(usize, usize),
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
    fn parse_open(c: &format::Command) -> Result<Command> {
        Ok(Command::Open(c.target.clone()))
    }

    fn parse_store_text(c: &format::Command) -> Result<Command> {
        let mut targets = Vec::new();
        for target in &c.targets {
            let (target, tag) = match target.get(0..2) {
                Some([target, tag]) => (target, tag),
                _ => Err(ParseError::LocatorFormatError(
                    "targets wrong format".to_owned(),
                ))?,
            };

            let location = parse_location(&target)?;

            let tag = tag
                .splitn(2, ':')
                .nth(1)
                .ok_or(ParseError::LocatorFormatError(
                    "type of selector is unknown".to_owned(),
                ))?;
            targets.push(Target {
                location,
                tag: Some(tag.to_owned()),
            })
        }
        let var = c.value.clone();
        let location = parse_location(&c.target)?;
        let target = Target::new(location);

        Ok(Command::StoreText {
            var,
            target,
            targets,
        })
    }

    fn parse_execute_script(c: &format::Command) -> Result<Command> {
        let var = if c.value.is_empty() {
            None
        } else {
            Some(c.value.clone())
        };
        let script = c.target.clone();

        Ok(Command::Execute { script, var })
    }

    fn parse_wait_for_visible(c: &format::Command) -> Result<Command> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Command::WaitForElementVisible { target, timeout })
    }

    fn parse_wait_for_editable(c: &format::Command) -> Result<Command> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Command::WaitForElementEditable { target, timeout })
    }

    fn parse_wait_for_not_present(c: &format::Command) -> Result<Command> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        let timeout = cast_timeout(&c.value)?;

        Ok(Command::WaitForElementNotPresent { target, timeout })
    }

    fn parse_select(c: &format::Command) -> Result<Command> {
        let locator = parse_select_locator(&c.value)?;
        let location = parse_location(&c.target)?;
        let target = Target::new(location);

        Ok(Command::Select { target, locator })
    }

    fn parse_echo(c: &format::Command) -> Result<Command> {
        Ok(Command::Echo(c.target.clone()))
    }

    fn parse_while(c: &format::Command) -> Result<Command> {
        Ok(Command::While(c.target.clone()))
    }

    fn parse_if(c: &format::Command) -> Result<Command> {
        Ok(Command::If(c.target.clone()))
    }

    fn parse_else(_: &format::Command) -> Result<Command> {
        Ok(Command::Else)
    }

    fn parse_else_if(c: &format::Command) -> Result<Command> {
        Ok(Command::ElseIf(c.target.clone()))
    }

    fn parse_end(_: &format::Command) -> Result<Command> {
        Ok(Command::End)
    }

    fn parse_pause(c: &format::Command) -> Result<Command> {
        let timeout = cast_timeout(&c.target)?;
        Ok(Command::Pause(timeout))
    }

    fn parse_click(c: &format::Command) -> Result<Command> {
        let location = parse_location(&c.target)?;
        let target = Target::new(location);
        Ok(Command::Click(target))
    }

    fn parse_store(c: &format::Command) -> Result<Command> {
        Ok(Command::Store {
            value: c.target.to_owned(),
            var: c.value.to_owned(),
        })
    }

    fn parse_set_window_size(c: &format::Command) -> Result<Command> {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    pub location: Location,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Location {
    XPath(String),
    Css(String),
    Id(String),
}

impl Location {
    fn new(tp: &str, path: &str) -> Result<Self> {
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

fn parse_location(text: &str) -> Result<Location> {
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

fn parse_select_locator(text: &str) -> Result<SelectLocator> {
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

fn cast_timeout(s: &str) -> result::Result<Duration, ParseError> {
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

pub mod format {
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
