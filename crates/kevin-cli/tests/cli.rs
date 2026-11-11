mod common;
use common::*;
use serde_json::json;
use std::fs;

#[test]
fn cli_init_routing_doctor_fallback_profile_edits_skill_install_and_generation() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    assert!(
        !run(cwd, &["doctor", "--config", "missing.json"], "")
            .status
            .success()
    );
    ok(cwd, &["init", "--language", "pt-BR"], "");
    assert!(!run(cwd, &["init"], "").status.success());
    assert_eq!(value(cwd, &["doctor"], "")["voices"], 12);
    fs::write(
        cwd.join("identity.json"),
        ok(cwd, &["identity", "export"], ""),
    )
    .unwrap();
    ok(cwd, &["identity", "import", "identity.json"], "");
    ok(
        cwd,
        &["voice", "create", "custom"],
        r#"{"description":"Custom","extends":"minimalist","traits":{"warmth":0.8}}"#,
    );
    assert!(
        !run(
            cwd,
            &["voice", "create", "../escape"],
            r#"{"description":"bad"}"#
        )
        .status
        .success()
    );
    ok(
        cwd,
        &["voice", "edit", "custom"],
        r#"{"description":"Changed"}"#,
    );
    let input = json!({"intent":"Acknowledge the coffee joke.","context":{"platform":"slack","audience":"teammate","seriousness":"ordinary","energy":"playful","existingHumor":true},"messages":["Next you will automate my coffee."]});
    fs::write(cwd.join("request.json"), input.to_string()).unwrap();
    let e = value(cwd, &["reply", "--request", "request.json", "--json"], "");
    assert_eq!(e["data"]["plan"]["route"]["voices"][0], "tony-stark");
    assert_eq!(e["data"]["plan"]["context"]["language"], "pt-BR");
    let prompt = run(cwd, &["reply"], "Hello.");
    assert!(prompt.status.success());
    assert!(String::from_utf8_lossy(&prompt.stderr).contains("prompt, not a generated reply"));
    assert!(String::from_utf8_lossy(&prompt.stdout).contains("INPUT DATA"));
    assert!(
        !run(cwd, &["reply", "--platform", "invalid"], "Hi")
            .status
            .success()
    );
    assert!(
        !run(cwd, &["reply", "--voice", "missing"], "Hi")
            .status
            .success()
    );
    assert_eq!(
        ok(
            cwd,
            &[
                "reply",
                "--request",
                "request.json",
                "--model-command",
                fixture().to_str().unwrap()
            ],
            ""
        )
        .trim(),
        "Coffee still needs a human in the loop."
    );
    ok(cwd, &["skill", "install", "codex"], "");
    assert!(
        fs::read_to_string(cwd.join(".agents/skills/kevin/SKILL.md"))
            .unwrap()
            .contains("name: kevin")
    );
    assert!(
        fs::read_to_string(cwd.join(".agents/skills/kevin/references/identity.md"))
            .unwrap()
            .contains("Calibration workflow")
    );
    assert!(
        !run(cwd, &["skill", "install", "codex"], "")
            .status
            .success()
    );
}
#[test]
fn composition_preserves_piped_message_with_explicit_intent() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let input = "  Can you check TEST-44?\n  It failed twice.\n";
    let intent = "Say I'll take a look after lunch.";

    let reply = value(cwd, &["reply", "--intent", intent, "--json"], input);
    assert_eq!(reply["data"]["request"]["intent"], intent);
    assert_eq!(reply["data"]["request"]["messages"], json!([input]));

    let rewrite = value(cwd, &["rewrite", "--intent", intent, "--json"], input);
    assert_eq!(rewrite["data"]["request"]["draft"], input.trim());
    assert_eq!(rewrite["data"]["request"]["messages"], json!([]));

    let implicit_intent = value(cwd, &["reply", "--json"], input);
    assert_eq!(implicit_intent["data"]["request"]["intent"], input.trim());
    assert_eq!(implicit_intent["data"]["request"]["messages"], json!([]));
}
#[test]
fn provider_rejects_invalid_json_failure_oversized_output_and_missing_executable() {
    let dir = init();
    for mode in ["invalid", "failure", "huge"] {
        assert!(
            !run(
                dir.path(),
                &[
                    "reply",
                    "--intent",
                    "Reply.",
                    "--model-command",
                    fixture().to_str().unwrap(),
                    "--model-arg",
                    mode
                ],
                ""
            )
            .status
            .success()
        );
    }
    let out = run(
        dir.path(),
        &[
            "reply",
            "--intent",
            "Reply.",
            "--model-command",
            "kevin-executable-that-does-not-exist",
        ],
        "",
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Cannot start"));
}
#[test]
fn description_driven_creation_previews_first_saves_only_explicitly() {
    let dir = init();
    let cwd = dir.path();
    let profile = cwd.join(".kevin/config.json");
    let before = fs::read(&profile).unwrap();
    assert_eq!(
        value(
            cwd,
            &[
                "voice",
                "create",
                "field-engineer",
                "--description",
                "Dry, concise engineer."
            ],
            ""
        )["question"]["id"],
        "context"
    );
    fs::write(
        cwd.join("answers.json"),
        r#"{"context":"slack","stance":"direct","humor":"dry","detail":"short","example":"a"}"#,
    )
    .unwrap();
    let args = [
        "voice",
        "create",
        "field-engineer",
        "--description",
        "Dry, concise engineer.",
        "--answers",
        "answers.json",
    ];
    assert_eq!(value(cwd, &args, "")["status"], "proposal");
    assert_eq!(fs::read(&profile).unwrap(), before);
    let mut save = args.to_vec();
    save.push("--save");
    ok(cwd, &save, "");
    let current: serde_json::Value = serde_json::from_slice(&fs::read(&profile).unwrap()).unwrap();
    let before: serde_json::Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(
        current["voices"]["field-engineer"]["description"],
        "Dry, concise engineer."
    );
    assert_eq!(current["identity"], before["identity"]);
    assert_eq!(current["routing"], before["routing"]);
    assert!(!run(cwd, &save, "").status.success());
    assert!(
        !run(
            cwd,
            &[
                "voice",
                "create",
                "incomplete",
                "--description",
                "Example.",
                "--save"
            ],
            ""
        )
        .status
        .success()
    );
}
#[test]
fn list_voices_change_default_reject_invalid_defaults_without_profile_changes() {
    let dir = init();
    let cwd = dir.path();
    assert_eq!(ok(cwd, &["voice", "default"], "").trim(), "tony-stark");
    assert!(ok(cwd, &["voices"], "").contains("tony-stark (default)"));
    ok(cwd, &["voice", "default", "minimalist"], "");
    assert_eq!(ok(cwd, &["voice", "default"], "").trim(), "minimalist");
    let list = value(cwd, &["voices", "--json"], "");
    assert_eq!(list["default"], "minimalist");
    assert!(list["voices"].get("dry-comedian").is_some());
    assert_eq!(
        value(cwd, &["reply", "--json"], "Explain.")["data"]["plan"]["route"]["voices"],
        json!(["minimalist"])
    );
    let profile = cwd.join(".kevin/config.json");
    let before = fs::read(&profile).unwrap();
    assert!(
        !run(cwd, &["voice", "default", "missing"], "")
            .status
            .success()
    );
    assert_eq!(fs::read(profile).unwrap(), before);
    assert_eq!(
        value(
            cwd,
            &["reply", "--json", "--voice", "friendly-teammate"],
            "Explain."
        )["data"]["plan"]["route"]["voices"],
        json!(["friendly-teammate"])
    );
}
#[test]
fn model_arguments_remain_literal_private_stderr_not_exposed() {
    let dir = init();
    let literal = "$(touch /tmp/kevin-shell-injection); `echo private`";
    let out = ok(
        dir.path(),
        &[
            "reply",
            "--intent",
            "Reply.",
            "--model-command",
            fixture().to_str().unwrap(),
            "--model-arg",
            "echo",
            "--model-arg",
            literal,
        ],
        "",
    );
    assert_eq!(out.trim(), literal);
    let out = run(
        dir.path(),
        &[
            "reply",
            "--intent",
            "Reply.",
            "--model-command",
            fixture().to_str().unwrap(),
            "--model-arg",
            "failure",
        ],
        "",
    );
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("PRIVATE-TOKEN"));
}
#[test]
fn cli_manages_channel_person_assignments_preserves_profile_on_invalid_rules() {
    let dir = init();
    let cwd = dir.path();
    ok(
        cwd,
        &[
            "rule",
            "set",
            "team",
            "--platform",
            "slack",
            "--workspace",
            "T_EXAMPLE",
            "--channel",
            "C_EXAMPLE",
            "--voice",
            "witty-engineer",
        ],
        "",
    );
    assert!(
        value(cwd, &["rules"], "")
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["name"] == "team")
    );
    assert_eq!(
        value(
            cwd,
            &[
                "reply",
                "--json",
                "--platform",
                "slack",
                "--workspace",
                "T_EXAMPLE",
                "--channel",
                "C_EXAMPLE"
            ],
            "Acknowledge."
        )["data"]["plan"]["route"]["voices"],
        json!(["witty-engineer"])
    );
    ok(
        cwd,
        &[
            "rule",
            "set",
            "person",
            "--recipient",
            "U_EXAMPLE",
            "--voice",
            "minimalist",
            "--priority",
            "85",
        ],
        "",
    );
    assert_eq!(
        value(
            cwd,
            &["reply", "--json", "--recipient", "U_EXAMPLE"],
            "Acknowledge."
        )["data"]["plan"]["route"]["voices"],
        json!(["minimalist"])
    );
    ok(
        cwd,
        &[
            "rule",
            "set",
            "team",
            "--channel",
            "C_EXAMPLE",
            "--voice",
            "friendly-teammate",
        ],
        "",
    );
    let profile = cwd.join(".kevin/config.json");
    let before = fs::read(&profile).unwrap();
    for args in [
        vec![
            "rule",
            "set",
            "bad",
            "--channel",
            "C_EXAMPLE",
            "--voice",
            "missing",
        ],
        vec!["rule", "set", "bad", "--voice", "neutral"],
        vec![
            "rule",
            "set",
            "bad",
            "--channel",
            "C_EXAMPLE",
            "--voice",
            "neutral",
            "--priority",
            "NaN",
        ],
    ] {
        assert!(!run(cwd, &args, "").status.success());
    }
    assert_eq!(fs::read(profile).unwrap(), before);
    ok(cwd, &["rule", "remove", "team"], "");
    assert!(!run(cwd, &["rule", "remove", "team"], "").status.success());
}
#[test]
fn cli_memory_explicit_writes_isolated_scopes_deletion_and_no_implicit_capture() {
    let dir = init();
    let cwd = dir.path();
    ok(cwd, &["reply", "--json"], "Temporary conversation only.");
    assert!(!cwd.join(".kevin/memory.json").exists());
    let global = value(
        cwd,
        &[
            "memory",
            "add",
            "--text",
            "Avoid corporate filler.",
            "--kind",
            "preference",
        ],
        "",
    );
    let gid = global["entry"]["id"].as_str().unwrap();
    let voice = value(
        cwd,
        &[
            "memory",
            "add",
            "--voice",
            "minimalist",
            "--text",
            "Private minimalist note.",
        ],
        "",
    );
    let vid = voice["entry"]["id"].as_str().unwrap();
    let scoped = value(cwd, &["reply", "--voice", "minimalist", "--json"], "Reply.");
    assert_eq!(
        scoped["data"]["memory"]["global"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        scoped["data"]["memory"]["voices"][0]["entries"][0]["id"],
        vid
    );
    assert!(
        !ok(cwd, &["reply", "--voice", "diplomat", "--json"], "Reply.")
            .contains("Private minimalist note.")
    );
    assert_eq!(
        value(cwd, &["memory", "list"], "")["entries"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        value(cwd, &["memory", "list", "--voice", "minimalist"], "")["entries"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        value(cwd, &["memory", "list", "--voice", "diplomat"], "")["entries"],
        json!([])
    );
    let path = cwd.join(".kevin/memory.json");
    let before = fs::read(&path).unwrap();
    assert!(
        !run(
            cwd,
            &["memory", "add", "--text", "No write.", "--voice", "missing"],
            ""
        )
        .status
        .success()
    );
    assert!(!run(cwd, &["memory", "remove", vid], "").status.success());
    assert_eq!(fs::read(&path).unwrap(), before);
    ok(cwd, &["memory", "remove", vid, "--voice", "minimalist"], "");
    ok(cwd, &["memory", "remove", gid], "");
    let all = value(cwd, &["memory", "list", "--all"], "");
    assert_eq!(all["global"], json!([]));
    assert_eq!(all["voices"], json!({}));
    assert!(
        !fs::read_to_string(path)
            .unwrap()
            .contains("Private minimalist note.")
    );
}
#[test]
fn relocated_native_binary_installs_complete_skill_without_checkout_or_node() {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir
        .path()
        .join(format!("kevin{}", std::env::consts::EXE_SUFFIX));
    fs::copy(binary(), &exe).unwrap();
    let out = run_exe(&exe, dir.path(), &["skill", "install", "generic"], "");
    assert!(out.status.success());
    for file in [
        "memory.md",
        "routing.md",
        "identity.md",
        "voices.md",
        "voice-interview.md",
        "communication.md",
    ] {
        assert!(
            dir.path()
                .join("skills/kevin/references")
                .join(file)
                .exists()
        );
    }
}
#[test]
fn evaluation_runs_all_original_scenarios_and_records_optional_fixture_outputs() {
    let dir = tempfile::tempdir().unwrap();
    assert!(ok(dir.path(), &["eval"], "").contains("30 routing scenarios passed"));
    ok(
        dir.path(),
        &[
            "eval",
            "--model-command",
            fixture().to_str().unwrap(),
            "--model-arg",
            "review",
            "--model-id",
            "transport-fixture-not-a-model",
            "--settings",
            "deterministic fixture",
            "--output-dir",
            "records",
        ],
        "",
    );
    let files: Vec<_> = fs::read_dir(dir.path().join("records")).unwrap().collect();
    assert_eq!(files.len(), 1);
    let data: serde_json::Value =
        serde_json::from_slice(&fs::read(files[0].as_ref().unwrap().path()).unwrap()).unwrap();
    assert_eq!(data["records"].as_array().unwrap().len(), 30);
    assert!(
        data["records"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["semanticReview"] == "pending")
    );
}
#[test]
fn profile_permissions_and_malformed_memory_are_preserved() {
    let dir = init();
    let cwd = dir.path();
    fs::write(cwd.join(".kevin/memory.json"), "broken").unwrap();
    assert!(
        !run(cwd, &["reply", "--intent", "Reply.", "--json"], "")
            .status
            .success()
    );
    assert_eq!(
        fs::read_to_string(cwd.join(".kevin/memory.json")).unwrap(),
        "broken"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(cwd.join(".kevin/config.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(cwd.join(".kevin"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
}

#[test]
fn consumer_review_project_keeps_its_voice_for_every_review_including_security() {
    let dir = tempfile::tempdir().unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let profile = root.join("examples/code-review-project/kevin.json");
    let request = root.join("examples/code-review-project/review-input.json");
    for intent in [
        "Write a code review comment.",
        "Write a security review comment.",
    ] {
        let out = value(
            dir.path(),
            &[
                "review",
                "--config",
                profile.to_str().unwrap(),
                "--request",
                request.to_str().unwrap(),
                "--intent",
                intent,
                "--json",
            ],
            "",
        );
        assert_eq!(
            out["data"]["plan"]["route"]["voices"],
            json!(["project-reviewer"])
        );
        assert_eq!(out["data"]["plan"]["humor"]["mode"], "off");
    }
    let out = ok(
        dir.path(),
        &[
            "review",
            "--config",
            profile.to_str().unwrap(),
            "--request",
            request.to_str().unwrap(),
            "--model-command",
            fixture().to_str().unwrap(),
            "--model-arg",
            "review",
        ],
        "",
    );
    assert!(out.contains("items[0].id"));
}

#[test]
fn first_use_has_starter_voices_without_setup_and_explicit_changes_persist() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    let list = value(cwd, &["voices", "--json"], "");
    assert_eq!(list["default"], "tony-stark");
    assert_eq!(list["voices"].as_object().unwrap().len(), 12);
    assert_eq!(list["voices"]["chandler"]["extends"], "friendly-teammate");
    let e = value(cwd, &["reply", "--intent", "Explain this.", "--json"], "");
    assert_eq!(e["data"]["plan"]["route"]["voices"], json!(["tony-stark"]));
    assert_eq!(e["data"]["plan"]["humor"]["mode"], "off");
    assert!(!cwd.join(".kevin").exists());
    ok(cwd, &["voice", "default", "minimalist"], "");
    assert!(cwd.join(".kevin/config.json").exists());
    assert_eq!(ok(cwd, &["voice", "default"], "").trim(), "minimalist");
    assert!(
        !run(
            cwd,
            &[
                "reply",
                "--config",
                "missing.json",
                "--intent",
                "Reply.",
                "--json"
            ],
            ""
        )
        .status
        .success()
    );
}
#[test]
fn existing_v1_profile_keeps_its_default_instead_of_silently_switching_to_stark() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("legacy.json"),
        include_str!("../../../tests/fixtures/default-config-v1.json"),
    )
    .unwrap();
    assert_eq!(
        value(
            dir.path(),
            &[
                "reply",
                "--config",
                "legacy.json",
                "--intent",
                "Reply.",
                "--json"
            ],
            ""
        )["data"]["plan"]["route"]["voices"],
        json!(["neutral"])
    );
}
