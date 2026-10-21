# Vitro Backend

Rust/Axum API for authentication, workspaces, datasets, assets, uploads, tasks, and task runs.

Run workflows from the repository root:

```shell
bun install
just setup
just db-up
just db-migrate
just dev-api
```

`just db-up` starts both the Postgres database and the object storage services (`storage`, `storage-init`) used for asset file uploads.

Use `just test-rust` for database-independent unit tests and `just test-e2e` for the database-backed flow. `db-migrate` requires `sqlx-cli` (`cargo install sqlx-cli`).

If you have an existing `.env` from before object storage support was added, `just setup` will not overwrite it — add the `OBJECT_STORAGE_*` vars from `.env.example` (endpoint, region, access key, secret key, bucket) to it manually, or the backend will panic on startup.
