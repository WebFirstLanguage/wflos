//! Command parser
//! Parses user input into commands

use super::commands::Command;

pub fn parse(input: &str) -> Result<Command<'_>, &'static str> {
    let input = input.trim();

    if input.is_empty() {
        return Ok(Command::Empty);
    }

    // Split into command and arguments
    let mut parts = input.split_whitespace();
    let cmd = parts.next().ok_or("No command")?;

    match cmd {
        "help" => Ok(Command::Help),
        "clear" => Ok(Command::Clear),
        "version" => Ok(Command::Version),
        "halt" => Ok(Command::Halt),
        "meminfo" => Ok(Command::MemInfo),
        "echo" => {
            // Get text after "echo"
            let text = input.strip_prefix("echo").unwrap_or("").trim();
            Ok(Command::Echo(text))
        }
        _ => Err("Unknown command. Type 'help' for available commands."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        let result = parse("help");
        assert!(matches!(result, Ok(Command::Help)));
    }

    #[test]
    fn test_parse_clear() {
        let result = parse("clear");
        assert!(matches!(result, Ok(Command::Clear)));
    }

    #[test]
    fn test_parse_version() {
        let result = parse("version");
        assert!(matches!(result, Ok(Command::Version)));
    }

    #[test]
    fn test_parse_echo() {
        let result = parse("echo hello world");
        if let Ok(Command::Echo(text)) = result {
            assert_eq!(text, "hello world");
        } else {
            panic!("Expected Echo command");
        }
    }

    #[test]
    fn test_parse_empty() {
        let result = parse("");
        assert!(matches!(result, Ok(Command::Empty)));
    }

    #[test]
    fn test_parse_whitespace() {
        let result = parse("   ");
        assert!(matches!(result, Ok(Command::Empty)));
    }

    #[test]
    fn test_parse_unknown() {
        let result = parse("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_extra_whitespace() {
        let result = parse("  help  ");
        assert!(matches!(result, Ok(Command::Help)));
    }
}
