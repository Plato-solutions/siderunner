// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// TODO: Mock webdriver
// TODO: interface for fantocini and possibly choose webdriver provider by feature
// TODO: provide more direct error location test + command + location(can be determined just by section (target/value etc.)) + cause
// TODO: Runner may contains basic information to handle relative url
// TODO: refactoring and test While and If commits
// TODO: hide hook after feature flag + add a sleep statistic hook

use crate::webdriver::{self, Locator, Webdriver};
use crate::{
    error::{RunnerError, RunnerErrorKind},
    parser::{Command, Location, SelectLocator, Test},
};
use serde_json::Value;
use std::collections::HashMap;

/// A runtime for running test
///
/// It runs commands and controls program flow(manages conditions and loops).
/// It manages usage of variables.
pub struct Runner<D> {
    webdriver: D,
    pub data: HashMap<String, Value>,
    echo_hook: Box<fn(&str)>,
}

impl<D> Runner<D> {
    /// Create a new runner
    pub fn _new(client: D) -> Runner<D> {
        Self {
            webdriver: client,
            data: HashMap::new(),
            echo_hook: Box::new(|s| println!("{}", s)),
        }
    }
}

impl<D, E> Runner<D>
where
    D: Webdriver<Element = E, Error = RunnerErrorKind>,
    E: webdriver::Element<Driver = D, Error = RunnerErrorKind>,
{
    /// Runs a test
    pub async fn run(&mut self, test: &Test) -> Result<(), RunnerError> {
        crate::validation::validate_conditions(&test.commands)?;
        let mut nodes = create_nodes(&test.commands);
        connect_nodes(&mut nodes);

        self.run_nodes(nodes).await
    }

    async fn run_nodes(&mut self, nodes: Vec<CommandNode>) -> Result<(), RunnerError> {
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
                    self.run_command(cmd)
                        .await
                        .map_err(|e| RunnerError::new(e, run.index))?;
                }
                Some(NodeTransition::Conditional(next, or_else)) => {
                    let condition = match &run.command {
                        Command::While(cond) => cond,
                        Command::ElseIf(cond) => cond,
                        Command::If(cond) => cond,
                        Command::RepeatIf(cond) => cond,
                        _ => unreachable!("unexpected condition"),
                    };

                    let script = format!("return {}", condition);
                    let res = self
                        .exec(&script)
                        .await
                        .map_err(|e| RunnerError::new(e, run.index))?;
                    match res.as_bool() {
                        Some(b) => {
                            if b {
                                i = next;
                            } else {
                                i = or_else;
                            }
                        }
                        None => {
                            return Err(RunnerError::new(
                                RunnerErrorKind::MismatchedType(
                                    "expected boolean type in condition".to_owned(),
                                ),
                                run.index,
                            ))
                        }
                    }
                }
                None => unreachable!(),
            };
        }

        Ok(())
    }

    async fn run_command(&mut self, cmd: &Command) -> Result<(), RunnerErrorKind> {
        // TODO: emit variables in value field too
        match cmd {
            Command::Open(url) => {
                // todo: unify emiting variables
                let url = self.emit(url);

                self.webdriver.goto(&url).await?;
                let url = self.webdriver.current_url().await?;
                assert_eq!(url.as_ref(), url.as_ref());
            }
            Command::StoreText { var, target, .. } => {
                let location = match &target.location {
                    // TODO: get back to the privious variant with IncompleteString.
                    Location::Css(css) => Location::Css(self.emit(css)),
                    Location::Id(id) => Location::Id(self.emit(id)),
                    Location::XPath(path) => Location::XPath(self.emit(path)),
                };
                let locator = match &location {
                    Location::Css(css) => Locator::Css(css.to_string()),
                    Location::Id(id) => Locator::Id(id.to_string()),
                    Location::XPath(path) => Locator::XPath(path.to_string()),
                };

                let value = self.webdriver.find(locator).await?.text().await?;

                let value = Value::String(value);
                self.data.insert(var.clone(), value);

                // TODO: if `target` not found we should look up targets?
            }
            Command::Store { var, value } => {
                self.data.insert(var.clone(), Value::String(value.clone()));
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
                let text = self.emit(text);
                // TODO: create a hook in library to call as a writer
                self.echo_hook.as_ref()(text.as_str());
            }
            Command::WaitForElementVisible { timeout, .. } => {
                // todo: implemented wrongly
                // it's implmenetation more suited for WaitForElementPresent
                //
                // TODO: timout implementation is a bit wrong since we need to 'gracefully' stop running feature
                // let locator = match &target.location {
                //     Location::Css(css) => Locator::Css(&css),
                //     Location::Id(id) => Locator::Id(&id),
                //     Location::XPath(path) => Locator::XPath(&path),
                // };

                // match tokio::time::timeout(*timeout, self.webdriver.wait_for_find(locator)).await {
                //     Ok(err) => {
                //         err?;
                //     }
                //     Err(..) => Err(SideRunnerError::Timeout(
                //         "waitForElemementVisible".to_string(),
                //     ))?,
                // }

                std::thread::sleep(*timeout);
            }
            Command::WaitForElementPresent { timeout, target } => {
                let locator = match &target.location {
                    Location::Css(css) => Locator::Css(css.to_string()),
                    Location::Id(id) => Locator::Id(id.to_string()),
                    Location::XPath(path) => Locator::XPath(path.to_string()),
                };

                self.webdriver
                    .wait_for_present(locator, *timeout)
                    .await
                    .map_err(|_| RunnerErrorKind::Timeout("WaitForElementPresent".to_owned()))?;
            }
            Command::WaitForElementNotPresent { timeout, target } => {
                let locator = match &target.location {
                    Location::Css(css) => Locator::Css(css.to_string()),
                    Location::Id(id) => Locator::Id(id.to_string()),
                    Location::XPath(path) => Locator::XPath(path.to_string()),
                };

                self.webdriver
                    .wait_for_not_present(locator, *timeout)
                    .await
                    .map_err(|_| RunnerErrorKind::Timeout("WaitForElementNotPresent".to_owned()))?;
            }
            Command::WaitForElementEditable { timeout, target } => {
                let locator = match &target.location {
                    Location::Css(css) => Locator::Css(css.to_string()),
                    Location::Id(id) => Locator::Id(id.to_string()),
                    Location::XPath(path) => Locator::XPath(path.to_string()),
                };

                self.webdriver
                    .wait_for_editable(locator, *timeout)
                    .await
                    .map_err(|_| RunnerErrorKind::Timeout("WaitForElementEditable".to_owned()))?;
            }
            Command::Select { locator, target } => {
                let select_locator = match &target.location {
                    Location::Css(css) => Locator::Css(css.to_string()),
                    Location::Id(id) => Locator::Id(id.to_string()),
                    Location::XPath(path) => Locator::XPath(path.to_string()),
                };

                let mut select = self.webdriver.find(select_locator).await?;
                match locator {
                    SelectLocator::Index(index) => {
                        let index = self.emit(index);
                        match index.parse() {
                            Ok(index) => {
                                select.select_by_index(index).await?;
                            }
                            // TODO: IlligalSyntax  Failed: Illegal Index: {version_counter}
                            Err(..) => {
                                return Err(RunnerErrorKind::MismatchedType(format!(
                                    "expected to get int type but got {:?}",
                                    index
                                )))?
                            }
                        }
                    }
                    SelectLocator::Value(value) => {
                        let value = self.emit(value);
                        select.select_by_value(&value).await?;
                    }
                    SelectLocator::Id(id) => {
                        let id = self.emit(id);
                        let locator = format!(r#"option[id='{}']"#, id);
                        select.find(Locator::Css(locator)).await?.click().await?;
                    }
                    SelectLocator::Label(label) => {
                        let label = self.emit(label);
                        let locator = format!(r#".//option[normalize-space(.)='{}']"#, label);
                        select.find(Locator::XPath(locator)).await?.click().await?;
                    }
                };
            }
            Command::Click(target) => {
                let location = match &target.location {
                    Location::Css(css) => Location::Css(self.emit(css)),
                    Location::Id(id) => Location::Id(self.emit(id)),
                    Location::XPath(path) => Location::XPath(self.emit(path)),
                };

                let locator = match &location {
                    Location::Css(css) => Locator::Css(css.to_string()),
                    Location::Id(id) => Locator::Id(id.to_string()),
                    Location::XPath(path) => Locator::XPath(path.to_string()),
                };

                self.webdriver.find(locator).await?.click().await?;
            }
            Command::Pause(timeout) => {
                tokio::time::sleep(*timeout).await;
            }
            Command::SetWindowSize(w, h) => {
                self.webdriver.set_window_size(*w, *h).await?;
            }
            Command::StoreXpathCount { var, xpath } => {
                let locator = Locator::XPath(xpath.clone());
                let elements = self.webdriver.find_all(locator).await?;
                self.data.insert(var.clone(), elements.len().into());
            }
            Command::Close => self.webdriver.close().await?,
            #[allow(unused_variables)]
            cmd => {} // CAN BE AN END command at least if we panic here there will be PRODUCED A WEARD ERORR such as Box<Any>...
        };

        Ok(())
    }
    // argument[0] -> argument[1] -> argument[2] goes to implementing JS formatting

    pub fn set_echo(&mut self, func: fn(&str)) {
        self.echo_hook = Box::new(func);
    }

    async fn exec(
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

    fn emit(&self, s: &str) -> String {
        emit_variables(s, &self.data)
    }

    pub async fn close(mut self) -> Result<(), RunnerErrorKind> {
        self.webdriver.close().await
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

    let new_text = emit_vars(text.as_ref(), |var| {
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
            .into_iter()
            .map(|v| print_plain_value(v))
            .collect::<Vec<_>>()
            .join(","),
        Value::Bool(val) => val.to_string(),
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
    let nodes = commands
        .iter()
        .zip(levels)
        .enumerate()
        // remove commented commands to not influence runtime
        .filter(|(_, (cmd, _))| !matches!(cmd, Command::Custom { .. }))
        // enumarate after deliting so nodes[i] != nodes.index
        .map(|(index, (cmd, lvl))| CommandNode::new(cmd.clone(), index, lvl, None))
        .collect::<Vec<_>>();

    nodes
}

fn connect_nodes(nodes: &mut [CommandNode]) {
    let mut state = Vec::new();
    (0..nodes.len()).for_each(|i| {
        connect_commands(nodes, i, i + 1, &mut state);
    });
}

// find a coresponding END
// make this END's next pointed to the while // OR DO IT WITH THE ELEMENT BEFORE END
// make while's end on END+1 element
// make while's beggining on the next element

//     Command::If(..) => {
//         // find a coresponding END
//         // find a else/else if structures
//         // DON'T AFRAID TO MAKE SOMETHING INEFFICHIENT FROM SCRATCH. THAT'S FINE.

// todo: refactoring index usage since its too complex
fn connect_commands(
    cmds: &mut [CommandNode],
    current: usize,
    next: usize,
    state: &mut Vec<(&'static str, usize)>,
) {
    match cmds[current].command {
        Command::While(..) => {
            let index_of_whiles_end =
                find_next_end_on_level(&cmds[next..], cmds[current].level).unwrap() + next;
            let index_of_element_after_end = next_index(cmds, index_of_whiles_end);

            cmds[current].next = Some(NodeTransition::Conditional(
                next,
                index_of_element_after_end,
            ));
            state.push(("while", current));
        }
        Command::Do => {
            state.push(("do", current));
            cmds[current].next = Some(NodeTransition::Next(next));
        }
        Command::If(..) => {
            let if_next_index = find_next_on_level(&cmds[next..], cmds[current].level).unwrap();
            let if_next = &cmds[next + if_next_index];
            let cond_end_index =
                find_next_end_on_level(&cmds[current..], cmds[current].level).unwrap() + current;
            let cond_end = &cmds[cond_end_index];

            // todo: doesn't we need to increment this value?
            // now it points to the end value which will point to the next one we could just point it to the next one?
            // but what is the reason of end in this case?
            state.push(("if", cond_end.index));

            let next_element = &cmds[next];
            if next_element.level != cmds[current].level {
                cmds[current].next = Some(NodeTransition::Conditional(next, if_next.index));
            } else {
                cmds[current].next =
                    Some(NodeTransition::Conditional(cond_end.index, if_next.index));
            }
        }
        Command::ElseIf(..) => {
            let elseif_end_i = find_next_on_level(&cmds[next..], cmds[current].level).unwrap();
            let elseif_end = &cmds[elseif_end_i + next];

            let next_element = &cmds[next];
            if next_element.level != cmds[current].level {
                cmds[current].next = Some(NodeTransition::Conditional(next, elseif_end.index));
            } else {
                let (_if, end_index) = state.last().unwrap();
                assert_eq!(*_if, "if");

                cmds[current].next =
                    Some(NodeTransition::Conditional(*end_index, elseif_end.index));
            }
        }
        Command::Else => {
            cmds[current].next = Some(NodeTransition::Next(next));
        }
        Command::RepeatIf(..) => {
            let (_do, do_index) = state.pop().unwrap();
            assert_eq!(_do, "do");
            cmds[current].next = Some(NodeTransition::Conditional(do_index, next));
        }
        Command::End => match state.last() {
            Some(("while", index)) => {
                cmds[current].next = Some(NodeTransition::Next(*index));
                state.pop();
            }
            Some(("if", _)) => {
                state.pop();
                cmds[current].next = Some(NodeTransition::Next(next));
            }
            _ => unreachable!("the syntax is broken"),
        },
        _ if next < cmds.len()
            && matches!(cmds[next].command, Command::Else | Command::ElseIf(..)) =>
        {
            let (_, index) = state.last().unwrap();
            cmds[current].next = Some(NodeTransition::Next(*index));
        }
        _ => {
            cmds[current].next = Some(NodeTransition::Next(next));
        }
    }
}

// TODO: wrap [CommandNode] list by a structure?
// and make it its methods.
/// next_index produces an next element's index in the list
///
/// The next element after the last one in the list has index which exceds a list.len().
/// Which indicates that the list is passed.
///
/// An index sometimes is not just an incremental value, so sometimes
/// `cmds[i].index + 1 !=  cmds[i+1].index`
/// It's caused by custom commands which are deleted before building a list.
#[inline]
fn next_index(cmds: &mut [CommandNode], current: usize) -> usize {
    assert!(current < cmds.len());

    if current + 1 < cmds.len() {
        cmds[current + 1].index
    } else {
        cmds[current].index + 1
    }
}

fn find_next<Cmp: Fn(&CommandNode) -> bool>(
    commands: &[CommandNode],
    comparator: Cmp,
) -> Option<usize> {
    for (i, cmd) in commands.iter().enumerate() {
        if comparator(cmd) {
            return Some(i);
        }
    }

    None
}

fn find_next_on_level(commands: &[CommandNode], level: usize) -> Option<usize> {
    find_next(commands, |cmd| cmd.level == level)
}

fn find_next_end_on_level(commands: &[CommandNode], level: usize) -> Option<usize> {
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
            Command::Do => {
                levels.push(level);
                level += 1;
            }
            Command::RepeatIf(..) => {
                level -= 1;
                levels.push(level);
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
    fn test_creating_run_list_with_commeted_commands() {
        let commands = vec![
            Command::Open("open".to_owned()),
            Command::empty_custom(),
            Command::empty_custom(),
            Command::Echo("echo".to_owned()),
            Command::empty_custom(),
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
                    3,
                    0,
                    Some(NodeTransition::Next(2))
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    5,
                    0,
                    Some(NodeTransition::Next(3))
                )
            ]
        )
    }

    #[test]
    fn test_creating_run_list_with_commeted_command_and_while() {
        let commands = vec![
            Command::While("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::End,
            Command::empty_custom(),
            Command::Echo("echo".to_owned()),
        ];
        let node = create_nodes(&commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::While("...".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Conditional(1, 4)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    1,
                    1,
                    Some(NodeTransition::Next(2)),
                ),
                CommandNode::new(Command::End, 2, 0, Some(NodeTransition::Next(0))),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    4,
                    0,
                    Some(NodeTransition::Next(4))
                ),
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
                    Some(NodeTransition::Conditional(4, 2)),
                ),
                CommandNode::new(
                    Command::ElseIf("...".to_owned()),
                    2,
                    0,
                    Some(NodeTransition::Conditional(4, 3)),
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
                    Some(NodeTransition::Conditional(6, 5)),
                ),
                CommandNode::new(Command::Else, 5, 2, Some(NodeTransition::Next(6)),),
                CommandNode::new(Command::End, 6, 2, Some(NodeTransition::Next(7))),
                CommandNode::new(Command::End, 7, 1, Some(NodeTransition::Next(2))),
                CommandNode::new(Command::End, 8, 0, Some(NodeTransition::Next(1))),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_while_with_if() {
        let mut commands = vec![
            Command::Open("open".to_owned()),
            Command::While("...".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::If("...".to_owned()),
            Command::Else,
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
                    Some(NodeTransition::Conditional(2, 7)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Next(3)),
                ),
                CommandNode::new(
                    Command::If("...".to_owned()),
                    3,
                    1,
                    Some(NodeTransition::Conditional(5, 4)),
                ),
                CommandNode::new(Command::Else, 4, 1, Some(NodeTransition::Next(5)),),
                CommandNode::new(Command::End, 5, 1, Some(NodeTransition::Next(6))),
                CommandNode::new(Command::End, 6, 0, Some(NodeTransition::Next(1))),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_repeat_if() {
        let commands = vec![
            Command::Open("open".to_owned()),
            Command::Do,
            Command::Echo("echo".to_owned()),
            Command::RepeatIf("...".to_owned()),
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
                CommandNode::new(Command::Do, 1, 0, Some(NodeTransition::Next(2)),),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Next(3)),
                ),
                CommandNode::new(
                    Command::RepeatIf("...".to_owned()),
                    3,
                    0,
                    Some(NodeTransition::Conditional(1, 4)),
                ),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_repeat_if_with_if() {
        let commands = vec![
            Command::Open("open".to_owned()),
            Command::Do,
            Command::If("".to_owned()),
            Command::ElseIf("".to_owned()),
            Command::Echo("echo".to_owned()),
            Command::Else,
            Command::End,
            Command::RepeatIf("...".to_owned()),
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
                CommandNode::new(Command::Do, 1, 0, Some(NodeTransition::Next(2))),
                CommandNode::new(
                    Command::If("".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Conditional(6, 3)),
                ),
                CommandNode::new(
                    Command::ElseIf("".to_owned()),
                    3,
                    1,
                    Some(NodeTransition::Conditional(4, 5)),
                ),
                CommandNode::new(
                    Command::Echo("echo".to_owned()),
                    4,
                    2,
                    Some(NodeTransition::Next(6)),
                ),
                CommandNode::new(Command::Else, 5, 1, Some(NodeTransition::Next(6)),),
                CommandNode::new(Command::End, 6, 1, Some(NodeTransition::Next(7)),),
                CommandNode::new(
                    Command::RepeatIf("...".to_owned()),
                    7,
                    0,
                    Some(NodeTransition::Conditional(1, 8)),
                ),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_repeat_if_and_while_and_if() {
        let commands = vec![
            Command::Open("".to_owned()),
            Command::Do,
            Command::While("".to_owned()),
            Command::If("".to_owned()),
            Command::ElseIf("".to_owned()),
            Command::Echo("".to_owned()),
            Command::Else,
            Command::End,
            Command::End,
            Command::RepeatIf("".to_owned()),
        ];
        let node = create_nodes(&commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::Open("".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Next(1)),
                ),
                CommandNode::new(Command::Do, 1, 0, Some(NodeTransition::Next(2))),
                CommandNode::new(
                    Command::While("".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Conditional(3, 9)),
                ),
                CommandNode::new(
                    Command::If("".to_owned()),
                    3,
                    2,
                    Some(NodeTransition::Conditional(7, 4)),
                ),
                CommandNode::new(
                    Command::ElseIf("".to_owned()),
                    4,
                    2,
                    Some(NodeTransition::Conditional(5, 6)),
                ),
                CommandNode::new(
                    Command::Echo("".to_owned()),
                    5,
                    3,
                    Some(NodeTransition::Next(7)),
                ),
                CommandNode::new(Command::Else, 6, 2, Some(NodeTransition::Next(7)),),
                CommandNode::new(Command::End, 7, 2, Some(NodeTransition::Next(8)),),
                CommandNode::new(Command::End, 8, 1, Some(NodeTransition::Next(2)),),
                CommandNode::new(
                    Command::RepeatIf("".to_owned()),
                    9,
                    0,
                    Some(NodeTransition::Conditional(1, 10)),
                ),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_with_while_and_repeat_if() {
        let commands = vec![
            Command::While("..".to_owned()),
            Command::Do,
            Command::RepeatIf("...".to_owned()),
            Command::End,
        ];
        let node = create_nodes(&commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::While("..".to_owned()),
                    0,
                    0,
                    Some(NodeTransition::Conditional(1, 4)),
                ),
                CommandNode::new(Command::Do, 1, 1, Some(NodeTransition::Next(2))),
                CommandNode::new(
                    Command::RepeatIf("...".to_owned()),
                    2,
                    1,
                    Some(NodeTransition::Conditional(1, 3)),
                ),
                CommandNode::new(Command::End, 3, 0, Some(NodeTransition::Next(0)),),
            ]
        )
    }

    #[test]
    fn test_creating_run_list_with_2_whiles_and_2_ifs() {
        let commands = vec![
            Command::While(String::new()),
            Command::If(String::new()),
            Command::End,
            Command::End,
            Command::While(String::new()),
            Command::End,
            Command::While(String::new()),
            Command::If(String::new()),
            Command::Else,
            Command::End,
            Command::End,
        ];
        let node = create_nodes(&commands);
        assert_eq!(
            node,
            vec![
                CommandNode::new(
                    Command::While(String::new()),
                    0,
                    0,
                    Some(NodeTransition::Conditional(1, 4)),
                ),
                CommandNode::new(
                    Command::If(String::new()),
                    1,
                    1,
                    Some(NodeTransition::Conditional(2, 2)),
                ),
                CommandNode::new(Command::End, 2, 1, Some(NodeTransition::Next(3))),
                CommandNode::new(Command::End, 3, 0, Some(NodeTransition::Next(0))),
                CommandNode::new(
                    Command::While(String::new()),
                    4,
                    0,
                    Some(NodeTransition::Conditional(5, 6)),
                ),
                CommandNode::new(Command::End, 5, 0, Some(NodeTransition::Next(4))),
                CommandNode::new(
                    Command::While(String::new()),
                    6,
                    0,
                    Some(NodeTransition::Conditional(7, 11)),
                ),
                CommandNode::new(
                    Command::If(String::new()),
                    7,
                    1,
                    Some(NodeTransition::Conditional(9, 8)),
                ),
                CommandNode::new(Command::Else, 8, 1, Some(NodeTransition::Next(9))),
                CommandNode::new(Command::End, 9, 1, Some(NodeTransition::Next(10))),
                CommandNode::new(Command::End, 10, 0, Some(NodeTransition::Next(6))),
            ]
        )
    }

    fn create_nodes(commands: &[Command]) -> Vec<CommandNode> {
        let mut nodes = super::create_nodes(&commands);
        super::connect_nodes(&mut nodes);
        nodes
    }
}

#[cfg(test)]
mod flow {
    use super::*;
    use crate::parser::Target;
    use mock::{Call, Client};

    #[tokio::test]
    async fn test_run() {
        let test = Test {
            name: String::new(),
            commands: vec![
                Command::Open("".to_owned()),
                Command::Click(Target::new(Location::Css("".to_owned()))),
            ],
        };
        let client = Client::new();
        let mut runner = Runner::_new(client.clone());

        let res = runner.run(&test).await;
        assert!(res.is_ok());
        let calls = client.calls();
        assert_eq!(calls[Call::Goto], 1);
        assert_eq!(calls[Call::Click], 1);
    }

    #[tokio::test]
    async fn test_run_with_custom_command() {
        let test = Test {
            name: String::new(),
            commands: vec![
                Command::empty_custom(),
                Command::Open("".to_owned()),
                Command::Click(Target::new(Location::Css("".to_owned()))),
            ],
        };
        let client = Client::new();
        let mut runner = Runner::_new(client.clone());

        let res = runner.run(&test).await;
        assert!(res.is_ok());
        let calls = client.calls();
        assert_eq!(calls[Call::Goto], 1);
        assert_eq!(calls[Call::Click], 1);
    }

    mod mock {
        use super::*;
        use crate::webdriver::{Element as WebElement, Locator, Webdriver};
        use serde_json::Value as Json;
        use std::ops::{Index, IndexMut};
        use std::sync::{Arc, Mutex};
        use std::time::Duration;

        #[derive(Default)]
        pub struct Client {
            pub calls: Mutex<CallCount>,
            pub res_find: Option<fn() -> Result<Element, RunnerErrorKind>>,
            pub res_curr_url: Option<fn() -> Result<url::Url, RunnerErrorKind>>,
            pub res_exec: Option<fn() -> Result<Json, RunnerErrorKind>>,
            pub res_set_w_size: Option<fn() -> Result<(), RunnerErrorKind>>,
            pub res_close: Option<fn() -> Result<(), RunnerErrorKind>>,
            pub res_goto: Option<fn() -> Result<(), RunnerErrorKind>>,
            pub res_w8_visib: Option<fn() -> Result<Option<Duration>, RunnerErrorKind>>,
            pub res_w8_pres: Option<fn() -> Result<Option<Duration>, RunnerErrorKind>>,
            pub res_w8_npres: Option<fn() -> Result<Option<Duration>, RunnerErrorKind>>,
            pub res_w8_edit: Option<fn() -> Result<Option<Duration>, RunnerErrorKind>>,
        }

        impl Client {
            pub fn new() -> Arc<Self> {
                Arc::new(Self::default())
            }

            pub fn calls(&self) -> CallCount {
                self.calls.lock().unwrap().clone()
            }

            pub fn inc(&self, c: Call) {
                let mut calls = self.calls.lock().unwrap();
                calls[c] += 1;
            }
        }

        #[async_trait::async_trait]
        impl<'a> Webdriver for Arc<Client> {
            type Element = Element;
            type Error = crate::error::RunnerErrorKind;

            async fn goto(&mut self, _: &str) -> Result<(), Self::Error> {
                self.inc(Call::Goto);
                Ok(())
            }

            async fn find(&mut self, _: Locator) -> Result<Self::Element, Self::Error> {
                self.inc(Call::Find);
                Ok(Element(Arc::clone(self)))
            }

            async fn find_all(&mut self, _: Locator) -> Result<Vec<Self::Element>, Self::Error> {
                self.inc(Call::FindAll);
                Ok(vec![Element(Arc::clone(self))])
            }

            async fn wait_for_visible(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), Self::Error> {
                self.inc(Call::W8Visib);
                Ok(())
            }

            async fn wait_for_not_present(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), Self::Error> {
                self.inc(Call::W8NPres);
                Ok(())
            }

            async fn wait_for_present(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), Self::Error> {
                self.inc(Call::W8Pres);
                Ok(())
            }

            async fn wait_for_editable(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), Self::Error> {
                self.inc(Call::W8Edit);
                Ok(())
            }

            async fn current_url(&mut self) -> Result<url::Url, Self::Error> {
                self.inc(Call::CurrentUrl);
                Ok(url::Url::parse("http://example.com").unwrap())
            }

            async fn set_window_size(&mut self, _: u32, _: u32) -> Result<(), Self::Error> {
                self.inc(Call::SetWSize);
                Ok(())
            }

            async fn execute(&mut self, _: &str, _: Vec<Json>) -> Result<Json, Self::Error> {
                self.inc(Call::Exec);
                Ok(Json::Null)
            }

            async fn close(&mut self) -> Result<(), Self::Error> {
                self.inc(Call::Close);
                Ok(())
            }
        }

        pub struct Element(Arc<Client>);

        impl Element {
            pub fn inc(&self, call: Call) {
                self.0.inc(call);
            }
        }

        #[async_trait::async_trait]
        impl WebElement for Element {
            type Driver = Arc<Client>;
            type Error = crate::error::RunnerErrorKind;

            async fn attr(&mut self, _: &str) -> Result<Option<String>, Self::Error> {
                self.inc(Call::Attr);
                Ok(None)
            }

            async fn prop(&mut self, _: &str) -> Result<Option<String>, Self::Error> {
                self.inc(Call::Prop);
                Ok(None)
            }

            async fn text(&mut self) -> Result<String, Self::Error> {
                self.inc(Call::Text);
                Ok("".to_string())
            }

            async fn html(&mut self, _: bool) -> Result<String, Self::Error> {
                self.inc(Call::Html);
                Ok("".to_string())
            }

            async fn find(&mut self, _: Locator) -> Result<Self, Self::Error>
            where
                Self: Sized,
            {
                self.inc(Call::Find);
                Ok(Element(self.0.clone()))
            }

            async fn click(mut self) -> Result<Self::Driver, Self::Error> {
                self.inc(Call::Click);
                Ok(self.0.clone())
            }

            async fn select_by_index(mut self, _: usize) -> Result<Self::Driver, Self::Error> {
                self.inc(Call::SelectByIndex);
                Ok(self.0.clone())
            }

            async fn select_by_value(mut self, _: &str) -> Result<Self::Driver, Self::Error> {
                self.inc(Call::SelectByValue);
                Ok(self.0.clone())
            }
        }

        #[derive(Clone, Default)]
        pub struct CallCount {
            open: usize,
            click: usize,
            find: usize,
            findall: usize,
            goto: usize,
            exec: usize,
            close: usize,
            current_url: usize,
            set_w_size: usize,
            w_8_visib: usize,
            w_8_pres: usize,
            w_8_npres: usize,
            w_8_edit: usize,
            attr: usize,
            prop: usize,
            text: usize,
            html: usize,
            select_by_index: usize,
            select_by_value: usize,
        }

        #[derive(Hash, PartialEq, Eq)]
        pub enum Call {
            Open,
            Click,
            Find,
            FindAll,
            Goto,
            Exec,
            Close,
            CurrentUrl,
            SetWSize,
            W8Visib,
            W8Pres,
            W8NPres,
            W8Edit,
            Attr,
            Prop,
            Text,
            Html,
            SelectByIndex,
            SelectByValue,
        }

        impl Index<Call> for CallCount {
            type Output = usize;

            fn index(&self, count: Call) -> &Self::Output {
                match count {
                    Call::Open => &self.open,
                    Call::Click => &self.click,
                    Call::Find => &self.find,
                    Call::FindAll => &self.findall,
                    Call::Goto => &self.goto,
                    Call::Exec => &self.exec,
                    Call::Close => &self.close,
                    Call::CurrentUrl => &self.current_url,
                    Call::SetWSize => &self.set_w_size,
                    Call::W8Visib => &self.w_8_visib,
                    Call::W8Pres => &self.w_8_pres,
                    Call::W8NPres => &self.w_8_npres,
                    Call::W8Edit => &self.w_8_edit,
                    Call::Attr => &self.attr,
                    Call::Prop => &self.prop,
                    Call::Text => &self.text,
                    Call::Html => &self.html,
                    Call::SelectByIndex => &self.select_by_index,
                    Call::SelectByValue => &self.select_by_value,
                }
            }
        }

        impl IndexMut<Call> for CallCount {
            fn index_mut(&mut self, count: Call) -> &mut Self::Output {
                match count {
                    Call::Open => &mut self.open,
                    Call::Click => &mut self.click,
                    Call::Find => &mut self.find,
                    Call::FindAll => &mut self.findall,
                    Call::Goto => &mut self.goto,
                    Call::Exec => &mut self.exec,
                    Call::Close => &mut self.close,
                    Call::CurrentUrl => &mut self.current_url,
                    Call::SetWSize => &mut self.set_w_size,
                    Call::W8Visib => &mut self.w_8_visib,
                    Call::W8Pres => &mut self.w_8_pres,
                    Call::W8NPres => &mut self.w_8_npres,
                    Call::W8Edit => &mut self.w_8_edit,
                    Call::Attr => &mut self.attr,
                    Call::Prop => &mut self.prop,
                    Call::Text => &mut self.text,
                    Call::Html => &mut self.html,
                    Call::SelectByIndex => &mut self.select_by_index,
                    Call::SelectByValue => &mut self.select_by_value,
                }
            }
        }
    }
}
