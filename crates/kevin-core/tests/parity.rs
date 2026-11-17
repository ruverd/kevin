mod common;
use common::*;
use kevin_core::*;
use serde_json::{Value, json};
#[test]
fn typescript_v1_envelopes_preserve_non_policy_fields_across_34_baselines() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/typescript-v1.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let result = compile(
            case["request"].clone(),
            case.get("config").cloned().unwrap_or_else(config),
            case.get("memory").cloned(),
        )
        .unwrap_or_else(|e| panic!("{}: {e}", case["id"]));
        let mut expected = case["expected"].clone();
        expected["system"] = json!(COMPOSITION_INSTRUCTIONS);
        let id = case["id"].as_str().unwrap();
        let optional_intensity = match id {
            "serious-review-forced"
            | "pr-description-forced"
            | "congratulations-forced"
            | "architecture-forced"
            | "casual-slack-forced"
            | "ambiguous-forced"
            | "pt-to-en-forced"
            | "quoted-injection-forced"
            | "no-reply-forced" => Some(0.65),
            "congratulations-auto"
            | "architecture-auto"
            | "casual-slack-auto"
            | "ambiguous-auto"
            | "pt-to-en-auto"
            | "quoted-injection-auto"
            | "no-reply-auto"
            | "custom-30"
            | "custom-31"
            | "custom-32" => Some(0.25),
            _ => None,
        };
        if let Some(intensity) = optional_intensity {
            expected["data"]["plan"]["route"]["traits"]["humor"] = json!(intensity);
            expected["data"]["plan"]["humor"] = json!({
                "mode": "optional",
                "styles": ["understatement", "dry observation", "playful exaggeration"],
                "reason": "Humor may fit; omit it whenever forced or less clear than a plain reply."
            });
        } else if matches!(id, "serious-review-auto" | "pr-description-auto") {
            expected["data"]["plan"]["humor"]["reason"] =
                json!("Selected identity or voice disables humor.");
        } else if expected["data"]["plan"]["humor"]["reason"]
            == "Seriousness is sensitive, critical, or unknown."
        {
            expected["data"]["plan"]["humor"]["reason"] =
                json!("Seriousness is sensitive or critical.");
        }
        if expected["data"]["plan"]["humor"]["mode"] != "off" {
            expected["data"]["plan"]["humor"]["styles"]
                .as_array_mut()
                .unwrap()
                .extend([
                    json!("situational irony"),
                    json!("brief personification"),
                    json!("expectation reversal"),
                ]);
        }
        equivalent(&serde_json::to_value(result).unwrap(), &expected);
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
