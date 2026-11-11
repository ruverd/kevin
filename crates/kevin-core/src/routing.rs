use crate::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::HashSet, sync::LazyLock};

static CRITICAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?-u:\b)(incident|outage|sev[ -]?[01]|data loss|security|vulnerability|credential leak|breach|incidente|indisponibilidade|perda de dados|vulnerabilidade|seguran[çc]a)(?-u:\b)").expect("constant regex")
});
static SENSITIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?-u:\b)(harassment|layoff|fired|ass[eé]dio|demiss[aã]o)(?-u:\b)")
        .expect("constant regex")
});
static CONTEXTUAL_SENSITIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?-u:\b)(blocked|approval|disagreement|stressed|bloquead[oa]|aprova[çc][aã]o|desentendimento|estressad[oa])(?-u:\b)").expect("constant regex")
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Analysis {
    pub context: Context,
    pub confidence: String,
    pub reasons: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResolvedVoice {
    pub traits: PartialTraits,
    pub descriptions: Vec<String>,
    pub behaviors: Vec<String>,
    pub avoid: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Route {
    pub voices: Vec<String>,
    pub descriptions: Vec<String>,
    pub traits: Traits,
    pub behaviors: Vec<String>,
    pub avoid: Vec<String>,
    pub preferences: Vec<String>,
    pub reasons: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HumorMode {
    Off,
    Optional,
    Invited,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HumorDecision {
    pub mode: HumorMode,
    pub styles: Vec<String>,
    pub reason: String,
}

pub fn analyze_context(request: &Request, config: &Config) -> Analysis {
    let mut context = request.context.resolve(&config.identity.primary_language);
    let content = std::iter::once(request.intent.as_str())
        .chain(std::iter::once(request.draft.as_deref().unwrap_or("")))
        .chain(request.facts.iter().map(String::as_str))
        .chain(request.messages.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    let mut reasons = vec![];
    if CRITICAL.is_match(&content) {
        context.seriousness = Seriousness::Critical;
        reasons.push("Conservative critical-topic detection.".into());
    } else if context.seriousness != Seriousness::Critical
        && (SENSITIVE.is_match(&content)
            || (CONTEXTUAL_SENSITIVE.is_match(&content)
                && !has_assigned_slack_voice(
                    request,
                    &context,
                    matching_rule(config, &context).ok().flatten(),
                )))
    {
        context.seriousness = Seriousness::Sensitive;
        reasons.push("Conservative sensitive-topic detection.".into());
    }
    let low = context.seriousness == Seriousness::Unknown || context.platform == Platform::Unknown;
    if low {
        reasons.push("Incomplete context: use matching configured rules or the default.".into());
    }
    Analysis {
        context,
        confidence: if low { "low" } else { "supplied" }.into(),
        reasons,
    }
}
pub fn resolve_voice(config: &Config, name: &str) -> Result<ResolvedVoice> {
    let mut seen = HashSet::new();
    let mut chain = vec![];
    let mut next = Some(name);
    while let Some(id) = next {
        if !seen.insert(id) {
            return Err("Voice inheritance cycle".into());
        }
        let voice = config
            .voices
            .get(id)
            .ok_or_else(|| format!("Unknown voice: {id}"))?;
        chain.push(voice);
        next = voice.extends.as_deref();
    }
    let mut resolved = ResolvedVoice::default();
    for voice in chain.into_iter().rev() {
        resolved.traits.overlay(&voice.traits);
        resolved.descriptions.push(voice.description.clone());
        resolved.behaviors.extend(voice.behaviors.clone());
        resolved.avoid.extend(voice.avoid.clone());
    }
    Ok(resolved)
}
pub fn route_voice(request: &Request, config: &Config, analysis: &Analysis) -> Result<Route> {
    route_with_rule(request, config, analysis).map(|(route, _)| route)
}

fn matching_rule<'a>(config: &'a Config, context: &Context) -> Result<Option<&'a Rule>> {
    let context = serde_json::to_value(context).map_err(|e| e.to_string())?;
    let mut matches = vec![];
    for (index, rule) in config.routing.rules.iter().enumerate() {
        let conditions = serde_json::to_value(&rule.when).map_err(|e| e.to_string())?;
        if conditions.as_object().is_some_and(|fields| {
            fields
                .iter()
                .all(|(key, value)| context.get(key) == Some(value))
        }) {
            matches.push((index, rule));
        }
    }
    Ok(matches
        .into_iter()
        .max_by_key(|(index, rule)| (rule.priority, Reverse(*index)))
        .map(|(_, rule)| rule))
}

fn has_assigned_slack_voice(request: &Request, context: &Context, rule: Option<&Rule>) -> bool {
    context.task == Task::Reply
        && rule.is_some_and(|rule| {
            rule.when.platform == Some(Platform::Slack)
                && request.voice.as_deref() == Some(rule.voice.as_str())
        })
}

fn route_with_rule<'a>(
    request: &Request,
    config: &'a Config,
    analysis: &Analysis,
) -> Result<(Route, Option<&'a Rule>)> {
    let rule = matching_rule(config, &analysis.context)?;
    let name = request
        .voice
        .as_deref()
        .unwrap_or_else(|| rule.map_or(config.routing.fallback.as_str(), |r| r.voice.as_str()));
    let mut reasons = analysis.reasons.clone();
    reasons.push(format!(
        "{} voice: {name}",
        if request.voice.is_some() {
            "Manual"
        } else {
            "Auto"
        }
    ));
    if request.voice.is_none()
        && let Some(rule) = rule
    {
        reasons.push(format!("Rule: {} ({})", rule.name, rule.priority));
    }
    let mut traits = config.identity.traits.clone();
    let mut preferences = config.identity.preferences.clone();
    if let Some(language) = config.identity.languages.get(&analysis.context.language) {
        traits.overlay(&language.traits);
        preferences.extend(language.preferences.clone());
    }
    let mut voices = vec![name.to_string()];
    voices.extend(request.blend.clone());
    let mut route = Route {
        voices,
        descriptions: vec![],
        traits,
        behaviors: vec![],
        avoid: config.identity.avoid.clone(),
        preferences,
        reasons,
    };
    for name in &route.voices {
        let voice = resolve_voice(config, name)?;
        route.traits.overlay(&voice.traits);
        route.descriptions.extend(voice.descriptions);
        route.behaviors.extend(voice.behaviors);
        route.avoid.extend(voice.avoid);
    }
    if request.voice.is_none()
        && let Some(rule) = rule
    {
        route.traits.overlay(&rule.overrides);
    }
    if let Some(formality) = analysis.context.formality {
        route.traits.formality = formality;
    }
    if !request.blend.is_empty() {
        route
            .reasons
            .push("Blending applies named traits in order; last explicit trait wins.".into());
    }
    Ok((route, rule))
}

pub(crate) fn plan_route(
    request: &Request,
    config: &Config,
    analysis: &Analysis,
) -> Result<(Route, HumorDecision)> {
    let (mut route, rule) = route_with_rule(request, config, analysis)?;
    let context = &analysis.context;
    let slack_voice_assignment = has_assigned_slack_voice(request, context, rule);
    let humor = constrain_humor_for_route(request, context, &mut route, slack_voice_assignment);
    Ok((route, humor))
}
pub fn constrain_humor(request: &Request, context: &Context, route: &mut Route) -> HumorDecision {
    constrain_humor_for_route(request, context, route, false)
}

fn constrain_humor_for_route(
    request: &Request,
    context: &Context,
    route: &mut Route,
    slack_voice_assignment: bool,
) -> HumorDecision {
    let reason = if request.humor == HumorSetting::Off {
        Some("User disabled humor.")
    } else if matches!(
        context.seriousness,
        Seriousness::Sensitive | Seriousness::Critical
    ) || (context.seriousness == Seriousness::Unknown && !slack_voice_assignment)
    {
        Some("Seriousness is sensitive, critical, or unknown.")
    } else if context.energy == Energy::Tense {
        Some("Tense context.")
    } else if matches!(context.task, Task::Review | Task::PrDescription) {
        Some("Technical findings and evidence take priority.")
    } else if matches!(context.audience, Audience::External | Audience::Public)
        || (context.audience == Audience::Unknown && !slack_voice_assignment)
    {
        Some("No established audience permission for humor.")
    } else if !context.existing_humor
        && context.energy != Energy::Playful
        && !slack_voice_assignment
    {
        Some("No conversational invitation for humor.")
    } else if route.traits.humor == 0.0 {
        Some("Selected identity or voice disables humor.")
    } else {
        None
    };
    if let Some(reason) = reason {
        route.traits.humor = 0.0;
        route.traits.sarcasm = 0.0;
        return HumorDecision {
            mode: HumorMode::Off,
            styles: vec![],
            reason: reason.into(),
        };
    }
    let invited = context.existing_humor
        && context.seriousness == Seriousness::Ordinary
        && context.audience != Audience::Unknown;
    if !invited {
        route.traits.sarcasm = 0.0;
    }
    HumorDecision {
        mode: if invited {
            HumorMode::Invited
        } else {
            HumorMode::Optional
        },
        styles: ["understatement", "dry observation", "playful exaggeration"]
            .map(String::from)
            .to_vec(),
        reason: "Humor may fit; omit it whenever forced or less clear than a plain reply.".into(),
    }
}
