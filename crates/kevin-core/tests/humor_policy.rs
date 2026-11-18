use kevin_core::*;
use serde_json::{Value, json};

fn plan(request: Value, config: &Config) -> Plan {
    compile_typed(&parse(request).unwrap(), config, &Memory::default())
        .unwrap()
        .data
        .plan
}

#[test]
fn manual_routed_and_fallback_voices_share_limits_across_tasks_and_missing_context() {
    for selection in ["manual", "rule", "fallback"] {
        let mut config = default_config();
        config.routing.fallback = "chandler".into();
        config.routing.rules.clear();
        if selection == "rule" {
            config.routing.rules.push(
                parse(json!({
                    "name": "team", "priority": 20, "when": {"platform": "slack"},
                    "voice": "chandler"
                }))
                .unwrap(),
            );
        }
        for platform in ["slack", "github", "unknown"] {
            for task in ["reply", "review", "pr-description"] {
                for context in [
                    json!({}),
                    json!({"seriousness": "ordinary", "audience": "teammate", "energy": "calm"}),
                    json!({"seriousness": "unknown", "audience": "unknown", "existingHumor": true}),
                ] {
                    let mut request =
                        json!({"intent": "Summarize the supplied result.", "context": context});
                    request["context"]["platform"] = json!(platform);
                    request["context"]["task"] = json!(task);
                    if selection == "manual" {
                        request["voice"] = json!("chandler");
                    }
                    let result = plan(request.clone(), &config);
                    assert_eq!(result.route.voices, ["chandler"]);
                    assert_eq!(
                        result.humor.mode,
                        HumorMode::Optional,
                        "{selection}: {request}"
                    );
                    assert_eq!(result.route.traits.humor, 0.5);
                    assert_eq!(result.route.traits.sarcasm, 0.0);
                    let parsed: Request = parse(request).unwrap();
                    let analysis = analyze_context(&parsed, &config);
                    let mut route = route_voice(&parsed, &config, &analysis).unwrap();
                    assert_eq!(
                        constrain_humor(&parsed, &analysis.context, &mut route),
                        result.humor
                    );
                    assert_eq!(route, result.route);
                }
            }
        }
    }
}

#[test]
fn review_priority_and_manual_selection_remain_distinct_from_humor_permission() {
    let mut config = default_config();
    config.routing.rules.push(
        parse(json!({
            "name": "slack", "priority": 20, "when": {"platform": "slack"}, "voice": "chandler"
        }))
        .unwrap(),
    );
    let mut request = json!({"intent": "Report the review result.", "context": {"platform": "slack", "task": "review"}});
    let result = plan(request.clone(), &config);
    assert_eq!(result.route.voices, ["professional-reviewer"]);
    assert_eq!(result.humor.mode, HumorMode::Off);
    assert_eq!(
        result.humor.reason,
        "Selected identity or voice disables humor."
    );
    request["voice"] = json!("chandler");
    assert_eq!(
        plan(request.clone(), &config).humor.mode,
        HumorMode::Optional
    );
    request.as_object_mut().unwrap().remove("voice");
    config.routing.rules.last_mut().unwrap().priority = 80;
    assert_eq!(plan(request, &config).humor.mode, HumorMode::Optional);
}

#[test]
fn ordinary_approval_and_unverified_claims_keep_uncertainty_and_exact_evidence() {
    for intent in [
        "Request approval for this PR.",
        "Peça aprovação para esta PR.",
    ] {
        let request = json!({
            "intent": intent, "voice": "chandler",
            "facts": ["No required changes remain.", "Low: an optional test improvement remains.",
                "The author reports the preview works; it was not reproduced.",
                "Deployed configuration was not verified."],
            "preserve": ["Low", "not reproduced", "not verified"],
            "context": {"platform": "slack", "task": "review"}
        });
        let parsed: Request = parse(request).unwrap();
        let envelope = compile_typed(&parsed, &default_config(), &Memory::default()).unwrap();
        assert_eq!(envelope.data.request, parsed);
        assert_eq!(envelope.data.plan.context.seriousness, Seriousness::Unknown);
        assert_eq!(envelope.data.plan.context.audience, Audience::Unknown);
        assert_eq!(envelope.data.plan.confidence, "low");
        assert_eq!(envelope.data.plan.humor.mode, HumorMode::Optional);
        let prompt = render_prompt(&envelope).unwrap();
        for fact in &parsed.facts {
            assert!(prompt.contains(fact));
        }
        assert!(prompt.contains("Permitted humor is optional unless"));
        assert!(
            compose(&envelope, &|_: &Envelope| Ok(
                json!({"status":"draft", "text":"Low: looks good."})
            ))
            .is_err()
        );
    }
}

