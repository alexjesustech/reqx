// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Execute API requests

use crate::config::Config;
use crate::http::Client;
use crate::output::{OutputFormatter, TableFormatter, JsonFormatter, JunitFormatter, TapFormatter};
use crate::parser::{parse_file, ReqxFile};
use crate::runtime::{ExecutionContext, ExecutionResult};
use anyhow::{Context, Result};
use colored::Colorize;
use glob::glob;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use super::OutputFormat;

pub struct RunOptions {
    pub path: PathBuf,
    pub env: Option<String>,
    pub output: OutputFormat,
    pub output_file: Option<PathBuf>,
    pub fail_fast: bool,
    pub parallel: usize,
    pub timeout: u64,
    pub retries: u32,
    pub retry_delay: u64,
    pub var: Vec<(String, String)>,
    pub var_file: Option<PathBuf>,
    pub filter: Option<String>,
    pub exclude: Option<String>,
    pub dry_run: bool,
    pub verbose: bool,
    pub no_color: bool,
}

pub async fn execute(options: RunOptions) -> Result<()> {
    // Load configuration
    let config = Config::load(options.env.as_deref())?;
    
    // Discover files to run
    let files = discover_files(&options.path, options.filter.as_deref(), options.exclude.as_deref())?;
    
    if files.is_empty() {
        println!("{}", "No .reqx files found".yellow());
        return Ok(());
    }

    if options.verbose {
        println!("Found {} file(s) to execute", files.len());
    }

    // Parse all files
    let mut parsed_files: Vec<(PathBuf, ReqxFile)> = Vec::new();
    for file_path in &files {
        match parse_file(file_path) {
            Ok(reqx_file) => {
                parsed_files.push((file_path.clone(), reqx_file));
            }
            Err(e) => {
                eprintln!("{}: {}", file_path.display().to_string().red(), e);
                if options.fail_fast {
                    std::process::exit(3);
                }
            }
        }
    }

    if options.dry_run {
        println!("{}", "Dry run - validation complete".cyan());
        for (path, _) in &parsed_files {
            println!("  ✓ {}", path.display());
        }
        return Ok(());
    }

    // Create HTTP client
    let client = Arc::new(Client::new(
        options.timeout,
        options.retries,
        options.retry_delay,
        config.http.clone(),
    )?);

    // Create execution context
    let mut context = ExecutionContext::new(config);
    
    // Add CLI variables
    for (key, value) in &options.var {
        context.set_variable(key.clone(), value.clone());
    }

    // Execute requests
    let start_time = Instant::now();
    let mut results: Vec<ExecutionResult> = Vec::new();

    if options.parallel <= 1 {
        // Sequential execution
        for (path, reqx_file) in parsed_files {
            let result = execute_request(&client, &mut context, &path, &reqx_file).await;
            
            if options.verbose {
                print_result_verbose(&result);
            }
            
            let failed = result.failed;
            results.push(result);
            
            if failed && options.fail_fast {
                break;
            }
        }
    } else {
        // TODO: Parallel execution
        // For now, fall back to sequential
        for (path, reqx_file) in parsed_files {
            let result = execute_request(&client, &mut context, &path, &reqx_file).await;
            results.push(result);
        }
    }

    let total_duration = start_time.elapsed();

    // Format and output results
    let formatter: Box<dyn OutputFormatter> = match options.output {
        OutputFormat::Table => Box::new(TableFormatter::new(!options.no_color)),
        OutputFormat::Json => Box::new(JsonFormatter::new()),
        OutputFormat::Junit => Box::new(JunitFormatter::new()),
        OutputFormat::Tap => Box::new(TapFormatter::new()),
        OutputFormat::Silent => {
            // Just return exit code
            let failed = results.iter().any(|r| r.failed);
            std::process::exit(if failed { 1 } else { 0 });
        }
    };

    let output = formatter.format(&results, total_duration);

    if let Some(output_file) = options.output_file {
        std::fs::write(&output_file, &output)
            .with_context(|| format!("Failed to write output to {}", output_file.display()))?;
        println!("Results written to {}", output_file.display());
    } else {
        println!("{}", output);
    }

    // Exit with appropriate code
    let passed = results.iter().filter(|r| !r.failed).count();
    let failed = results.iter().filter(|r| r.failed).count();

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn discover_files(
    path: &PathBuf,
    filter: Option<&str>,
    exclude: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.is_file() {
        if path.extension().map_or(false, |e| e == "reqx") {
            files.push(path.clone());
        }
    } else if path.is_dir() {
        let pattern = format!("{}/**/*.reqx", path.display());
        for entry in glob(&pattern)? {
            match entry {
                Ok(file_path) => {
                    // Apply filter
                    if let Some(filter_pattern) = filter {
                        let glob_pattern = glob::Pattern::new(filter_pattern)?;
                        if !glob_pattern.matches_path(&file_path) {
                            continue;
                        }
                    }
                    
                    // Apply exclude
                    if let Some(exclude_pattern) = exclude {
                        let glob_pattern = glob::Pattern::new(exclude_pattern)?;
                        if glob_pattern.matches_path(&file_path) {
                            continue;
                        }
                    }
                    
                    files.push(file_path);
                }
                Err(e) => {
                    eprintln!("Warning: {}", e);
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

async fn execute_request(
    client: &Client,
    context: &mut ExecutionContext,
    path: &PathBuf,
    reqx_file: &ReqxFile,
) -> ExecutionResult {
    let start = Instant::now();
    
    // Interpolate variables
    let interpolated = match context.interpolate(reqx_file) {
        Ok(r) => r,
        Err(e) => {
            return ExecutionResult {
                file: path.clone(),
                method: reqx_file.request.method.clone(),
                url: reqx_file.request.url.clone(),
                status: None,
                duration: start.elapsed(),
                assertions: vec![],
                failed: true,
                error: Some(format!("Interpolation error: {}", e)),
            };
        }
    };

    // Execute HTTP request
    let response = match client.execute(&interpolated).await {
        Ok(r) => r,
        Err(e) => {
            return ExecutionResult {
                file: path.clone(),
                method: reqx_file.request.method.clone(),
                url: interpolated.request.url.clone(),
                status: None,
                duration: start.elapsed(),
                assertions: vec![],
                failed: true,
                error: Some(format!("HTTP error: {}", e)),
            };
        }
    };

    // Run assertions
    let assertion_results = context.run_assertions(&interpolated, &response);
    let failed = assertion_results.iter().any(|a| !a.passed);

    // Run post-response scripts
    if !failed {
        if let Err(e) = context.run_post_response(&interpolated, &response) {
            eprintln!("Warning: post-response error: {}", e);
        }
    }

    ExecutionResult {
        file: path.clone(),
        method: interpolated.request.method,
        url: interpolated.request.url,
        status: Some(response.status),
        duration: start.elapsed(),
        assertions: assertion_results,
        failed,
        error: None,
    }
}

fn print_result_verbose(result: &ExecutionResult) {
    let status_str = result
        .status
        .map(|s| s.to_string())
        .unwrap_or_else(|| "ERR".to_string());
    
    let color = if result.failed { "red" } else { "green" };
    
    println!(
        "{} {} {} ({:?})",
        result.method,
        result.url,
        if result.failed {
            status_str.red()
        } else {
            status_str.green()
        },
        result.duration
    );
}
