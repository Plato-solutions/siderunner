use crate::{
    parser::{Cmd, Command},
    runner::{build_nodes, CommandNode, NodeTransition, Runner},
};

#[test]
fn test_creating_run_list_basic() {
    let commands = vec![
        blank_cmd(Cmd::Open("".to_owned())),
        blank_cmd(Cmd::Echo("".to_owned())),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("".to_owned()), 0, 0, NodeTransition::Next(1)),
            CommandNode::new(Cmd::Echo("".to_owned()), 1, 0, NodeTransition::Next(2))
        ]
    )
}

#[test]
fn test_creating_run_list_with_commeted_commands() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::empty_custom()),
        blank_cmd(Cmd::empty_custom()),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::empty_custom()),
        blank_cmd(Cmd::Echo("echo".to_owned())),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1)),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 3, 0, NodeTransition::Next(2)),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 5, 0, NodeTransition::Next(3)),
        ]
    )
}

#[test]
fn test_creating_run_list_with_commeted_command_and_while() {
    let commands = vec![
        blank_cmd(Cmd::While("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::empty_custom()),
        blank_cmd(Cmd::Echo("echo".to_owned())),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(
                Cmd::While("...".to_owned()),
                0,
                0,
                NodeTransition::Conditional(1, 4),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 1, 1, NodeTransition::Next(2)),
            CommandNode::new(Cmd::End, 2, 0, NodeTransition::Next(0)),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 4, 0, NodeTransition::Next(4)),
        ]
    )
}

#[test]
fn test_creating_run_list_while_loop() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::While("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::End),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(
                Cmd::While("...".to_owned()),
                1,
                0,
                NodeTransition::Conditional(2, 4),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 2, 1, NodeTransition::Next(3),),
            CommandNode::new(Cmd::End, 3, 0, NodeTransition::Next(1)),
        ]
    )
}

#[test]
fn test_creating_run_list_if() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::If("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::Echo("echo".to_owned())),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(
                Cmd::If("...".to_owned()),
                1,
                0,
                NodeTransition::Conditional(2, 3),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 2, 1, NodeTransition::Next(3),),
            CommandNode::new(Cmd::End, 3, 0, NodeTransition::Next(4)),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 4, 0, NodeTransition::Next(5)),
        ]
    )
}

#[test]
fn test_creating_run_list_if_complex_empty_conditions() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::If("...".to_owned())),
        blank_cmd(Cmd::ElseIf("...".to_owned())),
        blank_cmd(Cmd::Else),
        blank_cmd(Cmd::End),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(
                Cmd::If("...".to_owned()),
                1,
                0,
                NodeTransition::Conditional(4, 2),
            ),
            CommandNode::new(
                Cmd::ElseIf("...".to_owned()),
                2,
                0,
                NodeTransition::Conditional(4, 3),
            ),
            CommandNode::new(Cmd::Else, 3, 0, NodeTransition::Next(4)),
            CommandNode::new(Cmd::End, 4, 0, NodeTransition::Next(5)),
        ]
    )
}

#[test]
fn test_creating_run_list_if_complex() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::If("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::ElseIf("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::Else),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::End),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(
                Cmd::If("...".to_owned()),
                1,
                0,
                NodeTransition::Conditional(2, 3),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 2, 1, NodeTransition::Next(8),),
            CommandNode::new(
                Cmd::ElseIf("...".to_owned()),
                3,
                0,
                NodeTransition::Conditional(4, 6),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 4, 1, NodeTransition::Next(5),),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 5, 1, NodeTransition::Next(8),),
            CommandNode::new(Cmd::Else, 6, 0, NodeTransition::Next(7)),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 7, 1, NodeTransition::Next(8)),
            CommandNode::new(Cmd::End, 8, 0, NodeTransition::Next(9)),
        ]
    )
}

#[test]
fn test_creating_run_list_if_complex_without_else() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::If("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::ElseIf("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::End),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(
                Cmd::If("...".to_owned()),
                1,
                0,
                NodeTransition::Conditional(2, 3),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 2, 1, NodeTransition::Next(5),),
            CommandNode::new(
                Cmd::ElseIf("...".to_owned()),
                3,
                0,
                NodeTransition::Conditional(4, 5),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 4, 1, NodeTransition::Next(5)),
            CommandNode::new(Cmd::End, 5, 0, NodeTransition::Next(6)),
        ]
    )
}

