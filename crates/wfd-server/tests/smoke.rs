//! Smoke test of the binary (work-breakdown.md T11 acceptance `cargo test -p wfd-server --test smoke`): spawns
//! `warframe-dps --no-browser --port 0`, reads the printed URL (server-api.md §1.2 step 5) and requests `/api/meta`.
//!
//! Skeleton (T02d): `/api/meta` answers 501 `api.not_implemented`; T11 changes the expectation to 200 with
//! `app_id = "warframe-dps"` when it wires the route.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// Kills the server when the test ends, also on failure.
struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Plain HTTP/1.1 GET; returns the status code and the body.
fn get(host: &str, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(host).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("response");
    let (head, body) = response.split_once("\r\n\r\n").expect("headers");
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .expect("status line");
    (status, body.to_owned())
}

#[test]
fn binary_prints_its_url_and_answers() {
    let child = Command::new(env!("CARGO_BIN_EXE_warframe-dps"))
        .args(["--no-browser", "--port", "0"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn warframe-dps");
    let mut server = Server(child);
    let stdout = server.0.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(stdout).read_line(&mut line);
        let _ = tx.send(line);
    });
    let line = rx
        .recv_timeout(Duration::from_secs(60))
        .expect("no startup line within 60 s");
    assert!(line.starts_with("warframe-dps "), "{line:?}");
    let url = line
        .split_whitespace()
        .find(|w| w.starts_with("http://"))
        .unwrap_or_else(|| panic!("no URL in {line:?}"));
    let host = url
        .strip_prefix("http://")
        .and_then(|u| u.strip_suffix('/'))
        .unwrap_or_else(|| panic!("URL {url:?} is not http://host:port/"));
    assert!(host.starts_with("127.0.0.1:"), "{host}");
    assert_ne!(host, "127.0.0.1:0");

    let (status, body) = get(host, "/api/meta");
    assert_eq!(status, 501, "{body}");
    let body: serde_json::Value = serde_json::from_str(&body).expect("JSON body");
    assert_eq!(body["error"]["key"], "api.not_implemented");
    assert_eq!(body["error"]["args"]["route"], "GET /api/meta");
}
