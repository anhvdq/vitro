[working-directory: 'packages/web']
dev-web:
    bun run dev

[working-directory: 'packages/web']
build-web:
    bun run build

[working-directory: 'packages/web']
fmt-web:
    bun run format

[working-directory: 'packages/web']
check-web:
    bun run lint
    bun run format:check
    bun run typecheck

[working-directory: 'packages/web']
test-web:
    bun run test

# Rewrite the OpenAPI snapshot from the API, then regenerate the web client types
contracts:
    UPDATE_CONTRACTS=1 cargo test -p vitro-api --test openapi
    cd packages/web && bun run generate

# Fail if the generated client types differ from the OpenAPI snapshot
check-contracts:
    #!/usr/bin/env sh
    set -eu
    tmp=$(mktemp)
    (cd packages/web && bunx openapi-typescript ../contracts/openapi.json -o "$tmp" >/dev/null)
    diff -q "$tmp" packages/web/src/api/schema.d.ts >/dev/null || {
        echo "packages/web/src/api/schema.d.ts is out of date; run 'just contracts'" >&2
        exit 1
    }
