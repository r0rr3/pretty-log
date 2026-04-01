use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
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
/// Prints warning to stderr if file exists but is invalid YAML.
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
}
