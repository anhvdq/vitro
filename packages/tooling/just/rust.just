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

# Database-backed API flow; needs `just db-up db-migrate`
test-e2e:
    cargo test -p vitro-api --test test_e2e

# Start Postgres and object storage
[working-directory: 'packages/api']
db-up:
    docker compose up -d

[working-directory: 'packages/api']
db-down:
    docker compose down

# Apply SQL migrations (requires sqlx-cli)
[working-directory: 'packages/api']
db-migrate:
    sh scripts/migrate.sh
