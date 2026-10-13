//! The log file every Omarchist process writes to.
//!
//! The window, the automations service and every `omarchist flow run` write
//! to one file a day in `$XDG_STATE_HOME/omarchist/logs`
//! (`omarchist-2026-10-12.log`), so a bug that crosses processes (an
//! automation starts a flow, the flow fails) reads in order in one place.
//! Each process opens the file for appending and writes a record with one
//! `write`, so records from different processes never mix within a line. A
//! file that passes [`MAX_FILE_BYTES`] continues in `omarchist-<date>.1.log`.
//!
//! Records are written as they are made, never buffered: release builds
//! abort on panic, and a record still in a queue would be lost with the
//! process. The window and the service purge old files ([`purge`]); a flow
//! run never does.
//!
//! What is logged must never hold what a person typed or copied: no
//! variable values, flow input, clipboard, selection or answers. Commands
//! are logged as the flow writes them, before substitution.

use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::{DateTime, Local, NaiveDate};
use log::{Level, LevelFilter, Log, Metadata, Record};

use crate::error::Result;

/// A file passing this size continues in the next part.
pub const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;
/// The folder is cut back to this size, oldest files first.
pub const MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024;
/// How long logs are kept unless Settings says otherwise.
pub const DEFAULT_KEEP_DAYS: u32 = 7;
/// The choices Settings offers for how long logs are kept.
pub const KEEP_DAYS: &[u32] = &[3, 7, 30];
/// The environment variable that overrides the level: `debug`, or filters
/// such as `omarchist::flows=trace,gpui=info`.
pub const ENV: &str = "OMARCHIST_LOG";

/// Which kind of process wrote a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The window.
    App,
    /// `omarchist automations run`.
    Service,
    /// `omarchist flow run`.
    Flow,
    /// Any other command.
    Cli,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::App => "app",
            Role::Service => "service",
            Role::Flow => "flow",
            Role::Cli => "cli",
        }
    }

    /// The window and the service purge; a flow run is short and often
    /// one of several at once, so it never does.
    fn purges(self) -> bool {
        matches!(self, Role::App | Role::Service)
    }
}

/// `$XDG_STATE_HOME/omarchist/logs`.
pub fn log_dir() -> Result<PathBuf> {
    Ok(crate::system::flows::history::state_dir()?.join("logs"))
}

// ── Filters ────────────────────────────────────────────────────────────

/// Which records are written: a level for Omarchist's own modules, one for
/// everything else (gpui and the other libraries), and any per-module
/// levels, the longest matching prefix winning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    own: LevelFilter,
    others: LevelFilter,
    modules: Vec<(String, LevelFilter)>,
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter {
    /// Debug for Omarchist, warnings from libraries.
    pub fn new() -> Self {
        Self {
            own: LevelFilter::Debug,
            others: LevelFilter::Warn,
            modules: Vec::new(),
        }
    }

    /// Applies an [`ENV`] value on top: a bare level sets Omarchist's, a
    /// `module=level` sets that module's (`omarchist::flows` or a crate's
    /// name such as `gpui`), `all=level` sets the libraries'. Anything it
    /// cannot read is left out and named in the result's second half.
    pub fn parse(mut self, spec: &str) -> (Self, Vec<String>) {
        let mut ignored = Vec::new();
        for part in spec.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            match part.split_once('=') {
                None => match part.parse() {
                    Ok(level) => self.own = level,
                    Err(_) => ignored.push(part.to_string()),
                },
                Some((module, level)) => match (module.trim(), level.trim().parse()) {
                    ("all", Ok(level)) => self.others = level,
                    ("omarchist", Ok(level)) => self.own = level,
                    (module, Ok(level)) if !module.is_empty() => {
                        self.modules.push((module.to_string(), level))
                    }
                    _ => ignored.push(part.to_string()),
                },
            }
        }
        // Longest first, so the most specific prefix is found first.
        self.modules
            .sort_by_key(|(module, _)| std::cmp::Reverse(module.len()));
        (self, ignored)
    }

    pub fn level_for(&self, target: &str) -> LevelFilter {
        let in_module = |module: &str| {
            target == module
                || target
                    .strip_prefix(module)
                    .is_some_and(|rest| rest.starts_with("::"))
        };
        if let Some((_, level)) = self.modules.iter().find(|(m, _)| in_module(m)) {
            return *level;
        }
        if in_module("omarchist") {
            self.own
        } else {
            self.others
        }
    }

    /// The most detailed level any target can have, for `log::set_max_level`.
    pub fn max(&self) -> LevelFilter {
        self.modules
            .iter()
            .map(|(_, level)| *level)
            .chain([self.own, self.others])
            .max()
            .unwrap_or(LevelFilter::Off)
    }
}