#[test]
fn test_creating_run_list_multi_while() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::While("...".to_owned())),
        blank_cmd(Cmd::While("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::End),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(
                Cmd::While("...".to_owned()),
                1,
                0,
                NodeTransition::Conditional(2, 6),
            ),
            CommandNode::new(
                Cmd::While("...".to_owned()),
                2,
                1,
                NodeTransition::Conditional(3, 5),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 3, 2, NodeTransition::Next(4),),
            CommandNode::new(Cmd::End, 4, 1, NodeTransition::Next(2)),
            CommandNode::new(Cmd::End, 5, 0, NodeTransition::Next(1)),
        ]
    )
}

#[test]
fn test_creating_run_list_multi_while_with_if() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::While("...".to_owned())),
        blank_cmd(Cmd::While("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::If("...".to_owned())),
        blank_cmd(Cmd::Else),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::End),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(
                Cmd::While("...".to_owned()),
                1,
                0,
                NodeTransition::Conditional(2, 9),
            ),
            CommandNode::new(
                Cmd::While("...".to_owned()),
                2,
                1,
                NodeTransition::Conditional(3, 8),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 3, 2, NodeTransition::Next(4),),
            CommandNode::new(
                Cmd::If("...".to_owned()),
                4,
                2,
                NodeTransition::Conditional(6, 5),
            ),
            CommandNode::new(Cmd::Else, 5, 2, NodeTransition::Next(6)),
            CommandNode::new(Cmd::End, 6, 2, NodeTransition::Next(7)),
            CommandNode::new(Cmd::End, 7, 1, NodeTransition::Next(2)),
            CommandNode::new(Cmd::End, 8, 0, NodeTransition::Next(1)),
        ]
    )
}

#[test]
fn test_creating_run_list_while_with_if() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::While("...".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::If("...".to_owned())),
        blank_cmd(Cmd::Else),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::End),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(
                Cmd::While("...".to_owned()),
                1,
                0,
                NodeTransition::Conditional(2, 7),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 2, 1, NodeTransition::Next(3),),
            CommandNode::new(
                Cmd::If("...".to_owned()),
                3,
                1,
                NodeTransition::Conditional(5, 4),
            ),
            CommandNode::new(Cmd::Else, 4, 1, NodeTransition::Next(5)),
            CommandNode::new(Cmd::End, 5, 1, NodeTransition::Next(6)),
            CommandNode::new(Cmd::End, 6, 0, NodeTransition::Next(1)),
        ]
    )
}

#[test]
fn test_creating_run_list_repeat_if() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::Do),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::RepeatIf("...".to_owned())),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(Cmd::Do, 1, 0, NodeTransition::Next(2)),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 2, 1, NodeTransition::Next(3),),
            CommandNode::new(
                Cmd::RepeatIf("...".to_owned()),
                3,
                0,
                NodeTransition::Conditional(1, 4),
            ),
        ]
    )
}

#[test]
fn test_creating_run_list_repeat_if_with_if() {
    let commands = vec![
        blank_cmd(Cmd::Open("open".to_owned())),
        blank_cmd(Cmd::Do),
        blank_cmd(Cmd::If("".to_owned())),
        blank_cmd(Cmd::ElseIf("".to_owned())),
        blank_cmd(Cmd::Echo("echo".to_owned())),
        blank_cmd(Cmd::Else),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::RepeatIf("...".to_owned())),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("open".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(Cmd::Do, 1, 0, NodeTransition::Next(2)),
            CommandNode::new(
                Cmd::If("".to_owned()),
                2,
                1,
                NodeTransition::Conditional(6, 3),
            ),
            CommandNode::new(
                Cmd::ElseIf("".to_owned()),
                3,
                1,
                NodeTransition::Conditional(4, 5),
            ),
            CommandNode::new(Cmd::Echo("echo".to_owned()), 4, 2, NodeTransition::Next(6),),
            CommandNode::new(Cmd::Else, 5, 1, NodeTransition::Next(6)),
            CommandNode::new(Cmd::End, 6, 1, NodeTransition::Next(7)),
            CommandNode::new(
                Cmd::RepeatIf("...".to_owned()),
                7,
                0,
                NodeTransition::Conditional(1, 8),
            ),
        ]
    )
}

