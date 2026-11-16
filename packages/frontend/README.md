# Vitro Frontend

React and TanStack Start client for the Vitro data-profiling platform.

Run workflows from the repository root:

```shell
bun install
just setup
just dev-frontend
```

Use `just test-frontend`, `just check-frontend` and `just build-frontend` for verification. The frontend reads `VITE_API_BASE_URL` from `packages/frontend/.env`; the default local backend is `http://localhost:8000`.
