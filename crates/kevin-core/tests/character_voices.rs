use kevin_core::*;
use serde_json::{Value, json};

fn cases() -> Vec<Value> {
    serde_json::from_str::<Value>(include_str!("../../../evals/character-voices.json"))
        .unwrap()["cases"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn portable_character_cards_match_runtime_definitions_and_reach_the_prompt() {
    let config = default_config();
    let cards = include_str!("../../../skills/kevin/references/voices.md");
    for (voice, other) in [("chandler", "tony-stark"), ("tony-stark", "chandler")] {
        let definition = &config.voices[voice];
        let card = cards
            .split(&format!("### {voice}\n"))
            .nth(1)
            .unwrap()
            .split("\n### ")
            .next()
            .unwrap();
        let request: Request =
            parse(json!({"intent":"Reply to the update.", "voice":voice})).unwrap();
        let envelope = compile_typed(&request, &config, &Memory::default()).unwrap();
        assert_eq!(envelope.data.plan.route.voices, [voice]);
        assert_eq!(envelope.data.plan.humor.mode, HumorMode::Optional);
        assert_eq!(envelope.data.plan.route.traits.sarcasm, 0.0);
        for text in std::iter::once(&definition.description)
            .chain(&definition.behaviors)
            .chain(&definition.avoid)
        {
            assert!(card.contains(text), "{voice} card missing: {text}");
            let prompt_data: Value = serde_json::from_str(
                render_prompt(&envelope)
                    .unwrap()
                    .split_once("INPUT DATA (JSON):\n")
                    .unwrap()
                    .1,
            )
            .unwrap();
            assert!(
                prompt_data["plan"]["route"]
                    .to_string()
                    .contains(&serde_json::to_string(text).unwrap())
            );
        }
        assert!(
            !envelope
                .data
                .plan
                .route
                .descriptions
                .contains(&config.voices[other].description)
        );
    }
}

#[test]
fn skill_and_runtime_allow_contextual_closing_remarks_without_requiring_a_joke() {
    let skill = include_str!("../../../skills/kevin/SKILL.md");
    for instructions in [COMPOSITION_INSTRUCTIONS, skill] {
        assert!(instructions.contains("not permission to ignore the selected personality"));
        assert!(instructions.contains("closing sentence or aside"));
        assert!(instructions.contains("They can work with sarcasm at zero"));
        assert!(instructions.contains("existingHumor"));
        assert!(!instructions.contains("Skip detachable jokes"));
        assert!(!instructions.contains("Skip punchlines, asides"));
        assert!(!instructions.contains("closing quips"));
        assert!(!instructions.contains("already carries a necessary fact"));
    }
}

#[test]
fn synthetic_fixtures_deliver_history_and_enforce_protected_attribution_and_caveat_strings() {
    for case in cases() {
        for (voice, draft) in case["drafts"].as_object().unwrap() {
            let mut input = case["request"].clone();
            input["voice"] = json!(voice);
            let request: Request = parse(input).unwrap();
            let envelope = compile_typed(&request, &default_config(), &Memory::default()).unwrap();
            assert_eq!(envelope.data.request, request);
            assert_eq!(request.context.existing_humor, None);
            assert!(!envelope.data.plan.context.existing_humor);
            assert_eq!(
                envelope.data.plan.humor.mode,
                HumorMode::Optional,
                "{}",
                case["id"]
            );
            assert_eq!(envelope.data.plan.route.traits.sarcasm, 0.0);
            assert!(
                envelope
                    .data
                    .plan
                    .humor
                    .styles
                    .contains(&"brief personification".into())
            );
            let prompt = render_prompt(&envelope).unwrap();
            let delivered: Value =
                serde_json::from_str(prompt.split_once("INPUT DATA (JSON):\n").unwrap().1).unwrap();
            assert_eq!(delivered["request"]["messages"], json!(request.messages));
            assert_eq!(delivered["request"]["intent"], request.intent);
            assert!(
                envelope
                    .system
                    .contains("never adopt another participant's contribution as your own")
            );
            let fixture = |_: &Envelope| Ok(json!({"status":"draft", "text":draft}));
            let output = compose(&envelope, &fixture).unwrap();
            assert_eq!(output.text(), draft.as_str().unwrap());
            for protected in &request.preserve {
                let missing = draft.as_str().unwrap().replace(protected, "");
                assert!(
                    compose(&envelope, &|_: &Envelope| Ok(
                        json!({"status":"draft", "text":missing})
                    ))
                    .is_err()
                );
            }
        }
    }
}

#[test]
fn playful_history_does_not_override_explicit_limits() {
    let callback = cases()
        .into_iter()
        .find(|case| case["id"] == "cardboard-callback")
        .unwrap();
    for voice in ["chandler", "tony-stark"] {
        for (field, value) in [
            ("humor", "off"),
            ("seriousness", "critical"),
            ("seriousness", "sensitive"),
        ] {
            let mut request = callback["request"].clone();
            request["voice"] = json!(voice);
            if field == "humor" {
                request[field] = json!(value);
            } else {
                request["context"][field] = json!(value);
            }
            let envelope = compile(
                request,
                serde_json::to_value(default_config()).unwrap(),
                None,
            )
            .unwrap();
            assert_eq!(envelope.data.plan.route.voices, [voice]);
            assert_eq!(envelope.data.plan.humor.mode, HumorMode::Off);
            assert_eq!(envelope.data.plan.route.traits.humor, 0.0);
            assert_eq!(envelope.data.plan.route.traits.sarcasm, 0.0);
        }
    }
}