#[test]
fn test_creating_run_list_repeat_if_and_while_and_if() {
    let commands = vec![
        blank_cmd(Cmd::Open("".to_owned())),
        blank_cmd(Cmd::Do),
        blank_cmd(Cmd::While("".to_owned())),
        blank_cmd(Cmd::If("".to_owned())),
        blank_cmd(Cmd::ElseIf("".to_owned())),
        blank_cmd(Cmd::Echo("".to_owned())),
        blank_cmd(Cmd::Else),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::RepeatIf("".to_owned())),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(Cmd::Open("".to_owned()), 0, 0, NodeTransition::Next(1),),
            CommandNode::new(Cmd::Do, 1, 0, NodeTransition::Next(2)),
            CommandNode::new(
                Cmd::While("".to_owned()),
                2,
                1,
                NodeTransition::Conditional(3, 9),
            ),
            CommandNode::new(
                Cmd::If("".to_owned()),
                3,
                2,
                NodeTransition::Conditional(7, 4),
            ),
            CommandNode::new(
                Cmd::ElseIf("".to_owned()),
                4,
                2,
                NodeTransition::Conditional(5, 6),
            ),
            CommandNode::new(Cmd::Echo("".to_owned()), 5, 3, NodeTransition::Next(7),),
            CommandNode::new(Cmd::Else, 6, 2, NodeTransition::Next(7)),
            CommandNode::new(Cmd::End, 7, 2, NodeTransition::Next(8)),
            CommandNode::new(Cmd::End, 8, 1, NodeTransition::Next(2)),
            CommandNode::new(
                Cmd::RepeatIf("".to_owned()),
                9,
                0,
                NodeTransition::Conditional(1, 10),
            ),
        ]
    )
}

#[test]
fn test_creating_run_list_with_while_and_repeat_if() {
    let commands = vec![
        blank_cmd(Cmd::While("..".to_owned())),
        blank_cmd(Cmd::Do),
        blank_cmd(Cmd::RepeatIf("...".to_owned())),
        blank_cmd(Cmd::End),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(
                Cmd::While("..".to_owned()),
                0,
                0,
                NodeTransition::Conditional(1, 4),
            ),
            CommandNode::new(Cmd::Do, 1, 1, NodeTransition::Next(2)),
            CommandNode::new(
                Cmd::RepeatIf("...".to_owned()),
                2,
                1,
                NodeTransition::Conditional(1, 3),
            ),
            CommandNode::new(Cmd::End, 3, 0, NodeTransition::Next(0)),
        ]
    )
}

#[test]
fn test_creating_run_list_with_2_whiles_and_2_ifs() {
    let commands = vec![
        blank_cmd(Cmd::While(String::new())),
        blank_cmd(Cmd::If(String::new())),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::While(String::new())),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::While(String::new())),
        blank_cmd(Cmd::If(String::new())),
        blank_cmd(Cmd::Else),
        blank_cmd(Cmd::End),
        blank_cmd(Cmd::End),
    ];
    let node = build_nodes(&commands);
    assert_eq!(
        node,
        vec![
            CommandNode::new(
                Cmd::While(String::new()),
                0,
                0,
                NodeTransition::Conditional(1, 4),
            ),
            CommandNode::new(
                Cmd::If(String::new()),
                1,
                1,
                NodeTransition::Conditional(2, 2),
            ),
            CommandNode::new(Cmd::End, 2, 1, NodeTransition::Next(3)),
            CommandNode::new(Cmd::End, 3, 0, NodeTransition::Next(0)),
            CommandNode::new(
                Cmd::While(String::new()),
                4,
                0,
                NodeTransition::Conditional(5, 6),
            ),
            CommandNode::new(Cmd::End, 5, 0, NodeTransition::Next(4)),
            CommandNode::new(
                Cmd::While(String::new()),
                6,
                0,
                NodeTransition::Conditional(7, 11),
            ),
            CommandNode::new(
                Cmd::If(String::new()),
                7,
                1,
                NodeTransition::Conditional(9, 8),
            ),
            CommandNode::new(Cmd::Else, 8, 1, NodeTransition::Next(9)),
            CommandNode::new(Cmd::End, 9, 1, NodeTransition::Next(10)),
            CommandNode::new(Cmd::End, 10, 0, NodeTransition::Next(6)),
        ]
    )
}

