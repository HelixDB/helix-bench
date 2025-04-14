use anyhow::Result;
use serde_json::Value;

pub fn extract_string_field(val: &Value) -> Result<String> {
    if let Some(obj) = val.as_object() {
        for (_, value) in obj {
            if let Some(s) = value.as_str() {
                return Ok(s.to_string());
            }
        }
    }
    Ok(val.to_string())
}
