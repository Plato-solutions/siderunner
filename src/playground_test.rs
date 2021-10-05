#![cfg(test)]

use crate::{
    parser::{Cmd, Command},
    playground::{build_nodes, Node, Transition},
    runner::Runner,
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
            Node::new(Cmd::Open("".to_owned()), 0, 0, Transition::Next),
            Node::new(Cmd::Echo("".to_owned()), 1, 0, Transition::Next)
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(Cmd::Echo("echo".to_owned()), 3, 0, Transition::Next),
            Node::new(Cmd::Echo("echo".to_owned()), 5, 0, Transition::Next),
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
            Node::new(
                Cmd::While("...".to_owned()),
                0,
                0,
                Transition::Conditional { next: 1, end: 4 }
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 1, 1, Transition::Next),
            Node::new(Cmd::End, 2, 0, Transition::Move(0)),
            Node::new(Cmd::Echo("echo".to_owned()), 4, 0, Transition::Next),
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(
                Cmd::While("...".to_owned()),
                1,
                0,
                Transition::Conditional { next: 2, end: 4 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 2, 1, Transition::Next),
            Node::new(Cmd::End, 3, 0, Transition::Move(1)),
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(
                Cmd::If("...".to_owned()),
                1,
                0,
                Transition::Conditional { next: 2, end: 3 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 2, 1, Transition::Next),
            Node::new(Cmd::End, 3, 0, Transition::Next),
            Node::new(Cmd::Echo("echo".to_owned()), 4, 0, Transition::Next),
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(
                Cmd::If("...".to_owned()),
                1,
                0,
                Transition::Conditional { next: 4, end: 2 },
            ),
            Node::new(
                Cmd::ElseIf("...".to_owned()),
                2,
                0,
                Transition::Conditional { next: 4, end: 3 },
            ),
            Node::new(Cmd::Else, 3, 0, Transition::Next),
            Node::new(Cmd::End, 4, 0, Transition::Next),
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(
                Cmd::If("...".to_owned()),
                1,
                0,
                Transition::Conditional { next: 2, end: 3 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 2, 1, Transition::Move(8)),
            Node::new(
                Cmd::ElseIf("...".to_owned()),
                3,
                0,
                Transition::Conditional { next: 4, end: 6 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 4, 1, Transition::Next),
            Node::new(Cmd::Echo("echo".to_owned()), 5, 1, Transition::Move(8)),
            Node::new(Cmd::Else, 6, 0, Transition::Next),
            Node::new(Cmd::Echo("echo".to_owned()), 7, 1, Transition::Next),
            Node::new(Cmd::End, 8, 0, Transition::Next)
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(
                Cmd::If("...".to_owned()),
                1,
                0,
                Transition::Conditional { next: 2, end: 3 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 2, 1, Transition::Move(5)),
            Node::new(
                Cmd::ElseIf("...".to_owned()),
                3,
                0,
                Transition::Conditional { next: 4, end: 5 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 4, 1, Transition::Next),
            Node::new(Cmd::End, 5, 0, Transition::Next),
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(
                Cmd::While("...".to_owned()),
                1,
                0,
                Transition::Conditional { next: 2, end: 6 },
            ),
            Node::new(
                Cmd::While("...".to_owned()),
                2,
                1,
                Transition::Conditional { next: 3, end: 5 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 3, 2, Transition::Next),
            Node::new(Cmd::End, 4, 1, Transition::Move(2)),
            Node::new(Cmd::End, 5, 0, Transition::Move(1)),
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(
                Cmd::While("...".to_owned()),
                1,
                0,
                Transition::Conditional { next: 2, end: 9 },
            ),
            Node::new(
                Cmd::While("...".to_owned()),
                2,
                1,
                Transition::Conditional { next: 3, end: 8 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 3, 2, Transition::Next),
            Node::new(
                Cmd::If("...".to_owned()),
                4,
                2,
                Transition::Conditional { next: 6, end: 5 },
            ),
            Node::new(Cmd::Else, 5, 2, Transition::Next),
            Node::new(Cmd::End, 6, 2, Transition::Next),
            Node::new(Cmd::End, 7, 1, Transition::Move(2)),
            Node::new(Cmd::End, 8, 0, Transition::Move(1)),
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(
                Cmd::While("...".to_owned()),
                1,
                0,
                Transition::Conditional { next: 2, end: 7 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 2, 1, Transition::Next),
            Node::new(
                Cmd::If("...".to_owned()),
                3,
                1,
                Transition::Conditional { next: 5, end: 4 },
            ),
            Node::new(Cmd::Else, 4, 1, Transition::Next),
            Node::new(Cmd::End, 5, 1, Transition::Next),
            Node::new(Cmd::End, 6, 0, Transition::Move(1)),
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(Cmd::Do, 1, 0, Transition::Next),
            Node::new(Cmd::Echo("echo".to_owned()), 2, 1, Transition::Next),
            Node::new(
                Cmd::RepeatIf("...".to_owned()),
                3,
                0,
                Transition::Conditional { next: 1, end: 4 },
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
            Node::new(Cmd::Open("open".to_owned()), 0, 0, Transition::Next),
            Node::new(Cmd::Do, 1, 0, Transition::Next),
            Node::new(
                Cmd::If("".to_owned()),
                2,
                1,
                Transition::Conditional { next: 6, end: 3 }
            ),
            Node::new(
                Cmd::ElseIf("".to_owned()),
                3,
                1,
                Transition::Conditional { next: 4, end: 5 },
            ),
            Node::new(Cmd::Echo("echo".to_owned()), 4, 2, Transition::Move(6)),
            Node::new(Cmd::Else, 5, 1, Transition::Next),
            Node::new(Cmd::End, 6, 1, Transition::Next),
            Node::new(
                Cmd::RepeatIf("...".to_owned()),
                7,
                0,
                Transition::Conditional { next: 1, end: 8 },
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
            Node::new(Cmd::Open("".to_owned()), 0, 0, Transition::Next),
            Node::new(Cmd::Do, 1, 0, Transition::Next),
            Node::new(
                Cmd::While("".to_owned()),
                2,
                1,
                Transition::Conditional { next: 3, end: 9 },
            ),
            Node::new(
                Cmd::If("".to_owned()),
                3,
                2,
                Transition::Conditional { next: 7, end: 4 },
            ),
            Node::new(
                Cmd::ElseIf("".to_owned()),
                4,
                2,
                Transition::Conditional { next: 5, end: 6 },
            ),
            Node::new(Cmd::Echo("".to_owned()), 5, 3, Transition::Move(7)),
            Node::new(Cmd::Else, 6, 2, Transition::Next),
            Node::new(Cmd::End, 7, 2, Transition::Next),
            Node::new(Cmd::End, 8, 1, Transition::Move(2)),
            Node::new(
                Cmd::RepeatIf("".to_owned()),
                9,
                0,
                Transition::Conditional { next: 1, end: 10 },
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
            Node::new(
                Cmd::While("..".to_owned()),
                0,
                0,
                Transition::Conditional { next: 1, end: 4 },
            ),
            Node::new(Cmd::Do, 1, 1, Transition::Next),
            Node::new(
                Cmd::RepeatIf("...".to_owned()),
                2,
                1,
                Transition::Conditional { next: 1, end: 3 },
            ),
            Node::new(Cmd::End, 3, 0, Transition::Move(0)),
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
            Node::new(
                Cmd::While(String::new()),
                0,
                0,
                Transition::Conditional { next: 1, end: 4 },
            ),
            Node::new(
                Cmd::If(String::new()),
                1,
                1,
                Transition::Conditional { next: 2, end: 2 }
            ),
            Node::new(Cmd::End, 2, 1, Transition::Next),
            Node::new(Cmd::End, 3, 0, Transition::Move(0)),
            Node::new(
                Cmd::While(String::new()),
                4,
                0,
                Transition::Conditional { next: 5, end: 6 }
            ),
            Node::new(Cmd::End, 5, 0, Transition::Move(4)),
            Node::new(
                Cmd::While(String::new()),
                6,
                0,
                Transition::Conditional { next: 7, end: 11 }
            ),
            Node::new(
                Cmd::If(String::new()),
                7,
                1,
                Transition::Conditional { next: 9, end: 8 }
            ),
            Node::new(Cmd::Else, 8, 1, Transition::Next),
            Node::new(Cmd::End, 9, 1, Transition::Next),
            Node::new(Cmd::End, 10, 0, Transition::Move(6)),
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
    async fn test_times() {
        let file = blank_file(vec![
            Cmd::Open("http://example.com".to_string()),
            Cmd::Times("4".to_string()),
            Cmd::Echo("".to_string()),
            Cmd::End,
        ]);
        let client = Client::new();
        let mut runner = Runner::_new(client.clone());

        let echo_counter: Arc<Mutex<usize>> = Arc::default();
        let echo_counter1 = echo_counter.clone();
        runner.set_echo(move |_| *echo_counter1.lock().unwrap() += 1);

        let res = runner.run(&file).await;
        assert!(res.is_ok());

        assert_eq!(*echo_counter.lock().unwrap(), 4);
    }

    #[tokio::test]
    async fn test_times_with_var() {
        let file = blank_file(vec![
            Cmd::Open("http://example.com".to_string()),
            Cmd::Times("${N}".to_string()),
            Cmd::Echo("".to_string()),
            Cmd::End,
        ]);
        let client = Client::new();
        let mut runner = Runner::_new(client.clone());
        runner.save_value("N".to_string(), 4u64.into());

        let echo_counter: Arc<Mutex<usize>> = Arc::default();
        let echo_counter1 = echo_counter.clone();
        runner.set_echo(move |_| *echo_counter1.lock().unwrap() += 1);

        let res = runner.run(&file).await;
        assert!(res.is_ok());

        assert_eq!(*echo_counter.lock().unwrap(), 4);
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
        let commands = commands.into_iter().map(blank_cmd).collect();

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

            async fn goto(&mut self, url: &str) -> Result<(), RunnerErrorKind> {
                self.res_goto.as_ref().map(|f| (f)(url));
                self.inc(Call::Goto);
                Ok(())
            }

            async fn find(&mut self, _: Locator) -> Result<Self::Element, RunnerErrorKind> {
                self.inc(Call::Find);
                Ok(Element(Arc::clone(self)))
            }

            async fn find_all(
                &mut self,
                _: Locator,
            ) -> Result<Vec<Self::Element>, RunnerErrorKind> {
                self.inc(Call::FindAll);
                Ok(vec![Element(Arc::clone(self))])
            }

            async fn wait_for_visible(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), RunnerErrorKind> {
                self.inc(Call::W8Visib);
                Ok(())
            }

            async fn wait_for_not_visible(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), RunnerErrorKind> {
                self.inc(Call::W8NotVisib);
                Ok(())
            }

            async fn wait_for_not_present(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), RunnerErrorKind> {
                self.inc(Call::W8NPres);
                Ok(())
            }

            async fn wait_for_present(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), RunnerErrorKind> {
                self.inc(Call::W8Pres);
                Ok(())
            }

            async fn wait_for_editable(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), RunnerErrorKind> {
                self.inc(Call::W8Edit);
                Ok(())
            }

            async fn wait_for_not_editable(
                &mut self,
                _: Locator,
                _: Duration,
            ) -> Result<(), RunnerErrorKind> {
                self.inc(Call::W8NotEdit);
                Ok(())
            }

            async fn current_url(&mut self) -> Result<url::Url, RunnerErrorKind> {
                self.inc(Call::CurrentUrl);
                Ok(url::Url::parse("http://example.com").unwrap())
            }

            async fn set_window_size(&mut self, _: u32, _: u32) -> Result<(), RunnerErrorKind> {
                self.inc(Call::SetWSize);
                Ok(())
            }

            async fn execute(&mut self, _: &str, _: Vec<Json>) -> Result<Json, RunnerErrorKind> {
                self.inc(Call::Exec);
                Ok(Json::Null)
            }

            async fn execute_async(
                &mut self,
                _: &str,
                _: Vec<Json>,
            ) -> Result<Json, RunnerErrorKind> {
                self.inc(Call::ExecAsync);
                Ok(Json::Null)
            }

            async fn close(&mut self) -> Result<(), RunnerErrorKind> {
                self.inc(Call::Close);
                Ok(())
            }

            async fn alert_text(&mut self) -> Result<String, RunnerErrorKind> {
                self.inc(Call::AlertText);
                Ok("".to_string())
            }

            async fn alert_accept(&mut self) -> Result<(), RunnerErrorKind> {
                self.inc(Call::AlertAccept);
                Ok(())
            }

            async fn alert_dissmis(&mut self) -> Result<(), RunnerErrorKind> {
                self.inc(Call::AlertDissmis);
                Ok(())
            }

            async fn double_click(&mut self, _: Locator) -> Result<(), RunnerErrorKind> {
                self.inc(Call::DoubleClick);
                Ok(())
            }

            async fn mouse_down(&mut self, _: Locator) -> Result<(), RunnerErrorKind> {
                self.inc(Call::MouseDown);
                Ok(())
            }

            async fn mouse_up(&mut self, _: Locator) -> Result<(), RunnerErrorKind> {
                self.inc(Call::MouseUp);
                Ok(())
            }

            async fn title(&mut self) -> Result<String, RunnerErrorKind> {
                self.inc(Call::Title);
                Ok(String::new())
            }

            async fn click_at(&mut self, _: Locator, _: (i32, i32)) -> Result<(), RunnerErrorKind> {
                self.inc(Call::ClickAt);
                Ok(())
            }

            async fn double_click_at(
                &mut self,
                _: Locator,
                _: (i32, i32),
            ) -> Result<(), RunnerErrorKind> {
                self.inc(Call::DoubleClickAt);
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

            async fn attr(&mut self, _: &str) -> Result<Option<String>, RunnerErrorKind> {
                self.inc(Call::Attr);
                Ok(None)
            }

            async fn prop(&mut self, _: &str) -> Result<Option<String>, RunnerErrorKind> {
                self.inc(Call::Prop);
                Ok(None)
            }

            async fn text(&mut self) -> Result<String, RunnerErrorKind> {
                self.inc(Call::Text);
                Ok("".to_string())
            }

            async fn html(&mut self, _: bool) -> Result<String, RunnerErrorKind> {
                self.inc(Call::Html);
                Ok("".to_string())
            }

            async fn find(&mut self, _: Locator) -> Result<Self, RunnerErrorKind>
            where
                Self: Sized,
            {
                self.inc(Call::Find);
                Ok(Element(self.0.clone()))
            }

            async fn click(mut self) -> Result<Self::Driver, RunnerErrorKind> {
                self.inc(Call::Click);
                Ok(self.0)
            }

            async fn select_by_index(mut self, _: usize) -> Result<Self::Driver, RunnerErrorKind> {
                self.inc(Call::SelectByIndex);
                Ok(self.0)
            }

            async fn select_by_value(mut self, _: &str) -> Result<Self::Driver, RunnerErrorKind> {
                self.inc(Call::SelectByValue);
                Ok(self.0)
            }

            async fn send_keys(mut self, _: &str) -> Result<(), RunnerErrorKind> {
                self.inc(Call::SendKeys);
                Ok(())
            }

            async fn select_by_label(mut self, _: &str) -> Result<Self::Driver, RunnerErrorKind> {
                self.inc(Call::SelectByLabel);
                Ok(self.0)
            }

            async fn is_selected(&mut self) -> Result<bool, RunnerErrorKind> {
                self.inc(Call::IsSelected);
                Ok(true)
            }

            async fn is_present(&mut self) -> Result<bool, RunnerErrorKind> {
                self.inc(Call::IsPresent);
                Ok(true)
            }

            async fn is_enabled(&mut self) -> Result<bool, RunnerErrorKind> {
                self.inc(Call::IsEnabled);
                Ok(true)
            }
        }

        #[derive(Clone, Default)]
        pub struct CallCount {
            inner: std::collections::HashMap<Call, usize>,
        }

        #[derive(Hash, PartialEq, Eq, Clone)]
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
            W8NotVisib,
            W8Pres,
            W8NPres,
            W8Edit,
            W8NotEdit,
            Attr,
            Prop,
            Text,
            Html,
            SelectByIndex,
            SelectByValue,
            SelectByLabel,
            AlertText,
            AlertAccept,
            AlertDissmis,
            DoubleClick,
            SendKeys,
            MouseDown,
            MouseUp,
            Title,
            IsSelected,
            IsPresent,
            IsEnabled,
            ClickAt,
            DoubleClickAt,
        }

        impl Index<Call> for CallCount {
            type Output = usize;

            fn index(&self, call: Call) -> &Self::Output {
                self.inner.get(&call).unwrap()
            }
        }

        impl IndexMut<Call> for CallCount {
            fn index_mut(&mut self, call: Call) -> &mut Self::Output {
                self.inner.entry(call).or_default()
            }
        }
    }
}

fn blank_cmd(cmd: Cmd) -> Command {
    Command::new("".to_owned(), "".to_owned(), cmd)
}
