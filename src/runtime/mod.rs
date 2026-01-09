// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Runtime module for executing requests and assertions

use crate::config::Config;
use crate::http::Response;
use crate::parser::ReqxFile;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Execution context holding variables and configuration
pub struct ExecutionContext {
    pub config: Config,
    pub variables: HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new(config: Config) -> Self {
        let mut variables = HashMap::new();

        // Copy environment variables
        for (key, value) in &config.variables {
            variables.insert(key.clone(), value.clone());
        }

        Self { config, variables }
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Interpolate variables in a ReqxFile
    pub fn interpolate(&self, reqx_file: &ReqxFile) -> Result<ReqxFile> {
        let mut result = reqx_file.clone();

        // Interpolate URL
        result.request.url = self.interpolate_string(&result.request.url)?;

        // Interpolate headers
        for value in result.headers.values_mut() {
            *value = self.interpolate_string(value)?;
        }

        // Interpolate query params
        for value in result.query.values_mut() {
            *value = self.interpolate_string(value)?;
        }

        // Interpolate body (if JSON)
        if let Some(crate::parser::BodySection::Json(ref mut json)) = result.body {
            *json = self.interpolate_json(json)?;
        }

        Ok(result)
    }

    fn interpolate_string(&self, input: &str) -> Result<String> {
        let re = Regex::new(r"\{\{([^}]+)\}\}")?;
        let mut result = input.to_string();

        for cap in re.captures_iter(input) {
            let var_name = &cap[1];
            let full_match = &cap[0];

            let value = match var_name {
                "$uuid" => uuid::Uuid::new_v4().to_string(),
                "$timestamp" => chrono::Utc::now().timestamp().to_string(),
                "$random" => rand_number().to_string(),
                "$date" => chrono::Utc::now().format("%Y-%m-%d").to_string(),
                "$datetime" => chrono::Utc::now().to_rfc3339(),
                name => {
                    // Check if it's an env var reference
                    if name.starts_with('$') {
                        std::env::var(&name[1..]).unwrap_or_default()
                    } else {
                        self.variables.get(name).cloned().unwrap_or_else(|| {
                            // Try environment variable
                            std::env::var(name).unwrap_or_default()
                        })
                    }
                }
            };

            result = result.replace(full_match, &value);
        }

        Ok(result)
    }

    fn interpolate_json(&self, json: &serde_json::Value) -> Result<serde_json::Value> {
        match json {
            serde_json::Value::String(s) => {
                Ok(serde_json::Value::String(self.interpolate_string(s)?))
            }
            serde_json::Value::Array(arr) => {
                let new_arr: Result<Vec<_>> = arr.iter().map(|v| self.interpolate_json(v)).collect();
                Ok(serde_json::Value::Array(new_arr?))
            }
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (k, v) in obj {
                    new_obj.insert(k.clone(), self.interpolate_json(v)?);
                }
                Ok(serde_json::Value::Object(new_obj))
            }
            other => Ok(other.clone()),
        }
    }

    /// Run assertions against a response
    pub fn run_assertions(&self, reqx_file: &ReqxFile, response: &Response) -> Vec<AssertionResult> {
        let mut results = Vec::new();

        for assertion in &reqx_file.assertions {
            let result = self.evaluate_assertion(assertion, response);
            results.push(result);
        }

        results
    }

    fn evaluate_assertion(
        &self,
        assertion: &crate::parser::Assertion,
        response: &Response,
    ) -> AssertionResult {
        let expression = &assertion.expression;
        let expected = &assertion.expected;

        // Handle special case: status
        if expression == "status" {
            let actual = response.status.to_string();
            let passed = actual == *expected;
            return AssertionResult {
                expression: expression.clone(),
                expected: expected.clone(),
                actual: Some(actual),
                passed,
                message: if passed {
                    format!("status = {}", expected)
                } else {
                    format!("status: expected {}, got {}", expected, response.status)
                },
            };
        }

        // Handle body assertions
        if expression == "body" || expression.starts_with("body.") || expression.starts_with("body[") {
            return self.evaluate_body_assertion(expression, expected, response);
        }

        // Handle header assertions
        if expression.starts_with("headers.") {
            let header_name = expression.strip_prefix("headers.").unwrap();
            let actual = response.headers.get(header_name).cloned();
            let passed = actual.as_deref() == Some(expected);
            return AssertionResult {
                expression: expression.clone(),
                expected: expected.clone(),
                actual,
                passed,
                message: if passed {
                    format!("{} = {}", expression, expected)
                } else {
                    format!(
                        "{}: expected {}, got {:?}",
                        expression,
                        expected,
                        actual.unwrap_or_default()
                    )
                },
            };
        }

        AssertionResult {
            expression: expression.clone(),
            expected: expected.clone(),
            actual: None,
            passed: false,
            message: format!("Unknown assertion expression: {}", expression),
        }
    }

