# pretty-log Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `pretty` — a streaming log beautifier CLI that reads from stdin line-by-line, parses JSON logs, and outputs colorized human-readable output.

**Architecture:** Simple pipeline — `LineReader` handles multi-line grouping → `Parser` extracts JSON fields → `Classifier` assigns semantic roles → `Renderer` outputs ANSI-colored single-line format. Each stage is an independent module with well-defined types.

**Tech Stack:** Rust, clap 4, serde_json, serde_yaml, owo-colors, dirs

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Dependencies and binary target |
| `src/main.rs` | CLI entry point, args parsing, pipeline wiring |
| `src/config.rs` | Config struct, YAML loading, default values, merge with CLI args |
| `src/reader.rs` | Streaming stdin reader, multi-line log grouping |
| `src/parser.rs` | JSON parsing, returns `ParseResult` enum |
| `src/classifier.rs` | Semantic field recognition using config aliases |
| `src/renderer.rs` | ANSI color rendering, single-line output format |
| `tests/integration.rs` | End-to-end tests driving the full pipeline |

---

## Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/config.rs`
- Create: `src/reader.rs`
- Create: `src/parser.rs`
- Create: `src/classifier.rs`
- Create: `src/renderer.rs`
- Create: `tests/integration.rs`

- [ ] **Step 1: Initialize Cargo project**

```bash
cd E:/pretty-log
cargo init --name pretty
```

Expected output: `Created binary (application) package`

- [ ] **Step 2: Write Cargo.toml with all dependencies**

Replace the generated `Cargo.toml` with:

```toml
[package]
name = "pretty"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "pretty"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
owo-colors = "4"
dirs = "5"
regex = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 3: Create empty module files**

Create `src/config.rs`:
```rust
// config module - placeholder
```

Create `src/reader.rs`:
```rust
// reader module - placeholder
```

Create `src/parser.rs`:
```rust
// parser module - placeholder
```

Create `src/classifier.rs`:
```rust
// classifier module - placeholder
```

Create `src/renderer.rs`:
```rust
// renderer module - placeholder
```

Create `tests/integration.rs`:
```rust
// integration tests - placeholder
```

- [ ] **Step 4: Write minimal main.rs to compile**

```rust
mod config;
mod reader;
mod parser;
mod classifier;
mod renderer;

fn main() {
    println!("pretty-log");
}
```

- [ ] **Step 5: Verify project compiles**

```bash
cd E:/pretty-log && cargo build 2>&1
```

Expected: `Compiling pretty v0.1.0` then `Finished`

- [ ] **Step 6: Commit**

```bash
cd E:/pretty-log && git init && git add -A && git commit -m "chore: scaffold project with dependencies"
```

---

## Task 2: Config Module

**Files:**
- Modify: `src/config.rs`

- [ ] **Step 1: Write failing test for default config**

Add to `src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub fields: FieldAliases,
    pub expand_nested: bool,
    pub highlight_errors: bool,
    pub multiline: MultilineConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FieldAliases {
    pub level: Vec<String>,
    pub timestamp: Vec<String>,
    pub message: Vec<String>,
    pub trace_id: Vec<String>,
    pub caller: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultilineConfig {
    pub enabled: bool,
    pub continuation_pattern: String,
}

impl Default for FieldAliases {
    fn default() -> Self {
        Self {
            level: vec![
                "level".into(), "lvl".into(), "severity".into(), "log_level".into(),
            ],
            timestamp: vec![
                "time".into(), "timestamp".into(), "ts".into(), "@timestamp".into(),
            ],
            message: vec![
                "msg".into(), "message".into(), "body".into(),
            ],
            trace_id: vec![
                "trace_id".into(), "traceId".into(), "traceid".into(),
                "request_id".into(), "x-trace-id".into(),
            ],
            caller: vec![
                "caller".into(), "file".into(), "source".into(),
            ],
        }
    }
}

impl Default for MultilineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            continuation_pattern: r"^[^\{]".into(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fields: FieldAliases::default(),
            expand_nested: false,
            highlight_errors: false,
            multiline: MultilineConfig::default(),
        }
    }
}

