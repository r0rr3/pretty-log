//! Field classification and semantic recognition
//!
//! Recognizes semantic roles of JSON fields (level, timestamp, message, etc.)
//! using configurable field aliases.

use serde_json::Value;
use crate::config::Config;
use crate::parser::ParsedLine;

/// Recognized log level severity.
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Unknown(String),
}

impl LogLevel {
    /// Parse a string into a LogLevel (case-insensitive).
    ///
    /// Recognizes common aliases like "err", "fatal", "warning", etc.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" | "err" | "fatal" | "crit" | "critical" => LogLevel::Error,
            "warn" | "warning" => LogLevel::Warn,
            "info" | "information" => LogLevel::Info,
            "debug" | "dbg" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            other => LogLevel::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassifiedLine {
    pub level: Option<LogLevel>,
    pub timestamp: Option<String>,
    pub message: Option<String>,
    pub trace_id: Option<String>,
    pub caller: Option<String>,
    pub extras: Vec<(String, Value)>,
    pub continuation_lines: Vec<String>,
}

pub fn classify(parsed: ParsedLine, config: &Config) -> ClassifiedLine {
    let mut level = None;
    let mut timestamp = None;
    let mut message = None;
    let mut trace_id = None;
    let mut caller = None;
    let mut extras = Vec::new();

    for (key, value) in &parsed.fields {
        let k = key.as_str();
        if config.fields.level.iter().any(|a| a == k) {
            level = Some(LogLevel::from_str(&value_to_string(value)));
        } else if config.fields.timestamp.iter().any(|a| a == k) {
            timestamp = Some(value_to_string(value));
        } else if config.fields.message.iter().any(|a| a == k) {
            message = Some(value_to_string(value));
        } else if config.fields.trace_id.iter().any(|a| a == k) {
            trace_id = Some(value_to_string(value));
        } else if config.fields.caller.iter().any(|a| a == k) {
            caller = Some(value_to_string(value));
        } else {
            extras.push((key.clone(), value.clone()));
        }
    }

    // Sort extras for deterministic output
    extras.sort_by(|a, b| a.0.cmp(&b.0));

    ClassifiedLine {
        level,
        timestamp,
        message,
        trace_id,
        caller,
        extras,
        continuation_lines: parsed.continuation_lines,
    }
}

pub fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(_) | Value::Object(_) => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;

    fn make_parsed(json: &str) -> ParsedLine {
        let fields: Map<String, Value> = serde_json::from_str(json).unwrap();
        ParsedLine {
            fields,
            continuation_lines: vec![],
        }
    }

    #[test]
    fn classifies_level_field() {
        let parsed = make_parsed(r#"{"level":"error","msg":"oops"}"#);
        let classified = classify(parsed, &Config::default());
        assert_eq!(classified.level, Some(LogLevel::Error));
    }

    #[test]
    fn classifies_lvl_alias() {
        let parsed = make_parsed(r#"{"lvl":"warn","msg":"heads up"}"#);
        let classified = classify(parsed, &Config::default());
        assert_eq!(classified.level, Some(LogLevel::Warn));
    }

    #[test]
    fn classifies_message_field() {
        let parsed = make_parsed(r#"{"level":"info","msg":"hello world"}"#);
        let classified = classify(parsed, &Config::default());
        assert_eq!(classified.message, Some("hello world".to_string()));
    }

    #[test]
    fn puts_unknown_fields_in_extras() {
        let parsed =
            make_parsed(r#"{"level":"info","msg":"x","port":8080,"host":"localhost"}"#);
        let classified = classify(parsed, &Config::default());
        assert_eq!(classified.extras.len(), 2);
        assert_eq!(classified.extras[0].0, "host");
        assert_eq!(classified.extras[1].0, "port");
    }

    #[test]
    fn loglevel_case_insensitive() {
        assert_eq!(LogLevel::from_str("ERROR"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("Info"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("WARN"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
    }

    #[test]
    fn classifies_trace_id() {
        let parsed =
            make_parsed(r#"{"level":"info","msg":"req","trace_id":"abc-123"}"#);
        let classified = classify(parsed, &Config::default());
        assert_eq!(classified.trace_id, Some("abc-123".to_string()));
    }
}