#[cfg(test)]
mod flow {
    use super::*;
    use crate::parser::{Cmd, Command, File, Location, Target, Test};
    use mock::{Call, Client};
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_run() {
        let file = blank_file(vec![
            Cmd::Open("http://example.com".to_owned()),
            Cmd::Click(Target::new(Location::Css("".to_owned()))),
        ]);
        let client = Client::new();
        let mut runner = Runner::_new(client.clone());

        let res = runner.run(&file).await;
        assert!(res.is_ok());
        let calls = client.calls();
        assert_eq!(calls[Call::Goto], 1);
        assert_eq!(calls[Call::Click], 1);
    }

    #[tokio::test]
    async fn test_run_with_custom_command() {
        let file = blank_file(vec![
            Cmd::empty_custom(),
            Cmd::Open("http://example.com".to_owned()),
            Cmd::Click(Target::new(Location::Css("".to_owned()))),
        ]);
        let client = Client::new();
        let mut runner = Runner::_new(client.clone());

        let res = runner.run(&file).await;
        assert!(res.is_ok());
        let calls = client.calls();
        assert_eq!(calls[Call::Goto], 1);
        assert_eq!(calls[Call::Click], 1);
    }

    #[tokio::test]
    async fn test_for_each() {
        let file = blank_file(vec![
            Cmd::Open("http://example.com".to_owned()),
            Cmd::ForEach {
                var: "element".to_string(),
                iterator: "array".to_string(),
            },
            Cmd::Echo("${element}".to_string()),
            Cmd::End,
        ]);
        let client = Client::new();
        let mut runner = Runner::_new(client.clone());
        runner.save_value("array".to_string(), serde_json::json!(["E1", "E2", "E3"]));

        let echo_vector: Arc<Mutex<Vec<String>>> = Arc::default();
        let echo_vector1 = echo_vector.clone();
        runner.set_echo(move |e| echo_vector1.lock().unwrap().push(e.to_string()));

        let res = runner.run(&file).await;
        assert!(res.is_ok());

        assert_eq!(echo_vector.lock().unwrap().len(), 3);
        assert_eq!(echo_vector.lock().unwrap()[0], "E1");
        assert_eq!(echo_vector.lock().unwrap()[1], "E2");
        assert_eq!(echo_vector.lock().unwrap()[2], "E3");
    }

    #[tokio::test]
    async fn test_open_relative_url() {
        let file = File::new(
            "".into(),
            "".into(),
            "http://example.com".into(),
            "".into(),
            vec![Test {
                id: String::new(),
                name: String::new(),
                commands: vec![Command::new(
                    "".to_owned(),
                    "".to_owned(),
                    Cmd::Open("/index.html".to_owned()),
                )],
            }],
        );

        let client = Client::with_functions(
            None,
            None,
            None,
            None,
            None,
            Some(|url| {
                assert_eq!(url, "http://example.com/index.html");
                Ok(())
            }),
            None,
            None,
            None,
            None,
        );
        let mut runner = Runner::_new(client.clone());

        runner.run(&file).await.unwrap();

        assert_eq!(client.calls()[Call::Goto], 1);
    }

    fn blank_file(commands: Vec<Cmd>) -> File {
        let commands = commands.into_iter().map(|cmd| blank_cmd(cmd)).collect();

        File {
            id: String::new(),
            name: String::new(),
            url: String::new(),
            version: String::new(),
            tests: vec![Test {
                id: String::new(),
                name: String::new(),
                commands,
            }],
        }
    }

    mod mock {
        use crate::error::RunnerErrorKind;
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
            #[allow(clippy::type_complexity)]
            pub res_goto: Option<fn(&str) -> Result<(), RunnerErrorKind>>,
            pub res_w8_visib: Option<fn() -> Result<(), RunnerErrorKind>>,
            pub res_w8_pres: Option<fn() -> Result<(), RunnerErrorKind>>,
            pub res_w8_npres: Option<fn() -> Result<(), RunnerErrorKind>>,
            pub res_w8_edit: Option<fn() -> Result<(), RunnerErrorKind>>,
        }

