// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    error::{RunnerError, RunnerErrorKind},
    parser::{Cmd, Command},
};

/// Validate_conditions verifies a corrent state of command list.
/// That there's enough `Ends`, `Cycles` and `If` statements.
pub fn validate_conditions(commands: &[Command]) -> Result<(), RunnerError> {
    let mut state = Vec::new();
    for (index, command) in commands.iter().enumerate() {
        validate(&command.cmd, &mut state).map_err(|e| RunnerError::new(e, index))?
    }

    if !state.is_empty() {
        Err(RunnerError::new(
            RunnerErrorKind::BranchValidationError("incomplete block".to_owned()),
            commands.len(), // todo: investigate correct index
        ))
    } else {
        Ok(())
    }
}

enum State {
    While,
    If,
    ElseIf,
    Else,
    Do,
    ForEach,
    Times,
    #[allow(dead_code)]
    End,
}

fn validate(cmd: &Cmd, state: &mut Vec<State>) -> Result<(), RunnerErrorKind> {
    match cmd {
        Cmd::While(..) => {
            state.push(State::While);
            Ok(())
        }
        Cmd::If(..) => {
            state.push(State::If);
            Ok(())
        }
        Cmd::ElseIf(..) => validate_else_if(state),
        Cmd::Else => validate_else(state),
        Cmd::End => validate_end(state),
        Cmd::Do => {
            state.push(State::Do);
            Ok(())
        }
        Cmd::RepeatIf(..) => validate_do(state),
        Cmd::ForEach { .. } => {
            state.push(State::ForEach);
            Ok(())
        }
        Cmd::Times(..) => {
            state.push(State::Times);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_end(state: &mut Vec<State>) -> Result<(), RunnerErrorKind> {
    match state.last() {
        Some(st) if matches!(st, State::While | State::If | State::ForEach | State::Times) => {
            state.pop();
            Ok(())
        }
        Some(st) if matches!(st, State::ElseIf | State::Else) => {
            state.pop();
            validate_end(state)
        }
        _ => Err(RunnerErrorKind::BranchValidationError(
            "end used in wrong way".to_owned(),
        )),
    }
}

fn validate_else(state: &mut Vec<State>) -> Result<(), RunnerErrorKind> {
    match state.last() {
        Some(st) if matches!(st, State::If | State::ElseIf) => {
            state.push(State::Else);
            Ok(())
        }
        Some(st) if matches!(st, State::Else) => Err(RunnerErrorKind::BranchValidationError(
            "too many else operations".to_owned(),
        )),
        _ => Err(RunnerErrorKind::BranchValidationError(
            "else used out of if scope".to_owned(),
        )),
    }
}

fn validate_else_if(state: &mut Vec<State>) -> Result<(), RunnerErrorKind> {
    match state.last() {
        Some(st) if matches!(st, State::If | State::ElseIf) => {
            state.push(State::ElseIf);
            Ok(())
        }
        Some(st) if matches!(st, State::Else) => Err(RunnerErrorKind::BranchValidationError(
            "usage of elseif after else".to_owned(),
        )),
        _ => Err(RunnerErrorKind::BranchValidationError(
            "else if used outside the if scope".to_owned(),
        )),
    }
}

fn validate_do(state: &mut Vec<State>) -> Result<(), RunnerErrorKind> {
    match state.last() {
        Some(st) if matches!(st, State::Do) => {
            state.pop();
            Ok(())
        }
        _ => Err(RunnerErrorKind::BranchValidationError(
            "repeatIf used outside the if scope".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        assert!(
            validate_conditions(&commands(vec![Cmd::While("...".to_owned()), Cmd::End])).is_ok()
        );
        assert!(validate_conditions(&commands(vec![Cmd::If("...".to_owned()), Cmd::End])).is_ok());
        assert!(validate_conditions(&commands(vec![
            Cmd::If("...".to_owned()),
            Cmd::Else,
            Cmd::End
        ]))
        .is_ok());
        assert!(validate_conditions(&commands(vec![
            Cmd::If("...".to_owned()),
            Cmd::ElseIf("...".to_owned()),
            Cmd::Else,
            Cmd::End,
        ]))
        .is_ok());
        assert!(validate_conditions(&commands(vec![
            Cmd::If("...".to_owned()),
            Cmd::ElseIf("...".to_owned()),
            Cmd::ElseIf("...".to_owned()),
            Cmd::Else,
            Cmd::End,
        ]))
        .is_ok());
    }

    #[test]
    fn test_validation_missed_end() {
        assert!(validate_conditions(&commands(vec![Cmd::While("...".to_owned())])).is_err());
        assert!(validate_conditions(&commands(vec![Cmd::If("...".to_owned())])).is_err());
        assert!(
            validate_conditions(&commands(vec![Cmd::If("...".to_owned()), Cmd::Else])).is_err()
        );
        assert!(validate_conditions(&commands(vec![
            Cmd::If("...".to_owned()),
            Cmd::Else,
            Cmd::ElseIf("...".to_owned()),
        ]))
        .is_err());
        assert!(validate_conditions(&commands(vec![
            Cmd::If("...".to_owned()),
            Cmd::Else,
            Cmd::ElseIf("...".to_owned()),
            Cmd::ElseIf("...".to_owned()),
        ]))
        .is_err());
    }

    #[test]
    fn test_validation_wrong_end() {
        assert!(validate_conditions(&commands(vec![Cmd::End])).is_err());
        assert!(validate_conditions(&commands(vec![
            Cmd::If("...".to_owned()),
            Cmd::End,
            Cmd::Else
        ]))
        .is_err());
        assert!(validate_conditions(&commands(vec![
            Cmd::If("...".to_owned()),
            Cmd::End,
            Cmd::Else,
            Cmd::ElseIf("...".to_owned()),
        ]))
        .is_err());
        assert!(validate_conditions(&commands(vec![
            Cmd::If("...".to_owned()),
            Cmd::Else,
            Cmd::End,
            Cmd::ElseIf("...".to_owned()),
        ]))
        .is_err());
    }

    #[test]
    fn test_validation_missed_repeat_if() {
        assert!(validate_conditions(&commands(vec![
            Cmd::Open("".to_owned()),
            Cmd::Do,
            Cmd::Echo("".to_owned())
        ]))
        .is_err());
        assert!(validate_conditions(&commands(vec![
            Cmd::Open("".to_owned()),
            Cmd::Echo("".to_owned()),
            Cmd::RepeatIf("".to_owned()),
        ]))
        .is_err());
    }

    #[test]
    fn test_validation_for_each() {
        assert!(validate_conditions(&commands(vec![
            Cmd::ForEach {
                iterator: String::new(),
                var: String::new()
            },
            Cmd::Echo(String::new()),
            Cmd::End,
        ]))
        .is_ok());

        assert!(validate_conditions(&commands(vec![Cmd::ForEach {
            iterator: String::new(),
            var: String::new()
        }]))
        .is_err());
    }

    #[test]
    fn test_validation_times() {
        assert!(validate_conditions(&commands(vec![
            Cmd::Times(String::new()),
            Cmd::Echo(String::new()),
            Cmd::End,
        ]))
        .is_ok());

        assert!(validate_conditions(&commands(vec![Cmd::Times(String::new())])).is_err());
    }

    fn commands(cmds: Vec<Cmd>) -> Vec<Command> {
        cmds.into_iter().map(blank_cmd).collect()
    }

    fn blank_cmd(cmd: Cmd) -> Command {
        Command::new("".to_owned(), "".to_owned(), cmd)
    }
}
