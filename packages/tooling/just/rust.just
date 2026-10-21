# Run the API
[working-directory: 'packages/api']
dev-api:
    cargo run -p vitro-api

build-api:
    cargo build --release -p vitro-api

fmt-rust:
    cargo fmt --all

check-rust:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo check --workspace --all-targets --all-features

test-rust:
    cargo test --workspace --lib --bins

# Event queue and claim loop tests; needs DATABASE_URL to a Postgres the test user can create databases on
test-db:
    cargo test -p vitro-adapters -p vitro-worker --tests

# Database-backed API flow; needs `just db-up db-migrate`
test-e2e:
    cargo test -p vitro-api --test test_e2e

# Start Postgres and object storage
db-up:
    docker compose -f packages/infra/compose.yaml --env-file packages/api/.env up -d

db-down:
    docker compose -f packages/infra/compose.yaml --env-file packages/api/.env down

# Apply SQL migrations (requires sqlx-cli)
[working-directory: 'packages/api']
db-migrate:
    sh scripts/migrate.sh
