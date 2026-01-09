// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Export to other API client formats

use super::ExportFormat;
use crate::parser::parse_file;
use anyhow::{Context, Result};
use colored::Colorize;
use glob::glob;
use std::fs;
use std::path::PathBuf;

pub async fn execute(format: ExportFormat, path: PathBuf) -> Result<()> {
    println!("Exporting to {:?} from: {}", format, path.display());

    match format {
        ExportFormat::Postman => export_postman(&path).await,
        ExportFormat::Openapi => export_openapi(&path).await,
    }
}

async fn export_postman(path: &PathBuf) -> Result<()> {
    let pattern = format!("{}/**/*.reqx", path.display());
    let mut items = Vec::new();

    for entry in glob(&pattern)? {
        let file_path = entry?;
        match parse_file(&file_path) {
            Ok(reqx_file) => {
                let item = serde_json::json!({
                    "name": file_path.file_stem().unwrap_or_default().to_string_lossy(),
                    "request": {
                        "method": reqx_file.request.method,
                        "header": reqx_file.headers.iter().map(|(k, v)| {
                            serde_json::json!({
                                "key": k,
                                "value": v
                            })
                        }).collect::<Vec<_>>(),
                        "url": {
                            "raw": reqx_file.request.url
                        }
                    }
                });
                items.push(item);
                println!("  {} {}", "✓".green(), file_path.display());
            }
            Err(e) => {
                println!("  {} {} ({})", "✗".red(), file_path.display(), e);
            }
        }
    }

    let collection = serde_json::json!({
        "info": {
            "name": "Exported from reqx",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": items
    });

    let output_path = PathBuf::from("postman_collection.json");
    fs::write(&output_path, serde_json::to_string_pretty(&collection)?)?;

    println!();
    println!(
        "Exported {} request(s) to {}",
        items.len(),
        output_path.display()
    );

    Ok(())
}

async fn export_openapi(path: &PathBuf) -> Result<()> {
    let pattern = format!("{}/**/*.reqx", path.display());
    let mut paths = serde_json::Map::new();

    for entry in glob(&pattern)? {
        let file_path = entry?;
        match parse_file(&file_path) {
            Ok(reqx_file) => {
                // Extract path from URL (simplified)
                let url = &reqx_file.request.url;
                let path_part = url
                    .split("://")
                    .last()
                    .and_then(|s| s.split('/').skip(1).next())
                    .map(|s| format!("/{}", s))
                    .unwrap_or_else(|| "/".to_string());

                let method = reqx_file.request.method.to_lowercase();
                let operation = serde_json::json!({
                    "summary": file_path.file_stem().unwrap_or_default().to_string_lossy(),
                    "responses": {
                        "200": {
                            "description": "Successful response"
                        }
                    }
                });

                let path_item = paths
                    .entry(path_part)
                    .or_insert(serde_json::json!({}));

                if let serde_json::Value::Object(ref mut obj) = path_item {
                    obj.insert(method, operation);
                }

                println!("  {} {}", "✓".green(), file_path.display());
            }
            Err(e) => {
                println!("  {} {} ({})", "✗".red(), file_path.display(), e);
            }
        }
    }

    let openapi = serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Exported from reqx",
            "version": "1.0.0"
        },
        "servers": [
            {
                "url": "{{base_url}}"
            }
        ],
        "paths": paths
    });

    let output_path = PathBuf::from("openapi.json");
    fs::write(&output_path, serde_json::to_string_pretty(&openapi)?)?;

    println!();
    println!(
        "Exported {} path(s) to {}",
        paths.len(),
        output_path.display()
    );

    Ok(())
}
