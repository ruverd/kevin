use crate::{args::*, evaluation, provider, skills, storage, wizard};
use kevin_core::*;
use serde_json::{Value, json};
use std::{io::IsTerminal, path::Path};

fn context_flags(options: &Options) -> Value {
    let mut context = json!({});
    for (key, value) in [
        ("platform", &options.platform),
        ("task", &options.task),
        ("audience", &options.audience),
        ("seriousness", &options.seriousness),
        ("energy", &options.energy),
        ("language", &options.language),
        ("workspace", &options.workspace),
        ("channel", &options.channel),
        ("recipient", &options.recipient),
    ] {
        if let Some(value) = value {
            context[key] = json!(value);
        }
    }
    context
}
fn stdin_text() -> Result<String> {
    if std::io::stdin().is_terminal() {
        Ok(String::new())
    } else {
        Ok(storage::read_stdin()?.trim().into())
    }
}
fn read_stdin_value() -> Result<Value> {
    serde_json::from_str(&stdin_text()?).map_err(|e| format!("Invalid JSON input: {e}"))
}

pub fn run(args: Args) -> Result<()> {
    let Args { command, options } = args;
    let Some(command) = command else {
        use clap::CommandFactory;
        Args::command().print_help().map_err(|e| e.to_string())?;
        println!();
        return Ok(());
    };
    match command {
        Command::Eval => return evaluation::run(&options),
        Command::Ollama => return storage::print(&provider::ollama()?),
        Command::Skill {
            command: SkillCommand::Install { host },
        } => {
            let path = skills::install(&host, options.destination.as_deref())?;
            println!("Installed {}", path.display());
            return Ok(());
        }
        _ => {}
    }
    let config_path = storage::absolute(
        options
            .config
            .as_deref()
            .unwrap_or(Path::new(".kevin/config.json")),
    )?;
    if matches!(command, Command::Init) {
        let mut config = default_config();
        if let Some(language) = &options.language {
            config.identity.primary_language = language.clone();
        }
        if let Some(style) = &options.style {
            config.routing.fallback = style.clone();
        }
        storage::create_json(&config_path, &config)?;
        println!("Created {}", config_path.display());
        return Ok(());
    }
    let mut config = if options.config.is_none() && !config_path.exists() {
        default_config()
    } else {
        storage::load_config(&config_path)?
    };
    let memory_path = storage::absolute(&options.memory.clone().unwrap_or_else(|| {
        config_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("memory.json")
    }))?;
    match command {
        Command::Doctor => storage::print(
            &json!({"version":env!("CARGO_PKG_VERSION"),"runtime":"Rust native","config":config_path,"voices":config.voices.len(),"generation":"Agent or explicit --model-command; optional kevin ollama loopback bridge","network":"Only explicit Ollama loopback requests or user-selected model commands; no telemetry"}),
        ),
        Command::Voices => {
            if options.json {
                storage::print(&json!({"default":config.routing.fallback,"voices":config.voices}))
            } else {
                for (id, voice) in &config.voices {
                    println!(
                        "{id}{}\n  {}",
                        if *id == config.routing.fallback {
                            " (default)"
                        } else {
                            ""
                        },
                        voice.description
                    );
                }
                Ok(())
            }
        }
        Command::Voice {
            command: VoiceCommand::Default { name },
        } => {
            if let Some(name) = name {
                if !config.voices.contains_key(&name) {
                    return Err(format!(
                        "Unknown voice: {name}. Run kevin voices to list options."
                    ));
                }
                config.routing.fallback = name.clone();
                storage::save_json(&config_path, &config)?;
                println!(
                    "Default voice: {name}. Specific routing rules and --voice can override it."
                );
            } else {
                println!("{}", config.routing.fallback);
            }
            Ok(())
        }
        Command::Voice { command } => create_voice(command, &options, &mut config, &config_path),
        Command::Rules => storage::print(&config.routing.rules),
        Command::Rule { command } => {
            match command {
                RuleCommand::Set { name } => {
                    let when = context_flags(&options);
                    if when.as_object().is_none_or(|fields| fields.is_empty()) {
                        return Err("Supply at least one context condition, such as --task, --channel or --recipient.".into());
                    }
                    let rule: Rule = parse(
                        json!({"name":name,"voice":options.voice,"priority":options.priority.unwrap_or(80),"when":when}),
                    )?;
                    if let Some(index) = config.routing.rules.iter().position(|r| r.name == name) {
                        config.routing.rules[index] = rule;
                    } else {
                        config.routing.rules.push(rule);
                    }
                    storage::save_json(&config_path, &config)?;
                    println!("Saved rule {name}");
                }
                RuleCommand::Remove { name } => {
                    let index = config
                        .routing
                        .rules
                        .iter()
                        .position(|r| r.name == name)
                        .ok_or("Unknown rule. Run kevin rules to list assignments.")?;
                    config.routing.rules.remove(index);
                    storage::save_json(&config_path, &config)?;
                    println!("Removed rule {name}");
                }
            }
            Ok(())
        }
        Command::Identity { command } => match command {
            IdentityCommand::Export => storage::print(&config.identity),
            IdentityCommand::Import { path } => {
                config.identity = storage::read_json(&path)?;
                storage::save_json(&config_path, &config)?;
                println!("Identity imported.");
                Ok(())
            }
        },
        Command::Memory { command } => memory(command, &options, &config, &memory_path),
        Command::Reply => composition("reply", &options, &config, &memory_path),
        Command::Rewrite => composition("rewrite", &options, &config, &memory_path),
        Command::Review => composition("review", &options, &config, &memory_path),
        Command::Explain => composition("explain", &options, &config, &memory_path),
        Command::Summarize => composition("summarize", &options, &config, &memory_path),
        Command::Draft => composition("draft", &options, &config, &memory_path),
        Command::Init | Command::Skill { .. } | Command::Eval | Command::Ollama => {
            unreachable!("early dispatch")
        }
    }
}
fn create_voice(
    command: VoiceCommand,
    options: &Options,
    config: &mut Config,
    path: &Path,
) -> Result<()> {
    let (name, edit) = match command {
        VoiceCommand::Create { name } => (name, false),
        VoiceCommand::Edit { name } => (name, true),
        VoiceCommand::Default { .. } => return Err("Unexpected default command".into()),
    };
    if !valid_voice_id(&name) {
        return Err("Invalid voice name".into());
    }
    let exists = config.voices.contains_key(&name);
    if exists && !edit {
        return Err("Voice already exists. Use voice edit.".into());
    }
    if !exists && edit {
        return Err("Unknown voice. Use voice create.".into());
    }
    if let Some(description) = &options.description {
        if edit {
            return Err(
                "Description interviews create new voices. Use JSON stdin for voice edit.".into(),
            );
        }
        let answers = match &options.answers {
            Some(file) => storage::read_value(file)?,
            None => json!({}),
        };
        let result = if std::io::stdin().is_terminal() {
            wizard::interview(description, answers)?
        } else {
            Some(design_voice(description, answers)?)
        };
        let Some(result) = result else {
            println!("Cancelled; no voice saved.");
            return Ok(());
        };
        if let DesignResult::Question { ref question, .. } = result {
            if options.save {
                return Err(format!(
                    "Missing answer: {}. Run without --save to inspect the next question.",
                    question.id
                ));
            }
            return storage::print(&result);
        }
        storage::print(&result)?;
        let save = options.save || (std::io::stdin().is_terminal() && wizard::confirm_save()?);
        if !save {
            eprintln!(
                "Proposal only; no profile changed. Supply --save after reviewing complete answers."
            );
            return Ok(());
        }
        if let DesignResult::Proposal { voice, .. } = result {
            config.voices.insert(name.clone(), voice);
        }
        storage::save_json(path, config)?;
        eprintln!("Saved voice {name}. Identity and routing unchanged.");
        return Ok(());
    }
    if options.answers.is_some() || options.save {
        return Err("--answers and --save require --description.".into());
    }
    config
        .voices
        .insert(name.clone(), parse(read_stdin_value()?)?);
    storage::save_json(path, config)?;
    println!("Saved voice {name}");
    Ok(())
}
fn composition(mode: &str, options: &Options, config: &Config, memory_path: &Path) -> Result<()> {
    let input = match &options.request {
        Some(file) => Some(storage::read_json::<Request>(file)?),
        None => None,
    };
    let body = if input.is_none() {
        stdin_text()?
    } else {
        String::new()
    };
    let mut request = match &input {
        Some(request) => serde_json::to_value(request).map_err(|e| e.to_string())?,
        None => json!({"intent":body}),
    };
    if let Some(intent) = &options.intent {
        request["intent"] = json!(intent);
    }
    if input.is_none() && mode == "rewrite" {
        request["draft"] = json!(body);
    }
    if request.get("context").is_none() {
        request["context"] = json!({});
    }
    let fields = context_flags(options);
    for (key, value) in fields.as_object().ok_or("Invalid context")? {
        request["context"][key] = value.clone();
    }
    if request["context"].get("task").is_none() {
        request["context"]["task"] = json!(mode);
    }
    if let Some(voice) = &options.voice {
        request["voice"] = json!(voice);
    }
    if !options.blend.is_empty() {
        request["blend"] = json!(options.blend);
    }
    if options.no_humor {
        request["humor"] = json!("off");
    }
    let request: Request = parse(request)?;
    let envelope = compile_typed(&request, config, &storage::load_memory(memory_path)?)?;
    if options.json {
        return storage::print(&envelope);
    }
    if options.prompt || options.model_command.is_none() {
        if !options.prompt {
            eprintln!("No model selected. Output is a prompt, not a generated reply.");
        }
        print!("{}", render_prompt(&envelope)?);
        return Ok(());
    }
    let command = options
        .model_command
        .as_deref()
        .ok_or("Missing model command")?;
    let output = compose(
        &envelope,
        &provider::CommandGenerator::new(command, &options.model_arg),
    )?;
    if let Output::Draft { text } = output {
        println!("{text}");
    }
    Ok(())
}
fn memory(
    command: Option<MemoryCommand>,
    options: &Options,
    config: &Config,
    path: &Path,
) -> Result<()> {
    if let Some(voice) = &options.voice
        && !config.voices.contains_key(voice)
    {
        return Err("Unknown memory voice. Run kevin voices to list options.".into());
    }
    if options.voice.is_some() && options.all {
        return Err("Choose --voice or --all, not both.".into());
    }
    let mut memory = storage::load_memory(path)?;
    if matches!(command, None | Some(MemoryCommand::List)) {
        if options.all {
            return storage::print(&memory);
        }
        let items = match &options.voice {
            Some(voice) => memory.voices.get(voice).map(Vec::as_slice).unwrap_or(&[]),
            None => memory.global.as_slice(),
        };
        return storage::print(
            &json!({"scope":options.voice.as_deref().unwrap_or("global"),"entries":items}),
        );
    }
    if options.all {
        return Err("--all is available only for memory list.".into());
    }
    let items = if let Some(voice) = &options.voice {
        memory.voices.entry(voice.clone()).or_default()
    } else {
        &mut memory.global
    };
    match command {
        Some(MemoryCommand::Add) => {
            let entry: MemoryEntry = parse(
                json!({"id":uuid::Uuid::new_v4().to_string(),"kind":options.kind.as_deref().unwrap_or("note"),"text":options.text,"createdAt":chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis,true),"source":"user"}),
            )?;
            items.push(entry.clone());
            storage::save_json(path, &memory)?;
            storage::print(
                &json!({"scope":options.voice.as_deref().unwrap_or("global"),"entry":entry}),
            )
        }
        Some(MemoryCommand::Remove { id }) => {
            let index = items
                .iter()
                .position(|entry| entry.id == id)
                .ok_or("Memory ID not found in selected scope.")?;
            items.remove(index);
            if items.is_empty()
                && let Some(voice) = &options.voice
            {
                memory.voices.shift_remove(voice);
            }
            storage::save_json(path, &memory)?;
            storage::print(
                &json!({"removed":id,"scope":options.voice.as_deref().unwrap_or("global")}),
            )
        }
        _ => unreachable!("list dispatched above"),
    }
}
