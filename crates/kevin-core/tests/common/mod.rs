#![allow(dead_code)]
use kevin_core::*;
use serde_json::{Value, json};
pub fn config() -> Value {
    serde_json::from_str(include_str!(
        "../../../../tests/fixtures/default-config-v1.json"
    ))
    .unwrap()
}
pub fn ordinary() -> Value {
    json!({"platform":"slack","audience":"teammate","seriousness":"ordinary","energy":"playful","existingHumor":true})
}
pub fn env(request: Value) -> Envelope {
    compile(request, config(), None).unwrap()
}
pub fn with(request: Value, config: Value) -> Envelope {
    compile(request, config, None).unwrap()
}
pub fn scoped() -> Value {
    let mut config = config();
    let rules = config["routing"]["rules"].as_array_mut().unwrap();
    rules.push(json!({"name":"channel","priority":80,"when":{"platform":"slack","workspace":"T_EXAMPLE","channel":"C_EXAMPLE"},"voice":"witty-engineer"}));
    rules.push(json!({"name":"person","priority":85,"when":{"platform":"slack","workspace":"T_EXAMPLE","recipient":"U_EXAMPLE"},"voice":"minimalist"}));
    config
}
pub fn entry(id: &str, text: &str) -> Value {
    json!({"id":id,"text":text,"kind":"preference","source":"user","createdAt":"2026-11-03T12:00:00Z"})
}
pub fn equivalent(left: &Value, right: &Value) {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => assert_eq!(a.as_f64(), b.as_f64()),
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len(), "object fields differ: {left} vs {right}");
            for (key, value) in a {
                equivalent(
                    value,
                    b.get(key).unwrap_or_else(|| panic!("missing key {key}")),
                );
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                equivalent(a, b);
            }
        }
        _ => assert_eq!(left, right),
    }
}
