// TODO: Mock webdriver
// TODO: interface for fantocini and possibly choose webdriver provider by feature
// TODO: provide more direct error location test + command + location(can be determined just by section (target/value etc.)) + cause
// TODO: Runner may contains basic information to handle relative url

use crate::{error::SideRunnerError, Command, Location, Result, SelectLocator, Test};
use fantoccini::{Client, Locator};
use serde_json::Value;
use std::collections::HashMap;

pub struct Runner<'driver> {
    webdriver: &'driver mut Client,
    pub data: HashMap<String, Value>,
}

impl<'driver> Runner<'driver> {
    pub fn new(client: &'driver mut Client) -> Self {
        Self {
            webdriver: client,
            data: HashMap::new(),
        }
    }

    pub async fn run(&mut self, test: &Test) -> Result<()> {
        crate::validation::validate_conditions(&test.commands)?;
        let nodes = create_nodes(&test.commands);
        // for node in nodes {
        //     println!("{:?}", node);
        // }
        // return Ok(());

        if nodes.is_empty() {
            return Ok(());
        }

        let mut i = 0;
        loop {
            if i >= nodes.len() {
                break;
            }

            let run = &nodes[i];
            match run.next {
                Some(NodeTransition::Next(next)) => {
                    i = next;
                    let cmd = &run.command;
                    self.run_command(cmd).await?;
                }
                Some(NodeTransition::Conditional(next, or_else)) => {
                    let condition = match &run.command {
                        Command::While(cond) => cond,
                        Command::ElseIf(cond) => cond,
                        Command::If(cond) => cond,
                        _ => unreachable!(),
                    };

                    let script = format!("return {}", condition);
                    let res = self.exec(&script).await?;
                    println!("{}", res);
                    match res.as_bool() {
                        Some(b) => {
                            if b {
                                i = next;
                            } else {
                                i = or_else;
                            }
                        }
                        None => Err(SideRunnerError::MismatchedType(
                            "expected boolean type in condition".to_owned(),
                        ))?,
                    }
                }
                None => unreachable!(),
            };
        }

        Ok(())
    }