// ── Format ─────────────────────────────────────────────────────────────

/// One record as written: a line, and every further line of the message
/// indented under it, so `grep` on a timestamp or level finds the record
/// and a backtrace stays readable.
pub fn format_record(
    at: DateTime<Local>,
    level: Level,
    role: Role,
    pid: u32,
    target: &str,
    message: &str,
) -> String {
    let target = target
        .strip_prefix("omarchist::system::")
        .or_else(|| target.strip_prefix("omarchist::"))
        .unwrap_or(target);
    let mut line = String::with_capacity(64 + message.len());
    let _ = write!(
        line,
        "{} {:<5} [{}:{pid}] {target}: ",
        at.format("%Y-%m-%dT%H:%M:%S%.3f%:z"),
        level,
        role.name(),
    );
    let mut lines = message.trim_end().lines();
    line.push_str(lines.next().unwrap_or(""));
    line.push('\n');
    for rest in lines {
        line.push_str("    ");
        line.push_str(rest);
        line.push('\n');
    }
    line
}

/// The level of a written line, or `None` for a continuation line.
pub fn line_level(line: &str) -> Option<Level> {
    if line.starts_with(' ') {
        return None;
    }
    line.split_whitespace().nth(1)?.parse().ok()
}

/// The time of a written line, or `None` for a continuation line.
pub fn line_time(line: &str) -> Option<DateTime<chrono::FixedOffset>> {
    DateTime::parse_from_str(line.split_whitespace().next()?, "%Y-%m-%dT%H:%M:%S%.3f%:z").ok()
}

// ── Files ──────────────────────────────────────────────────────────────

/// A log file's day and part, read back from its name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogFile {
    pub date: NaiveDate,
    pub part: u32,
    pub path: PathBuf,
}

pub fn file_name(date: NaiveDate, part: u32) -> String {
    if part == 0 {
        format!("omarchist-{date}.log")
    } else {
        format!("omarchist-{date}.{part}.log")
    }
}

fn parse_name(name: &str) -> Option<(NaiveDate, u32)> {
    let stem = name.strip_prefix("omarchist-")?.strip_suffix(".log")?;
    let (date, part) = match stem.split_once('.') {
        Some((date, part)) => (date, part.parse().ok()?),
        None => (stem, 0),
    };
    Some((NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?, part))
}

/// Every log file in `dir`, oldest first. Other files are left alone.
pub fn list_files(dir: &Path) -> Vec<LogFile> {
    let mut files: Vec<LogFile> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let (date, part) = parse_name(entry.file_name().to_str()?)?;
            Some(LogFile {
                date,
                part,
                path: entry.path(),
            })
        })
        .collect();
    files.sort();
    files
}

/// The part of `date`'s log to append to: the last one, or the next one
/// when the last is full.
fn current_part(dir: &Path, date: NaiveDate, max_bytes: u64) -> u32 {
    let Some(last) = list_files(dir).into_iter().rfind(|f| f.date == date) else {
        return 0;
    };
    let size = fs::metadata(&last.path).map(|m| m.len()).unwrap_or(0);
    if size >= max_bytes {
        last.part + 1
    } else {
        last.part
    }
}

