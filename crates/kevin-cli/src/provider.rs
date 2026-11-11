use kevin_core::{Envelope, Generator, Result};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

pub struct CommandGenerator {
    pub executable: String,
    pub args: Vec<String>,
    pub timeout: Duration,
}
impl CommandGenerator {
    pub fn new(executable: &str, args: &[String]) -> Self {
        Self {
            executable: executable.into(),
            args: args.to_vec(),
            timeout: Duration::from_secs(30),
        }
    }
}
enum Event {
    Output(std::io::Result<Vec<u8>>),
    Input(std::io::Result<()>),
}
impl Generator for CommandGenerator {
    fn generate(&self, envelope: &Envelope) -> Result<Value> {
        let mut child = Command::new(&self.executable)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "Cannot start model command. Check executable path.")?;
        let result = (|| {
            let mut stdin = child.stdin.take().ok_or("Model stdin unavailable")?;
            let stdout = child.stdout.take().ok_or("Model stdout unavailable")?;
            let data = serde_json::to_vec(envelope).map_err(|e| e.to_string())?;
            let (tx, rx) = mpsc::channel();
            let writer = tx.clone();
            thread::spawn(move || {
                let result = stdin.write_all(&data);
                drop(stdin);
                let _ = writer.send(Event::Input(result));
            });
            thread::spawn(move || {
                let mut data = vec![];
                let result = stdout.take(1_000_001).read_to_end(&mut data).map(|_| data);
                let _ = tx.send(Event::Output(result));
            });
            let start = Instant::now();
            let mut output = None;
            let mut written = false;
            loop {
                if start.elapsed() > self.timeout {
                    return Err("Model command exceeded timeout (default 30 seconds).".into());
                }
                match rx.recv_timeout(Duration::from_millis(5)) {
                    Ok(Event::Input(result)) => {
                        result.map_err(|_| "Model command closed stdin before reading request.")?;
                        written = true;
                    }
                    Ok(Event::Output(result)) => {
                        let data = result.map_err(|_| "Cannot read model output")?;
                        if data.len() > 1_000_000 {
                            return Err("Model output exceeded 1 MB.".into());
                        }
                        output = Some(data);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => {
                    }
                }
                if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                    if !status.success() {
                        return Err(format!(
                            "Model command failed (exit {}).",
                            status
                                .code()
                                .map_or("signal".into(), |code| code.to_string())
                        ));
                    }
                    if written && let Some(data) = output.take() {
                        return serde_json::from_slice(&data).map_err(|_|"Model command must return one JSON object; no fences or logging on stdout.".into());
                    }
                }
            }
        })();
        if result.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        result
    }
}

pub struct OllamaGenerator<'a> {
    pub model: Option<&'a str>,
}

impl Generator for OllamaGenerator<'_> {
    fn generate(&self, envelope: &Envelope) -> Result<Value> {
        let model = self
            .model
            .map(str::to_owned)
            .or_else(|| std::env::var("KEVIN_MODEL").ok())
            .filter(|model| !model.trim().is_empty())
            .ok_or("Pass --model or set KEVIN_MODEL to an installed Ollama model.")?;
        generate_ollama(
            &envelope.system,
            &envelope.data,
            &envelope.output_schema,
            &model,
        )
    }
}

pub fn ollama() -> Result<Value> {
    let envelope: Value = serde_json::from_str(&crate::storage::read_stdin()?)
        .map_err(|_| "Invalid envelope JSON")?;
    let (Some(system), Some(data), Some(schema)) = (
        envelope.get("system").and_then(Value::as_str),
        envelope.get("data").filter(|value| value.is_object()),
        envelope
            .get("outputSchema")
            .filter(|value| value.is_object()),
    ) else {
        return Err("Invalid composition envelope".into());
    };
    if envelope.get("version").and_then(Value::as_f64) != Some(1.0) {
        return Err("Invalid composition envelope".into());
    }
    let model = std::env::var("KEVIN_MODEL")
        .ok()
        .filter(|s| !s.is_empty())
        .ok_or("Set KEVIN_MODEL to an already installed local model.")?;
    generate_ollama(system, data, schema, &model)
}

fn generate_ollama(
    system: &str,
    data: &impl Serialize,
    schema: &Value,
    model: &str,
) -> Result<Value> {
    let port = std::env::var("KEVIN_OLLAMA_PORT").unwrap_or_else(|_| "11434".into());
    if port.is_empty()
        || port.len() > 5
        || !port.bytes().all(|c| c.is_ascii_digit())
        || port.parse::<u16>().ok().filter(|p| *p > 0).is_none()
    {
        return Err("Invalid loopback port.".into());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(25))
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Cannot create local HTTP client")?;
    let data = serde_json::to_string(data).map_err(|e| e.to_string())?;
    let response = client
        .post(format!("http://127.0.0.1:{port}/api/chat"))
        .json(&json!({
            "model": model,
            "stream": false,
            "messages": [{"role": "system", "content": system}, {"role": "user", "content": data}],
            "format": schema,
            "options": {"temperature": 0, "seed": 42}
        }))
        .send()
        .map_err(|error| {
            if error.is_timeout() {
                "Local Ollama request timed out."
            } else {
                "Cannot reach local Ollama server. Check that it is running on the selected port."
            }
        })?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err("Ollama HTTP 404. Check that the selected model is installed.".into());
    }
    if !response.status().is_success() {
        return Err(format!("Ollama HTTP {}", response.status().as_u16()));
    }
    let mut bytes = vec![];
    response
        .take(1_000_001)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read Ollama response")?;
    if bytes.len() > 1_000_000 {
        return Err("Ollama response exceeded 1 MB".into());
    }
    let body: Value = serde_json::from_slice(&bytes).map_err(|_| "Invalid Ollama response")?;
    let content = body
        .pointer("/message/content")
        .and_then(Value::as_str)
        .ok_or("Invalid Ollama response")?;
    serde_json::from_str(content).map_err(|_| "Ollama content must be JSON".into())
}
