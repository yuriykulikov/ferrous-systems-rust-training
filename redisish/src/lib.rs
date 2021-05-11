#[derive(Eq, PartialEq, Debug)]
pub enum Command {
    Publish(String),
    Retrieve,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
    MissingNewline,
    NewlineInMessage,
    Malformed(String),
    UnknownVerb,
}

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
    let first_newline_pos = input.find('\n');

    match first_newline_pos {
        // A missing newline at the end of the message is an error
        None => { return Err(Error::MissingNewline); }
        // Messages cannot contain newlines
        Some(index) if index != input.len() - 1 => { return Err(Error::NewlineInMessage); }
        _ => { /* OK */ }
    }

    let mut split = input.trim_end_matches('\n').splitn(2, ' ');

    let verb = split.next();
    return match verb {
        Some("PUBLISH") => {
            let payload = split.next().unwrap_or("").into();
            Ok(Command::Publish(payload))
        }
        Some("RETRIEVE") => {
            if split.next().is_some() {
                return Err(Error::Malformed(format!("Malformed: {}", input)));
            }
            Ok(Command::Retrieve)
        }
        _ => Err(Error::UnknownVerb),
    };
}

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
}
