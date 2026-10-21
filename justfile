# Single entry point for repository workflows. Run `just --list` for recipes.

import 'packages/tooling/just/rust.just'
import 'packages/tooling/just/web.just'

# Create missing .env files from their templates (never overwrites)
setup:
    cargo x setup

# Run the API and the web app together
dev:
    #!/usr/bin/env sh
    trap 'kill 0' EXIT
    just dev-api & just dev-web & wait

# Build release artifacts
build: build-api build-web

# Write formatting
fmt: fmt-rust fmt-web

# Every check required before handoff
check: check-rust check-web

# Unit tests (no database)
test: test-rust test-web
