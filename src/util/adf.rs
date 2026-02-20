use serde_json::Value;

/// Extract plain text from Jira's Atlassian Document Format (ADF).
pub fn extract_text_from_adf(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().filter_map(extract_text_from_adf).collect();
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(" "))
            }
        }
        Value::Object(obj) => {
            if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                return obj.get("text").and_then(|v| v.as_str()).map(String::from);
            }
            if let Some(content) = obj.get("content") {
                return extract_text_from_adf(content);
            }
            None
        }
        _ => None,
    }
}
