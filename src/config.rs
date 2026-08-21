use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    pub name: String,
    pub channel: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub query: Option<String>,
}

fn default_limit() -> u32 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PresetsConfig {
    #[serde(default)]
    pub presets: HashMap<String, Preset>,
}

impl PresetsConfig {
    /// Load presets from YAML file path
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read preset file at {:?}", path))?;
        let config: PresetsConfig =
            serde_yaml::from_str(&content).context("Failed to parse YAML preset file.")?;

        Ok(config)
    }

    /// Retreive a specific preset by key name.
    pub fn get_preset(&self, key: &str) -> Option<&Preset> {
        self.presets.get(key)
    }
}
