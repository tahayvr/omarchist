//! Values that pass between the steps of one run: a step can save its
//! output under a name (`output = "url"`), and later steps use it as
//! `{{url}}` in a command, a notification, or a Hyprland action. A few
//! built-in variables (`{{clipboard}}`, `{{date}}`, ...) are read when a
//! step first needs them.
//!
//! Substitution never turns a value into code. In a shell command each
//! reference becomes a quoted expansion of an environment variable that
//! holds the value, quoted to fit where the reference sits (bare, inside
//! double quotes, or inside single quotes). In a Hyprland action a
//! reference may only sit inside a string literal, and the value is escaped
//! for it; the result must still pass the `hl.dsp.*(...)` literal guard.
use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};
use crate::system::keybinds::overrides::is_dsp_call;

/// Built-in variables, with what each holds, in the order the editor
/// offers them.
pub const BUILTINS: &[(&str, &str)] = &[
    ("clipboard", "What is on the clipboard"),
    ("selection", "The text selected anywhere"),
    ("date", "Today, as 2026-10-04"),
    ("time", "The time, as 14:05"),
    ("window", "The focused window's title"),
    ("app", "The focused window's app"),
    ("workspace", "The current workspace"),
    ("input", "What the flow was started with"),
];

/// Names the loop steps set for the steps inside them: the current line
/// of a "repeat with each", and the round, counting from 1.
pub const LOOP_NAMES: &[&str] = &["item", "index"];

/// A saved output is cut to this many bytes, so a chatty command cannot
/// fill memory or an environment block.
pub const MAX_VALUE_BYTES: usize = 64 * 1024;

/// Names a step can save its output under: a letter, then letters, digits,
/// spaces, hyphens or underscores, at most 32 characters. Matched without
/// regard to case or surrounding spaces.
pub fn is_name(name: &str) -> bool {
    let name = name.trim();
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
        && name.len() <= 32
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_'))
}

/// The form a name is stored and compared in.
pub fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

pub fn is_builtin(name: &str) -> bool {
    let name = normalize(name);
    BUILTINS.iter().any(|(b, _)| *b == name)
}

pub fn is_loop_name(name: &str) -> bool {
    LOOP_NAMES.contains(&normalize(name).as_str())
}

/// Whether a step cannot save its output under `name` because Omarchist
/// sets it itself.
pub fn is_reserved(name: &str) -> bool {
    is_builtin(name) || is_loop_name(name)
}

/// One `{{name}}` in a text: its byte range and the normalized name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub start: usize,
    pub end: usize,
    pub name: String,
}

/// Every `{{name}}` in `text`, in order. Text between braces that is not a
/// valid name (`{{ }}`, `{{a.b}}`) is left alone as plain text.
pub fn references(text: &str) -> Vec<Reference> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(open) = text[from..].find("{{").map(|i| from + i) {
        let Some(close) = text[open + 2..].find("}}").map(|i| open + 2 + i) else {
            break;
        };
        let inner = &text[open + 2..close];
        if is_name(inner) {
            found.push(Reference {
                start: open,
                end: close + 2,
                name: normalize(inner),
            });
            from = close + 2;
        } else {
            from = open + 2;
        }
    }
    found
}

/// Names referenced in `text`.
pub fn names_in(text: &str) -> Vec<String> {
    references(text).into_iter().map(|r| r.name).collect()
}

/// The values of one run: saved outputs, and built-ins once read.
#[derive(Debug, Default, Clone)]
pub struct Vars {
    values: HashMap<String, String>,
    /// Reads built-ins; replaced in tests.
    reader: Option<fn(&str) -> String>,
}

impl Vars {
    pub fn new() -> Self {
        Self::default()
    }

    /// Built-ins read through `reader` instead of the system.
    #[cfg(test)]
    pub fn with_reader(reader: fn(&str) -> String) -> Self {
        Self {
            values: HashMap::new(),
            reader: Some(reader),
        }
    }

    /// Saves a step's output, cut to [`MAX_VALUE_BYTES`] on a character
    /// boundary and without the trailing newline commands print.
    pub fn set(&mut self, name: &str, value: &str) {
        let value = value.strip_suffix('\n').unwrap_or(value);
        let value = value.strip_suffix('\r').unwrap_or(value);
        let mut end = value.len().min(MAX_VALUE_BYTES);
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        self.values
            .insert(normalize(name), value[..end].to_string());
    }

