//! One window per session. A second `omarchist` (from the bar widget, a
//! keybind, or a terminal) hands its `--view` to the running one over a
//! Unix socket in `$XDG_RUNTIME_DIR` and exits, so the existing window
//! comes forward on that page instead of a second window opening.
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

const SOCKET_NAME: &str = "omarchist.sock";
const REPLY_TIMEOUT: Duration = Duration::from_secs(2);

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

/// Binds the socket for this instance, replacing one left by a crashed
/// instance (a socket nobody answers on).
pub fn listen() -> Option<UnixListener> {
    let path = socket_path()?;
    if path.exists() {
        if UnixStream::connect(&path).is_ok() {
            return None;
        }
        let _ = std::fs::remove_file(&path);
    }
    UnixListener::bind(&path).ok()
}

/// Waits for one request and acknowledges it. `None` when the listener
/// is gone.
pub fn accept(listener: &UnixListener) -> Option<OpenRequest> {
    let (stream, _) = listener.accept().ok()?;
    let _ = stream.set_read_timeout(Some(REPLY_TIMEOUT));
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let request = parse_request(&line)?;
    let _ = reader.get_mut().write_all(b"ok\n");
    Some(request)
}

pub fn parse_request(line: &str) -> Option<OpenRequest> {
    serde_json::from_str(line.trim()).ok()
}

pub fn remove_socket() {
    if let Some(path) = socket_path() {
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

    #[test]
    fn forward_and_accept_meet_on_a_socket() {
        let dir = std::env::temp_dir().join(format!("omarchist-instance-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
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
            Some(OpenRequest {
                view: Some("flows".into()),
                theme: None
            })
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
