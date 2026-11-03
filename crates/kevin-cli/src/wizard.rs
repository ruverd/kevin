use kevin_core::{DesignResult, Result, design_voice};
use serde_json::Value;
use std::io::{self, Write};
fn ask(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut input = String::new();
    if io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?
        == 0
    {
        return Err("Input closed; no voice saved.".into());
    }
    Ok(input.trim().into())
}
pub fn interview(description: &str, mut answers: Value) -> Result<Option<DesignResult>> {
    loop {
        let result = design_voice(description, answers)?;
        let DesignResult::Question {
            answers: ref current,
            ref question,
            ..
        } = result
        else {
            return Ok(Some(result));
        };
        println!("\n{}\n{}", question.prompt, question.why);
        if let Some(evidence) = &question.evidence {
            println!("{evidence}");
        }
        for (index, option) in question.options.iter().enumerate() {
            println!(
                "{}. {}{}",
                index + 1,
                option.label,
                if option.id == question.recommended {
                    " (recommended)"
                } else {
                    ""
                }
            );
        }
        loop {
            let answer =
                ask("Choose number or option ID; Enter accepts recommendation; q cancels: ")?;
            if answer.eq_ignore_ascii_case("q") {
                return Ok(None);
            }
            let choice = if answer.is_empty() {
                question
                    .options
                    .iter()
                    .find(|o| o.id == question.recommended)
            } else {
                question
                    .options
                    .iter()
                    .enumerate()
                    .find(|(i, o)| o.id == answer || (i + 1).to_string() == answer)
                    .map(|(_, o)| o)
            };
            if let Some(choice) = choice {
                answers = serde_json::to_value(current).map_err(|e| e.to_string())?;
                answers[&question.id] = Value::String(choice.id.clone());
                break;
            }
            println!("Choose one listed option.");
        }
    }
}
pub fn confirm_save() -> Result<bool> {
    Ok(matches!(
        ask("Save this voice to the selected profile? [y/N] ")?
            .to_lowercase()
            .as_str(),
        "y" | "yes"
    ))
}
