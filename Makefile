.PHONY: help dev build build-release test lint clean install mock-api mock-api-down mock-api-logs mock-api-rebuild

# Default target
help:
	@echo "reqx - CLI-first API client"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Development:"
	@echo "  dev          Start development environment (Docker)"
	@echo "  dev-shell    Open shell in dev container"
	@echo "  dev-watch    Run cargo watch for auto-rebuild"
	@echo ""
	@echo "Build:"
	@echo "  build        Build debug binary"
	@echo "  build-release Build release binary"
	@echo "  build-static Build static binary (musl)"
	@echo "  build-windows Build Windows binary (cross-compile)"
	@echo "  build-all    Build for all platforms"
	@echo ""
	@echo "Mock API:"
	@echo "  mock-api     Start mock API server (Docker)"
	@echo "  mock-api-down Stop mock API server"
	@echo "  mock-api-logs View mock API logs"
	@echo ""
	@echo "Test:"
	@echo "  test         Run unit tests"
	@echo "  test-all     Run tests on all distros (Docker)"
	@echo "  test-integration Run integration tests"
	@echo ""
	@echo "Quality:"
	@echo "  lint         Run linters (fmt, clippy, audit)"
	@echo "  fmt          Format code"
	@echo "  coverage     Generate code coverage report"
	@echo ""
	@echo "Other:"
	@echo "  clean        Clean build artifacts"
	@echo "  install      Install locally"
	@echo "  docs         Generate documentation"

# ============================================================================
# Development
# ============================================================================

dev:
	docker compose -f .docker/dev/docker-compose.yml up -d
	docker compose -f .docker/dev/docker-compose.yml exec dev bash

dev-shell:
	docker compose -f .docker/dev/docker-compose.yml exec dev bash

dev-watch:
	docker compose -f .docker/dev/docker-compose.yml exec dev cargo watch -x "build" -x "test"

dev-down:
	docker compose -f .docker/dev/docker-compose.yml down

# ============================================================================
# Build
# ============================================================================

build:
	cargo build

build-release:
	cargo build --release

build-static:
	docker build -f .docker/build/Dockerfile.alpine -t reqx:alpine .
	docker create --name reqx-extract reqx:alpine
	docker cp reqx-extract:/reqx ./dist/reqx-linux-x64
	docker rm reqx-extract
	@echo "Binary: ./dist/reqx-linux-x64"

build-windows:
	docker build -f .docker/build/Dockerfile.cross-win -t reqx:windows --target export -o dist .
	@echo "Binary: ./dist/reqx-win64.exe"

build-all: build-static build-windows
	@echo ""
	@echo "Built binaries:"
	@ls -la ./dist/

# ============================================================================
# Mock API (for testing)
# ============================================================================

mock-api:
	docker compose -f docker-compose.test.yml up -d
	@echo "Mock API running at http://localhost:3333"
	@echo "Health check: http://localhost:3333/health"

mock-api-down:
	docker compose -f docker-compose.test.yml down

mock-api-logs:
	docker compose -f docker-compose.test.yml logs -f

mock-api-rebuild:
	docker compose -f docker-compose.test.yml up -d --build

# ============================================================================
# Test
# ============================================================================

test:
	cargo test

test-verbose:
	cargo test -- --nocapture

test-integration:
	cargo test --features integration

test-all:
	mkdir -p .docker/test/results
	docker compose -f .docker/test/docker-compose.test.yml up --build --abort-on-container-exit
	@echo ""
	@echo "Results:"
	@ls -la .docker/test/results/

# ============================================================================
# Quality
# ============================================================================

lint: fmt-check clippy audit

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

audit:
	cargo audit

coverage:
	cargo llvm-cov --html --output-dir coverage
	@echo "Coverage report: coverage/html/index.html"

# ============================================================================
# Documentation
# ============================================================================

docs:
	cargo doc --no-deps --open

# ============================================================================
# Install
# ============================================================================

install:
	cargo install --path .

install-local:
	cargo build --release
	cp target/release/reqx ~/.local/bin/

# ============================================================================
# Clean
# ============================================================================

clean:
	cargo clean
	rm -rf dist/
	rm -rf coverage/
	rm -rf .docker/test/results/

# ============================================================================
# Release
# ============================================================================

release:
ifndef VERSION
	$(error VERSION is required. Usage: make release VERSION=1.0.0)
endif
	@echo "Preparing release v$(VERSION)..."
	sed -i 's/^version = .*/version = "$(VERSION)"/' Cargo.toml
	cargo build --release
	git add Cargo.toml Cargo.lock
	git commit -m "chore: release v$(VERSION)"
	git tag -a "v$(VERSION)" -m "Release v$(VERSION)"
	@echo ""
	@echo "Release v$(VERSION) prepared. Run 'git push --follow-tags' to publish."
