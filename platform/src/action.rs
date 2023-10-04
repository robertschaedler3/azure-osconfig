use anyhow::Result;
use serde_json::Value;

pub fn read_file_content(path: &str) -> Result<Value> {
    let content = std::fs::read_to_string(path)?;
    Ok(Value::String(content))
}

pub fn script(path: &str) -> Result<Value> {
    let output = std::process::Command::new(path).output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(Value::String(stdout))
}