#[test]
fn sensitive_content_clamps_every_voice_even_with_high_priority_overrides_and_blends() {
    for voice in ["chandler", "tony-stark", "dry-comedian"] {
        let mut config = default_config();
        config.routing.rules.push(
            parse(json!({
                "name": "custom", "priority": 1000, "when": {}, "voice": voice,
                "overrides": {"humor": 1, "sarcasm": 1}
            }))
            .unwrap(),
        );
        for message in [
            "Work is blocked pending approval.",
            "Trabalho bloqueado.",
            "Possible security vulnerability.",
            "Incidente com perda de dados.",
            "She is grieving a death in the family.",
            "Diagnóstico médico e luto.",
            "A serious correction is needed.",
            "Harassment report.",
        ] {
            for manual in [false, true] {
                let mut request = json!({"intent": "Respond to the update.", "messages": [message],
                    "blend": ["witty-engineer"], "context": {"platform": "slack", "task": "review",
                    "seriousness": "ordinary", "audience": "teammate", "existingHumor": true}});
                if manual {
                    request["voice"] = json!(voice);
                }
                let result = plan(request, &config);
                assert_eq!(result.route.voices[0], voice);
                assert_eq!(result.humor.mode, HumorMode::Off, "{voice}: {message}");
                assert_eq!(result.route.traits.humor, 0.0);
                assert_eq!(result.route.traits.sarcasm, 0.0);
            }
        }
    }
}

#[test]
fn configured_floor_applies_to_matching_routes_and_preserves_explicit_limits() {
    let mut config = default_config();
    config.routing.rules.push(
        parse(json!({
            "name": "team", "priority": 110,
            "when": {"platform":"slack", "channel":"C_EXAMPLE"},
            "voice": "chandler", "humorPolicy": {"minimum":0.6, "exceptions":"personal"}
        }))
        .unwrap(),
    );
    let request = json!({
        "intent":"Share the operational update.",
        "messages":["Security incident blocked staging."],
        "context":{"platform":"slack","channel":"C_EXAMPLE","audience":"teammate","energy":"tense"}
    });
    let result = plan(request.clone(), &config);
    assert_eq!(result.humor.mode, HumorMode::Required);
    assert_eq!(result.humor.policy.unwrap().minimum, 0.6);
    assert_eq!(result.route.traits.humor, 0.6);
    assert_eq!(result.route.traits.sarcasm, 0.0);
    for (key, value) in [
        ("seriousness", "sensitive"),
        ("audience", "external"),
        ("channel", "C_OTHER"),
        ("platform", "github"),
    ] {
        let mut variant = request.clone();
        variant["context"][key] = json!(value);
        assert_eq!(plan(variant, &config).humor.mode, HumorMode::Off, "{key}");
    }
    let mut disabled = request.clone();
    disabled["humor"] = json!("off");
    assert_eq!(plan(disabled, &config).humor.mode, HumorMode::Off);
    let mut manual = request.clone();
    manual["voice"] = json!("chandler");
    assert_eq!(plan(manual, &config).humor.mode, HumorMode::Off);

    config
        .routing
        .rules
        .last_mut()
        .unwrap()
        .humor_policy
        .as_mut()
        .unwrap()
        .exceptions = HumorExceptions::Standard;
    assert_eq!(plan(request.clone(), &config).humor.mode, HumorMode::Off);
    let mut routine = request;
    routine["messages"] = json!(["Build passed."]);
    routine["context"]["energy"] = json!("calm");
    assert_eq!(plan(routine, &config).humor.mode, HumorMode::Required);
}
