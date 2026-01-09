// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Initialize a new reqx collection

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

pub async fn execute(force: bool) -> Result<()> {
    let reqx_dir = Path::new(".reqx");

    if reqx_dir.exists() && !force {
        anyhow::bail!(
            "Directory .reqx already exists. Use --force to overwrite."
        );
    }

    println!("{}", "Initializing reqx collection...".cyan());

    // Create directory structure
    fs::create_dir_all(".reqx/environments")?;
    fs::create_dir_all("examples")?;

    // Create config.toml
    let config_content = r#"# reqx configuration
# https://docs.reqx.dev/reference/configuration

[http]
timeout = 30000
follow_redirects = true
max_redirects = 10

[output]
default_format = "table"
colors = true

[execution]
parallel = 1
retries = 0
retry_delay = 1000
"#;

    fs::write(".reqx/config.toml", config_content)
        .context("Failed to create config.toml")?;

    // Create local environment
    let local_env = r#"# Local development environment
# https://docs.reqx.dev/reference/environments

[variables]
base_url = "http://localhost:3000"
# access_token = "${API_TOKEN}"  # Uncomment and set env var
"#;

    fs::write(".reqx/environments/local.toml", local_env)
        .context("Failed to create local.toml")?;

    // Create CI environment
    let ci_env = r#"# CI/CD environment
# Variables should be provided via environment variables

[variables]
base_url = "${API_BASE_URL}"
access_token = "${API_TOKEN}"
"#;

    fs::write(".reqx/environments/ci.toml", ci_env)
        .context("Failed to create ci.toml")?;

    // Create example request
    let example_request = r#"# Example API request
# https://docs.reqx.dev/reference/file-format

[request]
method = "GET"
url = "{{base_url}}/health"

[headers]
Accept = "application/json"

[assert]
status = 200
"#;

    fs::write("examples/health.reqx", example_request)
        .context("Failed to create example request")?;

    // Create .gitignore additions
    let gitignore = r#"# reqx
.reqx/environments/*.local.toml
*.reqx.log
"#;

    // Append to .gitignore if exists, create if not
    let gitignore_path = Path::new(".gitignore");
    if gitignore_path.exists() {
        let existing = fs::read_to_string(gitignore_path)?;
        if !existing.contains("# reqx") {
            fs::write(gitignore_path, format!("{}\n{}", existing, gitignore))?;
        }
    } else {
        fs::write(gitignore_path, gitignore)?;
    }

    println!("{}", "✓ Created .reqx/config.toml".green());
    println!("{}", "✓ Created .reqx/environments/local.toml".green());
    println!("{}", "✓ Created .reqx/environments/ci.toml".green());
    println!("{}", "✓ Created examples/health.reqx".green());
    println!("{}", "✓ Updated .gitignore".green());
    println!();
    println!("Next steps:");
    println!("  1. Edit .reqx/environments/local.toml with your API URL");
    println!("  2. Run: reqx run examples/health.reqx --env=local");
    println!();
    println!(
        "Documentation: {}",
        "https://docs.reqx.dev/getting-started".cyan()
    );

    Ok(())
}
