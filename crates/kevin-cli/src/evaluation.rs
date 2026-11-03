use crate::{args::Options, provider::CommandGenerator, storage};
use kevin_core::{Result, compile, compose, default_config};
use serde_json::{Value, json};
use std::{fs::OpenOptions, io::Write, time::Instant};

pub fn run(options: &Options) -> Result<()> {
    if options.model_command.is_some()
        && (options.model_id.is_none()
            || options.settings.is_none()
            || options.output_dir.is_none())
    {
        return Err("Model runs require --model-id, --settings and --output-dir for reproducibility. Settings are a recorded label; configure the actual generator separately.".into());
    }
    let scenarios: Vec<Value> = serde_json::from_str(include_str!("../../../evals/scenarios.json"))
        .map_err(|e| e.to_string())?;
    let config = serde_json::to_value(default_config()).map_err(|e| e.to_string())?;
    let mut records = vec![];
    let mut count = 0;
    let mut failed = false;
    for scenario in scenarios {
        for forced in [false, true] {
            let mut request = scenario["request"].clone();
            if forced {
                request["voice"] = json!("dry-comedian");
            }
            let envelope = compile(request, config.clone(), None)?;
            let expected = if forced {
                "dry-comedian"
            } else {
                scenario["expectedVoice"]
                    .as_str()
                    .ok_or("Invalid scenario")?
            };
            if envelope.data.plan.route.voices[0] != expected
                || serde_json::to_value(&envelope.data.plan.humor.mode)
                    .map_err(|e| e.to_string())?
                    != scenario["expectedHumor"]
            {
                return Err(format!("Routing expectation failed: {}", scenario["id"]));
            }
            if scenario["expectedHumor"] == "off"
                && (envelope.data.plan.route.traits.humor != 0.0
                    || envelope.data.plan.route.traits.sarcasm != 0.0)
            {
                return Err("Humor constraint failed".into());
            }
            count += 1;
            if let Some(command) = &options.model_command {
                let start = Instant::now();
                let mut record = json!({"id":scenario["id"],"forced":forced,"envelope":envelope,"semanticReview":"pending"});
                match compose(
                    &envelope,
                    &CommandGenerator::new(command, &options.model_arg),
                ) {
                    Ok(output) => {
                        record["output"] =
                            serde_json::to_value(output).map_err(|e| e.to_string())?;
                        record["elapsedMs"] = json!(start.elapsed().as_millis());
                    }
                    Err(error) => {
                        record["error"] = json!(error);
                        failed = true;
                    }
                }
                records.push(record);
            }
        }
    }
    if let Some(directory) = &options.output_dir
        && options.model_command.is_some()
    {
        storage::create_private_dir(directory)?;
        let path = directory.join(format!("run-{}.json", uuid::Uuid::new_v4()));
        let mut options_file = OpenOptions::new();
        options_file.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options_file.mode(0o600);
        }
        let mut file = options_file.open(&path).map_err(|e| e.to_string())?;
        let run = json!({"version":1,"date":chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis,true),"model":options.model_id,"settings":options.settings,"command":options.model_command,"args":options.model_arg,"records":records});
        file.write_all(
            serde_json::to_string_pretty(&run)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| e.to_string())?;
        println!(
            "Recorded model outputs at {}. Subjective quality requires review.",
            path.display()
        );
    }
    println!(
        "{count} routing scenarios passed (auto + forced). No subjective quality score inferred."
    );
    if failed {
        Err("One or more model generations failed; inspect recorded outputs.".into())
    } else {
        Ok(())
    }
}
