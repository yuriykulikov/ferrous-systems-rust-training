use std::fmt;
use std::str::SplitN;

/// Redisish command\
/// The protocol has two commands:
///
/// * PUBLISH <message>\n
/// * RETRIEVE\n
#[derive(Eq, PartialEq, Debug)]
pub enum Command {
    Publish(String),
    Retrieve,
}

/// Redisish parsing error
#[derive(Eq, PartialEq, Debug)]
pub enum Error {
    MissingNewline,
    NewlineInMessage,
    Malformed(String),
    UnknownVerb,
}

///
/// # Parse redisish command
/// Parses redisish command and returns [Command] if successful or an [Error] otherwise
///
/// ## Protocol Specification
/// https://ferrous-systems.github.io/teaching-material/assignments/redisish.html
///
/// The protocol has two commands:
///
/// PUBLISH <message>\n
/// RETRIEVE\n
///
/// Edge cases:
/// * Messages cannot contain newlines. => NewlineInMessage
/// * Data after the first newline is an error. => NewlineInMessage
/// * A missing newline at the end of the message is an error => MissingNewline
/// * Empty messages are allowed. In this case, the message is PUBLISH \n.
///
/// Other cases (not part of the task):
/// * RETRIEVE does not have the payload, the only valid RETRIEVE message is `RETRIEVE/n`
pub fn parse(input: &str) -> Result<Command, Error> {
    check_preconditions(input)?;

    let mut split = input.trim_end_matches('\n').splitn(2, ' ');

    let verb = split.next();
    match verb {
        Some("PUBLISH") => parse_publish(&mut split),
        Some("RETRIEVE") => parse_retrieve(input, &mut split),
        _ => Err(Error::UnknownVerb),
    }
}

/// Check universal preconditions such as newline positions
fn check_preconditions(input: &str) -> Result<(), Error> {
    let first_newline_pos = input.find('\n');

    match first_newline_pos {
        // A missing newline at the end of the message is an error
        None => {
            return Err(Error::MissingNewline);
        }
        // Messages cannot contain newlines
        Some(index) if index != input.len() - 1 => {
            return Err(Error::NewlineInMessage);
        }
        _ => { /* OK */ }
    }

    Ok(())
}

fn parse_retrieve(input: &str, split: &mut SplitN<char>) -> Result<Command, Error> {
    if split.next().is_some() {
        return Err(Error::Malformed(format!("Malformed: {}", input)));
    }
    Ok(Command::Retrieve)
}

fn parse_publish(split: &mut SplitN<char>) -> Result<Command, Error> {
    let payload = split.next().unwrap_or("").into();
    Ok(Command::Publish(payload))
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Malformed(message) => write!(f, "Redisish error, malformed string: {}", message),
            Error::MissingNewline => write!(
                f,
                "Redisish error, missing newline at the end of the string"
            ),
            Error::NewlineInMessage => {
                write!(f, "Redisish error, newline is not at the end of the string")
            }
            Error::UnknownVerb => write!(f, "Redisish error, verb is unknown"),
        }
    }
}

impl Command {
    pub fn as_string(&self) -> String {
        match self {
            Command::Publish(payload) => {
                format!("PUBLISH {}\n", payload)
            }
            Command::Retrieve => "RETRIEVE\n".to_owned(),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use crate::{Command, Error};

    use super::*;

    #[test]
    fn test_publish_missing_newline() {
        let line = "PUBLISH TestMessage";
        let result: Result<Command, Error> = parse(line);
        let expected = Err(Error::MissingNewline);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_publish_newline_in_the_message() {
        let line = "PUBLISH Test\nMessage\n";
        let result: Result<Command, Error> = parse(line);
        let expected = Err(Error::NewlineInMessage);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_publish_ok() {
        let line = "PUBLISH TestMessage\n";
        let result: Result<Command, Error> = parse(line);
        let expected = Ok(Command::Publish("TestMessage".into()));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_publish_with_spaces_ok() {
        let line = "PUBLISH Test Mess age\n";
        let result: Result<Command, Error> = parse(line);
        let expected = Ok(Command::Publish("Test Mess age".into()));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_publish_with_spaces_at_the_end_ok() {
        let line = "PUBLISH TestMessage \n";
        let result: Result<Command, Error> = parse(line);
        let expected = Ok(Command::Publish("TestMessage ".into()));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_publish_empty_ok() {
        let line = "PUBLISH \n";
        let result: Result<Command, Error> = parse(line);
        let expected = Ok(Command::Publish("".to_owned()));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_retrieve_ok() {
        let line = "RETRIEVE\n";
        let result: Result<Command, Error> = parse(line);
        let expected = Ok(Command::Retrieve);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_retrieve_with_space_errors_with_malformed() {
        let line = "RETRIEVE \n";
        let result: Result<Command, Error> = parse(line);
        let expected = Err(Error::Malformed("Malformed: RETRIEVE \n".into()));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_retrieve_with_payload_errors_with_malformed() {
        let line = "RETRIEVE not allowed\n";
        let result: Result<Command, Error> = parse(line);
        let expected = Err(Error::Malformed("Malformed: RETRIEVE not allowed\n".into()));
        assert_eq!(result, expected);
    }

    #[test]
    fn unknown_command_errors_with_unknown_verb() {
        let line = "FOOBAR TestMessage\n";
        let result: Result<Command, Error> = parse(line);
        assert!(result.is_err());
        assert_eq!(result, Err(Error::UnknownVerb));
    }

    #[test]
    fn display_error_test() {
        assert_eq!(
            format!("{}", Error::UnknownVerb),
            "Redisish error, verb is unknown"
        );
        assert_eq!(
            format!("{}", Error::Malformed("oops".to_owned())),
            "Redisish error, malformed string: oops"
        );
        assert_eq!(
            format!("{}", Error::NewlineInMessage),
            "Redisish error, newline is not at the end of the string"
        );
        assert_eq!(
            format!("{}", Error::MissingNewline),
            "Redisish error, missing newline at the end of the string"
        );
    }
}
