mod common;
use common::*;
use kevin_core::{compile, default_config};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[test]
fn ollama_bridge_sends_separate_roles_and_schema_to_real_loopback_protocol_fixture() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).unwrap();
    let server = thread::spawn(move || {
        let start = Instant::now();
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        start.elapsed() < Duration::from_secs(5),
                        "client never connected"
                    );
                    thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("{e}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = vec![];
        let end = loop {
            let mut buf = [0; 1024];
            let n = stream.read(&mut buf).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buf[..n]);
            if let Some(index) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                break index + 4;
            }
        };
        let header = String::from_utf8(bytes[..end].to_vec()).unwrap();
        assert!(header.starts_with("POST /api/chat HTTP/1.1"));
        let len: usize = header
            .lines()
            .find_map(|line| {
                line.split_once(':')
                    .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                    .map(|(_, v)| v.trim().parse().unwrap())
            })
            .unwrap();
        while bytes.len() < end + len {
            let mut buf = [0; 1024];
            let n = stream.read(&mut buf).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buf[..n]);
        }
        let request: Value = serde_json::from_slice(&bytes[end..end + len]).unwrap();
        let body=json!({"message":{"content":json!({"status":"draft","text":"Guard items[0].id first."}).to_string()}}).to_string();
        write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
        request
    });
    let dir = init();
    let request =
        json!({"intent":"Write a review.","context":{"task":"review"},"preserve":["items[0].id"]});
    std::fs::write(dir.path().join("request.json"), request.to_string()).unwrap();
    let child = Command::new(binary())
        .args([
            "review",
            "--request",
            "request.json",
            "--model-command",
            binary(),
            "--model-arg",
            "ollama",
        ])
        .current_dir(dir.path())
        .env("KEVIN_MODEL", "fixture")
        .env("KEVIN_OLLAMA_PORT", port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8(out.stdout).unwrap().trim(),
        "Guard items[0].id first."
    );
    let received = server.join().unwrap();
    let e = compile(
        request,
        serde_json::to_value(default_config()).unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(
        received,
        json!({"model":"fixture","stream":false,"messages":[{"role":"system","content":e.system},{"role":"user","content":serde_json::to_string(&e.data).unwrap()}],"format":e.output_schema,"options":{"temperature":0,"seed":42}})
    );
}
#[test]
fn ollama_rejects_remote_port_injection_and_invalid_envelopes_without_network() {
    for port in ["http://example.com", "0", "65536", "-1"] {
        let envelope = compile(
            json!({"intent":"Reply."}),
            serde_json::to_value(default_config()).unwrap(),
            None,
        )
        .unwrap();
        let mut child = Command::new(binary())
            .arg("ollama")
            .env("KEVIN_MODEL", "fixture")
            .env("KEVIN_OLLAMA_PORT", port)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(serde_json::to_string(&envelope).unwrap().as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains("Invalid loopback port"));
    }
    assert!(
        !run(std::path::Path::new("."), &["ollama"], "{}")
            .status
            .success()
    );
}