    async fn run_command(&mut self, cmd: &Command) -> Result<()> {
        // TODO: emit variables in value field too
        match cmd {
            Command::Open(url) => {
                self.webdriver.goto(url).await?;
                let url = self.webdriver.current_url().await?;
                assert_eq!(url.as_ref(), url.as_ref());
            }
            Command::StoreText {
                var,
                target,
                targets,
            } => {
                let location = match &target.location {
                    Location::Css(css) => {
                        Location::Css(emit_variables::<_, PlainPrinter>(css, &self.data))
                    }
                    Location::Id(id) => {
                        Location::Id(emit_variables::<_, PlainPrinter>(id, &self.data))
                    }
                    Location::XPath(path) => {
                        Location::XPath(emit_variables::<_, PlainPrinter>(path, &self.data))
                    }
                };

                let locator = match &location {
                    Location::Css(css) => Locator::Css(&css),
                    Location::Id(id) => Locator::Id(&id),
                    Location::XPath(path) => Locator::XPath(&path),
                };

                let value = self.webdriver.find(locator).await?.text().await?;

                let value = Value::String(value);
                self.data.insert(var.clone(), value);

                // TODO: if `target` not found we should look up targets?
            }
            Command::Execute { script, var } => {
                // TODO: the logic is different from Selenium IDE
                // If the element is not loaded on the page IDE will fail not emidiately but our implementation will.
                // they might wait a little bit or something but there's something there

                let res = self.exec(script).await?;
                match var {
                    Some(var) => {
                        self.data.insert(var.clone(), res);
                    }
                    None => (),
                }
            }
            Command::Echo(text) => {
                let text = emit_variables::<_, PlainPrinter>(text, &self.data);
                println!("{}", text);
            }
            Command::WaitForElementVisible { timeout, target } => {
                // todo: implemented wrongly
                // it's implmenetation more suited for WaitForElementPresent
                //
                // TODO: timout implementation is a bit wrong since we need to 'gracefully' stop running feature
                let locator = match &target.location {
                    Location::Css(css) => Locator::Css(&css),
                    Location::Id(id) => Locator::Id(&id),
                    Location::XPath(path) => Locator::XPath(&path),
                };

                let timeout = std::time::Duration::from_millis(*timeout);

                match tokio::time::timeout(timeout, self.webdriver.wait_for_find(locator)).await {
                    Ok(Err(..)) => println!("Error"),
                    Ok(..) => (),
                    Err(..) => println!("timeout"),
                }
            }
            Command::WaitForElementEditable { timeout, target } => {
                std::thread::sleep_ms(10000);
                // TODO: #issue https://github.com/jonhoo/fantoccini/issues/93
            }
            Command::Select { locator, target } => {
                let select_locator = match &target.location {
                    Location::Css(css) => Locator::Css(&css),
                    Location::Id(id) => Locator::Id(&id),
                    Location::XPath(path) => Locator::XPath(&path),
                };

                let select = self.webdriver.find(select_locator).await?;
                match locator {
                    SelectLocator::Index(index) => {
                        let index = emit_variables::<_, PlainPrinter>(index, &self.data);
                        match index.parse() {
                            Ok(index) => select.select_by_index(index).await?,
                            // TODO: IlligalSyntax  Failed: Illegal Index: {version_counter}
                            Err(..) => Err(SideRunnerError::MismatchedType(format!(
                                "expected to get int type but got {:?}",
                                index
                            )))?,
                        }
                    }
                };
            }
            Command::While(condition) => {
                let res = self.exec(condition).await?;
                match res.as_bool() {
                    Some(true) => {}
                    Some(false) => {}
                    None => Err(SideRunnerError::MismatchedType(
                        "unexpected conditional expression".to_owned(),
                    ))?,
                }
            }
            _ => (),
        };

        Ok(())
    }
    // argument[0] -> argument[1] -> argument[2] goes to implementing JS formatting

    async fn exec(
        &mut self,
        script: &str,
    ) -> std::result::Result<serde_json::Value, fantoccini::error::CmdError> {
        let (script, used_vars) = emit_variables_custom(script, &self.data);
        let args = used_vars.iter().map(|var| self.data[var].clone()).collect();
        self.webdriver
            .execute(
                &format!("return (function(arguments) {{ {} }})(arguments)", script),
                args,
            )
            .await
    }
}

