# Vitro Backend

Rust/Axum API for authentication, workspaces, datasets, assets, uploads, tasks, and task runs.

Run workflows from the repository root:

```shell
bun install
bun run setup
bun run db:up
bun run db:migrate
bun run dev:server
```

Use `bun run test:server` for database-independent unit tests and `bun run test:e2e` for the database-backed flow. `db:migrate` requires `sqlx-cli` (`cargo install sqlx-cli`).