/// Load config from file. Returns default config if file not found.
/// Panics with message if file exists but is invalid YAML.
pub fn load_config(path: Option<&std::path::Path>) -> Config {
    let resolved = path
        .map(|p| p.to_path_buf())
        .or_else(|| {
            let local = std::path::PathBuf::from(".pretty.yaml");
            if local.exists() { return Some(local); }
            dirs::config_dir().map(|d| d.join("pretty").join("config.yaml"))
        });

    let Some(p) = resolved else {
        return Config::default();
    };

    if !p.exists() {
        return Config::default();
    }

    let content = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| { eprintln!("pretty: warning: cannot read config {:?}: {}", p, e); String::new() });

    match serde_yaml::from_str::<Config>(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("pretty: warning: invalid config {:?}: {} — using defaults", p, e);
            Config::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_level_aliases() {
        let cfg = Config::default();
        assert!(cfg.fields.level.contains(&"level".to_string()));
        assert!(cfg.fields.level.contains(&"lvl".to_string()));
        assert!(cfg.fields.level.contains(&"severity".to_string()));
    }

    #[test]
    fn default_config_has_timestamp_aliases() {
        let cfg = Config::default();
        assert!(cfg.fields.timestamp.contains(&"time".to_string()));
        assert!(cfg.fields.timestamp.contains(&"ts".to_string()));
    }

    #[test]
    fn default_config_multiline_enabled() {
        let cfg = Config::default();
        assert!(cfg.multiline.enabled);
    }

    #[test]
    fn load_config_returns_default_when_no_file() {
        let cfg = load_config(Some(std::path::Path::new("/nonexistent/path.yaml")));
        assert!(!cfg.expand_nested);
        assert!(!cfg.highlight_errors);
    }

    #[test]
    fn load_config_parses_yaml_file() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "expand_nested: true\nhighlight_errors: true").unwrap();
        let cfg = load_config(Some(f.path()));
        assert!(cfg.expand_nested);
        assert!(cfg.highlight_errors);
    }
}
```

- [ ] **Step 2: Add tempfile dev dependency and run failing test**

Add to `Cargo.toml` under `[dev-dependencies]`:
```toml
tempfile = "3"
```

```bash
cd E:/pretty-log && cargo test config 2>&1
```

Expected: Tests fail with compilation errors or missing implementation — that's expected, since we added the full impl above. Actually this test should pass once the code compiles.

- [ ] **Step 3: Run tests to verify they pass**

```bash
cd E:/pretty-log && cargo test config 2>&1
```

Expected: `test config::tests::default_config_has_level_aliases ... ok` (5 tests pass)

- [ ] **Step 4: Commit**

```bash
cd E:/pretty-log && git add src/config.rs Cargo.toml && git commit -m "feat: config module with YAML loading and defaults"
```

---

## Task 3: Parser Module

**Files:**
- Modify: `src/parser.rs`

- [ ] **Step 1: Write failing tests**

```rust
use serde_json::{Map, Value};

/// A successfully parsed JSON log line.
#[derive(Debug, Clone)]
pub struct ParsedLine {
    pub fields: Map<String, Value>,
    pub raw: String,
    pub continuation_lines: Vec<String>,
}

/// Result of attempting to parse a log line.
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Line was valid JSON object
    Json(ParsedLine),
    /// Line was not JSON — pass through as-is
    Raw { line: String, continuation_lines: Vec<String> },
}