    /// The value of `name`, reading a built-in the first time.
    pub fn get(&mut self, name: &str) -> Result<String> {
        let name = normalize(name);
        if let Some(value) = self.values.get(&name) {
            return Ok(value.clone());
        }
        if is_builtin(&name) {
            let value = match self.reader {
                Some(read) => read(&name),
                None => read_builtin(&name),
            };
            self.values.insert(name, value.clone());
            return Ok(value);
        }
        Err(Error::Invalid(format!(
            "{{{{{name}}}}} has no value: the step that saves it did not run"
        )))
    }

    /// What `name` holds now, without reading a built-in.
    pub fn peek(&self, name: &str) -> Option<String> {
        self.values.get(&normalize(name)).cloned()
    }

    /// Forgets `name`, or puts back the value [`peek`](Self::peek) gave.
    pub fn restore(&mut self, name: &str, value: Option<String>) {
        match value {
            Some(value) => self.values.insert(normalize(name), value),
            None => self.values.remove(&normalize(name)),
        };
    }

    /// [`text`](Self::text), with a name nothing has set yet read as
    /// empty. Only for checking whether a value is there; anything that
    /// uses a value goes through `text`, `shell` or `lua`, which refuse an
    /// unset name rather than run with a hole in it.
    pub fn text_or_empty(&mut self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        for r in references(text) {
            out.push_str(&text[last..r.start]);
            out.push_str(&self.get(&r.name).unwrap_or_default());
            last = r.end;
        }
        out.push_str(&text[last..]);
        out
    }

    /// `text` with every reference replaced by its value, for text that is
    /// never parsed (a notification's title and body).
    pub fn text(&mut self, text: &str) -> Result<String> {
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        for r in references(text) {
            out.push_str(&text[last..r.start]);
            out.push_str(&self.get(&r.name)?);
            last = r.end;
        }
        out.push_str(&text[last..]);
        Ok(out)
    }

    /// A shell command with every reference replaced by a quoted expansion
    /// of an environment variable, and the variables to set for it.
    pub fn shell(&mut self, command: &str) -> Result<(String, Vec<(String, String)>)> {
        let refs = references(command);
        if refs.is_empty() {
            return Ok((command.to_string(), Vec::new()));
        }
        if let Some(name) = reference_in_arithmetic(command) {
            return Err(Error::Invalid(arithmetic_message(&name)));
        }
        let mut env: Vec<(String, String)> = Vec::new();
        let mut out = String::with_capacity(command.len() + refs.len() * 16);
        let mut context = ShellContext::new();
        let mut last = 0;
        for r in &refs {
            let before = &command[last..r.start];
            context.read(before);
            let quote = context.quote();
            out.push_str(before);
            let value = self.get(&r.name)?;
            let var = match env.iter().position(|(_, v)| *v == value) {
                Some(ix) => env[ix].0.clone(),
                None => {
                    let var = format!("OMARCHIST_VAR_{}", env.len());
                    env.push((var.clone(), value));
                    var
                }
            };
            out.push_str(&match quote {
                Quote::None => format!("\"${{{var}}}\""),
                Quote::Double => format!("${{{var}}}"),
                Quote::Single => format!("'\"${{{var}}}\"'"),
            });
            last = r.end;
        }
        out.push_str(&command[last..]);
        Ok((out, env))
    }