/// Deletes logs older than `keep_days` days, then the oldest until the
/// folder holds at most `max_total` bytes. Today's files are never
/// deleted. Returns what was removed.
pub fn purge(dir: &Path, today: NaiveDate, keep_days: u32, max_total: u64) -> Vec<PathBuf> {
    let oldest_kept = today - chrono::Days::new(u64::from(keep_days.max(1)) - 1);
    let mut removed = Vec::new();
    let mut kept = Vec::new();
    for file in list_files(dir) {
        if file.date < oldest_kept && fs::remove_file(&file.path).is_ok() {
            removed.push(file.path);
        } else {
            let size = fs::metadata(&file.path).map(|m| m.len()).unwrap_or(0);
            kept.push((file, size));
        }
    }
    let mut total: u64 = kept.iter().map(|(_, size)| size).sum();
    for (file, size) in kept {
        if total <= max_total || file.date >= today {
            break;
        }
        if fs::remove_file(&file.path).is_ok() {
            total -= size;
            removed.push(file.path);
        }
    }
    removed
}

// ── Reading ────────────────────────────────────────────────────────────

/// Reads `30s`, `15m`, `2h` or `3d` as a span of time.
pub fn parse_since(text: &str) -> Option<chrono::Duration> {
    let text = text.trim();
    let (number, unit) = text.split_at(text.find(|c: char| !c.is_ascii_digit())?);
    let n: i64 = number.parse().ok()?;
    match unit {
        "s" => Some(chrono::Duration::seconds(n)),
        "m" => Some(chrono::Duration::minutes(n)),
        "h" => Some(chrono::Duration::hours(n)),
        "d" => Some(chrono::Duration::days(n)),
        _ => None,
    }
}

/// Which records to show: at `level` or worse, from `since` on.
pub struct Wanted {
    pub level: LevelFilter,
    pub since: Option<DateTime<Local>>,
    /// Whether the record the next continuation line belongs to is shown.
    showing: bool,
}

impl Wanted {
    pub fn new(level: LevelFilter, since: Option<DateTime<Local>>) -> Self {
        Self {
            level,
            since,
            showing: false,
        }
    }

    /// Whether `line` is shown: a record by its level and time, a
    /// continuation line with the record it belongs to.
    pub fn wants(&mut self, line: &str) -> bool {
        if let Some(level) = line_level(line) {
            let recent = match (self.since, line_time(line)) {
                (Some(since), Some(at)) => at >= since,
                _ => true,
            };
            self.showing = level <= self.level && recent;
        }
        self.showing
    }
}

/// The files holding records from `since` on, or today's, oldest first.
pub fn files_since(dir: &Path, since: Option<DateTime<Local>>) -> Vec<LogFile> {
    let from = since.map_or_else(|| Local::now().date_naive(), |since| since.date_naive());
    list_files(dir)
        .into_iter()
        .filter(|file| file.date >= from)
        .collect()
}

// ── The logger ─────────────────────────────────────────────────────────

struct Open {
    date: NaiveDate,
    part: u32,
    file: File,
}

struct FileLogger {
    role: Role,
    pid: u32,
    dir: PathBuf,
    max_file_bytes: u64,
    /// Records also go to stderr from this level down.
    stderr: LevelFilter,
    filter: Filter,
    open: Mutex<Option<Open>>,
}

impl FileLogger {
    fn file_for(&self, open: &mut Option<Open>, date: NaiveDate) -> std::io::Result<()> {
        let full = |file: &File| {
            file.metadata()
                .map(|m| m.len() >= self.max_file_bytes)
                .unwrap_or(false)
        };
        if let Some(current) = open
            && current.date == date
            && !full(&current.file)
        {
            return Ok(());
        }
        // Another process may have moved on to a later part already.
        let mut part = current_part(&self.dir, date, self.max_file_bytes);
        if let Some(current) = open
            && current.date == date
        {
            part = part.max(current.part + 1);
        }
        fs::create_dir_all(&self.dir)?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join(file_name(date, part)))?;
        *open = Some(Open { date, part, file });
        Ok(())
    }

    fn write(&self, text: &str, date: NaiveDate) {
        let Ok(mut open) = self.open.lock() else {
            return;
        };
        if self.file_for(&mut open, date).is_ok()
            && let Some(current) = open.as_mut()
        {
            // One write per record: appends from several processes never
            // interleave within it.
            let _ = current.file.write_all(text.as_bytes());
        }
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.filter.level_for(metadata.target())
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let now = Local::now();
        let text = format_record(
            now,
            record.level(),
            self.role,
            self.pid,
            record.target(),
            &record.args().to_string(),
        );
        self.write(&text, now.date_naive());
        if record.level() <= self.stderr {
            let _ = std::io::stderr().write_all(text.as_bytes());
        }
    }

    fn flush(&self) {}
}

