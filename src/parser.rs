// TODO: parse basic information
// TODO: custom errors with test name + command + field information where the error occuried
// TODO: do we need Target tag?
// TODO: create a default errors?

use crate::{error::ParseError, Result};

pub fn parse<R: std::io::Read>(side_file: R) -> Result<Vec<Test>> {
    let side: format::SideFile =
        serde_json::from_reader(side_file).map_err(|err| ParseError::FormatError(err))?;
    let mut tests = Vec::new();
    for test in &side.tests {
        let mut commands = Vec::with_capacity(test.commands.len());
        for command in &test.commands {
            let command = match command.cmd.as_str() {
                "open" => {
                    let url = &command.target;
                    Command::Open(url.clone())
                }
                "storeText" => {
                    let var_name = &command.value;
                    let location = parse_location(&command.target)?;
                    let target = Target {
                        location,
                        tag: None,
                    };

                    let mut targets = Vec::new();
                    for target in &command.targets {
                        let (target, tag) = match target.get(0..2) {
                            Some([target, tag]) => (target, tag),
                            _ => Err(ParseError::LocatorFormatError(
                                "targets wrong format".to_owned(),
                            ))?,
                        };

                        let location = parse_location(&target)?;

                        let tag =
                            tag.splitn(2, ':')
                                .nth(1)
                                .ok_or(ParseError::LocatorFormatError(
                                    "type of selector is unknown".to_owned(),
                                ))?;
                        targets.push(Target {
                            location,
                            tag: Some(tag.to_owned()),
                        })
                    }

                    Command::StoreText {
                        var: var_name.clone(),
                        target,
                        targets,
                    }
                }
                "executeScript" => {
                    let var = if command.value.is_empty() {
                        None
                    } else {
                        Some(command.value.clone())
                    };
                    Command::Execute {
                        script: command.target.clone(),
                        var,
                    }
                }
                "waitForElementVisible" => {
                    let location = parse_location(&command.target)?;
                    let target = Target {
                        tag: None,
                        location,
                    };

                    // TODO: posibly there may be a variable not only a number
                    let timeout = command
                        .value
                        .parse()
                        .map_err(|_| ParseError::TypeError("expected to get int".to_owned()))?;

                    Command::WaitForElementVisible { target, timeout }
                }
                "waitForElementEditable" => {
                    let location = parse_location(&command.target)?;

                    let target = Target {
                        tag: None,
                        location,
                    };

                    let timeout = command
                        .value
                        .parse()
                        .map_err(|_| ParseError::TypeError("expected to get int".to_owned()))?;

                    Command::WaitForElementEditable { target, timeout }
                }
                "select" => {
                    let locator = parse_select_locator(&command.value)?;
                    let location = parse_location(&command.target)?;
                    let target = Target {
                        tag: None,
                        location,
                    };

                    Command::Select { target, locator }
                }
                "echo" => Command::Echo(command.target.clone()),
                _ => unimplemented!(),
            };

            commands.push(command);
        }

        tests.push(Test {
            name: test.name.clone(),
            commands,
        });
    }

    Ok(tests)
}

pub struct Test {
    pub name: String,
    pub commands: Vec<Command>,
}

#[derive(Debug)]
pub enum Command {
    Open(String),
    Echo(String),
    // todo: targets?
    Select {
        target: Target,
        locator: SelectLocator,
    },
    WaitForElementVisible {
        target: Target,
        timeout: u64,
    },
    WaitForElementEditable {
        target: Target,
        timeout: u64,
    },
    StoreText {
        var: String,
        target: Target,
        targets: Vec<Target>,
    },
    Execute {
        script: String,
        var: Option<String>,
    },
    // While(String),
    // End,
}

#[derive(Debug)]
pub struct Target {
    pub location: Location,
    pub tag: Option<String>,
}

#[derive(Debug)]
pub enum SelectLocator {
    // todo: Looks like we should handle ${} stored values right in parsing stage too?
    Index(String),
}

trait IncompleteStr<T> {
    fn eval(vars: Vec<usize>) -> T;
}

#[derive(Debug)]
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
