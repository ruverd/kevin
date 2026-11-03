use kevin_core::*;
use serde_json::json;
#[test]
fn developer_voice_pack_defaults_to_stark_and_keeps_serious_context_limits() {
    let config = default_config();
    assert_eq!(config.voices.len(), 11);
    assert_eq!(config.routing.fallback, "tony-stark");
    config.validate().unwrap();
    for name in [
        "tony-stark",
        "rubber-duck",
        "pragmatic-architect",
        "incident-commander",
    ] {
        assert!(config.voices.contains_key(name));
    }
    let request:Request=parse(json!({"intent":"Reply to the coffee joke.","context":{"platform":"slack","audience":"teammate","seriousness":"ordinary","existingHumor":true,"energy":"playful"}})).unwrap();
    let e = compile_typed(&request, &config, &Memory::default()).unwrap();
    assert_eq!(e.data.plan.route.voices, ["tony-stark"]);
    assert_eq!(e.data.plan.humor.mode, HumorMode::Invited);
    let request: Request =
        parse(json!({"intent":"Report an incident.","voice":"tony-stark"})).unwrap();
    let e = compile_typed(&request, &config, &Memory::default()).unwrap();
    assert_eq!(e.data.plan.route.voices, ["tony-stark"]);
    assert_eq!(e.data.plan.humor.mode, HumorMode::Off);
    assert_eq!(e.data.plan.route.traits.sarcasm, 0.0);
}
#[test]
fn original_thirty_routing_scenarios_remain_valid_with_original_v1_profile() {
    let profile: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/default-config-v1.json"
    ))
    .unwrap();
    let scenarios: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../tests/fixtures/scenarios-v1.json")).unwrap();
    for scenario in scenarios {
        for forced in [false, true] {
            let mut request = scenario["request"].clone();
            if forced {
                request["voice"] = json!("dry-comedian");
            }
            let e = compile(request, profile.clone(), None).unwrap();
            assert_eq!(
                e.data.plan.route.voices[0],
                if forced {
                    "dry-comedian"
                } else {
                    scenario["expectedVoice"].as_str().unwrap()
                }
            );
            assert_eq!(
                serde_json::to_value(e.data.plan.humor.mode).unwrap(),
                scenario["expectedHumor"]
            );
        }
    }
}
