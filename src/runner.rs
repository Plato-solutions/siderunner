// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// TODO: Mock webdriver
// TODO: interface for fantocini and possibly choose webdriver provider by feature
// TODO: provide more direct error location test + command + location(can be determined just by section (target/value etc.)) + cause
// TODO: Runner may contains basic information to handle relative url
// TODO: refactoring and test While and If commits
// TODO: hide hook after feature flag + add a sleep statistic hook

use crate::command::Command as Cmd1;
use crate::command::{
    AnswerOnNextPrompt, Assert, AssertAlert, AssertChecked, AssertNotChecked, Click, Close, Echo,
    Execute, ExecuteAsync, Open, Pause, RunScript, Select, SetWindowSize, Store, StoreText,
    StoreXpathCount, WaitForElementEditable, WaitForElementNotPresent, WaitForElementPresent,
    WaitForElementVisible,
};
use crate::parser::Target;
use crate::webdriver::{self, Locator, Webdriver};
use crate::File;
use crate::{
    error::{RunnerError, RunnerErrorKind},
    parser::{Cmd, Command, Location, Test},
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
    pub fn _new(client: D) -> Runner<D> {
        Self {
            webdriver: client,
            data: HashMap::new(),
            echo_hook: Box::new(|s| println!("{}", s)),
        }
    }

    pub(crate) fn save_value(&mut self, var: String, value: Value) {
        self.data.insert(var, value);
    }

    pub(crate) fn get_value(&mut self, var: &str) -> Option<&Value> {
        self.data.get(var)
    }

    pub(crate) fn echo(&self, message: &str) {
        self.echo_hook.as_ref()(message)
    }
}

