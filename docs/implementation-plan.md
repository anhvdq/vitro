# Plan: Import design docs + roadmap from current Vitro prototype

## Context
User supplied three design docs (MVP plan, System Design v2, Code Design). Step 1 is to store them in `docs/`; step 2 is a gap analysis of the repo against them and a phased implementation plan.

## Step 1 — copy docs (do now, on branch `claude/lucid-pasteur-ciwtp9`)
| Upload | Target |
| --- | --- |
| MVP_Plan_Data_Integration_and_Derived-Data_Platform_for_Small_Businesses.md | `docs/mvp-plan.md` |
| System_Design_v2_Open-Source_Local-First_Data_Platform.md | `docs/system-design.md` |
| Code_Design_Open-Source_Data_Platform.md | `docs/code-design.md` |
Byte-identical `cp`, verify with `cmp`, commit, push. No content edits.

## Current repo vs docs (key gaps)
| Area | Repo today | Docs want |
| --- | --- | --- |
| Frontend | React 19 + TanStack | SolidJS + Vite |
| Tooling | Bun + Turborepo, single `packages/backend` crate | `just` + `cargo x`, Cargo workspace of core/adapters/api/worker/tooling |
| Domain | workspace/dataset/asset/file/task/task_run | tenant, connection, stream, dataset(source/derived), dataset_version, schema_version, derivation |
| Auth | Argon2 + JWT (`jsonwebtoken`) | Argon2id + server-side sessions in PG, invite-only |
| Queue/workers | none (task placeholders) | `events`/`deliveries` with SKIP LOCKED, worker roles |
| Engine | none | Polars 2.0 typed JSON spec |
| Storage | `object_store` (aws) | RustFS via S3 API — reusable |

## Decision: follow docs fully (user choice)
- Frontend → SolidJS + Vite (`packages/web`), React app removed once parity reached.
- Tooling → `just` + `cargo x`; Turborepo removed; Bun kept only for web/Pulumi.
- Auth → server-side sessions (JWT removed).
- AGENTS.md updated in P0 to the new stack so guidance doesn't contradict the code.

## Phased roadmap (each phase = its own PR; mirrors MVP phases)
- **P0 Foundation** (split into four PRs: P0a workspace + `just`/xtask, removing Turborepo; P0b Compose + event queue; P0c sessions auth; P0d SolidJS scaffold): root Cargo workspace with `core`, `adapters`, `api`, `worker`, `tooling` (move `packages/backend` → `packages/api`), strict workspace lints, `justfile` + xtask, remove turbo; scaffold `packages/web` (SolidJS) and `packages/infra`; docker compose (Postgres, RustFS, Mailpit); `events`/`deliveries` migration + claim loop with tests (lease expiry, retry → dead); sessions auth.
- **P1 Ingest**: rename asset→stream/connection & dataset→source dataset via migrations; `file_upload` connector (presigned upload + upload-complete → `ConnectionSyncRequested`); normalize worker (CSV/Excel via calamine, NFC, encoding_rs, header/subtotal detection); dataset versions as Parquet + manifest; schema versions & conflict handling; PII detection; LLM profiling behind `LlmClient` port.
- **P2 Derivations**: spec enum → Polars lazy compiler (whitelist), tests-before-publish, planner with stale marking/cycle check, six templates.
- **P3 UI + chat**: port remaining screens to SolidJS, delete `packages/frontend`, dataset/derivation views, vi/en i18n, chat via named tools.
- **P4**: `google_sheets` connector, MCP + read-only REST with API keys and audit log.
- Existing `task`/`task_run` placeholders replaced by `derivation_runs` in P2 (flagged for removal, not blended).

## Verification
Step 1: `cmp` each file, `git status` clean after push. Each phase: `just check`, `just test`, `just test-e2e` (Bun/Turbo commands until P0 lands), plus phase-specific integration tests (e.g. queue crash/lease reclaim, derived totals reconcile).