static LOGGER: OnceLock<FileLogger> = OnceLock::new();

/// Starts writing the log for this process: reads how long to keep logs
/// from Settings, applies [`ENV`], purges old files when the
/// role does, and logs every panic before the process dies. Call it once,
/// first thing in `main`; tests never do, so they never write a log.
pub fn init(role: Role) {
    let settings = crate::system::config::config_setup::settings();
    let (filter, ignored) = Filter::new().parse(&std::env::var(ENV).unwrap_or_default());
    let Ok(dir) = log_dir() else {
        return;
    };
    // The window mirrors everything to a terminal (`cargo run`); the
    // service sends what it does to journald; a flow run in a terminal keeps it
    // clean for the flow's own output.
    let stderr = match role {
        Role::App if std::io::stderr().is_terminal() => LevelFilter::Trace,
        Role::Service => LevelFilter::Info,
        _ => LevelFilter::Off,
    };
    let max = filter.max();
    let logger = FileLogger {
        role,
        pid: std::process::id(),
        dir: dir.clone(),
        max_file_bytes: MAX_FILE_BYTES,
        stderr,
        filter,
        open: Mutex::new(None),
    };
    if LOGGER.set(logger).is_err() {
        return;
    }
    if log::set_logger(LOGGER.get().expect("just set")).is_err() {
        return;
    }
    log::set_max_level(max);
    install_panic_hook();
    for part in ignored {
        log::warn!("{ENV}: ignored '{part}'");
    }
    if role.purges() {
        purge_now(settings.log_keep_days);
    }
}

/// Deletes logs past the kept days or the size cap; the window and the
/// service run it at startup and the service again every day.
pub fn purge_now(keep_days: u32) {
    let Ok(dir) = log_dir() else {
        return;
    };
    let removed = purge(&dir, Local::now().date_naive(), keep_days, MAX_TOTAL_BYTES);
    if !removed.is_empty() {
        log::info!("removed {} old log file(s)", removed.len());
    }
}

/// Logs a command that ran: at `success` when it exited cleanly, as a
/// warning with how it exited and the end of its stderr otherwise.
/// `argv` is what the caller wrote, never text a person typed or copied.
pub fn ran(
    success: Level,
    argv: &[&str],
    result: &std::io::Result<std::process::Output>,
    started: std::time::Instant,
) {
    let command = argv.join(" ");
    let ms = started.elapsed().as_millis();
    match result {
        Ok(output) if output.status.success() => {
            log::log!(target: "omarchist::command", success, "{command} ({ms} ms)")
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let lines: Vec<&str> = stderr.trim_end().lines().collect();
            let tail = lines[lines.len().saturating_sub(STDERR_LINES)..].join("\n");
            log::warn!(
                target: "omarchist::command",
                "{command} failed ({}, {ms} ms){}{tail}",
                output.status,
                if tail.is_empty() { "" } else { ":\n" },
            )
        }
        Err(e) => log::warn!(target: "omarchist::command", "{command} could not start: {e}"),
    }
}

/// How much of a failed command's stderr [`ran`] keeps.
const STDERR_LINES: usize = 5;

