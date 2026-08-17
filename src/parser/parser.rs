use crate::parser::ast::{Command, Redirect};

pub fn parse(tokens: Vec<String>) -> Option<Command> {
    if tokens.is_empty() {
        return None;
    }

    let name = tokens[0].clone();
    let mut args = Vec::new();
    let mut stdout = Redirect::None;
    let mut stderr = Redirect::None;

    let mut iter = tokens.into_iter().skip(1);

    while let Some(token) = iter.next() {
        match token.as_str() {
            ">" | "1>" => {
                if let Some(file) = iter.next() { stdout = Redirect::Overwrite(file); }
            }
            ">>" | "1>>" => {
                if let Some(file) = iter.next() { stdout = Redirect::Append(file); }
            }
            "2>" => {
                if let Some(file) = iter.next() { stderr = Redirect::Overwrite(file); }
            }
            "2>>" => {
                if let Some(file) = iter.next() { stderr = Redirect::Append(file); }
            }
            _ => {
                args.push(token);
            }
        }
    }

    Some(Command {
        name,
        args,
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::parser::ast::{Command, Redirect};

    fn v(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse(v(&[])).is_none());
    }

    #[test]
    fn test_parse_single_command() {
        let cmd = parse(v(&["ls"])).unwrap();
        assert_eq!(cmd.name, "ls");
        assert!(cmd.args.is_empty());
        assert!(matches!(cmd.stdout, Redirect::None));
        assert!(matches!(cmd.stderr, Redirect::None));
    }

    #[test]
    fn test_parse_command_with_args() {
        let cmd = parse(v(&["echo", "hello", "world"])).unwrap();
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, v(&["hello", "world"]));
    }

    #[test]
    fn test_parse_stdout_overwrite() {
        let cmd = parse(v(&["echo", "hi", ">", "file.txt"])).unwrap();
        assert!(matches!(cmd.stdout, Redirect::Overwrite(ref s) if s == "file.txt"));
    }

    #[test]
    fn test_parse_stdout_append() {
        let cmd = parse(v(&["echo", "hi", ">>", "file.txt"])).unwrap();
        assert!(matches!(cmd.stdout, Redirect::Append(ref s) if s == "file.txt"));
    }

    #[test]
    fn test_parse_stderr_overwrite() {
        let cmd = parse(v(&["cmd", "2>", "err.txt"])).unwrap();
        assert!(matches!(cmd.stderr, Redirect::Overwrite(ref s) if s == "err.txt"));
    }

    #[test]
    fn test_parse_stderr_append() {
        let cmd = parse(v(&["cmd", "2>>", "err.txt"])).unwrap();
        assert!(matches!(cmd.stderr, Redirect::Append(ref s) if s == "err.txt"));
    }

    #[test]
    fn test_parse_both_redirects() {
        let cmd = parse(v(&["cmd", ">", "out", "2>", "err"])).unwrap();
        assert!(matches!(cmd.stdout, Redirect::Overwrite(ref s) if s == "out"));
        assert!(matches!(cmd.stderr, Redirect::Overwrite(ref s) if s == "err"));
    }

    #[test]
    fn test_parse_named_stdout() {
        let cmd = parse(v(&["cmd", "1>", "out"])).unwrap();
        assert!(matches!(cmd.stdout, Redirect::Overwrite(ref s) if s == "out"));
    }

    #[test]
    fn test_parse_named_stdout_append() {
        let cmd = parse(v(&["cmd", "1>>", "out"])).unwrap();
        assert!(matches!(cmd.stdout, Redirect::Append(ref s) if s == "out"));
    }

    #[test]
    fn test_parse_args_before_redirect() {
        let cmd = parse(v(&["grep", "-rn", "pat", ">", "out"])).unwrap();
        assert_eq!(cmd.args, v(&["-rn", "pat"]));
        assert!(matches!(cmd.stdout, Redirect::Overwrite(ref s) if s == "out"));
    }

    #[test]
    fn test_parse_args_after_redirect() {
        let cmd = parse(v(&["cmd", ">", "out", "extra"])).unwrap();
        assert_eq!(cmd.args, v(&["extra"]));
        assert!(matches!(cmd.stdout, Redirect::Overwrite(ref s) if s == "out"));
    }

    #[test]
    fn test_parse_dangling_redirect() {
        let cmd = parse(v(&["cmd", ">"])).unwrap();
        assert!(matches!(cmd.stdout, Redirect::None));
    }

    #[test]
    fn test_parse_multiple_stdout_last_wins() {
        let cmd = parse(v(&["cmd", ">", "a", ">", "b"])).unwrap();
        assert!(matches!(cmd.stdout, Redirect::Overwrite(ref s) if s == "b"));
    }

    #[test]
    fn test_parse_many_args() {
        let expected_args: Vec<String> = (0..49).map(|i| i.to_string()).collect();
        let mut tokens_str: Vec<String> = vec!["cmd".to_string()];
        tokens_str.extend(expected_args.clone());
        let cmd = parse(tokens_str).unwrap();
        assert_eq!(cmd.args, expected_args);
    }

    #[test]
    fn test_ast_redirect_none_default() {
        let cmd = Command {
            name: "test".to_string(),
            args: vec![],
            stdout: Redirect::None,
            stderr: Redirect::None,
        };
        assert!(matches!(cmd.stdout, Redirect::None));
    }

    #[test]
    fn test_ast_command_fields_complete() {
        let cmd = Command {
            name: "cmd".to_string(),
            args: v(&["arg1"]),
            stdout: Redirect::Overwrite("out".to_string()),
            stderr: Redirect::Append("err".to_string()),
        };
        assert_eq!(cmd.name, "cmd");
        assert_eq!(cmd.args, v(&["arg1"]));
        assert!(matches!(cmd.stdout, Redirect::Overwrite(ref s) if s == "out"));
        assert!(matches!(cmd.stderr, Redirect::Append(ref s) if s == "err"));
    }

    #[test]
    fn test_ast_redirect_overwrite_contains_path() {
        let redir = Redirect::Overwrite("file".to_string());
        assert!(matches!(redir, Redirect::Overwrite(ref s) if s == "file"));
    }
}
