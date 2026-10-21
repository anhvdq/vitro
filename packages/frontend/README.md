# Vitro Frontend

React and TanStack Start client for the Vitro data-profiling platform.

Run workflows from the repository root:

```shell
bun install
just setup
just dev-web
```

Use `just test-web`, `just check-web` and `just build-web` for verification. The frontend reads `VITE_API_BASE_URL` from `packages/frontend/.env`; the default local backend is `http://localhost:8000`.
