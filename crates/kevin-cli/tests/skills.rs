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
