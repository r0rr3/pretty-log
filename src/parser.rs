use serde_json::{Map, Value};

/// A successfully parsed JSON log line.
#[derive(Debug, Clone)]
pub struct ParsedLine {
    pub fields: Map<String, Value>,
    pub continuation_lines: Vec<String>,
}

/// Result of attempting to parse a log line.
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Line was valid JSON object
    Json(ParsedLine),
    /// Line was not JSON — pass through as-is
    Raw {
        line: String,
        continuation_lines: Vec<String>,
    },
}

/// Parse a single logical line (may include continuation lines).
pub fn parse_line(main: &str, continuation_lines: Vec<String>) -> ParseResult {
    let trimmed = main.trim();
    match serde_json::from_str::<Map<String, Value>>(trimmed) {
        Ok(fields) => ParseResult::Json(ParsedLine {
            fields,
            continuation_lines,
        }),
        Err(_) => ParseResult::Raw {
            line: main.to_string(),
            continuation_lines,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_json_object() {
        let result = parse_line(r#"{"level":"info","msg":"hello"}"#, vec![]);
        match result {
            ParseResult::Json(p) => {
                assert_eq!(p.fields["level"], Value::String("info".into()));
                assert_eq!(p.fields["msg"], Value::String("hello".into()));
            }
            ParseResult::Raw { .. } => panic!("expected Json"),
        }
    }

    #[test]
    fn returns_raw_for_non_json() {
        let result = parse_line("plain text log line", vec![]);
        match result {
            ParseResult::Raw { line, .. } => assert_eq!(line, "plain text log line"),
            ParseResult::Json(_) => panic!("expected Raw"),
        }
    }

    #[test]
    fn returns_raw_for_json_array() {
        let result = parse_line(r#"[1,2,3]"#, vec![]);
        match result {
            ParseResult::Raw { .. } => {}
            ParseResult::Json(_) => panic!("expected Raw for array"),
        }
    }

    #[test]
    fn preserves_continuation_lines() {
        let result = parse_line(
            r#"{"level":"error","msg":"crash"}"#,
            vec!["  at main.rs:10".into()],
        );
        match result {
            ParseResult::Json(p) => {
                assert_eq!(p.continuation_lines, vec!["  at main.rs:10"])
            }
            ParseResult::Raw { .. } => panic!("expected Json"),
        }
    }

    #[test]
    fn handles_whitespace_around_json() {
        let result = parse_line(r#"  {"level":"debug","msg":"ok"}  "#, vec![]);
        match result {
            ParseResult::Json(_) => {}
            ParseResult::Raw { .. } => panic!("should parse json with surrounding whitespace"),
        }
    }
}
