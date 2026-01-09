// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Import from other API client formats

use super::ImportFormat;
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub async fn execute(format: ImportFormat, path: PathBuf) -> Result<()> {
    println!("Importing from {:?}: {}", format, path.display());

    match format {
        ImportFormat::Postman => import_postman(&path).await,
        ImportFormat::Openapi => import_openapi(&path).await,
        ImportFormat::Curl => import_curl(&path).await,
        ImportFormat::Har => import_har(&path).await,
        ImportFormat::Insomnia => import_insomnia(&path).await,
    }
}

async fn import_postman(path: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let collection: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse Postman collection")?;

    let items = collection
        .get("item")
        .and_then(|i| i.as_array())
        .context("Invalid Postman collection format")?;

    let output_dir = PathBuf::from("imported");
    fs::create_dir_all(&output_dir)?;

    let mut count = 0;
    for item in items {
        if let Some(request) = item.get("request") {
            let name = item
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("request");

            let method = request
                .get("method")
                .and_then(|m| m.as_str())
                .unwrap_or("GET");

            let url = extract_url(request.get("url"));

            let reqx_content = format!(
                r#"# Imported from Postman: {}

[request]
method = "{}"
url = "{}"

[headers]
Content-Type = "application/json"

[assert]
status = 200
"#,
                name, method, url
            );

            let filename = sanitize_filename(name);
            let file_path = output_dir.join(format!("{}.reqx", filename));
            fs::write(&file_path, reqx_content)?;

            println!("  {} {}", "✓".green(), file_path.display());
            count += 1;
        }
    }

    println!();
    println!("{} request(s) imported to {}/", count, output_dir.display());

    Ok(())
}

async fn import_openapi(path: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let spec: serde_json::Value = if path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
        // TODO: Add YAML support
        anyhow::bail!("YAML OpenAPI files not yet supported. Please convert to JSON.");
    } else {
        serde_json::from_str(&content).context("Failed to parse OpenAPI spec")?
    };

    let paths = spec
        .get("paths")
        .and_then(|p| p.as_object())
        .context("Invalid OpenAPI spec: missing paths")?;

    let output_dir = PathBuf::from("imported");
    fs::create_dir_all(&output_dir)?;

    let base_url = spec
        .get("servers")
        .and_then(|s| s.as_array())
        .and_then(|s| s.first())
        .and_then(|s| s.get("url"))
        .and_then(|u| u.as_str())
        .unwrap_or("{{base_url}}");

    let mut count = 0;
    for (path_str, methods) in paths {
        if let Some(methods_obj) = methods.as_object() {
            for (method, _operation) in methods_obj {
                let method_upper = method.to_uppercase();
                if !["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]
                    .contains(&method_upper.as_str())
                {
                    continue;
                }

                let reqx_content = format!(
                    r#"# Imported from OpenAPI: {} {}

[request]
method = "{}"
url = "{}{}"

[headers]
Content-Type = "application/json"

[assert]
status = 200
"#,
                    method_upper, path_str, method_upper, base_url, path_str
                );

                let filename = format!(
                    "{}-{}",
                    method,
                    path_str.trim_start_matches('/').replace('/', "-")
                );
                let file_path = output_dir.join(format!("{}.reqx", sanitize_filename(&filename)));
                fs::write(&file_path, reqx_content)?;

                println!("  {} {}", "✓".green(), file_path.display());
                count += 1;
            }
        }
    }

    println!();
    println!("{} endpoint(s) imported to {}/", count, output_dir.display());

    Ok(())
}

async fn import_curl(path: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Basic curl parsing
    let mut method = "GET";
    let mut url = "";
    let mut headers: Vec<(&str, &str)> = Vec::new();

    let parts: Vec<&str> = content.split_whitespace().collect();
    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            "-X" | "--request" => {
                if i + 1 < parts.len() {
                    method = parts[i + 1];
                    i += 1;
                }
            }
            "-H" | "--header" => {
                if i + 1 < parts.len() {
                    let header = parts[i + 1].trim_matches('"');
                    if let Some(pos) = header.find(':') {
                        headers.push((&header[..pos], header[pos + 1..].trim()));
                    }
                    i += 1;
                }
            }
            s if s.starts_with("http://") || s.starts_with("https://") => {
                url = s.trim_matches('"').trim_matches('\'');
            }
            _ => {}
        }
        i += 1;
    }

    if url.is_empty() {
        anyhow::bail!("Could not parse URL from curl command");
    }

    let headers_toml: String = headers
        .iter()
        .map(|(k, v)| format!("{} = \"{}\"", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    let reqx_content = format!(
        r#"# Imported from curl

[request]
method = "{}"
url = "{}"

[headers]
{}

[assert]
status = 200
"#,
        method,
        url,
        if headers_toml.is_empty() {
            "Content-Type = \"application/json\"".to_string()
        } else {
            headers_toml
        }
    );

    let output_path = PathBuf::from("imported-curl.reqx");
    fs::write(&output_path, reqx_content)?;

    println!("  {} {}", "✓".green(), output_path.display());

    Ok(())
}

async fn import_har(_path: &PathBuf) -> Result<()> {
    // TODO: Implement HAR import
    anyhow::bail!("HAR import not yet implemented")
}

async fn import_insomnia(_path: &PathBuf) -> Result<()> {
    // TODO: Implement Insomnia import
    anyhow::bail!("Insomnia import not yet implemented")
}

fn extract_url(url_value: Option<&serde_json::Value>) -> String {
    match url_value {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Object(obj)) => obj
            .get("raw")
            .and_then(|r| r.as_str())
            .unwrap_or("{{base_url}}")
            .to_string(),
        _ => "{{base_url}}".to_string(),
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase()
}
