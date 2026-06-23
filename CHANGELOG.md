# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Encrypted, git-friendly secret store: `reqx secret set|list|rm` keeps
  credentials in an age passphrase-encrypted `.reqx/secrets/<env>.enc`
  (committable; passphrase from `REQX_SECRET_KEY`). Referenced as
  `{{secret.NAME}}` and masked in all output.
- Concurrent execution: `--parallel N` now actually runs requests
  concurrently (it was previously accepted but ignored).
- CLI integration test suite (assert_cmd + wiremock).
- `CONTRIBUTING.md` at the repository root (moved from `.claude/`, where the
  README and Code of Conduct links could not reach it).
- `AGENTS.md` with guidance for coding agents.
- `llms.txt` ([llmstxt.org](https://llmstxt.org/) standard) pointing LLM
  consumers to the main documentation.
- This `CHANGELOG.md`.

### Changed
- Agent tooling directories (`.claude/`, `.opencode/`, `.agents/`) are now
  ignored: personal agent configuration is local-only in this public
  repository.

### Fixed
- The build now compiles: removed a phantom `[[bench]]` and a conflicting
  `From<RequestError>` impl, and fixed a use-after-move in assertion messages.
- JSONPath wildcards (`body.items[*].field`) now evaluate against every
  matched element instead of silently using only the first.
- Process exit codes follow the documented contract: `2` for execution
  errors and `4` for config errors (previously both collapsed to `1`).
