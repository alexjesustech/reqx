// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! reqx - CLI-first API client for developers
//!
//! Git-native, local-first, privacy-focused API testing tool.

mod cli;
mod config;
mod http;
mod output;
mod parser;
mod runtime;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }

    match cli.command {
        Commands::Init { force } => {
            cli::init::execute(force).await?;
        }
        Commands::Run {
            path,
            env,
            output,
            output_file,
            fail_fast,
            parallel,
            timeout,
            retries,
            retry_delay,
            var,
            var_file,
            filter,
            exclude,
            dry_run,
        } => {
            cli::run::execute(cli::run::RunOptions {
                path,
                env,
                output,
                output_file,
                fail_fast,
                parallel,
                timeout,
                retries,
                retry_delay,
                var,
                var_file,
                filter,
                exclude,
                dry_run,
                verbose: cli.verbose,
                no_color: cli.no_color,
            })
            .await?;
        }
        Commands::Validate { path, strict } => {
            cli::validate::execute(path, strict).await?;
        }
        Commands::Watch {
            path,
            env,
            filter,
            debounce,
        } => {
            cli::watch::execute(path, env, filter, debounce).await?;
        }
        Commands::Health {
            path,
            retries,
            retry_delay,
            timeout,
        } => {
            cli::health::execute(path, retries, retry_delay, timeout).await?;
        }
        Commands::Config { action } => {
            cli::config::execute(action).await?;
        }
        Commands::Import { format, path } => {
            cli::import::execute(format, path).await?;
        }
        Commands::Export { format, path } => {
            cli::export::execute(format, path).await?;
        }
        Commands::Completions { shell } => {
            cli::completions::execute(shell);
        }
    }

    Ok(())
}
