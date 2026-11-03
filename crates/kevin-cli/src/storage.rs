use kevin_core::{Config, Memory, Result, Validate, parse};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub fn absolute(path: &Path) -> Result<PathBuf> {
    std::path::absolute(path).map_err(|e| e.to_string())
}
pub fn read_json<T: DeserializeOwned + Validate>(path: &Path) -> Result<T> {
    parse(read_value(path)?)
}
pub fn read_value(path: &Path) -> Result<serde_json::Value> {
    let input =
        fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&input).map_err(|e| format!("Invalid JSON in {}: {e}", path.display()))
}
pub fn load_config(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Err(format!(
            "No profile at {}. Run kevin init or pass --config.",
            path.display()
        ));
    }
    read_json(path)
}
pub fn load_memory(path: &Path) -> Result<Memory> {
    match fs::read_to_string(path) {
        Ok(text) => {
            parse(serde_json::from_str(&text).map_err(|e| format!("Invalid memory JSON: {e}"))?)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Memory::default()),
        Err(e) => Err(format!("Cannot read memory: {e}")),
    }
}
fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}
pub fn create_private_dir(path: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).map_err(|e| e.to_string())
}
fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}
pub fn create_json<T: Serialize + Validate>(path: &Path, value: &T) -> Result<()> {
    value.validate()?;
    create_private_dir(parent(path))?;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            "Destination already exists; no overwrite performed.".into()
        } else {
            e.to_string()
        }
    })?;
    file.write_all(&bytes(value)?).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())
}
pub fn save_json<T: Serialize + Validate>(path: &Path, value: &T) -> Result<()> {
    value.validate()?;
    create_private_dir(parent(path))?;
    let mut file = tempfile::NamedTempFile::new_in(parent(path)).map_err(|e| e.to_string())?;
    file.write_all(&bytes(value)?).map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn read_stdin() -> Result<String> {
    let mut data = String::new();
    std::io::stdin()
        .take(1_000_001)
        .read_to_string(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 1_000_000 {
        return Err("Input exceeded 1 MB.".into());
    }
    Ok(data)
}
pub fn print<T: Serialize>(value: &T) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())?
    );
    Ok(())
}
