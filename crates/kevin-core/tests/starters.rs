use kevin_core::*;
use serde_json::json;
#[test]
fn developer_voice_pack_defaults_to_stark_and_keeps_serious_context_limits() {
    let config = default_config();
    assert_eq!(config.voices.len(), 12);
    assert_eq!(config.routing.fallback, "tony-stark");
    config.validate().unwrap();
    for name in [
        "tony-stark",
        "chandler",
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
fn chandler_follows_invited_banter_but_cannot_override_context_limits() {
    let config = default_config();
    let request: Request = parse(json!({
        "intent": "Reply to the supplied joke without adding commitments.",
        "voice": "chandler",
        "context": {
            "platform": "slack", "task": "reply", "audience": "teammate",
            "seriousness": "ordinary", "energy": "playful", "existingHumor": true
        }
    }))
    .unwrap();
    let envelope = compile_typed(&request, &config, &Memory::default()).unwrap();
    assert_eq!(envelope.data.plan.route.voices, ["chandler"]);
    assert_eq!(envelope.data.plan.route.traits.warmth, 0.75);
    assert_eq!(envelope.data.plan.route.traits.verbosity, 0.15);
    assert_eq!(envelope.data.plan.humor.mode, HumorMode::Invited);
    for context in [
        json!({"task":"review", "seriousness":"ordinary", "audience":"teammate", "energy":"playful", "existingHumor":true}),
        json!({"seriousness":"critical", "audience":"teammate", "energy":"playful", "existingHumor":true}),
        json!({"seriousness":"sensitive", "audience":"teammate", "energy":"playful", "existingHumor":true}),
        json!({"seriousness":"ordinary", "audience":"teammate", "energy":"unknown", "existingHumor":false}),
        json!({}),
    ] {
        let request: Request = parse(json!({
            "intent": "Preserve the supplied facts.", "voice": "chandler", "context": context
        }))
        .unwrap();
        let envelope = compile_typed(&request, &config, &Memory::default()).unwrap();
        assert_eq!(envelope.data.plan.humor.mode, HumorMode::Off);
        assert_eq!(envelope.data.plan.route.traits.humor, 0.0);
        assert_eq!(envelope.data.plan.route.traits.sarcasm, 0.0);
    }
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

#[test]
fn configured_slack_voice_can_use_optional_humor_with_incomplete_context() {
    let mut config = default_config();
    config.routing.rules.push(
        parse(json!({
            "name": "slack-voice", "priority": 80,
            "when": {"platform": "slack"}, "voice": "chandler"
        }))
        .unwrap(),
    );

    let request = json!({
        "intent": "Explain the verified reason briefly.",
        "facts": ["Outcome: Changes requested", "The retry still drops the request."],
        "voice": "chandler",
        "context": {"platform": "slack", "task": "reply", "channel": "pr-queue"}
    });
    let parsed: Request = parse(request.clone()).unwrap();
    let envelope = compile_typed(&parsed, &config, &Memory::default()).unwrap();
    assert_eq!(envelope.data.request, parsed);
    let plan = envelope.data.plan;
    assert_eq!(plan.context.seriousness, Seriousness::Unknown);
    assert_eq!(plan.context.audience, Audience::Unknown);
    assert_eq!(plan.context.energy, Energy::Unknown);
    assert_eq!(plan.route.voices, ["chandler"]);
    assert_eq!(plan.humor.mode, HumorMode::Optional);
    assert_eq!(plan.route.traits.humor, 0.5);
    assert_eq!(plan.route.traits.sarcasm, 0.0);
    assert!(
        plan.route
            .behaviors
            .iter()
            .any(|b| b.contains("self-deprecation"))
    );

    for (field, value) in [
        ("seriousness", "sensitive"),
        ("seriousness", "critical"),
        ("energy", "tense"),
        ("audience", "external"),
        ("audience", "public"),
        ("task", "review"),
        ("platform", "github"),
    ] {
        let mut input = request.clone();
        input["context"][field] = json!(value);
        let plan = compile(input, serde_json::to_value(&config).unwrap(), None)
            .unwrap()
            .data
            .plan;
        assert_eq!(plan.humor.mode, HumorMode::Off, "{field}={value}");
    }

    let mut off = request.clone();
    off["humor"] = json!("off");
    let mut security = request.clone();
    security["facts"] = json!(["Security vulnerability confirmed."]);
    let mut other_voice = request.clone();
    other_voice["voice"] = json!("tony-stark");
    let mut automatic = request.clone();
    automatic.as_object_mut().unwrap().remove("voice");
    for input in [off, security, other_voice, automatic] {
        let parsed = parse(input).unwrap();
        let envelope = compile_typed(&parsed, &config, &Memory::default()).unwrap();
        assert_eq!(envelope.data.plan.humor.mode, HumorMode::Off);
    }

    config.voices.get_mut("chandler").unwrap().traits.humor = Some(0.0);
    let parsed = parse(request).unwrap();
    let envelope = compile_typed(&parsed, &config, &Memory::default()).unwrap();
    assert_eq!(envelope.data.plan.humor.mode, HumorMode::Off);
}
