//! The shells and their command lines.

/// The command line of `program` with `args`, quoted so that the program
/// reads the same arguments, or `None` if `program` has a quote, which no
/// file name can have.
///
/// Windows gives a program its command line as one text, and each program
/// splits it: almost all of them with the rules of the C runtime of
/// Microsoft, which `CommandLineToArgvW` also follows. An argument with
/// spaces or tabs, or empty, goes in quotes; a quote inside an argument is
/// written `\"`; and the backslashes before a quote, or before the end of a
/// quoted argument, are doubled. The other backslashes stay as they are, as
/// in `C:\Windows`. The program is only quoted if it has spaces or tabs:
/// `CreateProcessW` reads it up to the first one, or up to the closing quote.
pub fn command_line<S: AsRef<str>>(program: &str, args: &[S]) -> Option<String> {
    if program.contains('"') {
        return None;
    }
    let mut line = String::with_capacity(
        program.len() + args.iter().map(|arg| arg.as_ref().len() + 3).sum::<usize>() + 2,
    );
    if needs_quotes(program) || program.is_empty() {
        line.push('"');
        line.push_str(program);
        line.push('"');
    } else {
        line.push_str(program);
    }
    for arg in args {
        line.push(' ');
        push_argument(&mut line, arg.as_ref());
    }
    Some(line)
}

fn needs_quotes(arg: &str) -> bool {
    arg.contains([' ', '\t'])
}

/// Adds `arg` to `line`, quoted as the C runtime reads it.
fn push_argument(line: &mut String, arg: &str) {
    let quote = arg.is_empty() || needs_quotes(arg);
    if quote {
        line.push('"');
    }
    let mut backslashes = 0;
    for c in arg.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                // The backslashes before it, and the quote, are escaped.
                line.extend(std::iter::repeat_n('\\', backslashes + 1));
                backslashes = 0;
            }
            _ => backslashes = 0,
        }
        line.push(c);
    }
    if quote {
        // The closing quote must not be escaped by the backslashes.
        line.extend(std::iter::repeat_n('\\', backslashes));
        line.push('"');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(program: &str, args: &[&str]) -> String {
        command_line(program, args).unwrap()
    }

    #[test]
    fn simple_arguments_stay_as_they_are() {
        assert_eq!(line("pwsh", &[]), "pwsh");
        assert_eq!(
            line("pwsh", &["-NoProfile", "-File", r"C:\perf\spinner.ps1"]),
            r"pwsh -NoProfile -File C:\perf\spinner.ps1"
        );
    }

    #[test]
    fn spaces_tabs_and_empty_arguments_go_in_quotes() {
        assert_eq!(
            line(
                r"C:\Program Files\PowerShell\7\pwsh.exe",
                &["-File", r"C:\my scripts\a b.ps1", "", "a\tb"]
            ),
            "\"C:\\Program Files\\PowerShell\\7\\pwsh.exe\" -File \
             \"C:\\my scripts\\a b.ps1\" \"\" \"a\tb\""
        );
    }

    #[test]
    fn quotes_and_the_backslashes_before_them_are_escaped() {
        assert_eq!(line("x", &[r#"say "ola""#]), r#"x "say \"ola\"""#);
        assert_eq!(line("x", &[r#"a\"b"#]), r#"x a\\\"b"#);
        assert_eq!(line("x", &[r"a\\b"]), r"x a\\b");
        // A quoted argument that ends in backslashes.
        assert_eq!(line("x", &[r"C:\a b\"]), r#"x "C:\a b\\""#);
        assert_eq!(line("x", &[r"C:\a\"]), r"x C:\a\");
    }

    #[test]
    fn a_program_cannot_have_quotes() {
        assert_eq!(command_line::<&str>(r#"a"b"#, &[]), None);
        assert_eq!(command_line::<&str>("", &[]).as_deref(), Some("\"\""));
    }
}
