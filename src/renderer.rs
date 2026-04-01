use owo_colors::{OwoColorize, Style, Stream::Stdout};
use crate::classifier::{ClassifiedLine, LogLevel, value_to_string};
use crate::config::Config;

/// Render a classified line to a String.
/// no_color=true disables all ANSI escape codes.
pub fn render(line: &ClassifiedLine, config: &Config, no_color: bool) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(ref ts) = line.timestamp {
        let ts_short = shorten_timestamp(ts);
        if no_color {
            parts.push(ts_short);
        } else {
            parts.push(format!("{}", ts_short.if_supports_color(Stdout, |t| t.cyan())));
        }
    }

    if let Some(ref lvl) = line.level {
        let lvl_str = format_level(lvl);
        if no_color {
            parts.push(lvl_str);
        } else {
            parts.push(colorize_level(lvl, &lvl_str));
        }
    }

    if let Some(ref msg) = line.message {
        let display = if config.highlight_errors && contains_error_keyword(msg) {
            if no_color {
                msg.clone()
            } else {
                let style = Style::new().bold().red();
                format!("{}", msg.if_supports_color(Stdout, |t| t.style(style)))
            }
        } else if no_color {
            msg.clone()
        } else {
            let style = Style::new().bold().white();
            format!("{}", msg.if_supports_color(Stdout, |t| t.style(style)))
        };
        parts.push(display);
    }

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

/// Render a raw (non-JSON) line with optional continuation lines.
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
        LogLevel::Error => {
            let style = Style::new().bold().red();
            format!("{}", s.if_supports_color(Stdout, |t| t.style(style)))
        }
        LogLevel::Warn => {
            let style = Style::new().bold().yellow();
            format!("{}", s.if_supports_color(Stdout, |t| t.style(style)))
        }
        LogLevel::Info  => format!("{}", s.if_supports_color(Stdout, |t| t.green())),
        LogLevel::Debug => format!("{}", s.if_supports_color(Stdout, |t| t.blue())),
        LogLevel::Trace => format!("{}", s.if_supports_color(Stdout, |t| t.bright_black())),
        LogLevel::Unknown(_) => s.to_string(),
    }
}

pub fn shorten_timestamp(ts: &str) -> String {
    if let Some(t_pos) = ts.find('T') {
        let after_t = &ts[t_pos + 1..];
        let time_part = after_t.split('.').next().unwrap_or(after_t);
        let time_part = time_part.split('Z').next().unwrap_or(time_part);
        if time_part.len() >= 8 {
            return time_part[..8].to_string();
        }
    }
    ts.chars().take(19).collect()
}

pub fn contains_error_keyword(msg: &str) -> bool {
    msg.contains("error") || msg.contains("Error") || msg.contains("ERROR")
        || msg.contains("err") || msg.contains("Err")
}

fn expand_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => {
            if let Ok(nested) = serde_json::from_str::<serde_json::Value>(s) {
                if nested.is_object() || nested.is_array() {
                    return serde_json::to_string_pretty(&nested)
                        .unwrap_or_else(|_| s.clone());
                }
            }
            s.clone()
        }
        other => serde_json::to_string_pretty(other)
            .unwrap_or_else(|_| value_to_string(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::{ClassifiedLine, LogLevel};
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

    #[test]
    fn render_raw_with_continuations_indented() {
        let continuations = vec!["trace: xyz".to_string()];
        let out = render_raw("some raw log", &continuations, true);
        assert!(out.contains("some raw log"));
        assert!(out.contains("\n  trace: xyz"));
    }
}
