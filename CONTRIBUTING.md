# Contributing to reqx

Thank you for your interest in contributing to reqx! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Docker (for development environment)
- Git

### Development Setup

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/YOUR_USERNAME/reqx.git
   cd reqx
   ```

2. Start the development environment:
   ```bash
   make dev
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Making Changes

### Branching

- Create a feature branch from `develop`:
  ```bash
  git checkout develop
  git pull origin develop
  git checkout -b feature/your-feature-name
  ```

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Examples:
```
feat: add OAuth2 authentication support
fix: handle timeout errors gracefully
docs: update CLI reference for run command
```

### Code Style

- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add tests for new functionality
- Update documentation as needed

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests
cargo test --features integration
```

## Pull Requests

1. Ensure all tests pass
2. Update documentation if needed
3. Fill out the PR template
4. Request review from maintainers

### PR Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Changelog entry added (for user-facing changes)
- [ ] Conventional commit message used
- [ ] `cargo fmt` and `cargo clippy` pass

## Reporting Issues

### Bug Reports

Please include:
- reqx version (`reqx --version`)
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Relevant .reqx file content (if applicable)

### Feature Requests

Please include:
- Use case description
- Proposed solution (if any)
- Alternative solutions considered

## License

By contributing, you agree that your contributions will be licensed under the MPL-2.0 license.

## Questions?

Open a discussion on GitHub or reach out to maintainers.

Thank you for contributing! 🎉
