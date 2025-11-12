use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub rules: HashMap<String, RuleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub enabled: bool,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        let path = path.unwrap_or(Path::new(".meerjungfraulint.toml"));
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn default() -> Self {
        let mut rules = HashMap::new();
        rules.insert("duplicate-node".to_string(), RuleConfig { enabled: true, severity: Severity::Error });
        rules.insert("unknown-node".to_string(), RuleConfig { enabled: true, severity: Severity::Error });
        rules.insert("naming-convention".to_string(), RuleConfig { enabled: true, severity: Severity::Warning });
        rules.insert("large-diagram".to_string(), RuleConfig { enabled: true, severity: Severity::Info });
        rules.insert("missing-edge-label".to_string(), RuleConfig { enabled: false, severity: Severity::Warning });
        Self { rules }
    }
}