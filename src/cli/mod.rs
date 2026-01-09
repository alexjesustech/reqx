// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! CLI module - Command line interface definitions and handlers

pub mod completions;
pub mod config;
pub mod export;
pub mod health;
pub mod import;
pub mod init;
pub mod run;
pub mod validate;
pub mod watch;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// reqx - CLI-first API client for developers
#[derive(Parser, Debug)]
#[command(name = "reqx")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new reqx collection
    Init {
        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,
    },

    /// Execute API requests
    Run {
        /// Path to .reqx file or directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Environment to use
        #[arg(short, long)]
        env: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        output: OutputFormat,

        /// Output file path
        #[arg(long)]
        output_file: Option<PathBuf>,

        /// Stop on first failure
        #[arg(long)]
        fail_fast: bool,

        /// Number of parallel executions (0 = auto)
        #[arg(long, default_value = "1")]
        parallel: usize,

        /// Request timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,

        /// Number of retries on network error
        #[arg(long, default_value = "0")]
        retries: u32,

        /// Delay between retries in milliseconds
        #[arg(long, default_value = "1000")]
        retry_delay: u64,

        /// Override variable (KEY=VALUE)
        #[arg(long, value_parser = parse_key_value)]
        var: Vec<(String, String)>,

        /// Additional variables file
        #[arg(long)]
        var_file: Option<PathBuf>,

        /// Filter files by glob pattern
        #[arg(long)]
        filter: Option<String>,

        /// Exclude files by glob pattern
        #[arg(long)]
        exclude: Option<String>,

        /// Validate without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate .reqx files syntax
    Validate {
        /// Path to .reqx file or directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },

    /// Watch for file changes and re-run
    Watch {
        /// Path to watch
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Environment to use
        #[arg(short, long)]
        env: Option<String>,

        /// Filter files by glob pattern
        #[arg(long)]
        filter: Option<String>,

        /// Debounce delay in milliseconds
        #[arg(long, default_value = "500")]
        debounce: u64,
    },

    /// Wait for API to be ready
    Health {
        /// Path to health check .reqx file
        path: PathBuf,

        /// Maximum retries
        #[arg(long, default_value = "30")]
        retries: u32,

        /// Delay between retries in milliseconds
        #[arg(long, default_value = "2000")]
        retry_delay: u64,

        /// Request timeout in milliseconds
        #[arg(long, default_value = "5000")]
        timeout: u64,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Import from other formats
    Import {
        /// Source format
        #[arg(value_enum)]
        format: ImportFormat,

        /// Path to import file
        path: PathBuf,
    },

    /// Export to other formats
    Export {
        /// Target format
        #[arg(value_enum)]
        format: ExportFormat,

        /// Path to export directory
        path: PathBuf,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Get a configuration value
    Get { key: String },
    /// Set a configuration value
    Set { key: String, value: String },
    /// List all configuration
    List,
    /// Edit configuration in editor
    Edit,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Junit,
    Tap,
    Silent,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ImportFormat {
    Postman,
    Openapi,
    Curl,
    Har,
    Insomnia,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    Postman,
    Openapi,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

/// Parse KEY=VALUE pairs
fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
