mod common;
#[allow(dead_code)]
#[path = "../src/provider.rs"]
mod provider;
#[allow(dead_code)]
#[path = "../src/storage.rs"]
mod storage;
use kevin_core::{Generator, compile, default_config};
use serde_json::json;
use std::time::{Duration, Instant};
#[test]
fn model_timeout_kills_child_and_returns_without_waiting_for_its_completion() {
    let envelope = compile(
        json!({"intent":"Reply."}),
        serde_json::to_value(default_config()).unwrap(),
        None,
    )
    .unwrap();
    let generator = provider::CommandGenerator {
        executable: common::fixture().to_string_lossy().into_owned(),
        args: vec!["sleep".into()],
        timeout: Duration::from_millis(100),
    };
    let start = Instant::now();
    assert!(
        generator
            .generate(&envelope)
            .unwrap_err()
            .contains("timeout")
    );
    assert!(start.elapsed() < Duration::from_secs(1));
}
