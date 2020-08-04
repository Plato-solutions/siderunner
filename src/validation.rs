use crate::{error::ParseError, parser::Command, Result};

// TODO: error with position when
//      there's missing END
//      there's more ENDs
//      END used wrongly
pub fn validate_conditions(commands: &[Command]) -> Result<()> {
    let mut state = Vec::new();
    for cmd in commands {
        validate(cmd, &mut state)?
    }

    if !state.is_empty() {
        Err(ParseError::ValidationError("incomplete block".to_owned()))?
    } else {
        Ok(())
    }
}

enum State {
    While,
    If,
    ElseIf,
    Else,
    End,
}

fn validate(cmd: &Command, state: &mut Vec<State>) -> Result<()> {
    match cmd {
        Command::While(..) => {
            state.push(State::While);
            Ok(())
        }
        Command::If(..) => {
            state.push(State::If);
            Ok(())
        }
        Command::ElseIf(..) => validate_else_if(state),
        Command::Else => validate_else(state),
        Command::End => validate_end(state),
        _ => Ok(()),
    }
}

fn validate_end(state: &mut Vec<State>) -> Result<()> {
    match state.last() {
        Some(st) if matches!(st, State::While | State::If) => {
            state.pop();
            Ok(())
        }
        Some(st) if matches!(st, State::ElseIf | State::Else) => {
            state.pop();
            validate_end(state)
        }
        _ => Err(ParseError::ValidationError(
            "end used in wrong way".to_owned(),
        ))?,
    }
}

fn validate_else(state: &mut Vec<State>) -> Result<()> {
    match state.last() {
        Some(st) if matches!(st, State::If | State::ElseIf) => {
            state.push(State::Else);
            Ok(())
        }
        Some(st) if matches!(st, State::Else) => Err(ParseError::ValidationError(
            "too many else operations".to_owned(),
        ))?,
        _ => Err(ParseError::ValidationError(
            "else used out of if scope".to_owned(),
        ))?,
    }
}

fn validate_else_if(state: &mut Vec<State>) -> Result<()> {
    match state.last() {
        Some(st) if matches!(st, State::If | State::ElseIf) => {
            state.push(State::ElseIf);
            Ok(())
        }
        Some(st) if matches!(st, State::Else) => Err(ParseError::ValidationError(
            "usage of elseif after else".to_owned(),
        ))?,
        _ => Err(ParseError::ValidationError(
            "else if used outside the if scope".to_owned(),
        ))?,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let commands = vec![Command::While("...".to_owned()), Command::End];
        assert!(validate_conditions(&commands).is_ok());
        let commands = vec![Command::If("...".to_owned()), Command::End];
        assert!(validate_conditions(&commands).is_ok());
        let commands = vec![Command::If("...".to_owned()), Command::Else, Command::End];
        assert!(validate_conditions(&commands).is_ok());
        let commands = vec![
            Command::If("...".to_owned()),
            Command::ElseIf("...".to_owned()),
            Command::Else,
            Command::End,
        ];
        assert!(validate_conditions(&commands).is_ok());
        let commands = vec![
            Command::If("...".to_owned()),
            Command::ElseIf("...".to_owned()),
            Command::ElseIf("...".to_owned()),
            Command::Else,
            Command::End,
        ];
        assert!(validate_conditions(&commands).is_ok());
    }

    #[test]
    fn test_validation_missed_end() {
        let commands = vec![Command::While("...".to_owned())];
        assert!(validate_conditions(&commands).is_err());
        let commands = vec![Command::If("...".to_owned())];
        assert!(validate_conditions(&commands).is_err());
        let commands = vec![Command::If("...".to_owned()), Command::Else];
        assert!(validate_conditions(&commands).is_err());
        let commands = vec![
            Command::If("...".to_owned()),
            Command::Else,
            Command::ElseIf("...".to_owned()),
        ];
        assert!(validate_conditions(&commands).is_err());
        let commands = vec![
            Command::If("...".to_owned()),
            Command::Else,
            Command::ElseIf("...".to_owned()),
            Command::ElseIf("...".to_owned()),
        ];
        assert!(validate_conditions(&commands).is_err());
    }

    #[test]
    fn test_validation_wrong_end() {
        let commands = vec![Command::End];
        assert!(validate_conditions(&commands).is_err());
        let commands = vec![Command::If("...".to_owned()), Command::End, Command::Else];
        assert!(validate_conditions(&commands).is_err());
        let commands = vec![
            Command::If("...".to_owned()),
            Command::End,
            Command::Else,
            Command::ElseIf("...".to_owned()),
        ];
        assert!(validate_conditions(&commands).is_err());
        let commands = vec![
            Command::If("...".to_owned()),
            Command::Else,
            Command::End,
            Command::ElseIf("...".to_owned()),
        ];
        assert!(validate_conditions(&commands).is_err());
    }
}
