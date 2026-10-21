[working-directory: 'packages/frontend']
dev-web:
    bun run dev

[working-directory: 'packages/frontend']
build-web:
    bun run build

[working-directory: 'packages/frontend']
fmt-web:
    bun run format

[working-directory: 'packages/frontend']
check-web:
    bun run lint
    bun run format:check
    bun run typecheck

[working-directory: 'packages/frontend']
test-web:
    bun run test