        impl Client {
            pub fn new() -> Arc<Self> {
                Arc::new(Self::default())
            }

            #[allow(clippy::type_complexity)]
            #[allow(clippy::too_many_arguments)]
            #[allow(clippy::field_reassign_with_default)]
            pub fn with_functions(
                res_find: Option<fn() -> Result<Element, RunnerErrorKind>>,
                res_curr_url: Option<fn() -> Result<url::Url, RunnerErrorKind>>,
                res_exec: Option<fn() -> Result<Json, RunnerErrorKind>>,
                res_set_w_size: Option<fn() -> Result<(), RunnerErrorKind>>,
                res_close: Option<fn() -> Result<(), RunnerErrorKind>>,
                res_goto: Option<fn(&str) -> Result<(), RunnerErrorKind>>,
                res_w8_visib: Option<fn() -> Result<(), RunnerErrorKind>>,
                res_w8_pres: Option<fn() -> Result<(), RunnerErrorKind>>,
                res_w8_npres: Option<fn() -> Result<(), RunnerErrorKind>>,
                res_w8_edit: Option<fn() -> Result<(), RunnerErrorKind>>,
            ) -> Arc<Self> {
                let mut client = Self::default();
                client.res_find = res_find;
                client.res_curr_url = res_curr_url;
                client.res_exec = res_exec;
                client.res_set_w_size = res_set_w_size;
                client.res_close = res_close;
                client.res_goto = res_goto;
                client.res_w8_visib = res_w8_visib;
                client.res_w8_pres = res_w8_pres;
                client.res_w8_npres = res_w8_npres;
                client.res_w8_edit = res_w8_edit;

                Arc::new(client)
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

            async fn goto(&mut self, url: &str) -> Result<(), Self::Error> {
                self.res_goto.as_ref().map(|f| (f)(url));
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

            async fn execute_async(&mut self, _: &str, _: Vec<Json>) -> Result<Json, Self::Error> {
                self.inc(Call::ExecAsync);
                Ok(Json::Null)
            }

            async fn close(&mut self) -> Result<(), Self::Error> {
                self.inc(Call::Close);
                Ok(())
            }

            async fn alert_text(&mut self) -> Result<String, Self::Error> {
                self.inc(Call::AlertText);
                Ok("".to_string())
            }

            async fn alert_accept(&mut self) -> Result<(), Self::Error> {
                self.inc(Call::AlertAccept);
                Ok(())
            }

            async fn alert_dissmis(&mut self) -> Result<(), Self::Error> {
                self.inc(Call::AlertDissmis);
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
                Ok(self.0)
            }

            async fn select_by_index(mut self, _: usize) -> Result<Self::Driver, Self::Error> {
                self.inc(Call::SelectByIndex);
                Ok(self.0)
            }

            async fn select_by_value(mut self, _: &str) -> Result<Self::Driver, Self::Error> {
                self.inc(Call::SelectByValue);
                Ok(self.0)
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
            exec_async: usize,
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
            alert_text: usize,
            alert_accept: usize,
            alert_dissmis: usize,
        }

        #[derive(Hash, PartialEq, Eq)]
        pub enum Call {
            Open,
            Click,
            Find,
            FindAll,
            Goto,
            Exec,
            ExecAsync,
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
            AlertText,
            AlertAccept,
            AlertDissmis,
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
                    Call::ExecAsync => &self.exec_async,
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
                    Call::AlertText => &self.alert_text,
                    Call::AlertAccept => &self.alert_accept,
                    Call::AlertDissmis => &self.alert_dissmis,
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
                    Call::ExecAsync => &mut self.exec_async,
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
                    Call::AlertText => &mut self.alert_text,
                    Call::AlertAccept => &mut self.alert_accept,
                    Call::AlertDissmis => &mut self.alert_dissmis,
                }
            }
        }
    }
}

fn blank_cmd(cmd: Cmd) -> Command {
    Command::new("".to_owned(), "".to_owned(), cmd)
}