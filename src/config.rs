//! Configuration loading and merging
//!
//! Loads configuration from YAML files with fallback to built-in defaults.
//! Supports multiple config locations with a clear priority order.

use serde::{Deserialize, Serialize};

/// Top-level configuration for pretty.
///
/// Fields control parsing behavior, output formatting, and feature toggles.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub fields: FieldAliases,
    pub expand_nested: bool,
    pub highlight_errors: bool,
    pub multiline: MultilineConfig,
    pub table: TableConfig,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TableConfig {
    pub columns: Vec<String>,
    pub show_extras_in_detail: bool,
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

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            columns: vec![
                "time".into(),
                "level".into(),
                "message".into(),
            ],
            show_extras_in_detail: false,
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
            table: TableConfig::default(),
        }
    }
}

/// Load configuration from file with fallback to defaults.
///
/// Search order:
/// 1. Explicit path (if provided)
/// 2. `.pretty.yaml` in current directory
/// 3. `~/.config/pretty/config.yaml`
/// 4. Built-in defaults
///
/// # Behavior
/// - Returns defaults if no config file found
/// - Prints warning to stderr if file exists but is invalid YAML
/// - Merges with defaults for missing fields
///
/// # Arguments
/// * `path` - Optional explicit config file path
///
/// # Returns
/// Loaded configuration or defaults if file not found
pub fn load_config(path: Option<&std::path::Path>) -> Config {
    let resolved = path
        .map(|p| p.to_path_buf())
        .or_else(|| {
            let local = std::path::PathBuf::from(".pretty.yaml");
            if local.exists() {
                return Some(local);
            }
            dirs::config_dir().map(|d| d.join("pretty").join("config.yaml"))
        });

    let Some(p) = resolved else {
        return Config::default();
    };

    if !p.exists() {
        return Config::default();
    }

    let content = match std::fs::read_to_string(&p) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("pretty: warning: cannot read config {:?}: {}", p, e);
            return Config::default();
        }
    };

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

    #[test]
    fn default_table_config_has_expected_columns() {
        let cfg = Config::default();
        assert_eq!(cfg.table.columns, vec!["time", "level", "message"]);
        assert!(!cfg.table.show_extras_in_detail);
    }
}