    /// A Hyprland action with every reference, each inside a string
    /// literal, replaced by its escaped value. Fails when a reference sits
    /// outside a string or the result is no longer a plain call.
    pub fn lua(&mut self, expr: &str) -> Result<String> {
        let refs = references(expr);
        if refs.is_empty() {
            return Ok(expr.to_string());
        }
        // `hl.dsp.exec_cmd` hands its text to a shell, where the Lua
        // escaping means nothing; a value there is a Command step's job.
        if lua_runs_a_shell(expr) {
            return Err(Error::Invalid(format!(
                "{{{{{}}}}} cannot be used in {}, which runs its text in a shell: \
                 use a Command step instead",
                refs[0].name,
                lua_call_path(expr).unwrap_or("this action")
            )));
        }
        let mut out = String::with_capacity(expr.len());
        let mut last = 0;
        let mut quote: Option<char> = None;
        for r in &refs {
            let before = &expr[last..r.start];
            quote = lua_quote_after(quote, before);
            let Some(q) = quote else {
                return Err(Error::Invalid(format!(
                    "{{{{{}}}}} must be inside quotes in a Hyprland action",
                    r.name
                )));
            };
            out.push_str(before);
            out.push_str(&lua_escape(&self.get(&r.name)?, q));
            last = r.end;
        }
        out.push_str(&expr[last..]);
        if !is_dsp_call(&out) {
            return Err(Error::Invalid(
                "The Hyprland action is not a plain call once its variables are filled in"
                    .to_string(),
            ));
        }
        Ok(out)
    }
}

/// Shell quoting state at a point in a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quote {
    None,
    Single,
    Double,
}

/// What a `$(`, a backtick or a `$((` opened: each starts its quoting
/// afresh, and ends when its closer is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Frame {
    Top,
    Substitution,
    Backtick,
    Arithmetic,
}

/// Where in a command the reader is: a stack of the substitutions opened
/// so far, each with its own quoting state, as `sh` reads it.
#[derive(Debug, Clone)]
struct ShellContext {
    frames: Vec<(Frame, Quote)>,
}

impl ShellContext {
    fn new() -> Self {
        Self {
            frames: vec![(Frame::Top, Quote::None)],
        }
    }

    fn quote(&self) -> Quote {
        self.frames.last().map_or(Quote::None, |frame| frame.1)
    }

    fn set_quote(&mut self, quote: Quote) {
        if let Some(frame) = self.frames.last_mut() {
            frame.1 = quote;
        }
    }

    /// Whether the reader is inside `$(( ))` or `(( ))`, however deep.
    fn in_arithmetic(&self) -> bool {
        self.frames.iter().any(|frame| frame.0 == Frame::Arithmetic)
    }

    /// Reads `text`. Backslashes escape outside single quotes, as in `sh`.
    fn read(&mut self, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let next = chars.get(i + 1).copied();
            let (frame, quote) = self
                .frames
                .last()
                .copied()
                .unwrap_or((Frame::Top, Quote::None));
            if quote == Quote::Single {
                if c == '\'' {
                    self.set_quote(Quote::None);
                }
                i += 1;
                continue;
            }
            match c {
                '\\' => {
                    i += 2;
                    continue;
                }
                '\'' if quote == Quote::None => self.set_quote(Quote::Single),
                '"' => self.set_quote(if quote == Quote::Double {
                    Quote::None
                } else {
                    Quote::Double
                }),
                '$' if next == Some('(') => {
                    if chars.get(i + 2) == Some(&'(') {
                        self.frames.push((Frame::Arithmetic, Quote::None));
                        i += 3;
                    } else {
                        self.frames.push((Frame::Substitution, Quote::None));
                        i += 2;
                    }
                    continue;
                }
                '(' if next == Some('(') && quote == Quote::None => {
                    self.frames.push((Frame::Arithmetic, Quote::None));
                    i += 2;
                    continue;
                }
                '`' => {
                    if frame == Frame::Backtick {
                        self.frames.pop();
                    } else {
                        self.frames.push((Frame::Backtick, Quote::None));
                    }
                }
                ')' if quote == Quote::None => match frame {
                    Frame::Arithmetic if next == Some(')') => {
                        self.frames.pop();
                        i += 2;
                        continue;
                    }
                    Frame::Substitution => {
                        self.frames.pop();
                    }
                    _ => {}
                },
                _ => {}
            }
            i += 1;
        }
        if self.frames.is_empty() {
            self.frames.push((Frame::Top, Quote::None));
        }
    }
}

/// The first reference in `command` that sits inside shell arithmetic,
/// where bash evaluates the contents of a value as code.
pub fn reference_in_arithmetic(command: &str) -> Option<String> {
    let mut context = ShellContext::new();
    let mut last = 0;
    for r in references(command) {
        context.read(&command[last..r.start]);
        if context.in_arithmetic() {
            return Some(r.name);
        }
        last = r.end;
    }
    None
}

