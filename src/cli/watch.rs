// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Watch for file changes and re-run

use anyhow::Result;
use colored::Colorize;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

pub async fn execute(
    path: PathBuf,
    env: Option<String>,
    filter: Option<String>,
    debounce: u64,
) -> Result<()> {
    println!("{}", "Watching for changes... (Ctrl+C to stop)".cyan());
    println!("Path: {}", path.display());
    if let Some(ref e) = env {
        println!("Environment: {}", e);
    }
    println!();

    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(debounce), tx)?;

    debouncer.watcher().watch(&path, RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                let reqx_events: Vec<_> = events
                    .iter()
                    .filter(|e| {
                        e.path
                            .extension()
                            .map_or(false, |ext| ext == "reqx")
                    })
                    .collect();

                if !reqx_events.is_empty() {
                    println!("{}", "\n─────────────────────────────────".dimmed());
                    println!("{}", "Changes detected, re-running...".cyan());
                    
                    for event in &reqx_events {
                        println!("  Modified: {}", event.path.display());
                    }
                    println!();

                    // Run the changed files
                    let options = super::run::RunOptions {
                        path: path.clone(),
                        env: env.clone(),
                        output: super::OutputFormat::Table,
                        output_file: None,
                        fail_fast: false,
                        parallel: 1,
                        timeout: 30000,
                        retries: 0,
                        retry_delay: 1000,
                        var: vec![],
                        var_file: None,
                        filter: filter.clone(),
                        exclude: None,
                        dry_run: false,
                        verbose: false,
                        no_color: false,
                    };

                    if let Err(e) = super::run::execute(options).await {
                        eprintln!("{}: {}", "Error".red(), e);
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("Watch error: {:?}", e);
            }
            Err(e) => {
                eprintln!("Channel error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
