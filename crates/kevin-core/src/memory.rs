use crate::validation::{check, js_trim, string_length};
use crate::{Result, Validate, Version1};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryKind {
    Preference,
    Note,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MemorySource {
    User,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MemoryEntry {
    pub id: String,
    pub kind: MemoryKind,
    pub text: String,
    pub created_at: String,
    pub source: MemorySource,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    pub version: Version1,
    #[serde(default)]
    pub global: Vec<MemoryEntry>,
    #[serde(default)]
    pub voices: IndexMap<String, Vec<MemoryEntry>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceMemory {
    pub voice: String,
    pub entries: Vec<MemoryEntry>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectedMemory {
    pub global: Vec<MemoryEntry>,
    pub voices: Vec<VoiceMemory>,
}

pub fn select_memory(memory: &Memory, selected: &[String]) -> SelectedMemory {
    let mut seen = HashSet::new();
    SelectedMemory {
        global: memory.global.clone(),
        voices: selected
            .iter()
            .filter(|id| seen.insert(id.as_str()))
            .filter_map(|id| {
                memory
                    .voices
                    .get(id)
                    .filter(|items| !items.is_empty())
                    .map(|items| VoiceMemory {
                        voice: id.clone(),
                        entries: items.clone(),
                    })
            })
            .collect(),
    }
}
impl Validate for MemoryEntry {
    fn normalize(&mut self) {
        self.text = js_trim(&self.text).into();
    }
    fn validate(&self) -> Result<()> {
        string_length(&self.id, 1, 100, "memory id")?;
        string_length(&self.text, 1, 2000, "memory text")?;
        check(
            {
                static ISO: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
                    regex::Regex::new(
                        r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-5][0-9](?:\.[0-9]+)?Z$",
                    )
                    .expect("constant timestamp regex")
                });
                ISO.is_match(&self.created_at)
                    && chrono::DateTime::parse_from_rfc3339(&self.created_at).is_ok()
            },
            "Invalid ISO UTC timestamp",
        )
    }
}
impl Validate for Memory {
    fn normalize(&mut self) {
        for entry in self
            .global
            .iter_mut()
            .chain(self.voices.values_mut().flatten())
        {
            entry.normalize();
        }
    }
    fn validate(&self) -> Result<()> {
        for name in self.voices.keys() {
            check(crate::valid_voice_id(name), "Invalid memory voice name")?;
        }
        for items in std::iter::once(&self.global).chain(self.voices.values()) {
            check(items.len() <= 100, "Too many memory entries in scope")?;
            let mut ids = HashSet::new();
            for entry in items {
                entry.validate()?;
                check(ids.insert(&entry.id), "Duplicate memory ID in scope")?;
            }
        }
        Ok(())
    }
}
