// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Parser module for .reqx files

mod lexer;
mod ast;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parsed .reqx file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqxFile {
    pub request: RequestSection,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: Option<BodySection>,
    pub assertions: Vec<Assertion>,
    pub post_response: Vec<PostResponseScript>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSection {
    pub method: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BodySection {
    Json(serde_json::Value),
    Raw(String),
    FormData(HashMap<String, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    pub expression: String,
    pub expected: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponseScript {
    pub variable: String,
    pub expression: String,
}

/// Parse a .reqx file from path
pub fn parse_file(path: &Path) -> Result<ReqxFile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    parse_content(&content, path)
}

/// Parse .reqx content
pub fn parse_content(content: &str, path: &Path) -> Result<ReqxFile> {
    // Parse as TOML
    let parsed: toml::Value = toml::from_str(content)
        .with_context(|| format!("Failed to parse TOML in {}", path.display()))?;

    let table = parsed
        .as_table()
        .context("Invalid .reqx format: expected TOML table")?;

    // Parse [request] section
    let request_table = table
        .get("request")
        .and_then(|v| v.as_table())
        .context("Missing [request] section")?;

    let method = request_table
        .get("method")
        .and_then(|v| v.as_str())
        .context("Missing 'method' in [request]")?
        .to_uppercase();

    let url = request_table
        .get("url")
        .and_then(|v| v.as_str())
        .context("Missing 'url' in [request]")?
        .to_string();

    // Parse [headers] section
    let headers = table
        .get("headers")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    // Parse [query] section
    let query = table
        .get("query")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    // Parse [body] section
    let body = table.get("body").map(|v| {
        if let Some(table) = v.as_table() {
            let json_value: serde_json::Value = serde_json::to_value(table).unwrap_or_default();
            BodySection::Json(json_value)
        } else if let Some(s) = v.as_str() {
            BodySection::Raw(s.to_string())
        } else {
            BodySection::Raw(v.to_string())
        }
    });

    // Parse [assert] section
    let assertions = table
        .get("assert")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .map(|(k, v)| Assertion {
                    expression: k.clone(),
                    expected: v.as_str().unwrap_or(&v.to_string()).to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    // Parse [post-response] section
    let post_response = table
        .get("post-response")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .map(|(k, v)| PostResponseScript {
                    variable: k.clone(),
                    expression: v.as_str().unwrap_or(&v.to_string()).to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(ReqxFile {
        request: RequestSection { method, url },
        headers,
        query,
        body,
        assertions,
        post_response,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_request() {
        let content = r#"
[request]
method = "GET"
url = "https://api.example.com/users"

[headers]
Authorization = "Bearer token"

[assert]
status = "200"
"#;

        let result = parse_content(content, Path::new("test.reqx")).unwrap();
        assert_eq!(result.request.method, "GET");
        assert_eq!(result.request.url, "https://api.example.com/users");
        assert_eq!(result.headers.get("Authorization").unwrap(), "Bearer token");
    }

    #[test]
    fn test_parse_post_with_body() {
        let content = r#"
[request]
method = "POST"
url = "{{base_url}}/users"

[body]
name = "John"
email = "john@example.com"

[assert]
status = "201"
"#;

        let result = parse_content(content, Path::new("test.reqx")).unwrap();
        assert_eq!(result.request.method, "POST");
        assert!(result.body.is_some());
    }
}
