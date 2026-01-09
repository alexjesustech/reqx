# reqx

[![CI](https://github.com/reqx/reqx/actions/workflows/ci.yml/badge.svg)](https://github.com/reqx/reqx/actions/workflows/ci.yml)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)
[![Crates.io](https://img.shields.io/crates/v/reqx.svg)](https://crates.io/crates/reqx)

**CLI-first API client for developers.** Git-native, local-first, privacy-focused.

While others build 'platforms' to justify higher costs, reqx stays focused on what matters: helping developers test APIs efficiently. Sometimes, the best solution is the one that doesn't try to do everything.

## Features

- **Git-native**: API collections are versioned, branched, and merged just like code
- **Local-first**: Everything runs on your machine. No accounts, no cloud dependencies
- **Plain text**: Human-readable `.reqx` files that work with any editor or version control
- **Privacy by design**: Your data never leaves your computer
- **CI/CD ready**: JUnit, TAP, JSON outputs with proper exit codes
- **Zero admin overhead**: Uses your existing Git infrastructure for collaboration

## Quick Start

### Installation

```bash
# Linux/macOS
curl -fsSL https://reqx.dev/install.sh | sh

# Windows (PowerShell)
iwr -useb https://reqx.dev/install.ps1 | iex

# Cargo
cargo install reqx

# Homebrew
brew install reqx/tap/reqx

# Chocolatey (Windows)
choco install reqx
```

### First Request

```bash
# Initialize a new collection
reqx init

# Create your first request
cat > users/list.reqx << 'EOF'
[request]
method = "GET"
url = "https://jsonplaceholder.typicode.com/users"

[assert]
status = 200
body = "is_array"
body[0].id = "exists"
EOF

# Run it
reqx run users/list.reqx
```

## File Format (.reqx)

```toml
# users/create.reqx

[request]
method = "POST"
url = "{{base_url}}/api/users"

[headers]
Content-Type = "application/json"
Authorization = "Bearer {{access_token}}"

[body]
name = "{{user_name}}"
email = "{{user_email}}"

[assert]
status = 201
body.id = "exists"
body.name = "{{user_name}}"

[post-response]
created_user_id = "res.body.id"
```

## Environments

```bash
# .reqx/environments/local.toml
[variables]
base_url = "http://localhost:3000"
access_token = "${API_TOKEN}"  # From environment variable

# Run with specific environment
reqx run ./tests --env=local
```

## CI/CD Integration

```bash
# JUnit output for CI
reqx run ./tests --env=ci --output=junit --output-file=results.xml

# Exit codes
# 0 = All passed
# 1 = Assertion failed
# 2 = Execution error
# 3 = Parse error
# 4 = Config error
```

### GitHub Actions Example

```yaml
- name: Run API Tests
  run: |
    reqx run ./api-tests \
      --env=ci \
      --output=junit \
      --output-file=results.xml
      
- name: Publish Results
  uses: mikepenz/action-junit-report@v4
  with:
    report_paths: results.xml
```

## Commands

```bash
reqx init                          # Initialize collection structure
reqx run <path>                    # Execute requests
reqx validate <path>               # Validate syntax
reqx watch <path>                  # Re-run on file changes
reqx health <path>                 # Wait for API readiness
reqx import postman <file>         # Import from Postman
reqx export openapi <dir>          # Export to OpenAPI
```

## Comparison

| Feature | reqx | Postman | Insomnia | Bruno |
|---------|------|---------|----------|-------|
| Git-native | ✅ | ❌ | ❌ | ✅ |
| Local-first | ✅ | ❌ | ❌ | ✅ |
| CLI-first | ✅ | ❌ | ❌ | ❌ |
| No account required | ✅ | ❌ | ❌ | ✅ |
| Open source | ✅ | ❌ | ❌ | ✅ |
| Plain text format | ✅ | ❌ | ❌ | ✅ |

## Documentation

- [Getting Started](https://docs.reqx.dev/getting-started)
- [File Format Reference](https://docs.reqx.dev/reference/file-format)
- [CLI Reference](https://docs.reqx.dev/reference/cli)
- [CI/CD Guide](https://docs.reqx.dev/guides/ci-cd)
- [Migration from Postman](https://docs.reqx.dev/guides/migration-postman)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE).

