// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Configuration management commands

use super::ConfigAction;
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::process::Command;

const CONFIG_PATH: &str = ".reqx/config.toml";

pub async fn execute(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Get { key } => {
            let config = fs::read_to_string(CONFIG_PATH)
                .context("Failed to read config file. Run 'reqx init' first.")?;
            
            let parsed: toml::Value = toml::from_str(&config)?;
            
            if let Some(value) = get_nested_value(&parsed, &key) {
                println!("{}", value);
            } else {
                println!("{}", "Key not found".yellow());
            }
        }
        ConfigAction::Set { key, value } => {
            let config = fs::read_to_string(CONFIG_PATH)
                .context("Failed to read config file. Run 'reqx init' first.")?;
            
            let mut parsed: toml::Value = toml::from_str(&config)?;
            
            set_nested_value(&mut parsed, &key, &value)?;
            
            let output = toml::to_string_pretty(&parsed)?;
            fs::write(CONFIG_PATH, output)?;
            
            println!("{} = {}", key.cyan(), value.green());
        }
        ConfigAction::List => {
            let config = fs::read_to_string(CONFIG_PATH)
                .context("Failed to read config file. Run 'reqx init' first.")?;
            
            println!("{}", config);
        }
        ConfigAction::Edit => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            
            Command::new(&editor)
                .arg(CONFIG_PATH)
                .status()
                .with_context(|| format!("Failed to open editor: {}", editor))?;
        }
    }

    Ok(())
}

fn get_nested_value<'a>(value: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = value;

    for part in parts {
        current = current.get(part)?;
    }

    Some(current)
}

fn set_nested_value(value: &mut toml::Value, key: &str, new_value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = value;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let toml::Value::Table(table) = current {
                // Try to parse as number, bool, or keep as string
                let parsed_value = if new_value == "true" {
                    toml::Value::Boolean(true)
                } else if new_value == "false" {
                    toml::Value::Boolean(false)
                } else if let Ok(n) = new_value.parse::<i64>() {
                    toml::Value::Integer(n)
                } else if let Ok(f) = new_value.parse::<f64>() {
                    toml::Value::Float(f)
                } else {
                    toml::Value::String(new_value.to_string())
                };

                table.insert(part.to_string(), parsed_value);
            }
        } else {
            // Navigate deeper
            if let toml::Value::Table(table) = current {
                current = table
                    .entry(part.to_string())
                    .or_insert(toml::Value::Table(toml::map::Map::new()));
            }
        }
    }

    Ok(())
}
