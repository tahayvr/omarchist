// Runs the embedded `scan.lua` with the system `lua` interpreter and parses
// its tab-separated records. See the header of `scan.lua` for the protocol.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

use super::{BindOptions, Dispatcher};

const SCAN_SCRIPT: &str = include_str!("scan.lua");

/// Upper bound for one scan. Evaluating the config is a few milliseconds;
/// the limit only guards against a user config that blocks (a `while true`
/// or an interactive command run through `io.popen`).
const SCAN_TIMEOUT: Duration = Duration::from_secs(10);

pub const LUA_MISSING_MESSAGE: &str =
    "The 'lua' interpreter is required to read keybinds (install it with: sudo pacman -S lua)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedBind {
    pub seq: u32,
    pub source: PathBuf,
    pub keys: String,
    pub description: String,
    pub dispatcher: Dispatcher,
    pub options: BindOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanEvent {
    Bind(ScannedBind),
    Unbind {
        seq: u32,
        source: PathBuf,
        keys: String,
    },
    Error {
        seq: u32,
        message: String,
    },
    Done {
        seq: u32,
    },
}

/// The interpreter to run. `OMARCHIST_LUA` overrides it for tests and
/// debugging.
pub fn lua_binary() -> String {
    std::env::var("OMARCHIST_LUA").unwrap_or_else(|_| "lua".to_string())
}

pub fn run_scan(hyprland_lua: &Path) -> Result<Vec<ScanEvent>> {
    let output = run_lua(hyprland_lua)?;
    let events = parse_scan_output(&output.stdout);

    if !events.iter().any(|e| matches!(e, ScanEvent::Done { .. })) {
        let stderr = output.stderr.trim();
        let detail = if stderr.is_empty() {
            "the scanner produced no result".to_string()
        } else {
            stderr.to_string()
        };
        return Err(Error::Invalid(format!("Keybind scan failed: {detail}")));
    }

    Ok(events)
}

struct ScanOutput {
    stdout: String,
    stderr: String,
}

fn run_lua(hyprland_lua: &Path) -> Result<ScanOutput> {
    let mut child = Command::new(lua_binary())
        .arg("-")
        .arg(hyprland_lua)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::Invalid(LUA_MISSING_MESSAGE.to_string())
            } else {
                Error::io("Failed to start the lua interpreter", e)
            }
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        // A broken pipe here only means lua exited early; the missing
        // `done` record reports that below.
        let _ = stdin.write_all(SCAN_SCRIPT.as_bytes());
    }

    // Drain both pipes on helper threads so a chatty config cannot fill a
    // pipe buffer and deadlock against the timeout loop.
    let stdout = child.stdout.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = std::io::Read::read_to_end(&mut pipe, &mut buf);
            buf
        })
    });
    let stderr = child.stderr.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = std::io::Read::read_to_end(&mut pipe, &mut buf);
            buf
        })
    });

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < SCAN_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Error::Invalid(
                    "Keybind scan timed out: your hyprland.lua did not finish evaluating"
                        .to_string(),
                ));
            }
            Err(e) => return Err(Error::io("Failed to wait for the lua interpreter", e)),
        }
    }

    let collect = |handle: Option<std::thread::JoinHandle<Vec<u8>>>| {
        handle
            .and_then(|h| h.join().ok())
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_default()
    };

    Ok(ScanOutput {
        stdout: collect(stdout),
        stderr: collect(stderr),
    })
}

pub fn parse_scan_output(text: &str) -> Vec<ScanEvent> {
    text.lines().filter_map(parse_scan_line).collect()
}