/// Logs a panic, with where it happened and a backtrace, then runs the
/// hook installed before (the submap reset, then Rust's own message).
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "panic".into());
        let at = info
            .location()
            .map(|l| format!(" at {}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let thread = std::thread::current();
        log::error!(
            target: "omarchist::panic",
            "thread '{}' panicked{at}: {message}\n{backtrace}",
            thread.name().unwrap_or("unnamed")
        );
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::{
        Filter, LogFile, Role, file_name, format_record, line_level, line_time, list_files,
        parse_name, purge,
    };
    use chrono::{Local, NaiveDate, TimeZone};
    use log::{Level, LevelFilter};
    use std::fs;
    use std::path::PathBuf;

    fn tempdir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "omarchist-logging-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn a_record_is_one_line_and_its_further_lines_are_indented() {
        let at = Local.with_ymd_and_hms(2026, 10, 12, 16, 20, 1).unwrap();
        let text = format_record(
            at,
            Level::Warn,
            Role::Flow,
            4133,
            "omarchist::system::flows::runner",
            "step 3 failed\nbrightnessctl: not found\n",
        );
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("2026-10-12T16:20:01.000"));
        assert!(lines[0].ends_with("WARN  [flow:4133] flows::runner: step 3 failed"));
        assert_eq!(lines[1], "    brightnessctl: not found");
        assert!(text.ends_with('\n'));
        assert_eq!(line_level(lines[0]), Some(Level::Warn));
        assert_eq!(line_level(lines[1]), None);
        assert!(line_time(lines[0]).is_some());
        assert!(line_time(lines[1]).is_none());
    }

    #[test]
    fn the_filter_reads_levels_and_modules() {
        let filter = Filter::new();
        assert_eq!(filter.level_for("omarchist::ui"), LevelFilter::Debug);
        assert_eq!(filter.level_for("gpui::window"), LevelFilter::Warn);

        let (filter, ignored) =
            Filter::new().parse("info, omarchist::system::flows=trace,gpui=info,nope,x=loud");
        assert_eq!(ignored, vec!["nope".to_string(), "x=loud".to_string()]);
        assert_eq!(
            filter.level_for("omarchist::ui::app_view"),
            LevelFilter::Info
        );
        assert_eq!(
            filter.level_for("omarchist::system::flows::runner"),
            LevelFilter::Trace
        );
        assert_eq!(filter.level_for("gpui"), LevelFilter::Info);
        // A prefix only matches whole module names.
        assert_eq!(filter.level_for("gpuix"), LevelFilter::Warn);
        assert_eq!(filter.max(), LevelFilter::Trace);

        let (filter, _) = Filter::new().parse("all=off,omarchist=error");
        assert_eq!(filter.level_for("zbus"), LevelFilter::Off);
        assert_eq!(filter.level_for("omarchist::cli"), LevelFilter::Error);
    }

    #[test]
    fn reading_keeps_a_record_with_its_further_lines() {
        let at = |h| Local.with_ymd_and_hms(2026, 10, 12, h, 0, 0).unwrap();
        let text = [
            format_record(at(9), Level::Info, Role::App, 1, "omarchist::ui", "opened"),
            format_record(
                at(10),
                Level::Warn,
                Role::Flow,
                2,
                "omarchist::flows",
                "failed\nwhy",
            ),
            format_record(
                at(11),
                Level::Debug,
                Role::App,
                1,
                "omarchist::ui",
                "page\nmore",
            ),
            format_record(
                at(12),
                Level::Error,
                Role::App,
                1,
                "omarchist::panic",
                "boom\ntrace",
            ),
        ]
        .concat();
        let shown = |level, since| {
            let mut wanted = super::Wanted::new(level, since);
            text.lines()
                .filter(|line| wanted.wants(line))
                .map(|line| line.split(": ").last().unwrap().trim().to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            shown(LevelFilter::Warn, None),
            ["failed", "why", "boom", "trace"]
        );
        assert_eq!(
            shown(LevelFilter::Trace, Some(at(11))),
            ["page", "more", "boom", "trace"]
        );
        assert_eq!(
            super::parse_since("90m"),
            Some(chrono::Duration::minutes(90))
        );
        assert_eq!(super::parse_since("2d"), Some(chrono::Duration::days(2)));
        assert_eq!(super::parse_since("2 weeks"), None);
        assert_eq!(super::parse_since("h"), None);
    }

    #[test]
    fn file_names_carry_the_day_and_the_part() {
        assert_eq!(file_name(day("2026-10-12"), 0), "omarchist-2026-10-12.log");
        assert_eq!(
            file_name(day("2026-10-12"), 2),
            "omarchist-2026-10-12.2.log"
        );
        assert_eq!(
            parse_name("omarchist-2026-10-12.2.log"),
            Some((day("2026-10-12"), 2))
        );
        assert_eq!(
            parse_name("omarchist-2026-10-12.log"),
            Some((day("2026-10-12"), 0))
        );
        assert_eq!(parse_name("notes.txt"), None);
        assert_eq!(parse_name("omarchist-yesterday.log"), None);
    }

    #[test]
    fn purging_drops_old_days_then_the_oldest_until_the_folder_fits() {
        let dir = tempdir("purge");
        let write =
            |name: &str, bytes: usize| fs::write(dir.join(name), vec![b'x'; bytes]).unwrap();
        write("omarchist-2026-10-01.log", 10);
        write("omarchist-2026-10-09.log", 40);
        write("omarchist-2026-10-10.log", 40);
        write("omarchist-2026-10-10.1.log", 40);
        write("omarchist-2026-10-12.log", 100);
        write("keep-me.txt", 1000);

        // Seven days back from the 12th keeps the 6th onwards; then 220
        // bytes must come down to 150, so the 9th and the 10th's first part go.
        let removed = purge(&dir, day("2026-10-12"), 7, 150);
        let names = |files: Vec<LogFile>| {
            files
                .into_iter()
                .map(|f| f.path.file_name().unwrap().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(removed.len(), 3);
        assert_eq!(
            names(list_files(&dir)),
            vec!["omarchist-2026-10-10.1.log", "omarchist-2026-10-12.log"]
        );
        assert!(dir.join("keep-me.txt").exists(), "other files stay");

        // Today's file stays even when it alone is over the cap.
        purge(&dir, day("2026-10-12"), 7, 10);
        assert_eq!(names(list_files(&dir)), vec!["omarchist-2026-10-12.log"]);
        fs::remove_dir_all(&dir).unwrap();
    }

    fn writer(dir: PathBuf, max_file_bytes: u64) -> super::FileLogger {
        super::FileLogger {
            role: Role::App,
            pid: std::process::id(),
            dir,
            max_file_bytes,
            stderr: LevelFilter::Off,
            filter: Filter::new(),
            open: std::sync::Mutex::new(None),
        }
    }

    /// Run as a child by the test below: appends records to the folder it
    /// is given.
    #[test]
    fn child_writer() {
        let Ok(dir) = std::env::var("OMARCHIST_LOG_TEST_DIR") else {
            return;
        };
        let logger = writer(PathBuf::from(dir), u64::MAX);
        let tag = std::process::id();
        for ix in 0..2000 {
            logger.write(
                &format!("{tag} {ix} {}\n", "z".repeat(300)),
                day("2026-10-12"),
            );
        }
    }

    #[test]
    fn two_processes_appending_never_mix_within_a_line() {
        let dir = tempdir("procs");
        let spawn = || {
            std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "system::logging::tests::child_writer", "--quiet"])
                .env("OMARCHIST_LOG_TEST_DIR", &dir)
                .stdout(std::process::Stdio::null())
                .spawn()
                .unwrap()
        };
        let children = [spawn(), spawn()];
        for mut child in children {
            assert!(child.wait().unwrap().success());
        }
        let text = fs::read_to_string(dir.join("omarchist-2026-10-12.log")).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 4000);
        for line in lines {
            let parts: Vec<&str> = line.split(' ').collect();
            assert_eq!(parts.len(), 3, "a line holds one record: {line}");
            assert_eq!(parts[2].len(), 300);
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_full_file_continues_in_the_next_part() {
        let dir = tempdir("parts");
        let logger = writer(dir.clone(), 100);
        let today = day("2026-10-12");
        for _ in 0..3 {
            logger.write(&format!("{}\n", "x".repeat(59)), today);
        }
        let files = list_files(&dir);
        assert_eq!(files.iter().map(|f| f.part).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(fs::metadata(&files[0].path).unwrap().len(), 120);
        assert_eq!(fs::metadata(&files[1].path).unwrap().len(), 60);
        // A new day starts a new file.
        logger.write("y\n", day("2026-10-13"));
        assert_eq!(list_files(&dir).last().unwrap().date, day("2026-10-13"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
