use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const COMPOSITION_INSTRUCTIONS: &str =
    include_str!("../../../assets/composition-instructions.txt");
const REQUIRED_HUMOR_INSTRUCTIONS: &str = "When plan.humor.mode is required, include exactly one short, light observation grounded in the supplied facts or conversation. Kevin has already applied the context limits. Prior banter is unnecessary, and preferences that make humor optional do not apply in this mode. Keep the observation about the situation or wording. Preserve the verdict, findings, severity and technical evidence. Do not invent personal experiences or force sarcasm. Follow the plan's sarcasm limit.";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Plan {
    pub context: Context,
    pub confidence: String,
    pub route: Route,
    pub humor: HumorDecision,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompositionIdentity {
    pub name: String,
    pub language: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompositionData {
    pub request: Request,
    pub identity: CompositionIdentity,
    pub memory: SelectedMemory,
    pub plan: Plan,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Envelope {
    pub version: Version1,
    pub system: String,
    pub data: CompositionData,
    pub output_schema: Value,
}

pub fn compile(request: Value, config: Value, memory: Option<Value>) -> Result<Envelope> {
    let request: Request = parse(request)?;
    let config: Config = parse(config)?;
    let memory: Memory = match memory {
        Some(value) => parse(value)?,
        None => Memory::default(),
    };
    compile_typed(&request, &config, &memory)
}
pub fn compile_typed(request: &Request, config: &Config, memory: &Memory) -> Result<Envelope> {
    request.validate()?;
    config.validate()?;
    let mut memory = memory.clone();
    memory.normalize();
    memory.validate()?;
    let analysis = analyze_context(request, config);
    let (route, humor) = plan_route(request, config, &analysis)?;
    let memory = select_memory(&memory, &route.voices);
    let system = if humor.mode == HumorMode::Required {
        format!("{COMPOSITION_INSTRUCTIONS}\n\n{REQUIRED_HUMOR_INSTRUCTIONS}")
    } else {
        COMPOSITION_INSTRUCTIONS.into()
    };
    Ok(Envelope {
        version: Version1,
        system,
        data: CompositionData {
            request: request.clone(),
            identity: CompositionIdentity {
                name: config.identity.name.clone(),
                language: analysis.context.language.clone(),
            },
            memory,
            plan: Plan {
                context: analysis.context,
                confidence: analysis.confidence,
                route,
                humor,
            },
        },
        output_schema: json!({"oneOf":[
            {"type":"object","additionalProperties":false,"required":["status","text"],"properties":{"status":{"const":"draft"},"text":{"type":"string","minLength":1,"maxLength":40000}}},
            {"type":"object","additionalProperties":false,"required":["status","text"],"properties":{"status":{"const":"no-response"},"text":{"const":""}}}
        ]}),
    })
}
pub fn render_prompt(envelope: &Envelope) -> Result<String> {
    Ok(format!(
        "{}\n\nINPUT DATA (JSON):\n{}\n",
        envelope.system,
        serde_json::to_string_pretty(&envelope.data).map_err(|e| e.to_string())?
    ))
}
pub trait Generator {
    fn generate(&self, envelope: &Envelope) -> Result<Value>;
}
impl<F: Fn(&Envelope) -> Result<Value>> Generator for F {
    fn generate(&self, envelope: &Envelope) -> Result<Value> {
        self(envelope)
    }
}
pub fn compose(envelope: &Envelope, generator: &dyn Generator) -> Result<Output> {
    if !envelope.data.plan.context.response_needed {
        return Ok(Output::no_response());
    }
    let output: Output = parse(generator.generate(envelope)?)?;
    if !matches!(output, Output::Draft { .. }) {
        return Err(
            "Model declined a required draft; inspect context or set responseNeeded=false.".into(),
        );
    }
    for exact in &envelope.data.request.preserve {
        if !output.text().contains(exact) {
            return Err("Model omitted a protected string; draft rejected.".into());
        }
    }
    Ok(output)
}