/// Parses one record. Lines that are not records (a user config that
/// `print`s) yield `None` and are ignored.
pub fn parse_scan_line(line: &str) -> Option<ScanEvent> {
    let fields: Vec<String> = line.split('\t').map(unescape_field).collect();
    let seq = |ix: usize| fields.get(ix).and_then(|f| f.parse::<u32>().ok());

    match fields.first().map(String::as_str) {
        Some("bind") if fields.len() == 8 => {
            let dispatcher = match fields[5].as_str() {
                "exec" => Dispatcher::Exec(fields[6].clone()),
                "fn" => Dispatcher::Function,
                _ => Dispatcher::Lua(fields[6].clone()),
            };
            Some(ScanEvent::Bind(ScannedBind {
                seq: seq(1)?,
                source: PathBuf::from(&fields[2]),
                keys: fields[3].clone(),
                description: fields[4].clone(),
                dispatcher,
                options: BindOptions::from_flags(&fields[7]),
            }))
        }
        Some("unbind") if fields.len() == 4 => Some(ScanEvent::Unbind {
            seq: seq(1)?,
            source: PathBuf::from(&fields[2]),
            keys: fields[3].clone(),
        }),
        Some("error") if fields.len() == 3 => Some(ScanEvent::Error {
            seq: seq(1)?,
            message: fields[2].clone(),
        }),
        Some("done") if fields.len() == 2 => Some(ScanEvent::Done { seq: seq(1)? }),
        _ => None,
    }
}

