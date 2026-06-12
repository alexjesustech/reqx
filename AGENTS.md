# reqx — agent guide

CLI-first API client for developers, written in Rust. Git-native, local-first,
privacy-focused: API collections are plain-text `.reqx` (TOML) files versioned
with Git. Licensed under MPL 2.0.

## Build, test, lint

- `cargo build` / `cargo test`
- `make dev` — Docker-based development environment
- `cargo clippy` (config in `clippy.toml`) and `cargo fmt` (config in
  `rustfmt.toml`) must pass before a PR
- Dependency policy: `deny.toml` (`cargo deny check`)

## Layout

- `src/cli/` — command-line interface (`init`, `run`, `validate`, `watch`,
  `health`, `import`, `export`)
- `src/parser/` — `.reqx` file format (TOML) parsing
- `src/runtime/` — request execution, assertions, variable capture
- `src/http/` — HTTP client layer
- `src/output/` — output formats (human, JUnit, TAP, JSON) and exit codes
- `src/config/` — configuration and environments
- `examples/` — sample collections

## Conventions

- Conventional Commits (`feat`/`fix`/`docs`/`chore`/`refactor`/`test`);
  imperative subject ≤ ~72 chars.
- Work on feature branches and open a PR; never commit directly to `main`.
- Update `CHANGELOG.md` (`[Unreleased]`) with any user-facing change.
- Agent tooling directories (`.claude/`, `.opencode/`, `.agents/`) are
  local-only and must never be committed to this public repository.
- See `CONTRIBUTING.md` for the full contributor workflow.
