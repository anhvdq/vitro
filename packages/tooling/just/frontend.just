# The React app, replaced by packages/web in P3.

[working-directory: 'packages/frontend']
dev-frontend:
    bun run dev

[working-directory: 'packages/frontend']
build-frontend:
    bun run build

[working-directory: 'packages/frontend']
fmt-frontend:
    bun run format

[working-directory: 'packages/frontend']
check-frontend:
    bun run lint
    bun run format:check
    bun run typecheck

[working-directory: 'packages/frontend']
test-frontend:
    bun run test
