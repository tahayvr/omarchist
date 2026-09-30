//! One window per session. A second `omarchist` (from the bar widget, a
//! keybind, or a terminal) hands its `--view` to the running one over a
//! Unix socket in `$XDG_RUNTIME_DIR` and exits, so the existing window
//! comes forward on that page instead of a second window opening.
use std::io::Read;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};

const SOCKET_NAME: &str = "omarchist.sock";
const REPLY_TIMEOUT: Duration = Duration::from_secs(2);
/// A request is one short JSON line; anything longer is not one.
const MAX_REQUEST_BYTES: u64 = 4096;

static OWNS_SOCKET: AtomicBool = AtomicBool::new(false);

/// What a second launch asks the running window to show.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenRequest {
    /// A CLI view name; `None` only brings the window forward.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

pub fn socket_path() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.is_dir())
        .or_else(dirs::runtime_dir)?;
    Some(dir.join(SOCKET_NAME))
}

/// Hands the request to a running instance. `false` means there is none
/// (or it did not answer), so this process should open its own window.
pub fn forward(request: &OpenRequest) -> bool {
    let Some(path) = socket_path() else {
        return false;
    };
    let Ok(mut stream) = UnixStream::connect(&path) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(REPLY_TIMEOUT));
    let _ = stream.set_write_timeout(Some(REPLY_TIMEOUT));
    let Ok(line) = serde_json::to_string(request) else {
        return false;
    };
    if stream.write_all(format!("{line}\n").as_bytes()).is_err() {
        return false;
    }
    // The instance acknowledges, so a dead socket is not mistaken for one.
    let mut reply = String::new();
    BufReader::new(stream).read_line(&mut reply).is_ok() && reply.trim() == "ok"
}

/// Whether an instance is answering on the socket. A stale socket left by
/// a crash refuses the connection, so it does not count.
pub fn is_running() -> bool {
    socket_path().is_some_and(|path| UnixStream::connect(path).is_ok())
}

/// The outcome of trying to own the instance socket.
pub enum Listen {
    /// This process answers launches from now on.
    Bound(UnixListener),
    /// Another instance answered on the socket first (or is binding it):
    /// hand the request over instead of opening a window.
    Taken,
    /// No runtime directory or the socket cannot be bound; run without one.
    Unavailable,
}

/// Binds the socket for this instance, replacing one left by a crashed
/// instance (a socket nobody answers on).
pub fn listen() -> Listen {
    let Some(path) = socket_path() else {
        return Listen::Unavailable;
    };
    if path.exists() {
        if UnixStream::connect(&path).is_ok() {
            return Listen::Taken;
        }
        let _ = std::fs::remove_file(&path);
    }
    match UnixListener::bind(&path) {
        Ok(listener) => {
            OWNS_SOCKET.store(true, Ordering::SeqCst);
            Listen::Bound(listener)
        }
        // Lost the race to another launch binding at the same moment.
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => Listen::Taken,
        Err(_) => Listen::Unavailable,
    }
}

/// One connection's worth of the accept loop.
#[derive(Debug, PartialEq, Eq)]
pub enum Accepted {
    Request(OpenRequest),
    /// The peer sent nothing usable (no line, too long, not a request);
    /// the listener keeps serving.
    Rejected,
    /// The listener itself failed; nothing more will arrive.
    Gone,
}

/// Waits for one connection, answers it, and says what came of it.
pub fn accept(listener: &UnixListener) -> Accepted {
    let Ok((stream, _)) = listener.accept() else {
        return Accepted::Gone;
    };
    let _ = stream.set_read_timeout(Some(REPLY_TIMEOUT));
    let _ = stream.set_write_timeout(Some(REPLY_TIMEOUT));
    let mut line = String::new();
    // Capped, so a peer that never sends a newline cannot grow the buffer.
    let mut reader = BufReader::new((&stream).take(MAX_REQUEST_BYTES));
    if reader.read_line(&mut line).is_err() || !line.ends_with('\n') {
        return Accepted::Rejected;
    }
    let Some(request) = parse_request(&line) else {
        return Accepted::Rejected;
    };
    let _ = (&stream).write_all(b"ok\n");
    Accepted::Request(request)
}

pub fn parse_request(line: &str) -> Option<OpenRequest> {
    serde_json::from_str(line.trim()).ok()
}

/// Removes the socket, but only when this process bound it: a second
/// instance quitting must not cut the first one off.
pub fn remove_socket() {
    if OWNS_SOCKET.load(Ordering::SeqCst)
        && let Some(path) = socket_path()
    {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_round_trip_as_one_json_line() {
        let request = OpenRequest {
            view: Some("themes".into()),
            theme: Some("my-theme".into()),
        };
        let line = serde_json::to_string(&request).unwrap();
        assert!(!line.contains('\n'));
        assert_eq!(parse_request(&format!("{line}\n")), Some(request));
        assert_eq!(parse_request("{}"), Some(OpenRequest::default()));
        assert_eq!(parse_request("not json"), None);
    }

    fn socket_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("omarchist-instance-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn forward_and_accept_meet_on_a_socket() {
        let dir = socket_dir("meet");
        let path = dir.join(SOCKET_NAME);
        let listener = UnixListener::bind(&path).unwrap();
        let server = std::thread::spawn(move || accept(&listener));

        let mut stream = UnixStream::connect(&path).unwrap();
        stream.write_all(b"{\"view\":\"flows\"}\n").unwrap();
        let mut reply = String::new();
        BufReader::new(stream).read_line(&mut reply).unwrap();
        assert_eq!(reply.trim(), "ok");
        assert_eq!(
            server.join().unwrap(),
            Accepted::Request(OpenRequest {
                view: Some("flows".into()),
                theme: None
            })
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn bad_connections_are_rejected_and_the_listener_keeps_serving() {
        let dir = socket_dir("bad");
        let path = dir.join(SOCKET_NAME);
        let listener = UnixListener::bind(&path).unwrap();
        let server = std::thread::spawn(move || {
            let mut outcomes = Vec::new();
            for _ in 0..4 {
                outcomes.push(accept(&listener));
            }
            outcomes
        });

        // Connects and hangs up without a word.
        drop(UnixStream::connect(&path).unwrap());
        // Not a request.
        let mut stream = UnixStream::connect(&path).unwrap();
        stream.write_all(b"x\n").unwrap();
        drop(stream);
        // A line longer than the cap, never terminated.
        let mut stream = UnixStream::connect(&path).unwrap();
        stream
            .write_all(&vec![b'{'; MAX_REQUEST_BYTES as usize + 10])
            .unwrap();
        drop(stream);
        // A real request afterwards is still answered.
        let mut stream = UnixStream::connect(&path).unwrap();
        stream.write_all(b"{}\n").unwrap();
        let mut reply = String::new();
        BufReader::new(stream).read_line(&mut reply).unwrap();
        assert_eq!(reply.trim(), "ok");

        assert_eq!(
            server.join().unwrap(),
            vec![
                Accepted::Rejected,
                Accepted::Rejected,
                Accepted::Rejected,
                Accepted::Request(OpenRequest::default()),
            ]
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_process_that_did_not_bind_the_socket_does_not_remove_it() {
        // `OWNS_SOCKET` is false unless `listen` bound it in this process.
        assert!(!OWNS_SOCKET.load(Ordering::SeqCst));
        remove_socket();
    }
}
