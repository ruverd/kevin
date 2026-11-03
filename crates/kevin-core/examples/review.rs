use kevin_core::{compile, default_config, render_prompt};
use serde_json::json;
fn main() -> Result<(), String> {
    let envelope = compile(
        json!({
            "intent":"Write one constructive review comment.",
            "context":{"platform":"github","task":"review","seriousness":"ordinary"},
            "facts":["items[0].id throws when items is empty."],
            "preserve":["items[0].id"]
        }),
        serde_json::to_value(default_config()).map_err(|e| e.to_string())?,
        None,
    )?;
    print!("{}", render_prompt(&envelope)?);
    Ok(())
}
