use serde_json::Value;

use crate::{
    error::RunnerErrorKind, parser::Cmd, runner::Runner, validation::validate_conditions,
    webdriver, Command, File, RunnerError,
};

pub struct Playground;

impl Playground {
    pub async fn run_test<D: webdriver::Webdriver>(
        runner: &mut Runner<D>,
        file: &File,
        test_index: usize,
    ) -> Result<(), RunnerError> {
        let test = &file.tests[test_index];
        let err_wrap = |mut e: RunnerError| {
            e.test = Some(test.name.clone());
            e
        };
        validate_conditions(&test.commands).map_err(err_wrap)?;
        let nodes = build_nodes(&test.commands);
        run_nodes(runner, nodes, &file.url).await.map_err(err_wrap)
    }
}

pub(crate) fn build_nodes(commands: &[Command]) -> Vec<CommandNode> {
    let mut nodes = create_nodes(commands);
    connect_nodes(&mut nodes);
    nodes
}

async fn run_nodes<D: webdriver::Webdriver>(
    runner: &mut Runner<D>,
    nodes: Vec<CommandNode>,
    file_url: &str,
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
                runner
                    .run_command(file_url, cmd)
                    .await
                    .map_err(|e| RunnerError::new(e, node.index))?;
            }
            NodeTransition::Conditional(next, or_else) => {
                match &node.command {
                    Cmd::While(condition)
                    | Cmd::ElseIf(condition)
                    | Cmd::If(condition)
                    | Cmd::RepeatIf(condition) => {
                        let cond = run_condition(runner, condition)
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
                        match runner.get_value_mut(&key) {
                            Some(Value::Array(array)) => {
                                if array.is_empty() {
                                    i = or_else;
                                } else {
                                    let e = array.remove(0);
                                    runner.save_value(var.clone(), e);
                                    i = next;
                                }
                            }
                            None => {
                                let mut array = match runner.get_value(iterator) {
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
                                    runner.save_value(var.clone(), e);
                                    runner.save_value(key, array);
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

async fn run_condition<D: webdriver::Webdriver>(
    runner: &mut Runner<D>,
    condition: &str,
) -> Result<bool, RunnerErrorKind> {
    let script = format!("return {}", condition);
    let res = runner.exec(&script).await?;
    match res.as_bool() {
        Some(b) => Ok(b),
        None => Err(RunnerErrorKind::MismatchedType(
            "expected boolean type in condition".to_owned(),
        )),
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
