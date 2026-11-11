use kevin_core::Result;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{ErrorKind, Write},
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
const MANIFEST: &str = ".kevin-install.json";
const UPDATE_LOCK: &str = ".kevin-update.lock";
// v0.2.0, v0.2.1, and db6f19a shipped these bytes without an installation manifest.
const LEGACY_HASHES: &str = include_str!("../../../assets/legacy-skill-hashes.json");

type Hashes = BTreeMap<String, String>;
type LegacyHashes = BTreeMap<String, Vec<String>>;

fn skill_path(host: &str, destination: Option<&Path>) -> Result<PathBuf> {
    let default = match host {
        "codex" => ".agents/skills",
        "claude" => ".claude/skills",
        "cursor" => ".cursor/skills",
        "generic" => "skills",
        _ => return Err("Choose codex, claude, cursor, or generic.".into()),
    };
    crate::storage::absolute(&destination.unwrap_or(Path::new(default)).join("kevin"))
}

fn metadata(path: &Path) -> Result<Option<fs::Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Cannot inspect {}: {error}", path.display())),
    }
}

fn directory_exists(path: &Path) -> Result<bool> {
    match metadata(path)? {
        Some(metadata) if metadata.is_dir() => Ok(true),
        Some(_) => Err(format!("Expected a directory at {}.", path.display())),
        None => Ok(false),
    }
}

fn file_metadata(path: &Path) -> Result<Option<fs::Metadata>> {
    match metadata(path)? {
        Some(metadata) if !metadata.is_file() => {
            Err(format!("Expected a regular file at {}.", path.display()))
        }
        metadata => Ok(metadata),
    }
}

fn read_file(path: &Path) -> Result<Option<Vec<u8>>> {
    file_metadata(path)?
        .map(|_| fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display())))
        .transpose()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn bundled_hashes() -> Hashes {
    FILES
        .iter()
        .map(|(name, text)| (name.to_string(), digest(text.as_bytes())))
        .collect()
}

pub fn install(host: &str, destination: Option<&Path>) -> Result<()> {
    let path = skill_path(host, destination)?;
    if directory_exists(&path)? {
        directory_exists(&path.join("references"))?;
        for (name, text) in FILES {
            if read_file(&path.join(name))?.as_deref() != Some(text.as_bytes()) {
                return Err(format!(
                    "Skill differs at {name}. Preview changes with kevin skill update {host} --dry-run."
                ));
            }
        }
        println!("Already installed {}", path.display());
        return Ok(());
    }
    let parent = path.parent().ok_or("Invalid skill destination")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let staged = tempfile::tempdir_in(parent).map_err(|e| e.to_string())?;
    fs::create_dir(staged.path().join("references")).map_err(|e| e.to_string())?;
    for (name, text) in FILES {
        fs::write(staged.path().join(name), text).map_err(|e| e.to_string())?;
    }
    let manifest = serde_json::to_vec_pretty(&bundled_hashes()).map_err(|e| e.to_string())?;
    fs::write(staged.path().join(MANIFEST), manifest).map_err(|e| e.to_string())?;
    if directory_exists(&path)? {
        return Err("Skill destination appeared during installation. Retry the command.".into());
    }
    fs::rename(staged.path(), &path)
        .map_err(|e| format!("Cannot install {}: {e}", path.display()))?;
    println!("Installed {}", path.display());
    Ok(())
}

struct Replacement {
    name: &'static str,
    text: &'static str,
    previous: Option<String>,
}

pub fn update(host: &str, destination: Option<&Path>, dry_run: bool) -> Result<()> {
    let path = skill_path(host, destination)?;
    if !directory_exists(&path)? {
        return Err(format!(
            "Skill is missing at {}. Run kevin skill install {host} first.",
            path.display()
        ));
    }
    directory_exists(&path.join("references"))?;
    let _lock = if dry_run {
        None
    } else {
        let lock_path = path.join(UPDATE_LOCK);
        file_metadata(&lock_path)?;
        let lock = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| format!("Cannot open {}: {e}", lock_path.display()))?;
        lock.try_lock()
            .map_err(|e| format!("Cannot lock skill update: {e}"))?;
        Some(lock)
    };
    let manifest_path = path.join(MANIFEST);
    let previous = read_file(&manifest_path)?
        .map(|bytes| serde_json::from_slice::<Hashes>(&bytes))
        .transpose()
        .map_err(|e| format!("Invalid skill installation manifest: {e}"))?;
    let legacy: LegacyHashes = serde_json::from_str(LEGACY_HASHES).expect("bundled skill hashes");
    let desired = bundled_hashes();
    let mut writes = Vec::new();
    let mut conflicts = false;
    for &(name, text) in FILES {
        let current = read_file(&path.join(name))?.map(|bytes| digest(&bytes));
        if current.as_ref() == desired.get(name) {
            continue;
        }
        let safe = match &previous {
            Some(hashes) => current.as_ref() == hashes.get(name),
            None => match (&current, legacy.get(name)) {
                (Some(current), Some(known)) => known.contains(current),
                (None, None) => true,
                _ => false,
            },
        };
        if safe {
            if dry_run {
                let action = if current.is_some() { "Replace" } else { "Add" };
                println!("{action} {name}");
            }
            writes.push(Replacement {
                name,
                text,
                previous: current,
            });
        } else {
            eprintln!("Local change or unknown version: {name}");
            conflicts = true;
        }
    }
    if conflicts {
        return Err("Skill update blocked. Review the listed files before trying again.".into());
    }
    if dry_run {
        if writes.is_empty() {
            println!("Skill is up to date.");
        }
        return Ok(());
    }
    for change in &writes {
        let target = path.join(change.name);
        let current = read_file(&target)?.map(|bytes| digest(&bytes));
        if current != change.previous {
            return Err(format!(
                "{} changed during the update. Retry after reviewing it.",
                change.name
            ));
        }
        write_file(&target, change.text.as_bytes())?;
    }
    if previous.as_ref() != Some(&desired) {
        let bytes = serde_json::to_vec_pretty(&desired).map_err(|e| e.to_string())?;
        write_file(&manifest_path, &bytes)?;
    }
    if writes.is_empty() {
        println!("Skill is up to date.");
    } else {
        println!("Updated skill at {}", path.display());
    }
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("Invalid skill file path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let existing = file_metadata(path)?;
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    staged.write_all(bytes).map_err(|e| e.to_string())?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    if let Some(metadata) = existing {
        staged
            .as_file()
            .set_permissions(metadata.permissions())
            .map_err(|e| e.to_string())?;
        staged.persist(path)
    } else {
        staged.persist_noclobber(path)
    }
    .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
    Ok(())
}
