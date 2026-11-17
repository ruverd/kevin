use kevin_core::*;
use serde_json::json;
mod common;
use common::current_scenario_humor;
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
fn chandler_can_follow_existing_banter_within_context_limits() {
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
    assert_eq!(envelope.data.plan.route.traits.sarcasm, 0.1);
    for context in [
        json!({"seriousness":"critical", "audience":"teammate", "energy":"playful", "existingHumor":true}),
        json!({"seriousness":"sensitive", "audience":"teammate", "energy":"playful", "existingHumor":true}),
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
fn original_routing_choices_remain_valid_with_updated_humor_policy() {
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
                current_scenario_humor(&scenario, forced)
            );
        }
    }
}

#[test]
fn assigned_chandler_uses_light_humor_for_routine_slack_review_replies() {
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
        "facts": [
            "Outcome: Changes requested",
            "Medium: The authenticated acceptance walkthrough is missing.",
            "Automated tests and CI passed."
        ],
        "preserve": ["Changes requested"],
        "voice": "chandler",
        "context": {"platform": "slack", "task": "reply", "channel": "pr-queue"}
    });
    let parsed: Request = parse(request.clone()).unwrap();
    let envelope = compile_typed(&parsed, &config, &Memory::default()).unwrap();
    assert_eq!(envelope.data.request, parsed);
    let prompt = render_prompt(&envelope).unwrap();
    assert!(prompt.contains("Permitted humor is always optional"));
    assert!(
        prompt.contains("A short closing sentence or aside may carry the humorous observation")
    );
    assert!(prompt.contains("It need not contain a new fact"));
    assert!(prompt.contains("plan.humor.styles, plan.route.traits.humor"));
    assert!(prompt.contains("not permission to ignore the selected personality"));
    assert!(prompt.contains(r#""humor": 0.5"#));
    assert!(prompt.contains(r#""mode": "optional""#));
    for fact in request["facts"].as_array().unwrap() {
        assert!(prompt.contains(fact.as_str().unwrap()));
    }
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

    for message in [
        "Approval is pending a routine review.",
        "A aprovação está pendente de revisão.",
        "The CI runner could not resolve security.ubuntu.com while installing Chromium.",
        "The package mirror <https://security.ubuntu.com/ubuntu|security.ubuntu.com> was unreachable from CI.",
    ] {
        let mut input = request.clone();
        input["messages"] = json!([message]);
        let parsed = parse(input).unwrap();
        let plan = compile_typed(&parsed, &config, &Memory::default())
            .unwrap()
            .data
            .plan;
        assert_eq!(plan.context.seriousness, Seriousness::Unknown);
        assert_eq!(plan.humor.mode, HumorMode::Optional);
    }

    for (field, value) in [
        ("seriousness", "sensitive"),
        ("seriousness", "critical"),
        ("energy", "tense"),
        ("audience", "external"),
        ("audience", "public"),
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
    let sensitive_inputs = [
        "CI is blocked pending approval.",
        "O CI está bloqueado e a aprovação está pendente.",
        "A security vulnerability was confirmed on security.ubuntu.com.",
        "Our manager is handling a harassment report.",
        "I'm grieving the death of a family member.",
        "I'm stressed and need to make a serious correction.",
        "She received a medical diagnosis.",
        "We are investigating an outage.",
    ]
    .map(|message| {
        let mut input = request.clone();
        input["messages"] = json!([message]);
        input
    });
    for input in [off].into_iter().chain(sensitive_inputs) {
        let parsed = parse(input).unwrap();
        let envelope = compile_typed(&parsed, &config, &Memory::default()).unwrap();
        assert_eq!(envelope.data.plan.humor.mode, HumorMode::Off);
        assert_eq!(envelope.data.plan.route.traits.humor, 0.0);
        assert_eq!(envelope.data.plan.route.traits.sarcasm, 0.0);
        assert_eq!(envelope.system, COMPOSITION_INSTRUCTIONS);
    }

    config.voices.get_mut("chandler").unwrap().traits.humor = Some(0.0);
    let parsed = parse(request).unwrap();
    let envelope = compile_typed(&parsed, &config, &Memory::default()).unwrap();
    assert_eq!(envelope.data.plan.humor.mode, HumorMode::Off);
}

#[test]
fn casual_chandler_slack_reply_permits_optional_humor_without_banter() {
    let config = serde_json::to_value(default_config()).unwrap();
    let request = json!({
        "intent": "Reply with the updated timing.",
        "facts": ["The team's coffee break moved to 3 PM."],
        "voice": "chandler",
        "context": {
            "platform": "slack",
            "task": "reply",
            "audience": "teammate",
            "seriousness": "ordinary",
            "energy": "calm"
        }
    });
    let envelope = compile(request.clone(), config.clone(), None).unwrap();
    assert_eq!(envelope.data.plan.humor.mode, HumorMode::Optional);
    assert_eq!(envelope.data.plan.route.traits.sarcasm, 0.0);

    for (field, value) in [
        ("platform", "email"),
        ("platform", "github"),
        ("task", "draft"),
    ] {
        let mut input = request.clone();
        input["context"][field] = json!(value);
        input["context"]["energy"] = json!("playful");
        let envelope = compile(input, config.clone(), None).unwrap();
        assert_eq!(envelope.data.plan.humor.mode, HumorMode::Optional);
        assert_eq!(envelope.system, COMPOSITION_INSTRUCTIONS);
    }

    let mut other_voice = request;
    other_voice["voice"] = json!("tony-stark");
    other_voice["context"]["energy"] = json!("playful");
    let envelope = compile(other_voice, config, None).unwrap();
    assert_eq!(envelope.data.plan.humor.mode, HumorMode::Optional);
    assert_eq!(envelope.system, COMPOSITION_INSTRUCTIONS);
}

#[test]
fn slack_channel_traits_reach_the_composer_at_the_resolved_intensity() {
    let mut config = default_config();
    for (channel, voice, intensity) in [
        ("quiet-queue", "chandler", 0.15),
        ("plain-queue", "chandler", 0.0),
        ("team-chat", "witty-engineer", 0.75),
    ] {
        config.routing.rules.push(
            parse(json!({
                "name": channel, "priority": 110,
                "when": {"platform": "slack", "channel": channel},
                "voice": voice, "overrides": {"humor": intensity}
            }))
            .unwrap(),
        );
    }
    let config = serde_json::to_value(config).unwrap();

    for (channel, voice, intensity, mode) in [
        ("quiet-queue", "chandler", 0.15, HumorMode::Optional),
        ("plain-queue", "chandler", 0.0, HumorMode::Off),
        ("team-chat", "witty-engineer", 0.75, HumorMode::Optional),
    ] {
        let envelope = compile(
            json!({
                "intent": "Reply with the updated demo time.",
                "facts": ["The demo moved to 3 PM."],
                "context": {
                    "platform": "slack", "channel": channel, "task": "reply",
                    "audience": "teammate", "seriousness": "ordinary", "energy": "playful"
                }
            }),
            config.clone(),
            None,
        )
        .unwrap();
        assert_eq!(envelope.data.plan.route.voices, [voice]);
        assert_eq!(envelope.data.plan.route.traits.humor, intensity);
        assert_eq!(envelope.data.plan.humor.mode, mode);
        assert_eq!(envelope.system, COMPOSITION_INSTRUCTIONS);
        let prompt = render_prompt(&envelope).unwrap();
        assert!(prompt.contains(&format!(r#""humor": {intensity:?}"#)));
    }
}
