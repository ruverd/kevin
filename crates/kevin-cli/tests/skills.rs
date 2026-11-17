mod common;

use common::{ok, run};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn skill_update_previews_changes_and_protects_local_files() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    ok(cwd, &["skill", "install", "codex"], "");
    let skill = cwd.join(".agents/skills/kevin");
    let routing = skill.join("references/routing.md");
    let skill_file = skill.join("SKILL.md");
    let manifest_file = skill.join(".kevin-install.json");
    let bundled_routing = fs::read(&routing).unwrap();
    let bundled_skill = fs::read(&skill_file).unwrap();
    let old_routing = b"Previous bundled routing guide.";
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_file).unwrap()).unwrap();
    manifest["references/routing.md"] = format!("{:x}", Sha256::digest(old_routing)).into();
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    fs::write(&manifest_file, &manifest_bytes).unwrap();
    fs::write(&routing, old_routing).unwrap();
    fs::write(skill.join("custom.md"), "Local reference").unwrap();

    let preview = ok(cwd, &["skill", "update", "codex", "--dry-run"], "");
    assert!(preview.contains("Replace references/routing.md"));
    assert_eq!(fs::read(&routing).unwrap(), old_routing);
    assert_eq!(fs::read(&manifest_file).unwrap(), manifest_bytes);
    assert!(!skill.join(".kevin-update.lock").exists());

    fs::write(&skill_file, "Local skill edits").unwrap();
    let output = run(cwd, &["skill", "update", "codex"], "");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Local change or unknown version: SKILL.md")
    );
    assert_eq!(
        fs::read_to_string(&skill_file).unwrap(),
        "Local skill edits"
    );
    assert_eq!(fs::read(&routing).unwrap(), old_routing);
    assert_eq!(fs::read(&manifest_file).unwrap(), manifest_bytes);

    fs::write(&skill_file, bundled_skill).unwrap();
    let output = ok(cwd, &["skill", "update", "codex"], "");
    assert!(output.contains("Updated skill"));
    assert_eq!(fs::read(&routing).unwrap(), bundled_routing);
    assert_eq!(
        fs::read_to_string(skill.join("custom.md")).unwrap(),
        "Local reference"
    );
    assert!(ok(cwd, &["skill", "update", "codex"], "").contains("Skill is up to date"));
}

#[test]
fn skill_refresh_keeps_saved_character_definitions_and_assignments() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path();
    ok(cwd, &["init"], "");
    let path = cwd.join(".kevin/config.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    config["routing"]["fallback"] = serde_json::json!("chandler");
    for voice in ["chandler", "tony-stark"] {
        config["voices"][voice]["description"] =
            serde_json::json!(format!("Saved {voice} description."));
        config["voices"][voice]["behaviors"] =
            serde_json::json!([format!("Saved {voice} behavior.")]);
    }
    let before = serde_json::to_vec(&config).unwrap();
    fs::write(&path, &before).unwrap();
    ok(cwd, &["skill", "install", "generic"], "");
    ok(cwd, &["skill", "update", "generic"], "");
    assert_eq!(fs::read(&path).unwrap(), before);
    for voice in ["chandler", "tony-stark"] {
        let envelope = common::value(
            cwd,
            &[
                "reply",
                "--voice",
                voice,
                "--intent",
                "Reply to the update.",
                "--json",
            ],
            "",
        );
        let route = &envelope["data"]["plan"]["route"];
        assert_eq!(route["voices"], serde_json::json!([voice]));
        assert!(
            route["descriptions"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!(format!("Saved {voice} description.")))
        );
        assert!(
            route["behaviors"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!(format!("Saved {voice} behavior.")))
        );
    }
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(
        common::value(cwd, &["voices", "--json"], "")["default"],
        "chandler"
    );
}

#[test]
fn structured_history_reaches_cli_composer_in_order_without_inferred_banter() {
    let dir = tempfile::tempdir().unwrap();
    let fixtures: Value =
        serde_json::from_str(include_str!("../../../evals/character-voices.json")).unwrap();
    let case = fixtures["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "cardboard-followup")
        .unwrap();
    let mut request = case["request"].clone();
    request["voice"] = serde_json::json!("chandler");
    let output = common::value(
        dir.path(),
        &["reply", "--request", "-", "--json"],
        &request.to_string(),
    );
    let delivered = &output["data"]["request"];
    assert_eq!(delivered["messages"], request["messages"]);
    assert_eq!(delivered["intent"], request["intent"]);
    assert_eq!(delivered["facts"], request["facts"]);
    assert_eq!(delivered["preserve"], request["preserve"]);
    assert!(delivered["context"].get("existingHumor").is_none());
    assert_eq!(output["data"]["plan"]["context"]["existingHumor"], false);
    assert_eq!(output["data"]["plan"]["humor"]["mode"], "optional");
    assert_eq!(output["data"]["plan"]["route"]["traits"]["sarcasm"], 0.0);
}