pub fn unescape_field(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    let mut chars = field.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescapes_scanner_fields() {
        assert_eq!(unescape_field("a\\tb\\nc\\\\d"), "a\tb\nc\\d");
        assert_eq!(unescape_field("plain"), "plain");
        assert_eq!(unescape_field("trailing\\"), "trailing\\");
        assert_eq!(unescape_field("\\q"), "\\q");
    }

    #[test]
    fn parses_each_record_type() {
        let bind = parse_scan_line(
            "bind\t3\t/x/bindings/media.lua\tXF86AudioRaiseVolume\tVolume up\texec\tomarchy-audio-output-volume raise\tlocked,repeating",
        )
        .unwrap();
        assert_eq!(
            bind,
            ScanEvent::Bind(ScannedBind {
                seq: 3,
                source: PathBuf::from("/x/bindings/media.lua"),
                keys: "XF86AudioRaiseVolume".into(),
                description: "Volume up".into(),
                dispatcher: Dispatcher::Exec("omarchy-audio-output-volume raise".into()),
                options: BindOptions {
                    locked: true,
                    repeating: true,
                    ..BindOptions::default()
                },
            })
        );

        let lua = parse_scan_line(
            "bind\t4\t/x/tiling.lua\tSUPER + W\tClose window\tlua\thl.dsp.window.close()\t",
        )
        .unwrap();
        let ScanEvent::Bind(lua) = lua else { panic!() };
        assert_eq!(
            lua.dispatcher,
            Dispatcher::Lua("hl.dsp.window.close()".into())
        );
        assert_eq!(lua.options, BindOptions::default());

        let func = parse_scan_line("bind\t5\t/x/u.lua\tSUPER + CTRL + Z\tZoom in\tfn\t\t").unwrap();
        let ScanEvent::Bind(func) = func else {
            panic!()
        };
        assert_eq!(func.dispatcher, Dispatcher::Function);

        let undescribed = parse_scan_line(
            "bind\t6\t/x/u.lua\tswitch:on:Lid Switch\t\texec\tomarchy-system-lid-close\tlocked",
        )
        .unwrap();
        let ScanEvent::Bind(undescribed) = undescribed else {
            panic!()
        };
        assert_eq!(undescribed.description, "");

        assert_eq!(
            parse_scan_line("unbind\t7\t/home/u/.config/hypr/bindings.lua\tSUPER + SHIFT + C")
                .unwrap(),
            ScanEvent::Unbind {
                seq: 7,
                source: PathBuf::from("/home/u/.config/hypr/bindings.lua"),
                keys: "SUPER + SHIFT + C".into(),
            }
        );
        assert_eq!(
            parse_scan_line("error\t7\tboom").unwrap(),
            ScanEvent::Error {
                seq: 7,
                message: "boom".into()
            }
        );
        assert_eq!(
            parse_scan_line("done\t7").unwrap(),
            ScanEvent::Done { seq: 7 }
        );
    }

    #[test]
    fn ignores_noise_and_malformed_lines() {
        assert!(parse_scan_line("hello from the user's config").is_none());
        assert!(parse_scan_line("bind\tnot-a-number\ta\tb\tc\td\te\tf").is_none());
        assert!(parse_scan_line("bind\t1\ttoo\tfew").is_none());
        assert!(parse_scan_line("").is_none());
    }

    #[test]
    fn escaped_tabs_survive_in_fields() {
        let ScanEvent::Bind(bind) =
            parse_scan_line("bind\t1\t/x.lua\tSUPER + K\tA\\tB\texec\techo \"a\\\\b\"\t").unwrap()
        else {
            panic!()
        };
        assert_eq!(bind.description, "A\tB");
        assert_eq!(bind.dispatcher, Dispatcher::Exec("echo \"a\\b\"".into()));
    }

    // Exercises the real interpreter and the embedded script. Skipped when
    // `lua` is not installed so CI without Lua still passes.
    #[test]
    fn scans_a_fixture_config_with_the_real_interpreter() {
        if Command::new(lua_binary()).arg("-v").output().is_err() {
            eprintln!("skipping: lua not installed");
            return;
        }

        let dir = std::env::temp_dir().join(format!("omarchist-scan-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let helpers = dir.join("default/hypr/helpers.lua");
        std::fs::create_dir_all(helpers.parent().unwrap()).unwrap();
        std::fs::write(
            &helpers,
            r#"
o = {}
function o.bind(keys, description, dispatcher, options)
  local opts = options or {}
  opts.description = description
  if type(dispatcher) == "string" then dispatcher = hl.dsp.exec_cmd(dispatcher) end
  hl.bind(keys, dispatcher, opts)
end
"#,
        )
        .unwrap();
        let config = dir.join("hyprland.lua");
        std::fs::write(
            &config,
            format!(
                r#"
dofile("{helpers}")
print("user noise")
o.bind("SUPER + K", "Keybindings", "omarchy-menu-keybindings")
o.bind("SUPER + W", "Close window", hl.dsp.window.close())
o.bind("SUPER + 1", "Workspace 1", hl.dsp.focus({{ workspace = "1" }}))
hl.bind("F9", function() end, {{ description = "Zoom", release = true }})
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("mute"), {{ description = "Mute", locked = true }})
hl.unbind("SUPER + K")
hl.on("layer.opened", function() end)
hl.config({{ general = {{ gaps_in = 5 }} }})
"#,
                helpers = helpers.display()
            ),
        )
        .unwrap();

        let events = run_scan(&config).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        let binds: Vec<&ScannedBind> = events
            .iter()
            .filter_map(|e| match e {
                ScanEvent::Bind(b) => Some(b),
                _ => None,
            })
            .collect();
        assert_eq!(binds.len(), 5, "{events:?}");
        assert_eq!(binds[0].keys, "SUPER + K");
        assert_eq!(binds[0].description, "Keybindings");
        assert_eq!(
            binds[0].dispatcher,
            Dispatcher::Exec("omarchy-menu-keybindings".into())
        );
        assert_eq!(
            binds[0].source, config,
            "o.bind must be attributed past helpers.lua"
        );
        assert_eq!(
            binds[1].dispatcher,
            Dispatcher::Lua("hl.dsp.window.close()".into())
        );
        assert_eq!(
            binds[2].dispatcher,
            Dispatcher::Lua("hl.dsp.focus({ workspace = \"1\" })".into())
        );
        assert_eq!(binds[3].dispatcher, Dispatcher::Function);
        assert!(binds[3].options.release);
        assert!(binds[4].options.locked);
        assert!(matches!(events[5], ScanEvent::Unbind { ref keys, .. } if keys == "SUPER + K"));
        assert!(matches!(events.last(), Some(ScanEvent::Done { seq: 6 })));
        assert!(!events.iter().any(|e| matches!(e, ScanEvent::Error { .. })));
    }
}
