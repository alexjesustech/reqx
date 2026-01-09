// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Configuration module

use crate::http::HttpConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub http: HttpConfig,

    #[serde(default)]
    pub output: OutputConfig,

    #[serde(default)]
    pub execution: ExecutionConfig,

    #[serde(default)]
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub default_format: String,

    #[serde(default = "default_true")]
    pub colors: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: "table".to_string(),
            colors: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    #[serde(default = "default_one")]
    pub parallel: usize,

    #[serde(default)]
    pub retries: u32,

    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            parallel: 1,
            retries: 0,
            retry_delay: 1000,
        }
    }
}

fn default_format() -> String {
    "table".to_string()
}

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}

fn default_retry_delay() -> u64 {
    1000
}

impl Config {
    /// Load configuration from .reqx/config.toml and optional environment
    pub fn load(env: Option<&str>) -> Result<Self> {
        let config_path = Path::new(".reqx/config.toml");

        let mut config = if config_path.exists() {
            let content = fs::read_to_string(config_path)
                .context("Failed to read config.toml")?;
            toml::from_str(&content).context("Failed to parse config.toml")?
        } else {
            Config::default()
        };

        // Load environment-specific config
        if let Some(env_name) = env {
            let env_path = Path::new(".reqx/environments").join(format!("{}.toml", env_name));

            if env_path.exists() {
                let env_content = fs::read_to_string(&env_path)
                    .with_context(|| format!("Failed to read environment file: {}", env_path.display()))?;

                let env_config: EnvironmentConfig = toml::from_str(&env_content)
                    .with_context(|| format!("Failed to parse environment file: {}", env_path.display()))?;

                // Merge environment variables
                for (key, value) in env_config.variables {
                    let resolved = resolve_env_vars(&value);
                    config.variables.insert(key, resolved);
                }
            } else {
                anyhow::bail!("Environment '{}' not found. Create .reqx/environments/{}.toml", env_name, env_name);
            }
        }

        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
struct EnvironmentConfig {
    #[serde(default)]
    variables: HashMap<String, String>,
}

/// Resolve ${VAR} references to environment variables
fn resolve_env_vars(value: &str) -> String {
    let mut result = value.to_string();

    // Match ${VAR_NAME} pattern
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();

    for cap in re.captures_iter(value) {
        let var_name = &cap[1];
        let full_match = &cap[0];

        if let Ok(env_value) = std::env::var(var_name) {
            result = result.replace(full_match, &env_value);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_env_vars() {
        std::env::set_var("TEST_VAR", "hello");
        let result = resolve_env_vars("${TEST_VAR} world");
        assert_eq!(result, "hello world");
        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.execution.parallel, 1);
        assert!(config.output.colors);
    }
}
