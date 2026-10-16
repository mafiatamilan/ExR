//! Minimal blocking QMP client over a unix socket.
//!
//! ## Why our own client (decision, 2026-10-16)
//!
//! Two existing options were evaluated against the live crates.io index:
//!
//! - `qmp` 0.1.1 — a real QMP *client*, but a single 0.x release with ~100
//!   downloads. Too immature to anchor our control plane on.
//! - `qapi-qmp` 0.15.x — mature and widely used, but it is *types only*
//!   (generated QAPI bindings); it ships no socket, handshake, or event-loop.
//!
//! Our needs are narrow and stable: greeting handshake, `qmp_capabilities`,
//! a dozen lifecycle/query commands, and `screendump` bytes. A ~200-line
//! blocking client over [`std::os::unix::net::UnixStream`] with `serde_json`
//! gives us full control over timeouts, cleanup, and error shapes, with zero
//! extra dependencies. Revisit in M5 if the command surface grows beyond
//! lifecycle + snapshots + display + input.
//!
//! Protocol notes: QMP speaks newline-delimited JSON. The server emits
//! asynchronous `{"event": ...}` objects at any time; [`QmpClient::execute`]
//! skips those and returns the first `return`/`error` object. All socket IO
//! uses read timeouts so a wedged guest fails loudly instead of hanging us.

use crate::error::VmError;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

/// Blocking QMP connection. One client per VM; not shared across threads
/// (callers hold it behind a mutex — see `qemu.rs`).
pub struct QmpClient {
    reader: BufReader<UnixStream>,
}

impl std::fmt::Debug for QmpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QmpClient").finish_non_exhaustive()
    }
}

