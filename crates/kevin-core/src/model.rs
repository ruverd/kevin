use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Version1;
impl Serialize for Version1 {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_u8(1)
    }
}
impl<'de> Deserialize<'de> for Version1 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;
        if value == 1.0 {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom("Unsupported version; expected 1"))
        }
    }
}

pub(crate) fn non_null<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Traits {
    pub directness: f64,
    pub formality: f64,
    pub warmth: f64,
    pub humor: f64,
    pub sarcasm: f64,
    pub verbosity: f64,
}
impl Default for Traits {
    fn default() -> Self {
        Self {
            directness: 0.75,
            formality: 0.35,
            warmth: 0.5,
            humor: 0.25,
            sarcasm: 0.0,
            verbosity: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct PartialTraits {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub directness: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub formality: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub warmth: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub humor: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub sarcasm: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub verbosity: Option<f64>,
}
impl Traits {
    pub fn overlay(&mut self, traits: &PartialTraits) {
        if let Some(v) = traits.directness {
            self.directness = v;
        }
        if let Some(v) = traits.formality {
            self.formality = v;
        }
        if let Some(v) = traits.warmth {
            self.warmth = v;
        }
        if let Some(v) = traits.humor {
            self.humor = v;
        }
        if let Some(v) = traits.sarcasm {
            self.sarcasm = v;
        }
        if let Some(v) = traits.verbosity {
            self.verbosity = v;
        }
    }
}
impl PartialTraits {
    pub fn overlay(&mut self, traits: &Self) {
        if traits.directness.is_some() {
            self.directness = traits.directness;
        }
        if traits.formality.is_some() {
            self.formality = traits.formality;
        }
        if traits.warmth.is_some() {
            self.warmth = traits.warmth;
        }
        if traits.humor.is_some() {
            self.humor = traits.humor;
        }
        if traits.sarcasm.is_some() {
            self.sarcasm = traits.sarcasm;
        }
        if traits.verbosity.is_some() {
            self.verbosity = traits.verbosity;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct Voice {
    pub description: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub extends: Option<String>,
    #[serde(default)]
    pub traits: PartialTraits,
    #[serde(default)]
    pub behaviors: Vec<String>,
    #[serde(default)]
    pub avoid: Vec<String>,
}

macro_rules! context_enum {
    ($name:ident, $default:ident, [$($variant:ident $(=> $wire:literal)?),+]) => {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
        #[serde(rename_all = "lowercase")]
        pub enum $name { #[default] $default, $( $(#[serde(rename = $wire)])? $variant),+ }
    };
}
context_enum!(Platform, Unknown, [Slack, Github, Email, Chat, Docs]);
context_enum!(Task, Reply, [Rewrite, Review, PrDescription => "pr-description", Explain, Summarize, Draft]);
context_enum!(Audience, Unknown, [Teammate, Manager, External, Public]);
context_enum!(Seriousness, Unknown, [Ordinary, Sensitive, Critical]);
context_enum!(Energy, Unknown, [Calm, Playful, Tense]);
context_enum!(HumorSetting, Auto, [Off]);

pub fn primary_language() -> String {
    "en-US".into()
}
fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Context {
    #[serde(default)]
    pub platform: Platform,
    #[serde(default)]
    pub task: Task,
    #[serde(default)]
    pub audience: Audience,
    #[serde(default)]
    pub seriousness: Seriousness,
    #[serde(default)]
    pub energy: Energy,
    #[serde(default)]
    pub existing_humor: bool,
    #[serde(default = "yes")]
    pub response_needed: bool,
    #[serde(default = "primary_language")]
    pub language: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub formality: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub workspace: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub channel: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub recipient: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContextInput {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub platform: Option<Platform>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub task: Option<Task>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub audience: Option<Audience>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub seriousness: Option<Seriousness>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub energy: Option<Energy>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub existing_humor: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub response_needed: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub language: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub workspace: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub channel: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub recipient: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub formality: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Conditions {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub platform: Option<Platform>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub task: Option<Task>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub audience: Option<Audience>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub seriousness: Option<Seriousness>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub energy: Option<Energy>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub existing_humor: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub response_needed: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub language: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub workspace: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub channel: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub recipient: Option<String>,
}
impl ContextInput {
    pub fn resolve(&self, language: &str) -> Context {
        Context {
            platform: self.platform.unwrap_or_default(),
            task: self.task.unwrap_or_default(),
            audience: self.audience.unwrap_or_default(),
            seriousness: self.seriousness.unwrap_or_default(),
            energy: self.energy.unwrap_or_default(),
            existing_humor: self.existing_humor.unwrap_or(false),
            response_needed: self.response_needed.unwrap_or(true),
            language: self.language.clone().unwrap_or_else(|| language.into()),
            formality: self.formality,
            workspace: self.workspace.clone(),
            channel: self.channel.clone(),
            recipient: self.recipient.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub name: String,
    #[serde(default, deserialize_with = "safe_integer")]
    pub priority: i64,
    pub when: Conditions,
    pub voice: String,
    #[serde(default)]
    pub overrides: PartialTraits,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct LanguageIdentity {
    #[serde(default)]
    pub traits: PartialTraits,
    #[serde(default)]
    pub preferences: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Identity {
    #[serde(default)]
    pub name: String,
    #[serde(default = "primary_language")]
    pub primary_language: String,
    #[serde(default)]
    pub traits: Traits,
    #[serde(default)]
    pub preferences: Vec<String>,
    #[serde(default)]
    pub avoid: Vec<String>,
    #[serde(default)]
    pub languages: IndexMap<String, LanguageIdentity>,
}
impl Default for Identity {
    fn default() -> Self {
        Self {
            name: String::new(),
            primary_language: primary_language(),
            traits: Traits::default(),
            preferences: vec![],
            avoid: vec![],
            languages: IndexMap::new(),
        }
    }
}
fn neutral() -> String {
    "neutral".into()
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Routing {
    #[serde(default = "neutral")]
    pub fallback: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
}
impl Default for Routing {
    fn default() -> Self {
        Self {
            fallback: neutral(),
            rules: vec![],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: Version1,
    #[serde(default)]
    pub identity: Identity,
    pub voices: IndexMap<String, Voice>,
    #[serde(default)]
    pub routing: Routing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub intent: String,
    #[serde(default)]
    pub context: ContextInput,
    #[serde(default)]
    pub messages: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub draft: Option<String>,
    #[serde(default)]
    pub facts: Vec<String>,
    #[serde(default)]
    pub preserve: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub voice: Option<String>,
    #[serde(default)]
    pub blend: Vec<String>,
    #[serde(default)]
    pub humor: HumorSetting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Output {
    Draft { text: String },
    NoResponse { text: String },
}
impl Output {
    pub fn no_response() -> Self {
        Self::NoResponse {
            text: String::new(),
        }
    }
    pub fn text(&self) -> &str {
        match self {
            Self::Draft { text } | Self::NoResponse { text } => text,
        }
    }
}

fn safe_integer<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<i64, D::Error> {
    let number = f64::deserialize(deserializer)?;
    if number.is_finite() && number.fract() == 0.0 && number.abs() <= 9_007_199_254_740_991.0 {
        Ok(number as i64)
    } else {
        Err(serde::de::Error::custom("Expected a safe integer priority"))
    }
}
