use crate::validation::{check, js_trim, string_length};
use crate::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DesignContext {
    Slack,
    Review,
    Email,
    Mixed,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Stance {
    Direct,
    Warm,
    Tactful,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DesignHumor {
    None,
    Dry,
    Playful,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Detail {
    Short,
    Balanced,
    Deep,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExampleChoice {
    A,
    B,
    C,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct DesignAnswers {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub context: Option<DesignContext>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub stance: Option<Stance>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub humor: Option<DesignHumor>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub detail: Option<Detail>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_null"
    )]
    pub example: Option<ExampleChoice>,
    #[serde(default)]
    pub avoid: Vec<String>,
}
impl Validate for DesignAnswers {
    fn validate(&self) -> Result<()> {
        check(self.avoid.len() <= 20, "Too many avoid entries")?;
        for text in &self.avoid {
            string_length(text, 1, 1000, "avoid")?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesignOption {
    pub id: String,
    pub label: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesignQuestion {
    pub id: String,
    pub prompt: String,
    pub why: String,
    pub recommended: String,
    pub options: Vec<DesignOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesignPreview {
    pub kind: String,
    pub context: String,
    pub evidence: String,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum DesignResult {
    Question {
        description: String,
        answers: DesignAnswers,
        question: DesignQuestion,
    },
    Proposal {
        description: String,
        answers: DesignAnswers,
        voice: Voice,
        previews: Vec<DesignPreview>,
    },
}
#[derive(Deserialize)]
struct QuestionBank {
    questions: Vec<DesignQuestion>,
    examples: IndexMap<String, DesignQuestion>,
}
static BANK: LazyLock<QuestionBank> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/voice-questions.json"))
        .expect("embedded question bank")
});

pub fn design_voice(description: &str, answers: serde_json::Value) -> Result<DesignResult> {
    let description = js_trim(description).to_owned();
    string_length(&description, 1, 20_000, "description")?;
    let answers: DesignAnswers = parse(answers)?;
    let answered = serde_json::to_value(&answers).map_err(|e| e.to_string())?;
    if let Some(question) = BANK
        .questions
        .iter()
        .find(|question| answered.get(&question.id).is_none())
    {
        return Ok(DesignResult::Question {
            description,
            answers,
            question: question.clone(),
        });
    }
    let context = match answers.context {
        Some(DesignContext::Review) => "review",
        Some(DesignContext::Email) => "email",
        Some(DesignContext::Mixed) => "mixed",
        _ => "slack",
    };
    let example = &BANK.examples[if context == "mixed" { "slack" } else { context }];
    let Some(choice) = answers.example else {
        return Ok(DesignResult::Question {
            description,
            answers,
            question: example.clone(),
        });
    };
    let choice = match choice {
        ExampleChoice::A => "a",
        ExampleChoice::B => "b",
        ExampleChoice::C => "c",
    };
    let selected = example
        .options
        .iter()
        .find(|option| option.id == choice)
        .ok_or("Unknown example selection")?;
    let parent = match answers.stance {
        Some(Stance::Warm) => "friendly-teammate",
        Some(Stance::Tactful) => "diplomat",
        _ => "minimalist",
    };
    let mut behaviors = vec![
        format!("Preferred starting context: {context}. Adapt to the actual audience and task."),
        match answers.example {
            Some(ExampleChoice::B) => "Use a brief collaborative acknowledgement when useful.",
            Some(ExampleChoice::C) => "Prefer complete sentences and a measured register.",
            _ => "Lead with the useful point.",
        }
        .into(),
    ];
    match answers.humor {
        Some(DesignHumor::Dry) => {
            behaviors.push("When invited, prefer understated situational observations.".into())
        }
        Some(DesignHumor::Playful) => {
            behaviors.push("When invited, light situational exaggeration is acceptable.".into())
        }
        _ => {}
    }
    let mut avoid = vec![
        "Invented facts or commitments".into(),
        "Forced humor".into(),
    ];
    avoid.extend(answers.avoid.clone());
    let voice = Voice {
        description: description.clone(),
        extends: Some(parent.into()),
        traits: PartialTraits {
            directness: Some(match answers.stance {
                Some(Stance::Direct) => 0.9,
                Some(Stance::Warm) => 0.75,
                _ => 0.65,
            }),
            warmth: Some(if answers.stance == Some(Stance::Warm) {
                0.75
            } else {
                0.5
            }),
            formality: Some(
                if answers.context == Some(DesignContext::Email)
                    || answers.example == Some(ExampleChoice::C)
                {
                    0.7
                } else {
                    0.3
                },
            ),
            humor: Some(match answers.humor {
                Some(DesignHumor::None) => 0.0,
                Some(DesignHumor::Dry) => 0.35,
                _ => 0.5,
            }),
            sarcasm: Some(0.0),
            verbosity: Some(match answers.detail {
                Some(Detail::Short) => 0.15,
                Some(Detail::Deep) => 0.75,
                _ => 0.4,
            }),
        },
        behaviors,
        avoid,
    };
    voice.validate()?;
    let previews = vec![
        DesignPreview {
            kind: "illustrative-fixture".into(),
            context: format!("{context} example"),
            evidence: example.evidence.clone().unwrap_or_default(),
            text: selected.label.clone(),
        },
        DesignPreview {
            kind: "illustrative-fixture".into(),
            context: "Critical incident; humor disabled regardless of voice".into(),
            evidence:
                "Synthetic incident: requests are failing; cause is unknown; no ETA supplied."
                    .into(),
            text: "Requests are failing. The cause is still unknown.".into(),
        },
    ];
    Ok(DesignResult::Proposal {
        description,
        answers,
        voice,
        previews,
    })
}