impl QmpClient {
    /// Connect to `sock`, consume the greeting, negotiate capabilities.
    pub fn connect(sock: &Path, timeout: Duration) -> Result<Self, VmError> {
        let stream = UnixStream::connect(sock).map_err(|e| VmError::Io {
            ctx: "qmp connect",
            source: e,
        })?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| VmError::Io {
                ctx: "qmp set timeout",
                source: e,
            })?;
        let mut client = Self {
            reader: BufReader::new(stream),
        };
        let greeting = client.read_object()?;
        if greeting.get("QMP").is_none() {
            return Err(VmError::QmpHandshake(format!(
                "first object is not a QMP greeting: {greeting}"
            )));
        }
        client.execute("qmp_capabilities", None)?;
        Ok(client)
    }

    /// Run one command; returns the `return` payload (events are skipped).
    pub fn execute(&mut self, cmd: &str, args: Option<Value>) -> Result<Value, VmError> {
        let mut req = serde_json::json!({ "execute": cmd });
        if let Some(a) = args {
            req["arguments"] = a;
        }
        let line = serde_json::to_string(&req).map_err(|e| VmError::QmpTransport(e.to_string()))?;
        self.reader
            .get_mut()
            .write_all(line.as_bytes())
            .and_then(|()| self.reader.get_mut().write_all(b"\n"))
            .and_then(|()| self.reader.get_mut().flush())
            .map_err(|e| VmError::Io {
                ctx: "qmp write",
                source: e,
            })?;
        loop {
            let obj = self.read_object()?;
            if obj.get("event").is_some() {
                continue; // async event (e.g. SHUTDOWN, STOP) — not our answer
            }
            if let Some(err) = obj.get("error") {
                return Err(VmError::QmpCommand {
                    cmd: cmd.to_string(),
                    detail: err.to_string(),
                });
            }
            if obj.get("return").is_some() {
                return Ok(obj["return"].clone());
            }
            return Err(VmError::QmpTransport(format!(
                "unexpected QMP object without return/error/event: {obj}"
            )));
        }
    }

    /// `query-status` → true while the vCPUs run.
    pub fn is_running(&mut self) -> Result<bool, VmError> {
        let ret = self.execute("query-status", None)?;
        Ok(ret.get("running").and_then(Value::as_bool).unwrap_or(false))
    }

    /// Ask QEMU to write a PPM screendump to `path` (server-side path).
    pub fn screendump(&mut self, path: &Path) -> Result<(), VmError> {
        let Some(s) = path.to_str() else {
            return Err(VmError::InvalidSpec("screendump path is not UTF-8".into()));
        };
        self.execute("screendump", Some(serde_json::json!({ "filename": s })))?;
        Ok(())
    }

    /// Graceful ACPI power-button request.
    pub fn powerdown(&mut self) -> Result<(), VmError> {
        self.execute("system_powerdown", None)?;
        Ok(())
    }

    /// Hard stop of vCPUs (guest keeps running state, execution halts).
    pub fn stop_vcpus(&mut self) -> Result<(), VmError> {
        self.execute("stop", None)?;
        Ok(())
    }

    /// End the QEMU process via QMP.
    pub fn quit(&mut self) -> Result<(), VmError> {
        // `quit` closes the connection without a return object; tolerate EOF.
        let req = serde_json::json!({ "execute": "quit" });
        let line = serde_json::to_string(&req).map_err(|e| VmError::QmpTransport(e.to_string()))?;
        let _ = self.reader.get_mut().write_all(line.as_bytes());
        let _ = self.reader.get_mut().write_all(b"\n");
        let _ = self.reader.get_mut().flush();
        Ok(())
    }

    fn read_object(&mut self) -> Result<Value, VmError> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line).map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut
                || e.kind() == std::io::ErrorKind::WouldBlock
            {
                VmError::QmpTransport("QMP read timed out".into())
            } else {
                VmError::Io {
                    ctx: "qmp read",
                    source: e,
                }
            }
        })?;
        if n == 0 {
            return Err(VmError::QmpTransport("QMP connection closed (EOF)".into()));
        }
        serde_json::from_str(&line).map_err(|e| VmError::QmpTransport(format!("bad QMP JSON: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;

    /// Drive the client against a scripted fake server on a socketpair-like
    /// listener: greeting + capabilities answer + one command answer.
    fn scripted(responses: &[&str]) -> (PathBuf, std::thread::JoinHandle<()>) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("exr-qmp-test-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let sock = dir.join("q.sock");
        let responses: Vec<String> = responses.iter().map(|s| s.to_string()).collect();
        let listener = UnixListener::bind(&sock).unwrap();
        let h = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            use std::io::{BufRead, BufReader, Write};
            let mut r = BufReader::new(s.try_clone().unwrap());
            let mut line = String::new();
            // greeting first
            s.write_all(b"{\"QMP\": {\"version\": {\"qemu\": {\"micro\": 0, \"minor\": 1, \"major\": 11}, \"package\": \"\"}, \"capabilities\": []}}\n").unwrap();
            let mut it = responses.into_iter();
            // capabilities exchange: one line in, one object out
            line.clear();
            r.read_line(&mut line).unwrap();
            let first = it.next().unwrap();
            s.write_all(first.as_bytes()).unwrap();
            s.write_all(b"\n").unwrap();
            // the test command: one line in, ALL remaining objects out
            // (lets tests interleave async events before the answer)
            line.clear();
            r.read_line(&mut line).unwrap();
            for resp in it {
                s.write_all(resp.as_bytes()).unwrap();
                s.write_all(b"\n").unwrap();
            }
        });
        (sock, h)
    }

    use std::path::PathBuf;

    #[test]
    fn handshake_and_command() {
        let (sock, h) = scripted(&["{\"return\": {}}", "{\"return\": {\"running\": true}}"]);
        let mut c = QmpClient::connect(&sock, Duration::from_secs(5)).unwrap();
        assert!(c.is_running().unwrap());
        h.join().unwrap();
    }

    #[test]
    fn events_are_skipped() {
        let (sock, h) = scripted(&[
            "{\"return\": {}}",
            "{\"timestamp\": {\"seconds\": 1, \"microseconds\": 0}, \"event\": \"STOP\"}",
            "{\"return\": {\"running\": false}}",
        ]);
        let mut c = QmpClient::connect(&sock, Duration::from_secs(5)).unwrap();
        assert!(!c.is_running().unwrap());
        h.join().unwrap();
    }

    #[test]
    fn command_error_surfaces() {
        let (sock, h) = scripted(&[
            "{\"return\": {}}",
            "{\"error\": {\"class\": \"GenericError\", \"desc\": \"nope\"}}",
        ]);
        let mut c = QmpClient::connect(&sock, Duration::from_secs(5)).unwrap();
        let err = c.is_running().unwrap_err();
        assert!(matches!(err, VmError::QmpCommand { .. }));
        h.join().unwrap();
    }

    #[test]
    fn bad_greeting_rejected() {
        let dir = std::env::temp_dir().join(format!("exr-qmp-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let sock = dir.join("q.sock");
        let listener = UnixListener::bind(&sock).unwrap();
        let h = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            use std::io::Write;
            s.write_all(b"{\"hello\": 1}\n").unwrap();
        });
        let err = QmpClient::connect(&sock, Duration::from_secs(5)).unwrap_err();
        assert!(matches!(err, VmError::QmpHandshake(_)));
        h.join().unwrap();
    }
}