fn emit_variables<S: AsRef<str>, P: VarPrinter>(
    text: S,
    variables: &HashMap<String, Value>,
) -> String {
    // TODO: check how to emit string in quotes or not
    //
    // regex look up for variable name in brackets #{var}
    // it exclude " sign to manage cases like ${var} }
    // it's important in emiting vars in JSON
    let re = regex::Regex::new(r#"\$\{(\w+?)\}"#).unwrap();
    let replacer = VarReplacer::<P> {
        data: variables,
        printer: std::marker::PhantomData::default(),
    };

    let new_text = re.replace_all(text.as_ref(), replacer);

    new_text.into()
}

fn emit_variables_custom<S: AsRef<str>>(
    text: S,
    variables: &HashMap<String, Value>,
) -> (String, Vec<String>) {
    // TODO: check how to emit string in quotes or not
    //
    // regex look up for variable name in brackets #{var}
    // it exclude " sign to manage cases like ${var} }
    // it's important in emiting vars in JSON
    //
    // https://github.com/SeleniumHQ/selenium-ide/blob/dd0c8ce313171672d2f0670cfb05786611f85b73/packages/side-runtime/src/preprocessors.js#L119
    let re = regex::Regex::new(r#"\$\{(.*?)\}"#).unwrap();
    let mut replacer = PuttingArg {
        emited_vars: HashMap::new(),
        index: 0,
    };

    let new_text = re.replace_all(text.as_ref(), &mut replacer);

    let count_positions = replacer.index;
    let mut vars = Vec::new();
    for i in 0..count_positions {
        vars.push(replacer.emited_vars[&i].clone());
    }

    (new_text.into(), vars)
}

struct PuttingArg {
    emited_vars: HashMap<usize, String>,
    index: usize,
}

// TODO: clean this up.
// it's library dependent ?
impl regex::Replacer for &mut PuttingArg {
    fn replace_append(&mut self, caps: &regex::Captures, dst: &mut String) {
        let var = caps.get(1).unwrap().as_str();

        let index = if let Some((pos, _)) = self.emited_vars.iter().find(|(_, v)| v.as_str() == var)
        {
            *pos
        } else {
            let index = self.index;
            self.emited_vars.insert(index, var.to_owned());
            self.index += 1;
            index
        };
        let replacement = format!("arguments[{}]", index);

        dst.push_str(replacement.as_str());
    }
}

struct VarReplacer<'a, P: VarPrinter> {
    data: &'a HashMap<String, Value>,
    printer: std::marker::PhantomData<P>,
}

trait VarPrinter {
    fn print(val: &Value) -> String;
}

struct JSPrinter {}

impl VarPrinter for JSPrinter {
    fn print(val: &Value) -> String {
        val.to_string()
    }
}

struct PlainPrinter {}

impl VarPrinter for PlainPrinter {
    fn print(val: &Value) -> String {
        match val {
            Value::String(val) => val.clone(),
            Value::Null => "".to_string(),
            Value::Number(val) => val.to_string(),
            Value::Object(..) => "[object Object]".to_string(), // is it ok behaviour?
            Value::Array(values) => values
                .into_iter()
                .map(|v| Self::print(v))
                .collect::<Vec<_>>()
                .join(","),
            Value::Bool(val) => val.to_string(),
        }
    }
}

impl<P: VarPrinter> regex::Replacer for VarReplacer<'_, P> {
    fn replace_append(&mut self, caps: &regex::Captures, dst: &mut String) {
        let var = caps.get(1).unwrap().as_str();
        eprintln!("{}", var);
        let replacement = match self.data.get(var) {
            Some(value) => P::print(value),
            None => "".to_string(),
        };

        dst.push_str(replacement.as_str());
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CommandNode {
    command: Command,
    index: usize,
    level: usize,
    next: Option<NodeTransition>,
}

impl CommandNode {
    fn new(cmd: Command, index: usize, level: usize, transition: Option<NodeTransition>) -> Self {
        Self {
            command: cmd,
            index,
            level,
            next: transition,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum NodeTransition {
    Next(usize),
    Conditional(usize, usize),
}

fn create_nodes(commands: &[Command]) -> Vec<CommandNode> {
    let levels = compute_levels(commands);
    let mut nodes = commands
        .iter()
        .zip(levels)
        .enumerate()
        .map(|(index, (cmd, lvl))| CommandNode::new(cmd.clone(), index, lvl, None))
        .collect::<Vec<_>>();
    let mut state = Vec::new();
    (0..nodes.len()).for_each(|i| {
        connect_commands(&mut nodes[i..], &mut state);
    });

    nodes
}

// find a coresponding END
// make this END's next pointed to the while // OR DO IT WITH THE ELEMENT BEFORE END
// make while's end on END+1 element
// make while's beggining on the next element

//     Command::If(..) => {
//         // find a coresponding END
//         // find a else/else if structures
//         // DON'T AFRAID TO MAKE SOMETHING INEFFICHIENT FROM SCRATCH. THAT'S FINE.

fn connect_commands(cmds: &mut [CommandNode], state: &mut Vec<(&'static str, usize)>) {
    match cmds[0].command {
        Command::While(..) => {
            let while_end = find_next_end_on_level(&cmds[1..], cmds[0].level).unwrap();
            cmds[0].next = Some(NodeTransition::Conditional(
                cmds[1].index,
                while_end.index + 1,
            ));
            state.push(("while", cmds[0].index));
        }
        Command::If(..) => {
            let if_next = find_next_on_level(&cmds[1..], cmds[0].level).unwrap();
            let cond_end = find_next_end_on_level(&cmds[1..], cmds[0].level).unwrap();
            state.push(("if", cond_end.index));
            cmds[0].next = Some(NodeTransition::Conditional(cmds[1].index, if_next.index));
        }
        Command::ElseIf(..) => {
            let elseif_end = find_next_on_level(&cmds[1..], cmds[0].level).unwrap();
            cmds[0].next = Some(NodeTransition::Conditional(cmds[1].index, elseif_end.index));
        }
        Command::Else => {
            cmds[0].next = Some(NodeTransition::Next(cmds[1].index));
        }
        Command::End => match state.last() {
            Some(("while", index)) => {
                cmds[0].next = Some(NodeTransition::Next(*index));
                state.pop();
            }
            Some(("if", _)) => {
                state.pop();
                cmds[0].next = Some(NodeTransition::Next(cmds[0].index + 1));
            }
            _ => {
                cmds[0].next = Some(NodeTransition::Next(cmds[0].index + 1));
            }
        },
        _ => {
            if cmds.len() > 1 {
                match cmds[1].command {
                    // Command::End => match state.pop() {
                    //     Some(("if", ..)) => {
                    //         // state.pop();
                    //         cmds[0].next = Some(NodeTransition::Next(cmds[1].index));
                    //     }
                    //     _ => {
                    //         cmds[0].next = Some(NodeTransition::Next(cmds[1].index));
                    //     }
                    // },
                    Command::Else => {
                        let (_, index) = state.pop().unwrap();
                        cmds[0].next = Some(NodeTransition::Next(index));
                    }
                    Command::ElseIf(..) => {
                        let (_, index) = state.last().unwrap();
                        cmds[0].next = Some(NodeTransition::Next(*index));
                    }
                    _ => {
                        cmds[0].next = Some(NodeTransition::Next(cmds[1].index));
                    }
                };
            } else {
                cmds[0].next = Some(NodeTransition::Next(cmds[0].index + 1));
            }
        }
    }
}

fn index_or_none(index: usize, cmds: &[CommandNode]) -> Option<usize> {
    if index > cmds.len() {
        None
    } else {
        Some(index)
    }
}

fn find_next<Cmp: Fn(&CommandNode) -> bool>(
    commands: &[CommandNode],
    comparator: Cmp,
) -> Option<&CommandNode> {
    for cmd in commands {
        if comparator(cmd) {
            return Some(cmd);
        }
    }

    None
}

fn find_next_on_level(commands: &[CommandNode], level: usize) -> Option<&CommandNode> {
    find_next(commands, |cmd| cmd.level == level)
}

fn find_next_end_on_level(commands: &[CommandNode], level: usize) -> Option<&CommandNode> {
    find_next(commands, |cmd| {
        cmd.level == level && matches!(cmd.command, Command::End)
    })
}

fn compute_levels(commands: &[Command]) -> Vec<usize> {
    let mut level = 0;
    let mut levels = Vec::with_capacity(commands.len());
    for cmd in commands {
        match cmd {
            Command::While(..) | Command::If(..) => {
                levels.push(level);
                level += 1;
            }
            Command::End => {
                level -= 1;
                levels.push(level);
            }
            Command::Else | Command::ElseIf(..) => {
                level -= 1;
                levels.push(level);
                level += 1;
            }
            _ => {
                levels.push(level);
            }
        }
    }

    levels
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

        assert_eq!(
            "\"Hello\"",
            emit_variables::<_, JSPrinter>("${hello}", &vars)
        );
        assert_eq!(
            "\"Hello\" \"World\"!",
            emit_variables::<_, JSPrinter>("${hello} ${world}!", &vars)
        );
        assert_eq!(
            "There are no vars here",
            emit_variables::<_, JSPrinter>("There are no vars here", &vars)
        );
        assert_eq!(
            "${\"XXX\"}",
            emit_variables::<_, JSPrinter>("${${something}}", &vars)
        );

        assert_eq!(
            "\"\"World\"\"",
            emit_variables::<_, JSPrinter>("\"${world}\"", &vars)
        );
        assert_eq!(
            "\"World\"\" }",
            emit_variables::<_, JSPrinter>("${world}\" }", &vars)
        );
        assert_eq!(
            "\"World\"\"}",
            emit_variables::<_, JSPrinter>("${world}\"}", &vars)
        );
        assert_eq!(
            "\"World\" }",
            emit_variables::<_, JSPrinter>("${world} }", &vars)
        );
        assert_eq!(
            "\"World\"}",
            emit_variables::<_, JSPrinter>("${world}}", &vars)
        );
    }

    #[test]
    fn test_emit_variables_types() {
        let mut vars = HashMap::new();

        vars.insert("test".to_string(), json!("string"));
        assert_eq!(
            r#""string""#,
            emit_variables::<_, JSPrinter>("${test}", &vars)
        );

        vars.insert("test".to_string(), json!(2));
        assert_eq!("2", emit_variables::<_, JSPrinter>("${test}", &vars));

        vars.insert("test".to_string(), json!({"h3": 3}));
        assert_eq!(
            r#"{"h3":3}"#,
            emit_variables::<_, JSPrinter>("${test}", &vars)
        );

        vars.insert("test".to_string(), json!(["h4", 4]));
        assert_eq!(
            r#"["h4",4]"#,
            emit_variables::<_, JSPrinter>("${test}", &vars)
        );
    }

    #[test]
    fn test_creating_run_list_basic() {
        let commands = vec![
            Command::Open("open".to_owned()),
            Command::Echo("echo".to_owned()),
        ];
        let node = create_nodes(&commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::Open("open".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Next(1)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    1,
                    0,
                    Some(NodeTransition::Next(2))
                )
            ]
        )
    }

    #[test]
    fn test_creating_run_list_while_loop() {
        let mut commands = vec![
            Command::Open("open".to_owned()),
            Command::While("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::End,
        ];
        let node = create_nodes(&mut commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::Open("open".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Next(1)),
                ),
                CommandNode::new(
                    Command::While("...".to_owned()),
                    1,
                    0,
                    Some(NodeTransition::Conditional(2, 4)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Next(3)),
                ),
                CommandNode::new(Command::End, 3, 0, Some(NodeTransition::Next(1))),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_if() {
        let mut commands = vec![
            Command::Open("open".to_owned()),
            Command::If("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::End,
            Command::Echo("echo".to_owned()),
        ];
        let node = create_nodes(&mut commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::Open("open".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Next(1)),
                ),
                CommandNode::new(
                    Command::If("...".to_owned()),
                    1,
                    0,
                    Some(NodeTransition::Conditional(2, 3)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Next(3)),
                ),
                CommandNode::new(Command::End, 3, 0, Some(NodeTransition::Next(4))),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    4,
                    0,
                    Some(NodeTransition::Next(5))
                ),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_if_complex_empty_conditions() {
        let mut commands = vec![
            Command::Open("open".to_owned()),
            Command::If("...".to_owned()),
            Command::ElseIf("...".to_owned()),
            Command::Else,
            Command::End,
        ];
        let node = create_nodes(&mut commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::Open("open".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Next(1)),
                ),
                CommandNode::new(
                    Command::If("...".to_owned()),
                    1,
                    0,
                    Some(NodeTransition::Conditional(2, 2)),
                ),
                CommandNode::new(
                    Command::ElseIf("...".to_owned()),
                    2,
                    0,
                    Some(NodeTransition::Conditional(3, 3)),
                ),
                CommandNode::new(Command::Else, 3, 0, Some(NodeTransition::Next(4)),),
                CommandNode::new(Command::End, 4, 0, Some(NodeTransition::Next(5))),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_if_complex() {
        let mut commands = vec![
            Command::Open("open".to_owned()),
            Command::If("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::ElseIf("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::Else,
            Command::Echo("echo".to_owned()),
            Command::End,
        ];
        let node = create_nodes(&mut commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::Open("open".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Next(1)),
                ),
                CommandNode::new(
                    Command::If("...".to_owned()),
                    1,
                    0,
                    Some(NodeTransition::Conditional(2, 3)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Next(8)),
                ),
                CommandNode::new(
                    Command::ElseIf("...".to_owned()),
                    3,
                    0,
                    Some(NodeTransition::Conditional(4, 6)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    4,
                    1,
                    Some(NodeTransition::Next(5)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    5,
                    1,
                    Some(NodeTransition::Next(8)),
                ),
                CommandNode::new(Command::Else, 6, 0, Some(NodeTransition::Next(7))),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    7,
                    1,
                    Some(NodeTransition::Next(8))
                ),
                CommandNode::new(Command::End, 8, 0, Some(NodeTransition::Next(9))),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_if_complex_without_else() {
        let mut commands = vec![
            Command::Open("open".to_owned()),
            Command::If("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::ElseIf("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::End,
        ];
        let node = create_nodes(&mut commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::Open("open".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Next(1)),
                ),
                CommandNode::new(
                    Command::If("...".to_owned()),
                    1,
                    0,
                    Some(NodeTransition::Conditional(2, 3)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Next(5)),
                ),
                CommandNode::new(
                    Command::ElseIf("...".to_owned()),
                    3,
                    0,
                    Some(NodeTransition::Conditional(4, 5)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    4,
                    1,
                    Some(NodeTransition::Next(5))
                ),
                CommandNode::new(Command::End, 5, 0, Some(NodeTransition::Next(6))),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_multi_while() {
        let mut commands = vec![
            Command::Open("open".to_owned()),
            Command::While("...".to_owned()),
            Command::While("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::End,
            Command::End,
        ];
        let node = create_nodes(&mut commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::Open("open".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Next(1)),
                ),
                CommandNode::new(
                    Command::While("...".to_owned()),
                    1,
                    0,
                    Some(NodeTransition::Conditional(2, 6)),
                ),
                CommandNode::new(
                    Command::While("...".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Conditional(3, 5)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    3,
                    2,
                    Some(NodeTransition::Next(4)),
                ),
                CommandNode::new(Command::End, 4, 1, Some(NodeTransition::Next(2))),
                CommandNode::new(Command::End, 5, 0, Some(NodeTransition::Next(1))),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_multi_while_with_if() {
        let mut commands = vec![
            Command::Open("open".to_owned()),
            Command::While("...".to_owned()),
            Command::While("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::If("...".to_owned()),
            Command::Else,
            Command::End,
            Command::End,
            Command::End,
        ];
        let node = create_nodes(&mut commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::Open("open".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Next(1)),
                ),
                CommandNode::new(
                    Command::While("...".to_owned()),
                    1,
                    0,
                    Some(NodeTransition::Conditional(2, 9)),
                ),
                CommandNode::new(
                    Command::While("...".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Conditional(3, 8)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    3,
                    2,
                    Some(NodeTransition::Next(4)),
                ),
                CommandNode::new(
                    Command::If("...".to_owned()),
                    4,
                    2,
                    Some(NodeTransition::Conditional(5, 5)),
                ),
                CommandNode::new(Command::Else, 5, 2, Some(NodeTransition::Next(6)),),
                CommandNode::new(Command::End, 6, 2, Some(NodeTransition::Next(7))),
                CommandNode::new(Command::End, 7, 1, Some(NodeTransition::Next(2))),
                CommandNode::new(Command::End, 8, 0, Some(NodeTransition::Next(1))),
            ]
        )
    }
}
