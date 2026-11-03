use kevin_core::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

const FILES: &[(&str, &str)] = &[
    ("SKILL.md", include_str!("../../../skills/kevin/SKILL.md")),
    (
        "references/communication.md",
        include_str!("../../../skills/kevin/references/communication.md"),
    ),
    (
        "references/identity.md",
        include_str!("../../../skills/kevin/references/identity.md"),
    ),
    (
        "references/voices.md",
        include_str!("../../../skills/kevin/references/voices.md"),
    ),
    (
        "references/voice-interview.md",
        include_str!("../../../skills/kevin/references/voice-interview.md"),
    ),
    (
        "references/routing.md",
        include_str!("../../../skills/kevin/references/routing.md"),
    ),
    (
        "references/memory.md",
        include_str!("../../../skills/kevin/references/memory.md"),
    ),
];
pub fn install(host: &str, destination: Option<&Path>) -> Result<PathBuf> {
    let default = match host {
        "codex" => ".agents/skills",
        "claude" => ".claude/skills",
        "cursor" => ".cursor/skills",
        "generic" => "skills",
        _ => return Err("Choose codex, claude, cursor, or generic.".into()),
    };
    let path = crate::storage::absolute(&destination.unwrap_or(Path::new(default)).join("kevin"))?;
    fs::create_dir_all(path.parent().ok_or("Invalid skill destination")?)
        .map_err(|e| e.to_string())?;
    fs::create_dir(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            "Destination already exists; no overwrite performed.".into()
        } else {
            e.to_string()
        }
    })?;
    for (name, text) in FILES {
        let target = path.join(name);
        fs::create_dir_all(target.parent().ok_or("Invalid resource path")?)
            .map_err(|e| e.to_string())?;
        fs::write(target, text).map_err(|e| e.to_string())?;
    }
    Ok(path)
}