    fn evaluate_body_assertion(
        &self,
        expression: &str,
        expected: &str,
        response: &Response,
    ) -> AssertionResult {
        // Simple body assertion
        if expression == "body" {
            let passed = match expected {
                "is_array" => response.body.is_array(),
                "is_object" => response.body.is_object(),
                "is_string" => response.body.is_string(),
                "is_number" => response.body.is_number(),
                "exists" => !response.body.is_null(),
                _ => false,
            };
            return AssertionResult {
                expression: expression.to_string(),
                expected: expected.to_string(),
                actual: Some(format!("{:?}", response.body)),
                passed,
                message: if passed {
                    format!("body {}", expected)
                } else {
                    format!("body: expected {}", expected)
                },
            };
        }

        // JSONPath-like assertion
        let path = expression.strip_prefix("body").unwrap_or(expression);
        let value = extract_json_path(&response.body, path);

        let (passed, actual) = match value {
            Some(v) => {
                let actual_str = json_value_to_string(&v);
                let passed = match expected {
                    "exists" => true,
                    "is_array" => v.is_array(),
                    "is_object" => v.is_object(),
                    "is_string" => v.is_string(),
                    "is_number" => v.is_number(),
                    "is_uuid" => is_uuid(&actual_str),
                    "is_iso8601" => is_iso8601(&actual_str),
                    _ => actual_str == *expected,
                };
                (passed, Some(actual_str))
            }
            None => {
                let passed = expected == "!exists";
                (passed, None)
            }
        };

        AssertionResult {
            expression: expression.to_string(),
            expected: expected.to_string(),
            actual,
            passed,
            message: if passed {
                format!("{} = {}", expression, expected)
            } else {
                format!("{}: expected {}", expression, expected)
            },
        }
    }

    /// Run post-response scripts
    pub fn run_post_response(&mut self, reqx_file: &ReqxFile, response: &Response) -> Result<()> {
        for script in &reqx_file.post_response {
            let value = self.evaluate_expression(&script.expression, response)?;
            self.variables.insert(script.variable.clone(), value);
        }
        Ok(())
    }

    fn evaluate_expression(&self, expression: &str, response: &Response) -> Result<String> {
        // Handle res.body.* expressions
        if expression.starts_with("res.body") {
            let path = expression.strip_prefix("res.body").unwrap_or("");
            if let Some(value) = extract_json_path(&response.body, path) {
                return Ok(json_value_to_string(&value));
            }
        }

        // Handle res.status
        if expression == "res.status" {
            return Ok(response.status.to_string());
        }

        // Handle res.headers.*
        if expression.starts_with("res.headers.") {
            let header = expression.strip_prefix("res.headers.").unwrap();
            if let Some(value) = response.headers.get(header) {
                return Ok(value.clone());
            }
        }

        // Handle pipe expressions (e.g., "res.body.data | length")
        if expression.contains(" | ") {
            let parts: Vec<&str> = expression.split(" | ").collect();
            if parts.len() == 2 {
                let base_value = self.evaluate_expression(parts[0].trim(), response)?;
                return self.apply_function(parts[1].trim(), &base_value);
            }
        }

        Ok(String::new())
    }

    fn apply_function(&self, func: &str, value: &str) -> Result<String> {
        match func {
            "length" => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
                    if let Some(arr) = json.as_array() {
                        return Ok(arr.len().to_string());
                    }
                    if let Some(s) = json.as_str() {
                        return Ok(s.len().to_string());
                    }
                }
                Ok(value.len().to_string())
            }
            "first" => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
                    if let Some(arr) = json.as_array() {
                        if let Some(first) = arr.first() {
                            return Ok(json_value_to_string(first));
                        }
                    }
                }
                Ok(String::new())
            }
            "last" => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
                    if let Some(arr) = json.as_array() {
                        if let Some(last) = arr.last() {
                            return Ok(json_value_to_string(last));
                        }
                    }
                }
                Ok(String::new())
            }
            _ => Ok(value.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub expression: String,
    pub expected: String,
    pub actual: Option<String>,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub file: PathBuf,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub duration: Duration,
    pub assertions: Vec<AssertionResult>,
    pub failed: bool,
    pub error: Option<String>,
}

// Helper functions

fn extract_json_path<'a>(json: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(json);
    }

    let path = path.trim_start_matches('.');
    let mut current = json;

    for segment in split_path(path) {
        match segment {
            PathSegment::Property(name) => {
                current = current.get(&name)?;
            }
            PathSegment::Index(idx) => {
                current = current.get(idx)?;
            }
            PathSegment::Wildcard => {
                // Return first element for wildcard
                if let Some(arr) = current.as_array() {
                    current = arr.first()?;
                } else {
                    return None;
                }
            }
        }
    }

    Some(current)
}

enum PathSegment {
    Property(String),
    Index(usize),
    Wildcard,
}

fn split_path(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_bracket = false;

    for c in path.chars() {
        match c {
            '.' if !in_bracket => {
                if !current.is_empty() {
                    segments.push(PathSegment::Property(current.clone()));
                    current.clear();
                }
            }
            '[' => {
                if !current.is_empty() {
                    segments.push(PathSegment::Property(current.clone()));
                    current.clear();
                }
                in_bracket = true;
            }
            ']' => {
                if current == "*" {
                    segments.push(PathSegment::Wildcard);
                } else if let Ok(idx) = current.parse::<usize>() {
                    segments.push(PathSegment::Index(idx));
                }
                current.clear();
                in_bracket = false;
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        segments.push(PathSegment::Property(current));
    }

    segments
}

fn json_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

fn is_uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok()
}

fn is_iso8601(s: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(s).is_ok()
        || chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok()
}

fn rand_number() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    (duration.as_nanos() % 1_000_000) as u32
}