/// Why a reference cannot sit in `$(( ))`.
pub fn arithmetic_message(name: &str) -> String {
    format!(
        "{{{{{name}}}}} cannot be used inside $(( )), where the shell runs code hidden in a \
         value; count with expr instead, as in expr {{{{{name}}}}} '*' 60"
    )
}

/// The `hl.dsp.<path>` a Hyprland action calls, before its `(`.
fn lua_call_path(expr: &str) -> Option<&str> {
    let path = expr.trim().split('(').next()?.trim();
    path.starts_with("hl.dsp.").then_some(path)
}

/// Whether a Hyprland action hands its text to a shell (`hl.dsp.exec_cmd`
/// and the like), so a value in it would be read as shell code.
pub fn lua_runs_a_shell(expr: &str) -> bool {
    lua_call_path(expr).is_some_and(|path| path.starts_with("hl.dsp.exec"))
}

/// The Lua string quote open after reading `text`, if any.
fn lua_quote_after(mut quote: Option<char>, text: &str) -> Option<char> {
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match quote {
            Some(_) if c == '\\' => {
                chars.next();
            }
            Some(q) if c == q => quote = None,
            None if c == '"' || c == '\'' => quote = Some(c),
            _ => {}
        }
    }
    quote
}

/// `value` escaped for a Lua string delimited by `quote`.
fn lua_escape(value: &str, quote: char) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

