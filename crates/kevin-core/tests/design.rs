mod common;
use kevin_core::*;
use serde_json::json;
fn answers() -> serde_json::Value {
    json!({"context":"slack","stance":"direct","humor":"dry","detail":"short","example":"a","avoid":["Generic praise"]})
}
#[test]
fn interview_asks_one_gap_at_a_time_with_meaningful_recommendations() {
    let DesignResult::Question {
        description,
        question,
        ..
    } = design_voice("Confident, concise, dry humor.", json!({})).unwrap()
    else {
        panic!("Expected question")
    };
    assert_eq!(question.id, "context");
    assert!(
        question
            .options
            .iter()
            .any(|o| o.id == question.recommended)
    );
    let DesignResult::Question { question, .. } =
        design_voice(&description, json!({"context":"review"})).unwrap()
    else {
        panic!("Expected question")
    };
    assert_eq!(question.id, "stance");
}
#[test]
fn example_choices_change_with_context_and_preserve_synthetic_facts() {
    for context in ["slack", "review", "email"] {
        let mut a = answers();
        a["context"] = json!(context);
        a.as_object_mut().unwrap().remove("example");
        let DesignResult::Question { question, .. } =
            design_voice("Technical colleague.", a).unwrap()
        else {
            panic!("Expected question")
        };
        assert_eq!(question.id, "example");
        assert!(question.evidence.unwrap().starts_with("Synthetic"));
        if context == "review" {
            assert!(
                question
                    .options
                    .iter()
                    .all(|o| o.label.contains("items[0].id"))
            );
        }
    }
}
#[test]
fn complete_choices_produce_voice_proposal_previews_no_identity_mutation() {
    let c = default_config();
    let before = c.clone();
    let DesignResult::Proposal {
        voice, previews, ..
    } = design_voice("Confident, concise, dry humor, no arrogance.", answers()).unwrap()
    else {
        panic!("Expected proposal")
    };
    assert_eq!(voice.extends.as_deref(), Some("minimalist"));
    assert_eq!(voice.traits.humor, Some(0.35));
    assert_eq!(previews.len(), 2);
    assert!(previews.iter().all(|p| p.kind == "illustrative-fixture"));
    assert_eq!(c, before);
    let mut c = c;
    c.voices.insert("custom".into(), voice.clone());
    let r: Request = parse(json!({"intent":"Report an incident.","voice":"custom"})).unwrap();
    let e = compile_typed(&r, &c, &Memory::default()).unwrap();
    assert_eq!(e.data.plan.humor.mode, HumorMode::Off);
    assert!(e.data.plan.route.descriptions.contains(&voice.description));
}
#[test]
fn invalid_choices_and_description_fail_before_voice_exists() {
    assert!(design_voice("", answers()).is_err());
    let mut a = answers();
    a["humor"] = json!("always");
    assert!(design_voice("A voice.", a).is_err());
    let mut a = answers();
    a["autoSend"] = json!(true);
    assert!(design_voice("A voice.", a).is_err());
}
