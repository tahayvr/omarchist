//! What a flow expects to find on the machine: the program each command
//! step starts and whatever `meta.requires` lists. Missing ones are shown
//! before saving so a shared flow does not fail at run time.
use std::collections::BTreeSet;
use std::path::Path;

use super::{Flow, StepKind};

/// Words the shell handles itself; a step starting with one is not a
/// program to look for.
const BUILTINS: &[&str] = &[
    ".", "[", "alias", "builtin", "case", "cd", "command", "eval", "exec", "exit", "export", "for",
    "function", "if", "local", "read", "return", "set", "shift", "source", "test", "then", "trap",
    "type", "unset", "until", "wait", "while",
];

/// The program a command step starts, when the command is a plain program
/// invocation rather than shell syntax (`FOO=1 cmd`, `$EDITOR`, `{ ...; }`)
/// or a builtin. A path is kept as a path, with `~` expanded, so it is
/// checked where it says rather than on `PATH`.
pub fn program_of(kind: &StepKind) -> Option<String> {
    let StepKind::Exec { command, .. } = kind else {
        return None;
    };
    let word = command.split_whitespace().next()?;
    let plain = !word.contains('=')
        && !word.starts_with(['$', '(', '{', '|', '&', ';', '!', '"', '\''])
        && !BUILTINS.contains(&word);
    if !plain {
        return None;
    }
    let expanded = match word.strip_prefix("~/") {
        Some(rest) => dirs::home_dir()?.join(rest).display().to_string(),
        None => word.to_string(),
    };
    Some(expanded)
}

/// Programs from the enabled command steps and `meta.requires` that are not
/// on `PATH`, each once, in order.
pub fn missing_programs(flow: &Flow) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut missing = Vec::new();
    let from_steps = flow
        .steps
        .iter()
        .filter(|step| step.enabled)
        .filter_map(|step| program_of(&step.kind));
    for program in from_steps.chain(flow.meta.requires.iter().cloned()) {
        if seen.insert(program.clone()) && !is_installed(&program) {
            missing.push(program);
        }
    }
    missing
}

/// Whether `program` is an executable file, by absolute path or on `PATH`.
pub fn is_installed(program: &str) -> bool {
    if program.contains('/') {
        return is_executable(Path::new(program));
    }
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).any(|dir| is_executable(&dir.join(program))))
        .unwrap_or(false)
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::super::{Meta, Step};
    use super::*;

    fn exec(command: &str) -> Step {
        Step::new(StepKind::Exec {
            command: command.into(),
            wait: false,
        })
    }

    #[test]
    fn plain_programs_are_named_and_shell_syntax_is_not() {
        assert_eq!(program_of(&exec("sh -c 'x'").kind).as_deref(), Some("sh"));
        assert_eq!(
            program_of(&exec("/usr/bin/env python").kind).as_deref(),
            Some("/usr/bin/env")
        );
        assert_eq!(program_of(&exec("FOO=1 cmd").kind), None);
        assert_eq!(program_of(&exec("$EDITOR file").kind), None);
        assert_eq!(program_of(&exec("cd ~/proj && code .").kind), None);
        assert_eq!(program_of(&exec("exec omarchy-launch-terminal").kind), None);
        assert_eq!(program_of(&StepKind::Wait { ms: 1 }), None);
        let home = dirs::home_dir().unwrap();
        assert_eq!(
            program_of(&exec("~/scripts/deploy.sh now").kind),
            Some(home.join("scripts/deploy.sh").display().to_string())
        );
    }

    #[test]
    fn missing_programs_come_from_enabled_steps_and_requires_once() {
        let mut flow = Flow::new("demo".into(), "Demo".into());
        flow.steps.push(exec("sh -c true"));
        flow.steps.push(exec("omarchist-surely-missing-program"));
        flow.steps
            .push(exec("omarchist-surely-missing-program --again"));
        let mut off = exec("omarchist-also-missing-but-off");
        off.enabled = false;
        flow.steps.push(off);
        flow.meta = Meta {
            requires: vec!["sh".into(), "omarchist-missing-requirement".into()],
            ..Meta::default()
        };
        assert_eq!(
            missing_programs(&flow),
            vec![
                "omarchist-surely-missing-program".to_string(),
                "omarchist-missing-requirement".to_string()
            ]
        );
    }

    #[test]
    fn installed_looks_at_path_and_absolute_paths() {
        assert!(is_installed("sh"));
        assert!(is_installed("/bin/sh"));
        assert!(!is_installed("omarchist-surely-missing-program"));
        assert!(!is_installed("/nonexistent/omarchist"));
    }
}
