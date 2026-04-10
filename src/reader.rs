//! Streaming line reader with multi-line grouping support
//!
//! Groups JSON log lines with their continuation lines (e.g., stack traces).
//! Uses regex patterns to detect when a line belongs to the previous entry.

use std::io::BufRead;
use crate::config::MultilineConfig;
use regex::Regex;

/// A logical log entry consisting of a main line and optional continuation lines.
///
/// For example:
/// ```text
/// {"level":"error","msg":"crash"}    // main
/// goroutine 1 [running]:             // continuation
/// main.go:42                         // continuation
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalLine {
    pub main: String,
    pub continuations: Vec<String>,
}

/// Reads logical log lines from a buffered input stream.
///
/// Groups continuation lines (e.g., stack traces) with their preceding JSON entry.
/// Uses configurable regex patterns to detect continuation lines.
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
                    let line = raw
                        .trim_end_matches('\n')
                        .trim_end_matches('\r')
                        .to_string();
                    if line.is_empty() {
                        continue;
                    }
                    if self.is_continuation(&line) {
                        if let Some(ref mut pending) = self.pending {
                            pending.continuations.push(line);
                        } else {
                            // Continuation with no preceding line — treat as standalone
                            self.pending =
                                Some(LogicalLine { main: line, continuations: vec![] });
                        }
                    } else {
                        let prev = self.pending.take();
                        self.pending =
                            Some(LogicalLine { main: line, continuations: vec![] });
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

/// Build a closure that tests whether a line is a multiline continuation.
/// Compiles the regex once at construction time so callers pay no per-line
/// compilation cost.
pub fn make_continuation_checker(
    config: &MultilineConfig,
) -> impl Fn(&str) -> bool + Send + 'static {
    let enabled = config.enabled;
    let re: Option<Regex> = if enabled {
        Regex::new(&config.continuation_pattern).ok()
    } else {
        None
    };
    move |line: &str| -> bool {
        if !enabled {
            return false;
        }
        match &re {
            Some(r) => r.is_match(line),
            None => !line.trim_start().starts_with('{'),
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
        let input =
            "{\"level\":\"error\",\"msg\":\"crash\"}\ngoroutine 1 [running]:\nmain.go:42\n";
        let lines: Vec<_> = reader(input).collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].continuations,
            vec!["goroutine 1 [running]:", "main.go:42"]
        );
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