impl<D, E> Runner<D>
where
    D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
    E: webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send,
{
    /// Run all tests in a side file
    pub async fn run(&mut self, file: &File) -> Result<(), RunnerError> {
        for test in 0..file.tests.len() {
            self.run_test(file, test).await?;
        }

        Ok(())
    }

    pub async fn run_test(&mut self, file: &File, test_index: usize) -> Result<(), RunnerError> {
        let test = &file.tests[test_index];
        let err_wrap = |mut e: RunnerError| {
            e.test = Some(test.name.clone());
            e
        };
        let nodes = Self::build_nodes(test).map_err(err_wrap)?;
        self.run_nodes(&file.url, nodes).await.map_err(err_wrap)?;
        Ok(())
    }

    fn build_nodes(test: &Test) -> Result<Vec<CommandNode>, RunnerError> {
        crate::validation::validate_conditions(&test.commands)?;
        Ok(build_nodes(&test.commands))
    }

    /// Close underlying webdriver client.
    ///
    /// It must be run as some backends require it's call to release a Webdriver session.
    pub async fn close(mut self) -> Result<(), RunnerErrorKind> {
        self.webdriver.close().await
    }

    /// Sets a callback which will be run on each Echo command.
    pub fn set_echo<F: Fn(&str) + Send + 'static>(&mut self, func: F) {
        self.echo_hook = Box::new(func);
    }

    /// Gets a list of variables which were collected over the runs.
    pub fn get_data(&self) -> &HashMap<String, Value> {
        &self.data
    }

    pub fn get_webdriver(&mut self) -> &mut D {
        &mut self.webdriver
    }

    async fn run_nodes(
        &mut self,
        file_url: &str,
        nodes: Vec<CommandNode>,
    ) -> Result<(), RunnerError> {
        if nodes.is_empty() {
            return Ok(());
        }

        let mut i = 0;
        loop {
            if i >= nodes.len() {
                break;
            }
            let node = &nodes[i];
            match node.next {
                NodeTransition::Next(next) if matches!(node.command, Cmd::End) => {
                    i = next;
                }
                NodeTransition::Next(next) => {
                    i = next;
                    let cmd = &node.command;
                    self.run_command(file_url, cmd)
                        .await
                        .map_err(|e| RunnerError::new(e, node.index))?;
                }
                NodeTransition::Conditional(next, or_else) => {
                    match &node.command {
                        Cmd::While(condition)
                        | Cmd::ElseIf(condition)
                        | Cmd::If(condition)
                        | Cmd::RepeatIf(condition) => {
                            let cond = self
                                .run_condition(condition)
                                .await
                                .map_err(|e| RunnerError::new(e, i))?;
                            if cond {
                                i = next;
                            } else {
                                i = or_else;
                            }
                        }
                        Cmd::ForEach { iterator, var } => {
                            let key = format!("ITERATOR_INDEX_{}_{}", iterator, var);
                            match self.data.get_mut(&key) {
                                Some(Value::Array(array)) => {
                                    if array.is_empty() {
                                        i = or_else;
                                    } else {
                                        let e = array.remove(0);
                                        self.data.insert(var.clone(), e);
                                        i = next;
                                    }
                                }
                                None => {
                                    let mut array = match self.data.get(iterator) {
                                        Some(Value::Array(arr)) => {
                                            serde_json::json!(arr)
                                        }
                                        Some(Value::String(s)) => {
                                            let arr = s.chars().collect::<Vec<_>>();
                                            serde_json::json!(arr)
                                        }
                                        // Itarator is invalid; skip inner block
                                        _ => {
                                            i = or_else;
                                            continue;
                                        }
                                    };

                                    let arr = array.as_array_mut().unwrap();
                                    if arr.is_empty() {
                                        i = or_else;
                                    } else {
                                        let e = arr.remove(0);
                                        self.data.insert(var.clone(), e);
                                        self.data.insert(key, array);
                                        i = next;
                                    }
                                }
                                Some(_) => unreachable!(),
                            }
                        }
                        _ => unreachable!("unexpected condition"),
                    };
                }
            };
        }

        Ok(())
    }

    async fn run_condition(&mut self, condition: &str) -> Result<bool, RunnerErrorKind> {
        let script = format!("return {}", condition);
        let res = self.exec(&script).await?;
        match res.as_bool() {
            Some(b) => Ok(b),
            None => Err(RunnerErrorKind::MismatchedType(
                "expected boolean type in condition".to_owned(),
            )),
        }
    }

    async fn run_command(&mut self, file_url: &str, cmd: &Cmd) -> Result<(), RunnerErrorKind> {
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
            Cmd::While(..)
            | Cmd::Else
            | Cmd::If(..)
            | Cmd::ElseIf(..)
            | Cmd::ForEach { .. }
            | Cmd::RepeatIf(..)
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CommandNode {
    command: Cmd,
    index: usize,
    level: usize,
    next: NodeTransition,
}

impl CommandNode {
    pub(crate) fn new(cmd: Cmd, index: usize, level: usize, transition: NodeTransition) -> Self {
        Self {
            command: cmd,
            index,
            level,
            next: transition,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum NodeTransition {
    Next(usize),
    Conditional(usize, usize),
}

pub(crate) fn build_nodes(commands: &[Command]) -> Vec<CommandNode> {
    let mut nodes = create_nodes(commands);
    connect_nodes(&mut nodes);
    nodes
}

fn create_nodes(commands: &[Command]) -> Vec<CommandNode> {
    let levels = compute_levels(commands);
    let nodes = commands
        .iter()
        .zip(levels)
        .enumerate()
        // remove commented commands to not influence runtime
        .filter(|(_, (cmd, _))| !matches!(cmd.cmd, Cmd::Custom { .. }))
        // enumarate after deliting so nodes[i] != nodes.index
        .map(|(index, (cmd, lvl))| {
            CommandNode::new(cmd.cmd.clone(), index, lvl, NodeTransition::Next(0))
            //  NodeTransition::Next(0) will be recalculated later
        })
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

//     Cmd::If(..) => {
//         // find a coresponding END
//         // find a else/else if structures
//         // DON'T AFRAID TO MAKE SOMETHING INEFFICHIENT FROM SCRATCH. THAT'S FINE.

// todo: refactoring index usage since its too complex
fn connect_commands(
    nodes: &mut [CommandNode],
    current: usize,
    next: usize,
    state: &mut Vec<(&'static str, usize)>,
) {
    match nodes[current].command {
        Cmd::While(..) => {
            let index_of_whiles_end =
                find_next_end_on_level(&nodes[next..], nodes[current].level).unwrap() + next;
            let index_of_element_after_end = next_index(nodes, index_of_whiles_end);

            nodes[current].next = NodeTransition::Conditional(next, index_of_element_after_end);
            state.push(("while", current));
        }
        Cmd::ForEach { .. } => {
            let index_end =
                find_next_end_on_level(&nodes[next..], nodes[current].level).unwrap() + next;
            let index_after_end = next_index(nodes, index_end);

            nodes[current].next = NodeTransition::Conditional(next, index_after_end);
            state.push(("forEach", current));
        }
        Cmd::Do => {
            state.push(("do", current));
            nodes[current].next = NodeTransition::Next(next);
        }
        Cmd::If(..) => {
            let if_next_index = find_next_on_level(&nodes[next..], nodes[current].level).unwrap();
            let if_next = &nodes[next + if_next_index];
            let cond_end_index =
                find_next_end_on_level(&nodes[current..], nodes[current].level).unwrap() + current;
            let cond_end = &nodes[cond_end_index];

            // todo: doesn't we need to increment this value?
            // now it points to the end value which will point to the next one we could just point it to the next one?
            // but what is the reason of end in this case?
            state.push(("if", cond_end.index));

            let next_element = &nodes[next];
            if next_element.level != nodes[current].level {
                nodes[current].next = NodeTransition::Conditional(next, if_next.index);
            } else {
                nodes[current].next = NodeTransition::Conditional(cond_end.index, if_next.index);
            }
        }
        Cmd::ElseIf(..) => {
            let elseif_end_i = find_next_on_level(&nodes[next..], nodes[current].level).unwrap();
            let elseif_end = &nodes[elseif_end_i + next];

            let next_element = &nodes[next];
            if next_element.level != nodes[current].level {
                nodes[current].next = NodeTransition::Conditional(next, elseif_end.index);
            } else {
                let (_if, end_index) = state.last().unwrap();
                assert_eq!(*_if, "if");

                nodes[current].next = NodeTransition::Conditional(*end_index, elseif_end.index);
            }
        }
        Cmd::Else => {
            nodes[current].next = NodeTransition::Next(next);
        }
        Cmd::RepeatIf(..) => {
            let (_do, do_index) = state.pop().unwrap();
            assert_eq!(_do, "do");
            nodes[current].next = NodeTransition::Conditional(do_index, next);
        }
        Cmd::End => match state.last() {
            Some(("while", index)) | Some(("forEach", index)) => {
                nodes[current].next = NodeTransition::Next(*index);
                state.pop();
            }
            Some(("if", _)) => {
                state.pop();
                nodes[current].next = NodeTransition::Next(next);
            }
            _ => unreachable!("the syntax is broken"),
        },
        _ if next < nodes.len() && matches!(nodes[next].command, Cmd::Else | Cmd::ElseIf(..)) => {
            let (_, index) = state.last().unwrap();
            nodes[current].next = NodeTransition::Next(*index);
        }
        _ => {
            nodes[current].next = NodeTransition::Next(next);
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
/// `nodes[i].index + 1 !=  nodes[i+1].index`
/// It's caused by custom commands which are deleted before building a list.
#[inline]
fn next_index(nodes: &mut [CommandNode], current: usize) -> usize {
    assert!(current < nodes.len());

    if current + 1 < nodes.len() {
        nodes[current + 1].index
    } else {
        nodes[current].index + 1
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

fn find_next_on_level(nodes: &[CommandNode], level: usize) -> Option<usize> {
    find_next(nodes, |node| node.level == level)
}

fn find_next_end_on_level(commands: &[CommandNode], level: usize) -> Option<usize> {
    find_next(commands, |node| {
        node.level == level && matches!(node.command, Cmd::End)
    })
}

fn compute_levels(commands: &[Command]) -> Vec<usize> {
    let mut level = 0;
    let mut levels = Vec::with_capacity(commands.len());
    for cmd in commands {
        match cmd.cmd {
            Cmd::While(..) | Cmd::If(..) | Cmd::ForEach { .. } => {
                levels.push(level);
                level += 1;
            }
            Cmd::End => {
                level -= 1;
                levels.push(level);
            }
            Cmd::Else | Cmd::ElseIf(..) => {
                level -= 1;
                levels.push(level);
                level += 1;
            }
            Cmd::Do => {
                levels.push(level);
                level += 1;
            }
            Cmd::RepeatIf(..) => {
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
