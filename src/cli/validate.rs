// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Validate .reqx files

use crate::parser::parse_file;
use anyhow::Result;
use colored::Colorize;
use glob::glob;
use std::path::PathBuf;

pub async fn execute(path: PathBuf, strict: bool) -> Result<()> {
    let mut files = Vec::new();
    let mut errors = 0;
    let mut warnings = 0;

    if path.is_file() {
        files.push(path);
    } else {
        let pattern = format!("{}/**/*.reqx", path.display());
        for entry in glob(&pattern)? {
            if let Ok(file_path) = entry {
                files.push(file_path);
            }
        }
    }

    if files.is_empty() {
        println!("{}", "No .reqx files found".yellow());
        return Ok(());
    }

    println!("Validating {} file(s)...\n", files.len());

    for file_path in &files {
        match parse_file(file_path) {
            Ok(reqx_file) => {
                // Check for warnings
                let file_warnings = validate_warnings(&reqx_file);
                if file_warnings.is_empty() {
                    println!("  {} {}", "✓".green(), file_path.display());
                } else {
                    println!("  {} {} ({} warning(s))", "⚠".yellow(), file_path.display(), file_warnings.len());
                    for warning in &file_warnings {
                        println!("    {}", warning.yellow());
                        warnings += 1;
                    }
                }
            }
            Err(e) => {
                println!("  {} {}", "✗".red(), file_path.display());
                println!("    {}", e.to_string().red());
                errors += 1;
            }
        }
    }

    println!();
    println!(
        "Validated {} file(s): {} error(s), {} warning(s)",
        files.len(),
        errors,
        warnings
    );

    if errors > 0 || (strict && warnings > 0) {
        std::process::exit(3);
    }

    Ok(())
}

fn validate_warnings(reqx_file: &crate::parser::ReqxFile) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check for missing assertions
    if reqx_file.assertions.is_empty() {
        warnings.push("No assertions defined".to_string());
    }

    // Check for hardcoded URLs (should use variables)
    if !reqx_file.request.url.contains("{{") {
        warnings.push("URL does not use variables - consider using {{base_url}}".to_string());
    }

    warnings
}
