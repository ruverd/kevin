use crate::*;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashSet;

pub trait Validate {
    fn validate(&self) -> Result<()>;
    fn normalize(&mut self) {}
}
pub fn parse<T: DeserializeOwned + Validate>(value: Value) -> Result<T> {
    let mut parsed: T =
        serde_json::from_value(value).map_err(|error| format!("Invalid data: {error}"))?;
    parsed.normalize();
    parsed.validate()?;
    Ok(parsed)
}
pub fn parse_config(value: Value) -> Result<Config> {
    parse(value)
}
pub fn default_config() -> Config {
    parse_config(
        serde_json::from_str(include_str!("../../../assets/default-config.json"))
            .expect("embedded JSON"),
    )
    .expect("embedded default configuration")
}
pub(crate) fn check(valid: bool, message: impl Into<String>) -> Result<()> {
    if valid { Ok(()) } else { Err(message.into()) }
}
pub(crate) fn string_length(value: &str, min: usize, max: usize, field: &str) -> Result<()> {
    // The v1 JavaScript contract measures strings in UTF-16 code units.
    check(
        (min..=max).contains(&value.encode_utf16().count()),
        format!("{field} must contain {min}..={max} UTF-16 units"),
    )
}
pub fn valid_voice_id(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn voice_id(value: &str) -> Result<()> {
    check(
        valid_voice_id(value),
        format!("Invalid voice name: {value}"),
    )
}
fn unit(value: f64) -> Result<()> {
    check(
        value.is_finite() && (0.0..=1.0).contains(&value),
        "Trait must be a finite number in [0, 1]",
    )
}
fn strings(values: &[String], count: usize, min: usize, max: usize) -> Result<()> {
    check(values.len() <= count, "Too many entries")?;
    for value in values {
        string_length(value, min, max, "text")?;
    }
    Ok(())
}
pub(crate) fn js_trim(value: &str) -> &str {
    value.trim_matches(|c| matches!(c, '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'))
}
impl Validate for Traits {
    fn validate(&self) -> Result<()> {
        for n in [
            self.directness,
            self.formality,
            self.warmth,
            self.humor,
            self.sarcasm,
            self.verbosity,
        ] {
            unit(n)?;
        }
        Ok(())
    }
}
impl Validate for PartialTraits {
    fn validate(&self) -> Result<()> {
        for n in [
            self.directness,
            self.formality,
            self.warmth,
            self.humor,
            self.sarcasm,
            self.verbosity,
        ]
        .into_iter()
        .flatten()
        {
            unit(n)?;
        }
        Ok(())
    }
}
impl Validate for Voice {
    fn validate(&self) -> Result<()> {
        string_length(&self.description, 0, 20_000, "description")?;
        if let Some(parent) = &self.extends {
            voice_id(parent)?;
        }
        self.traits.validate()?;
        strings(&self.behaviors, 30, 0, 20_000)?;
        strings(&self.avoid, 30, 0, 20_000)
    }
}
impl Validate for Identity {
    fn validate(&self) -> Result<()> {
        string_length(&self.name, 0, 100, "identity.name")?;
        string_length(&self.primary_language, 2, 35, "identity.primaryLanguage")?;
        self.traits.validate()?;
        strings(&self.preferences, 100, 0, 20_000)?;
        strings(&self.avoid, 100, 0, 20_000)?;
        for language in self.languages.values() {
            language.traits.validate()?;
            strings(&language.preferences, usize::MAX, 0, 20_000)?;
        }
        Ok(())
    }
}
impl Validate for ContextInput {
    fn validate(&self) -> Result<()> {
        if let Some(language) = &self.language {
            string_length(language, 2, 35, "language")?;
        }
        for value in [&self.workspace, &self.channel, &self.recipient]
            .into_iter()
            .flatten()
        {
            string_length(value, 1, 200, "context identifier")?;
        }
        if let Some(value) = self.formality {
            unit(value)?;
        }
        Ok(())
    }
}
impl Validate for Conditions {
    fn validate(&self) -> Result<()> {
        let input: ContextInput =
            serde_json::from_value(serde_json::to_value(self).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        input.validate()
    }
}
impl Validate for Rule {
    fn validate(&self) -> Result<()> {
        string_length(&self.name, 1, 100, "rule name")?;
        check(
            self.priority.unsigned_abs() <= 9_007_199_254_740_991,
            "Rule priority exceeds safe integer range",
        )?;
        voice_id(&self.voice)?;
        self.when.validate()?;
        self.overrides.validate()
    }
}
impl Validate for Config {
    fn validate(&self) -> Result<()> {
        self.identity.validate()?;
        voice_id(&self.routing.fallback)?;
        check(
            self.voices.contains_key(&self.routing.fallback),
            format!("Unknown voice: {}", self.routing.fallback),
        )?;
        check(self.routing.rules.len() <= 100, "Too many rules")?;
        let mut names = HashSet::new();
        for rule in &self.routing.rules {
            rule.validate()?;
            check(names.insert(&rule.name), "Duplicate rule name")?;
            check(
                self.voices.contains_key(&rule.voice),
                format!("Unknown voice: {}", rule.voice),
            )?;
        }
        for (name, voice) in &self.voices {
            voice_id(name)?;
            voice.validate()?;
            let mut seen = HashSet::new();
            let mut current = Some(name.as_str());
            while let Some(id) = current {
                check(seen.insert(id), "Voice inheritance cycle")?;
                let value = self
                    .voices
                    .get(id)
                    .ok_or_else(|| format!("Unknown voice: {id}"))?;
                current = value.extends.as_deref();
            }
        }
        Ok(())
    }
}
impl Validate for Request {
    fn validate(&self) -> Result<()> {
        string_length(&self.intent, 1, 20_000, "intent")?;
        self.context.validate()?;
        strings(&self.messages, 100, 0, 20_000)?;
        strings(&self.facts, 100, 0, 20_000)?;
        strings(&self.preserve, 100, 1, 2000)?;
        if let Some(draft) = &self.draft {
            string_length(draft, 0, 20_000, "draft")?;
        }
        if let Some(voice) = &self.voice {
            voice_id(voice)?;
        }
        check(self.blend.len() <= 5, "Too many blended voices")?;
        for voice in &self.blend {
            voice_id(voice)?;
        }
        Ok(())
    }
}
impl Validate for Output {
    fn normalize(&mut self) {
        if let Self::Draft { text } = self {
            *text = js_trim(text).into();
        }
    }
    fn validate(&self) -> Result<()> {
        match self {
            Self::Draft { text } => string_length(text, 1, 40_000, "draft text"),
            Self::NoResponse { text } => check(text.is_empty(), "no-response text must be empty"),
        }
    }
}
