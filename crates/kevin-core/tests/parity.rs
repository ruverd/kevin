mod common;
use common::*;
use kevin_core::*;
use serde_json::Value;
#[test]
fn typescript_v1_composition_envelopes_match_all_34_baselines() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/typescript-v1.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let result = compile(
            case["request"].clone(),
            case.get("config").cloned().unwrap_or_else(config),
            case.get("memory").cloned(),
        )
        .unwrap_or_else(|e| panic!("{}: {e}", case["id"]));
        equivalent(&serde_json::to_value(result).unwrap(), &case["expected"]);
    }
}
#[test]
fn typescript_v1_interview_states_match_all_five_baselines() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../../../tests/fixtures/voice-design-v1.json")).unwrap();
    for case in cases {
        let result = design_voice(
            case["description"].as_str().unwrap(),
            case["answers"].clone(),
        )
        .unwrap();
        equivalent(&serde_json::to_value(result).unwrap(), &case["expected"]);
    }
}
