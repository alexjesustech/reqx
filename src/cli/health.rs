// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Wait for API to be ready (health check)

use crate::config::Config;
use crate::http::Client;
use crate::parser::parse_file;
use crate::runtime::ExecutionContext;
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub async fn execute(path: PathBuf, retries: u32, retry_delay: u64, timeout: u64) -> Result<()> {
    println!("{}", "Waiting for API to be ready...".cyan());
    println!("Health check: {}", path.display());
    println!("Max retries: {}, delay: {}ms, timeout: {}ms", retries, retry_delay, timeout);
    println!();

    let config = Config::load(None)?;
    let reqx_file = parse_file(&path)?;
    
    let client = Arc::new(Client::new(timeout, 0, 0, config.http.clone())?);
    let mut context = ExecutionContext::new(config);

    for attempt in 1..=retries {
        print!("Attempt {}/{}: ", attempt, retries);

        let interpolated = match context.interpolate(&reqx_file) {
            Ok(r) => r,
            Err(e) => {
                println!("{} (interpolation error: {})", "FAIL".red(), e);
                sleep(Duration::from_millis(retry_delay)).await;
                continue;
            }
        };

        match client.execute(&interpolated).await {
            Ok(response) => {
                let assertion_results = context.run_assertions(&interpolated, &response);
                let failed = assertion_results.iter().any(|a| !a.passed);

                if !failed {
                    println!("{} (status: {})", "OK".green(), response.status);
                    println!();
                    println!("{}", "API is ready!".green().bold());
                    return Ok(());
                } else {
                    println!("{} (assertions failed)", "FAIL".red());
                    for result in assertion_results.iter().filter(|a| !a.passed) {
                        println!("  - {}", result.message);
                    }
                }
            }
            Err(e) => {
                println!("{} ({})", "FAIL".red(), e);
            }
        }

        if attempt < retries {
            sleep(Duration::from_millis(retry_delay)).await;
        }
    }

    println!();
    println!("{}", "API did not become ready in time".red().bold());
    std::process::exit(2);
}
