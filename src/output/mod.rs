// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Output formatters for test results

use crate::runtime::ExecutionResult;
use colored::Colorize;
use std::time::Duration;

pub trait OutputFormatter {
    fn format(&self, results: &[ExecutionResult], total_duration: Duration) -> String;
}

/// Table formatter (default, colorized)
pub struct TableFormatter {
    colorized: bool,
}

impl TableFormatter {
    pub fn new(colorized: bool) -> Self {
        Self { colorized }
    }
}

impl OutputFormatter for TableFormatter {
    fn format(&self, results: &[ExecutionResult], total_duration: Duration) -> String {
        let mut output = String::new();

        let passed = results.iter().filter(|r| !r.failed).count();
        let failed = results.iter().filter(|r| r.failed).count();

        output.push_str(&format!("\n{}\n\n", "─".repeat(60)));

        for result in results {
            let status_str = result
                .status
                .map(|s| s.to_string())
                .unwrap_or_else(|| "ERR".to_string());

            let status_display = if self.colorized {
                if result.failed {
                    status_str.red().to_string()
                } else {
                    status_str.green().to_string()
                }
            } else {
                status_str
            };

            let icon = if result.failed { "✗" } else { "✓" };
            let icon_display = if self.colorized {
                if result.failed {
                    icon.red().to_string()
                } else {
                    icon.green().to_string()
                }
            } else {
                icon.to_string()
            };

            output.push_str(&format!(
                "{} {} {} {} ({:?})\n",
                icon_display,
                result.method,
                result.url,
                status_display,
                result.duration
            ));

            // Show assertion details for failures
            if result.failed {
                for assertion in &result.assertions {
                    if !assertion.passed {
                        output.push_str(&format!("  └─ {}\n", assertion.message));
                    }
                }
                if let Some(ref error) = result.error {
                    output.push_str(&format!("  └─ Error: {}\n", error));
                }
            }
        }

        output.push_str(&format!("\n{}\n", "─".repeat(60)));

        let summary = format!(
            "Total: {} | Passed: {} | Failed: {} | Duration: {:?}",
            results.len(),
            passed,
            failed,
            total_duration
        );

        if self.colorized {
            if failed > 0 {
                output.push_str(&summary.red().to_string());
            } else {
                output.push_str(&summary.green().to_string());
            }
        } else {
            output.push_str(&summary);
        }

        output.push('\n');
        output
    }
}

/// JSON formatter
pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for JsonFormatter {
    fn format(&self, results: &[ExecutionResult], total_duration: Duration) -> String {
        let passed = results.iter().filter(|r| !r.failed).count();
        let failed = results.iter().filter(|r| r.failed).count();

        let output = serde_json::json!({
            "summary": {
                "total": results.len(),
                "passed": passed,
                "failed": failed,
                "duration_ms": total_duration.as_millis()
            },
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "file": r.file.to_string_lossy(),
                    "method": r.method,
                    "url": r.url,
                    "status": r.status,
                    "duration_ms": r.duration.as_millis(),
                    "passed": !r.failed,
                    "assertions": r.assertions,
                    "error": r.error
                })
            }).collect::<Vec<_>>()
        });

        serde_json::to_string_pretty(&output).unwrap_or_default()
    }
}

/// JUnit XML formatter
pub struct JunitFormatter;

impl JunitFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for JunitFormatter {
    fn format(&self, results: &[ExecutionResult], total_duration: Duration) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

        let total = results.len();
        let failures = results.iter().filter(|r| r.failed).count();
        let time = total_duration.as_secs_f64();

        xml.push_str(&format!(
            "<testsuites name=\"reqx\" tests=\"{}\" failures=\"{}\" errors=\"0\" time=\"{:.3}\">\n",
            total, failures, time
        ));

        // Group by directory
        let mut suites: std::collections::HashMap<String, Vec<&ExecutionResult>> =
            std::collections::HashMap::new();

        for result in results {
            let suite_name = result
                .file
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("default")
                .to_string();

            suites.entry(suite_name).or_default().push(result);
        }

        for (suite_name, suite_results) in suites {
            let suite_failures = suite_results.iter().filter(|r| r.failed).count();
            let suite_time: f64 = suite_results.iter().map(|r| r.duration.as_secs_f64()).sum();

            xml.push_str(&format!(
                "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">\n",
                escape_xml(&suite_name),
                suite_results.len(),
                suite_failures,
                suite_time
            ));

            for result in suite_results {
                let test_name = result
                    .file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                xml.push_str(&format!(
                    "    <testcase name=\"{}\" classname=\"{}\" time=\"{:.3}\">\n",
                    escape_xml(test_name),
                    escape_xml(&suite_name),
                    result.duration.as_secs_f64()
                ));

                xml.push_str(&format!(
                    "      <properties>\n        <property name=\"method\" value=\"{}\"/>\n        <property name=\"url\" value=\"{}\"/>\n",
                    escape_xml(&result.method),
                    escape_xml(&result.url)
                ));

                if let Some(status) = result.status {
                    xml.push_str(&format!(
                        "        <property name=\"status\" value=\"{}\"/>\n",
                        status
                    ));
                }

                xml.push_str("      </properties>\n");

                if result.failed {
                    let message = result
                        .assertions
                        .iter()
                        .filter(|a| !a.passed)
                        .map(|a| a.message.clone())
                        .collect::<Vec<_>>()
                        .join("; ");

                    let error_message = result.error.clone().unwrap_or_default();

                    xml.push_str(&format!(
                        "      <failure message=\"{}\" type=\"AssertionError\">\n{}\n      </failure>\n",
                        escape_xml(&message),
                        escape_xml(&error_message)
                    ));
                }

                xml.push_str("    </testcase>\n");
            }

            xml.push_str("  </testsuite>\n");
        }

        xml.push_str("</testsuites>\n");
        xml
    }
}

/// TAP (Test Anything Protocol) formatter
pub struct TapFormatter;

impl TapFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for TapFormatter {
    fn format(&self, results: &[ExecutionResult], _total_duration: Duration) -> String {
        let mut output = String::new();

        output.push_str("TAP version 14\n");
        output.push_str(&format!("1..{}\n", results.len()));

        for (i, result) in results.iter().enumerate() {
            let test_num = i + 1;
            let test_name = result
                .file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            if result.failed {
                output.push_str(&format!(
                    "not ok {} - {} ({:?})\n",
                    test_num, test_name, result.duration
                ));

                output.push_str("  ---\n");
                for assertion in &result.assertions {
                    if !assertion.passed {
                        output.push_str(&format!("  message: \"{}\"\n", assertion.message));
                    }
                }
                if let Some(ref error) = result.error {
                    output.push_str(&format!("  error: \"{}\"\n", error));
                }
                output.push_str("  ...\n");
            } else {
                output.push_str(&format!(
                    "ok {} - {} ({:?})\n",
                    test_num, test_name, result.duration
                ));
            }
        }

        output
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