/// Reads a built-in from the system. A missing tool or an empty clipboard
/// reads as empty text rather than failing the step.
fn read_builtin(name: &str) -> String {
    let run = |program: &str, args: &[&str]| -> String {
        Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim_end().to_string())
            .unwrap_or_default()
    };
    let active_window = |field: &str| -> String {
        let json = run("hyprctl", &["-j", "activewindow"]);
        serde_json::from_str::<serde_json::Value>(&json)
            .ok()
            .and_then(|v| v.get(field).and_then(|f| f.as_str()).map(str::to_string))
            .unwrap_or_default()
    };
    match name {
        "clipboard" => run("wl-paste", &["--no-newline"]),
        "selection" => run("wl-paste", &["--primary", "--no-newline"]),
        "date" => chrono::Local::now().format("%Y-%m-%d").to_string(),
        "time" => chrono::Local::now().format("%H:%M").to_string(),
        "window" => active_window("title"),
        "app" => active_window("class"),
        "workspace" => {
            let json = run("hyprctl", &["-j", "activeworkspace"]);
            serde_json::from_str::<serde_json::Value>(&json)
                .ok()
                .and_then(|v| v.get("name").and_then(|f| f.as_str()).map(str::to_string))
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Vars, is_name, names_in, reference_in_arithmetic, references};

    fn reader(name: &str) -> String {
        match name {
            "clipboard" => "it's \"quoted\" $(rm -rf ~)".to_string(),
            "date" => "2026-10-04".to_string(),
            _ => String::new(),
        }
    }

    /// Runs `command` under `sh` with the variables set and returns stdout.
    fn sh(command: &str, env: &[(String, String)]) -> String {
        let out = std::process::Command::new("sh")
            .args(["-c", command])
            .envs(env.iter().map(|(k, v)| (k, v)))
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    #[test]
    fn names_follow_the_rules() {
        assert!(is_name("url"));
        assert!(is_name(" Search term "));
        assert!(is_name("step_2-out"));
        assert!(!is_name("2nd"));
        assert!(!is_name(""));
        assert!(!is_name("a.b"));
        assert!(!is_name(&"a".repeat(33)));
    }

    #[test]
    fn references_are_found_and_normalized() {
        assert_eq!(names_in("open {{URL}} then {{ date }}"), ["url", "date"]);
        // Not names: left as text.
        assert!(references("{{ }} and {{a.b}} and {{").is_empty());
    }

    #[test]
    fn a_value_never_runs_as_code_wherever_the_reference_sits() {
        let mut vars = Vars::with_reader(reader);
        let value = reader("clipboard");
        for command in [
            "printf %s {{clipboard}}",
            "printf %s \"{{clipboard}}\"",
            "printf %s '{{clipboard}}'",
            "printf %s \"before {{clipboard}} after\"",
            "printf %s 'before {{clipboard}} after'",
            // A substitution starts its quoting afresh.
            "printf %s \"$(printf %s {{clipboard}})\"",
            "printf %s \"$(printf %s '{{clipboard}}')\"",
            "printf %s \"$(printf %s \"{{clipboard}}\")\"",
            "printf %s \"`printf %s {{clipboard}}`\"",
        ] {
            let (rewritten, env) = vars.shell(command).unwrap();
            let printed = sh(&rewritten, &env);
            assert!(
                printed.contains(&value),
                "{command} -> {rewritten} printed {printed:?}"
            );
        }
    }

    #[test]
    fn a_reference_in_arithmetic_is_refused() {
        let mut vars = Vars::with_reader(reader);
        vars.set("n", "a[$(echo pwned >&2)]");
        for command in [
            "sleep $(( {{n}} * 60 ))",
            "sleep $(( $(printf %s {{n}}) ))",
            "(( {{n}} > 1 )) && echo big",
            "echo \"$(( {{n}} ))\"",
        ] {
            assert!(vars.shell(command).is_err(), "{command}");
            assert_eq!(reference_in_arithmetic(command).as_deref(), Some("n"));
        }
        // Closed arithmetic before the reference is fine.
        assert_eq!(reference_in_arithmetic("echo $((1+1)) {{n}}"), None);
        assert_eq!(reference_in_arithmetic("echo \"$(echo {{n}})\""), None);
        assert!(vars.shell("echo $((1+1)) {{n}}").is_ok());
    }

    #[test]
    fn a_reference_in_a_shell_dispatcher_is_refused() {
        let mut vars = Vars::with_reader(reader);
        assert!(
            vars.lua("hl.dsp.exec_cmd(\"xdg-open {{clipboard}}\")")
                .is_err()
        );
        assert!(vars.lua("hl.dsp.exec(\"{{clipboard}}\")").is_err());
        assert!(vars.lua("hl.dsp.exec_cmd(\"xdg-open https://x\")").is_ok());
        assert!(
            vars.lua("hl.dsp.focus({ workspace = \"{{clipboard}}\" })")
                .is_ok()
        );
    }

    #[test]
    fn one_value_used_twice_shares_a_variable() {
        let mut vars = Vars::with_reader(reader);
        vars.set("q", "a b");
        let (command, env) = vars.shell("echo {{q}} {{q}}").unwrap();
        assert_eq!(env.len(), 1);
        assert_eq!(sh(&command, &env), "a b a b\n");
    }

    #[test]
    fn outputs_lose_the_trailing_newline_and_are_capped() {
        let mut vars = Vars::new();
        vars.set("a", "line\n");
        assert_eq!(vars.get("A").unwrap(), "line");
        vars.set("big", &"é".repeat(super::MAX_VALUE_BYTES));
        assert!(vars.get("big").unwrap().len() <= super::MAX_VALUE_BYTES);
    }

    #[test]
    fn an_unset_name_is_an_error() {
        let mut vars = Vars::new();
        assert!(vars.text("{{missing}}").is_err());
    }

    #[test]
    fn notification_text_is_filled_in_plainly() {
        let mut vars = Vars::with_reader(reader);
        assert_eq!(
            vars.text("Today is {{date}}").unwrap(),
            "Today is 2026-10-04"
        );
    }

    #[test]
    fn hyprland_values_are_escaped_inside_their_string() {
        let mut vars = Vars::with_reader(reader);
        vars.set("ws", "2\"), os.execute(\"x");
        let expr = vars
            .lua("hl.dsp.focus({ workspace = \"{{ws}}\" })")
            .unwrap();
        assert_eq!(
            expr,
            "hl.dsp.focus({ workspace = \"2\\\"), os.execute(\\\"x\" })"
        );
        assert!(crate::system::keybinds::overrides::is_dsp_call(&expr));
        // Outside a string, a reference is refused rather than spliced in.
        assert!(vars.lua("hl.dsp.focus({ workspace = {{ws}} })").is_err());
    }
}