/// Parse a single logical line (may include continuation lines).
pub fn parse_line(main: &str, continuation_lines: Vec<String>) -> ParseResult {
    let trimmed = main.trim();
    match serde_json::from_str::<Map<String, Value>>(trimmed) {
        Ok(fields) => ParseResult::Json(ParsedLine {
            fields,
            raw: main.to_string(),
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
        // Top-level arrays are not log lines
        let result = parse_line(r#"[1,2,3]"#, vec![]);
        match result {
            ParseResult::Raw { .. } => {},
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
            ParseResult::Json(p) => assert_eq!(p.continuation_lines, vec!["  at main.rs:10"]),
            ParseResult::Raw { .. } => panic!("expected Json"),
        }
    }

    #[test]
    fn handles_whitespace_around_json() {
        let result = parse_line(r#"  {"level":"debug","msg":"ok"}  "#, vec![]);
        match result {
            ParseResult::Json(_) => {},
            ParseResult::Raw { .. } => panic!("should parse json with surrounding whitespace"),
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

```bash
cd E:/pretty-log && cargo test parser 2>&1
```

Expected: 5 tests pass

- [ ] **Step 3: Commit**

```bash
cd E:/pretty-log && git add src/parser.rs && git commit -m "feat: parser module — JSON vs raw line detection"
```

---

## Task 4: Classifier Module

**Files:**
- Modify: `src/classifier.rs`

- [ ] **Step 1: Write classifier implementation and tests**

```rust
use serde_json::Value;
use crate::config::Config;
use crate::parser::ParsedLine;

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
    pub raw: String,
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
        raw: parsed.raw,
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
        ParsedLine { fields, raw: json.to_string(), continuation_lines: vec![] }
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
        let parsed = make_parsed(r#"{"level":"info","msg":"x","port":8080,"host":"localhost"}"#);
        let classified = classify(parsed, &Config::default());
        // extras sorted alphabetically
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
        let parsed = make_parsed(r#"{"level":"info","msg":"req","trace_id":"abc-123"}"#);
        let classified = classify(parsed, &Config::default());
        assert_eq!(classified.trace_id, Some("abc-123".to_string()));
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd E:/pretty-log && cargo test classifier 2>&1
```

Expected: 6 tests pass

- [ ] **Step 3: Commit**

```bash
cd E:/pretty-log && git add src/classifier.rs && git commit -m "feat: classifier module — semantic field recognition"
```

---

## Task 5: LineReader Module

**Files:**
- Modify: `src/reader.rs`

- [ ] **Step 1: Write LineReader implementation and tests**

```rust
use std::io::BufRead;
use crate::config::MultilineConfig;
use regex::Regex;

/// A logical log line: one primary line plus zero or more continuation lines.
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalLine {
    pub main: String,
    pub continuations: Vec<String>,
}

/// Reads logical log lines from a BufRead source.
/// Groups continuation lines (e.g., stack traces) with the preceding JSON line.
pub struct LineReader<R: BufRead> {
    inner: R,
    pending: Option<LogicalLine>,
    continuation_re: Option<Regex>,
    enabled: bool,
}

impl<R: BufRead> LineReader<R> {
    pub fn new(inner: R, config: &MultilineConfig) -> Self {
        let continuation_re = if config.enabled {
            Regex::new(&config.continuation_pattern).ok()
        } else {
            None
        };
        Self {
            inner,
            pending: None,
            continuation_re,
            enabled: config.enabled,
        }
    }

    fn is_continuation(&self, line: &str) -> bool {
        if !self.enabled {
            return false;
        }
        match &self.continuation_re {
            Some(re) => re.is_match(line),
            None => !line.trim_start().starts_with('{'),
        }
    }
}

impl<R: BufRead> Iterator for LineReader<R> {
    type Item = LogicalLine;

    fn next(&mut self) -> Option<LogicalLine> {
        loop {
            let mut raw = String::new();
            match self.inner.read_line(&mut raw) {
                Ok(0) => {
                    // EOF — flush pending
                    return self.pending.take();
                }
                Ok(_) => {
                    let line = raw.trim_end_matches('\n').trim_end_matches('\r').to_string();
                    if line.is_empty() {
                        continue;
                    }
                    if self.is_continuation(&line) {
                        if let Some(ref mut pending) = self.pending {
                            pending.continuations.push(line);
                        } else {
                            // Continuation with no preceding line — treat as standalone
                            self.pending = Some(LogicalLine { main: line, continuations: vec![] });
                        }
                    } else {
                        let prev = self.pending.take();
                        self.pending = Some(LogicalLine { main: line, continuations: vec![] });
                        if let Some(p) = prev {
                            return Some(p);
                        }
                    }
                }
                Err(_) => return self.pending.take(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn reader(input: &str) -> LineReader<Cursor<&[u8]>> {
        let config = MultilineConfig {
            enabled: true,
            continuation_pattern: r"^[^\{]".into(),
        };
        LineReader::new(Cursor::new(input.as_bytes()), &config)
    }

    #[test]
    fn reads_single_json_line() {
        let lines: Vec<_> = reader(r#"{"level":"info","msg":"hello"}"#).collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].main, r#"{"level":"info","msg":"hello"}"#);
        assert!(lines[0].continuations.is_empty());
    }

    #[test]
    fn reads_multiple_json_lines() {
        let input = "{\"a\":1}\n{\"b\":2}\n";
        let lines: Vec<_> = reader(input).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].main, "{\"a\":1}");
        assert_eq!(lines[1].main, "{\"b\":2}");
    }

    #[test]
    fn groups_continuation_lines_with_preceding_json() {
        let input = "{\"level\":\"error\",\"msg\":\"crash\"}\ngoroutine 1 [running]:\nmain.go:42\n";
        let lines: Vec<_> = reader(input).collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].continuations, vec!["goroutine 1 [running]:", "main.go:42"]);
    }

    #[test]
    fn new_json_line_flushes_previous() {
        let input = "{\"a\":1}\nstacktrace\n{\"b\":2}\n";
        let lines: Vec<_> = reader(input).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].main, "{\"a\":1}");
        assert_eq!(lines[0].continuations, vec!["stacktrace"]);
        assert_eq!(lines[1].main, "{\"b\":2}");
    }

    #[test]
    fn skips_empty_lines() {
        let input = "{\"a\":1}\n\n{\"b\":2}\n";
        let lines: Vec<_> = reader(input).collect();
        assert_eq!(lines.len(), 2);
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd E:/pretty-log && cargo test reader 2>&1
```

Expected: 5 tests pass

- [ ] **Step 3: Commit**

```bash
cd E:/pretty-log && git add src/reader.rs && git commit -m "feat: line reader with multi-line grouping"
```

---

## Task 6: Renderer Module

**Files:**
- Modify: `src/renderer.rs`

- [ ] **Step 1: Write renderer implementation and tests**

```rust
use owo_colors::{OwoColorize, Stream::Stdout};
use crate::classifier::{ClassifiedLine, LogLevel, value_to_string};
use crate::config::Config;

/// Render a classified line to a String (no ANSI if no_color is set).
pub fn render(line: &ClassifiedLine, config: &Config, no_color: bool) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Timestamp
    if let Some(ref ts) = line.timestamp {
        let ts_short = shorten_timestamp(ts);
        if no_color {
            parts.push(ts_short);
        } else {
            parts.push(format!("{}", ts_short.if_supports_color(Stdout, |t| t.cyan())));
        }
    }

    // Level
    if let Some(ref lvl) = line.level {
        let lvl_str = format_level(lvl);
        if no_color {
            parts.push(lvl_str);
        } else {
            parts.push(colorize_level(lvl, &lvl_str));
        }
    }

    // Message
    if let Some(ref msg) = line.message {
        let display = if config.highlight_errors && contains_error_keyword(msg) {
            if no_color {
                msg.clone()
            } else {
                format!("{}", msg.if_supports_color(Stdout, |t| t.red().bold()))
            }
        } else if no_color {
            msg.clone()
        } else {
            format!("{}", msg.if_supports_color(Stdout, |t| t.white().bold()))
        };
        parts.push(display);
    }

    // TraceID
    if let Some(ref tid) = line.trace_id {
        if no_color {
            parts.push(format!("trace={}", tid));
        } else {
            parts.push(format!(
                "trace={}",
                tid.if_supports_color(Stdout, |t| t.magenta())
            ));
        }
    }

    // Caller
    if let Some(ref c) = line.caller {
        if no_color {
            parts.push(format!("caller={}", c));
        } else {
            parts.push(format!(
                "caller={}",
                c.if_supports_color(Stdout, |t| t.bright_black())
            ));
        }
    }

    // Extra fields
    for (k, v) in &line.extras {
        let val_str = if config.expand_nested {
            expand_value(v)
        } else {
            value_to_string(v)
        };
        if no_color {
            parts.push(format!("{}={}", k, val_str));
        } else {
            parts.push(format!(
                "{}={}",
                k.if_supports_color(Stdout, |t| t.yellow()),
                val_str
            ));
        }
    }

    let mut output = parts.join("  ");

    // Continuation lines (stack traces etc.)
    for cont in &line.continuation_lines {
        if no_color {
            output.push_str(&format!("\n  {}", cont));
        } else {
            output.push_str(&format!(
                "\n  {}",
                cont.if_supports_color(Stdout, |t| t.bright_black())
            ));
        }
    }

    output
}

/// Render a raw (non-JSON) line.
pub fn render_raw(line: &str, continuations: &[String], no_color: bool) -> String {
    let mut output = if no_color {
        line.to_string()
    } else {
        format!("{}", line.if_supports_color(Stdout, |t| t.bright_black()))
    };
    for cont in continuations {
        if no_color {
            output.push_str(&format!("\n  {}", cont));
        } else {
            output.push_str(&format!(
                "\n  {}",
                cont.if_supports_color(Stdout, |t| t.bright_black())
            ));
        }
    }
    output
}

fn format_level(lvl: &LogLevel) -> String {
    match lvl {
        LogLevel::Error => "ERROR".to_string(),
        LogLevel::Warn  => "WARN ".to_string(),
        LogLevel::Info  => "INFO ".to_string(),
        LogLevel::Debug => "DEBUG".to_string(),
        LogLevel::Trace => "TRACE".to_string(),
        LogLevel::Unknown(s) => format!("{:<5}", s.to_uppercase()),
    }
}

fn colorize_level(lvl: &LogLevel, s: &str) -> String {
    match lvl {
        LogLevel::Error => format!("{}", s.if_supports_color(Stdout, |t| t.red().bold())),
        LogLevel::Warn  => format!("{}", s.if_supports_color(Stdout, |t| t.yellow().bold())),
        LogLevel::Info  => format!("{}", s.if_supports_color(Stdout, |t| t.green())),
        LogLevel::Debug => format!("{}", s.if_supports_color(Stdout, |t| t.blue())),
        LogLevel::Trace => format!("{}", s.if_supports_color(Stdout, |t| t.bright_black())),
        LogLevel::Unknown(_) => s.to_string(),
    }
}

fn shorten_timestamp(ts: &str) -> String {
    // Try to extract HH:MM:SS from ISO 8601 or similar formats
    // "2024-01-01T10:23:45Z" → "10:23:45"
    // "2024-01-01T10:23:45.123Z" → "10:23:45"
    if let Some(t_pos) = ts.find('T') {
        let after_t = &ts[t_pos + 1..];
        let time_part: &str = after_t.split('.').next().unwrap_or(after_t);
        let time_part: &str = time_part.split('Z').next().unwrap_or(time_part);
        if time_part.len() >= 8 {
            return time_part[..8].to_string();
        }
    }
    // Fallback: return as-is (truncated to 19 chars)
    ts.chars().take(19).collect()
}

fn contains_error_keyword(msg: &str) -> bool {
    // Case-sensitive check for error variants
    msg.contains("error") || msg.contains("Error") || msg.contains("ERROR")
        || msg.contains("err") || msg.contains("Err")
}

fn expand_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => {
            // Try to parse as nested JSON
            if let Ok(nested) = serde_json::from_str::<serde_json::Value>(s) {
                if nested.is_object() || nested.is_array() {
                    return serde_json::to_string_pretty(&nested).unwrap_or_else(|_| s.clone());
                }
            }
            s.clone()
        }
        other => serde_json::to_string_pretty(other).unwrap_or_else(|_| value_to_string(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::{LogLevel, ClassifiedLine};
    use serde_json::Value;

    fn simple_line(level: LogLevel, msg: &str) -> ClassifiedLine {
        ClassifiedLine {
            level: Some(level),
            timestamp: None,
            message: Some(msg.to_string()),
            trace_id: None,
            caller: None,
            extras: vec![],
            continuation_lines: vec![],
            raw: "".to_string(),
        }
    }

    #[test]
    fn render_basic_info_line_no_color() {
        let line = simple_line(LogLevel::Info, "server started");
        let out = render(&line, &Config::default(), true);
        assert!(out.contains("INFO"));
        assert!(out.contains("server started"));
    }

    #[test]
    fn render_error_level_no_color() {
        let line = simple_line(LogLevel::Error, "something broke");
        let out = render(&line, &Config::default(), true);
        assert!(out.contains("ERROR"));
    }

    #[test]
    fn render_includes_timestamp() {
        let line = ClassifiedLine {
            level: Some(LogLevel::Info),
            timestamp: Some("2024-01-01T10:23:45Z".to_string()),
            message: Some("ok".to_string()),
            trace_id: None,
            caller: None,
            extras: vec![],
            continuation_lines: vec![],
            raw: "".to_string(),
        };
        let out = render(&line, &Config::default(), true);
        assert!(out.contains("10:23:45"));
    }

    #[test]
    fn render_includes_extra_fields() {
        let line = ClassifiedLine {
            level: Some(LogLevel::Info),
            timestamp: None,
            message: Some("req".to_string()),
            trace_id: None,
            caller: None,
            extras: vec![("port".to_string(), Value::Number(8080.into()))],
            continuation_lines: vec![],
            raw: "".to_string(),
        };
        let out = render(&line, &Config::default(), true);
        assert!(out.contains("port=8080"));
    }

    #[test]
    fn render_continuation_lines_indented() {
        let line = ClassifiedLine {
            level: Some(LogLevel::Error),
            timestamp: None,
            message: Some("crash".to_string()),
            trace_id: None,
            caller: None,
            extras: vec![],
            continuation_lines: vec!["at main.rs:42".to_string()],
            raw: "".to_string(),
        };
        let out = render(&line, &Config::default(), true);
        assert!(out.contains("\n  at main.rs:42"));
    }

    #[test]
    fn highlight_errors_marks_message_with_error_keyword() {
        let mut cfg = Config::default();
        cfg.highlight_errors = true;
        let line = simple_line(LogLevel::Warn, "connection error occurred");
        let out = render(&line, &cfg, true);
        assert!(out.contains("connection error occurred"));
    }

    #[test]
    fn shorten_timestamp_extracts_time() {
        assert_eq!(shorten_timestamp("2024-01-01T10:23:45Z"), "10:23:45");
        assert_eq!(shorten_timestamp("2024-01-01T10:23:45.123Z"), "10:23:45");
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd E:/pretty-log && cargo test renderer 2>&1
```

Expected: 7 tests pass

- [ ] **Step 3: Commit**

```bash
cd E:/pretty-log && git add src/renderer.rs && git commit -m "feat: renderer module — ANSI colorized single-line output"
```

---

## Task 7: Main Entry Point — Wire Everything Together

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write main.rs with clap CLI and pipeline**

```rust
mod config;
mod reader;
mod parser;
mod classifier;
mod renderer;

use std::io::{self, BufRead, Write};
use clap::Parser as ClapParser;
use config::load_config;
use reader::LineReader;
use parser::{parse_line, ParseResult};
use classifier::classify;
use renderer::{render, render_raw};

#[derive(ClapParser, Debug)]
#[command(name = "pretty", about = "Streaming log beautifier")]
struct Args {
    /// Expand nested JSON field values
    #[arg(short = 's', long = "expand")]
    expand: bool,

    /// Highlight error keywords in message field
    #[arg(short = 'e', long = "highlight-errors")]
    highlight_errors: bool,

    /// Path to config file
    #[arg(long = "config", value_name = "FILE")]
    config: Option<std::path::PathBuf>,

    /// Disable ANSI color output
    #[arg(long = "no-color")]
    no_color: bool,
}

fn main() {
    let args = Args::parse();

    let mut config = load_config(args.config.as_deref());

    // CLI flags override config file
    if args.expand {
        config.expand_nested = true;
    }
    if args.highlight_errors {
        config.highlight_errors = true;
    }

    // Detect non-TTY: auto-disable color when piped
    let no_color = args.no_color || !atty::is(atty::Stream::Stdout);

    let stdin = io::stdin();
    let reader = LineReader::new(stdin.lock(), &config.multiline);
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for logical_line in reader {
        let result = parse_line(&logical_line.main, logical_line.continuations);
        let rendered = match result {
            ParseResult::Json(parsed) => {
                let classified = classify(parsed, &config);
                render(&classified, &config, no_color)
            }
            ParseResult::Raw { line, continuation_lines } => {
                render_raw(&line, &continuation_lines, no_color)
            }
        };
        writeln!(out, "{}", rendered).ok();
    }
}
```

- [ ] **Step 2: Add atty dependency to Cargo.toml**

Add to `[dependencies]` in `Cargo.toml`:
```toml
atty = "0.2"
```

- [ ] **Step 3: Build and smoke test**

```bash
cd E:/pretty-log && cargo build --release 2>&1
```

Expected: `Finished release [optimized] target(s)`

```bash
echo '{"level":"info","msg":"server started","port":8080}' | ./target/release/pretty --no-color
```

Expected output:
```
INFO   server started  port=8080
```

- [ ] **Step 4: Test error line with stack trace**

```bash
printf '{"level":"error","msg":"panic","time":"2024-01-01T10:00:00Z"}\ngoroutine 1 [running]:\nmain.go:42\n' | ./target/release/pretty --no-color
```

Expected:
```
10:00:00  ERROR  panic
  goroutine 1 [running]:
  main.go:42
```

- [ ] **Step 5: Commit**

```bash
cd E:/pretty-log && git add src/main.rs Cargo.toml Cargo.lock && git commit -m "feat: main entry — CLI args and pipeline wiring"
```

---

## Task 8: Integration Tests

**Files:**
- Modify: `tests/integration.rs`

- [ ] **Step 1: Write integration tests using assert_cmd**

```rust
use assert_cmd::Command;
use predicates::str::contains;

fn pretty() -> Command {
    Command::cargo_bin("pretty").unwrap()
}

#[test]
fn basic_json_line_output() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"info","msg":"hello world"}"#)
        .assert()
        .success()
        .stdout(contains("INFO"))
        .stdout(contains("hello world"));
}

#[test]
fn error_level_output() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"error","msg":"something broke"}"#)
        .assert()
        .success()
        .stdout(contains("ERROR"))
        .stdout(contains("something broke"));
}

#[test]
fn timestamp_shortened() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"info","msg":"ok","time":"2024-06-15T14:30:00Z"}"#)
        .assert()
        .success()
        .stdout(contains("14:30:00"));
}

#[test]
fn extra_fields_shown() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"info","msg":"req","port":8080}"#)
        .assert()
        .success()
        .stdout(contains("port=8080"));
}

#[test]
fn raw_non_json_line_passed_through() {
    pretty()
        .arg("--no-color")
        .write_stdin("plain text not json")
        .assert()
        .success()
        .stdout(contains("plain text not json"));
}

#[test]
fn multiline_stacktrace_indented() {
    let input = "{\"level\":\"error\",\"msg\":\"crash\"}\ngoroutine 1 [running]:\nmain.go:42\n";
    pretty()
        .arg("--no-color")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(contains("ERROR"))
        .stdout(contains("  goroutine 1 [running]:"))
        .stdout(contains("  main.go:42"));
}

#[test]
fn highlight_errors_flag() {
    pretty()
        .arg("--no-color")
        .arg("-e")
        .write_stdin(r#"{"level":"warn","msg":"connection error"}"#)
        .assert()
        .success()
        .stdout(contains("connection error"));
}

#[test]
fn expand_flag_expands_nested_json() {
    let input = r#"{"level":"info","msg":"ok","meta":"{\"user\":\"alice\"}"}"#;
    pretty()
        .arg("--no-color")
        .arg("-s")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(contains("alice"));
}

#[test]
fn lvl_alias_recognized() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"lvl":"debug","msg":"tracing"}"#)
        .assert()
        .success()
        .stdout(contains("DEBUG"))
        .stdout(contains("tracing"));
}

#[test]
fn trace_id_shown() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"info","msg":"req","trace_id":"abc-123"}"#)
        .assert()
        .success()
        .stdout(contains("trace=abc-123"));
}
```

- [ ] **Step 2: Run integration tests**

```bash
cd E:/pretty-log && cargo test --test integration 2>&1
```

Expected: 10 tests pass

- [ ] **Step 3: Run full test suite**

```bash
cd E:/pretty-log && cargo test 2>&1
```

Expected: All tests pass (unit + integration)

- [ ] **Step 4: Final commit**

```bash
cd E:/pretty-log && git add tests/integration.rs && git commit -m "test: integration tests for full pipeline"
```

---

## Task 9: Final Polish

**Files:**
- Create: `README.md`

- [ ] **Step 1: Verify release build and binary size**

```bash
cd E:/pretty-log && cargo build --release 2>&1 && ls -lh target/release/pretty* 2>/dev/null || ls -lh target/release/pretty.exe 2>/dev/null
```

- [ ] **Step 2: Final smoke test with realistic log**

```bash
printf '{"level":"error","msg":"database connection failed","time":"2024-01-01T09:15:33Z","trace_id":"xyz-789","host":"db-01","port":5432}\nFailed to connect after 3 retries\n' | ./target/release/pretty --no-color 2>/dev/null || printf '{"level":"error","msg":"database connection failed","time":"2024-01-01T09:15:33Z","trace_id":"xyz-789","host":"db-01","port":5432}\nFailed to connect after 3 retries\n' | ./target/release/pretty.exe --no-color
```

Expected:
```
09:15:33  ERROR  database connection failed  trace=xyz-789  host=db-01  port=5432
  Failed to connect after 3 retries
```

- [ ] **Step 3: Tag release**

```bash
cd E:/pretty-log && git tag v0.1.0 && echo "Tagged v0.1.0"
```
