# Code Design: Open-Source Data Platform

Oct 10, 2026 · @Duc Quoc Anh Van

## Summary and scope

This is the code design for the platform in System Design v2, organised as one monorepo. The backend is Rust, the web app is SolidJS with TypeScript, and infrastructure is Docker Compose plus Pulumi. The repo tooling follows the setup of the macro-inc/macro repository: a Cargo workspace, `just` recipes as the single entry point, a Rust `xtask` launcher for logic, strict clippy, ast-grep rules for boundaries the compiler cannot see, Biome and oxlint for TypeScript, and an offline sqlx query cache. I left out Nix, Doppler, Turborepo and the workspace-hack crate, for the reasons in the toolchain section. Anything I did not check against current documentation is marked as unverified.

Seven decisions shape it.

- **One repo, one entry point.** `just setup`, `just dev`, `just check`, `just test`, `just build` and `just deploy-pilot` at the root cover everything. Recipes stay short and call `cargo x` (the xtask launcher) when they need logic.
- **A package is a deployable or a shared concept.** There are eight packages. Four are deployables that each run as their own instance (web, api, worker, infra), and four are shared packages that hold concepts used by two or more deployables (core, adapters, contracts, tooling).
- **Promote on second use.** Code that one deployable needs lives inside it. When a second deployable needs it, it moves into the matching shared package. Layers live inside packages as modules.
- **The compiler enforces most boundaries.** In Rust a crate can only use what its manifest declares, so an undeclared import does not build. ast-grep rules, `cargo deny` and clippy cover what the compiler cannot see, such as layers inside a crate.
- **Strict is the default for every package.** Rust uses workspace lints with warnings as errors. TypeScript uses the strictest `tsconfig` flags. `cargo x check-repo` fails any package that opts out.
- **Rust types are the source of truth for contracts.** Event payloads and API types derive JSON Schema and OpenAPI. A task writes them to `contracts/`, CI fails if the committed files differ, and the web app generates its TypeScript client from them.
- **The backend design carries over.** A pure domain, a pure planner, ports with one default adapter each, a worker loop with explicit transactions, and classified errors. Rust makes some of these rules compile-time facts, for example a handler cannot forget to complete its delivery.

## Toolchain and principles

The table shows what each tool does and what it does not do, so no two tools overlap.

| Tool | Does | Does not |
| --- | --- | --- |
| Cargo workspace | Builds and tests every Rust package with one `Cargo.lock`, resolver 2 and edition 2024, shared `[workspace.dependencies]` and shared `[workspace.lints]` | Build the web app or the infrastructure |
| rust-toolchain.toml | Pins the Rust toolchain and the rustfmt and clippy components | Pin other tools |
| just | The single entry point. Recipes are split into modules under `packages/tooling/just/` (check, rust, web, infra) | Hold logic: anything longer than a few lines becomes an xtask command |
| xtask (`cargo x`) | A Rust binary with the repo logic: change detection, contract generation, migrations, package scaffolding, repo checks | Replace Cargo or just |
| rustfmt, taplo | Format Rust and TOML | Lint |
| clippy | Lints with `-D warnings` and `-D clippy::disallowed_methods`, configured in `clippy.toml` | Check boundaries between modules |
| cargo-deny | Checks licences, advisories, banned crates and duplicate versions, configured in `deny.toml` | Check code |
| ast-grep | Structural rules with stable IDs (RS- for Rust, FE- for the web app) for layers, SQL placement and `mod.rs` | Replace clippy |
| sqlx offline cache | Commits `.sqlx/` so queries are checked at compile time without a database (`SQLX_OFFLINE=true` in CI and image builds) | Run migrations |
| Bun | Installs and runs the TypeScript packages (web and the Pulumi program) | Manage Rust |
| Biome and oxlint | Biome formats and lints TypeScript. oxlint adds fast extra lints and is pinned through `bunx` | Typecheck: `tsc --noEmit` does that |
| Vitest, Playwright | Component tests and end-to-end flows for the web app | Test the backend |
| Pulumi | Defines the infrastructure in TypeScript | Run the application: Compose does that |
| SOPS and age | Encrypt per-environment secret files | Inject secrets at run time beyond `sops exec-env` |

What I left out of the setup I studied, and why.

- **Nix.** You chose to skip it. Tool versions are pinned in `rust-toolchain.toml` and `.tool-versions` instead, which is weaker for reproducibility but needs nothing to learn.
- **Doppler.** I read your answer "skip this tool" as dropping Doppler and keeping SOPS and age. Tell me if you meant something else.
- **Turborepo.** Macro does not use it either. `just` and `cargo x` order the tasks, and the Rust and TypeScript sides barely depend on each other.
- **Workspace-hack crate (hakari).** It speeds up builds in very large workspaces. Eight packages do not need it yet.
- **One xtask crate per command.** Macro splits its xtask commands into crates. I start with one crate and split it when build time hurts.

Rules of the repo

- One repository. One lockfile per ecosystem (`Cargo.lock` and `bun.lock`), both committed.
- Packages live only directly under `packages/`, side by side in one flat folder.
- Everything runs from the root with `just <recipe>`. A recipe that grows logic moves into `cargo x`.
- A package declares what it needs in its own manifest, and no package reaches into another's folder by relative path.
- Generated code and generated contract files are never edited by hand. A task produces them and CI checks that they are current.
- Nothing is published. Every package is private (`publish = false`, `"private": true`), and deployment is by git SHA.
- Dependencies are declared once in `[workspace.dependencies]` and inherited with `workspace = true`.

## Design principles

These rules settle most day-to-day code questions.

- **The domain is pure.** It does no I/O and never reads the clock or generates IDs itself. A `Clock` and an `IdGenerator` are passed in, so every test is deterministic. Clippy's `disallowed_methods` bans `Utc::now` and `Ulid::new` outside the two adapters that wrap them.
- **Decisions are separate from actions.** A pure function returns a list of actions such as `MarkStale` and `RequestRun`, and an adapter executes them. The planner is the model case.
- **Use the model only for judgment.** The LLM profiles datasets, drafts derivation specs and words answers. Routing, retries, status handling, state transitions and arithmetic are plain code.
- **Ports are small and owned by the application.** A port is a trait with the fewest methods that serve the use cases. It takes and returns domain types and leaks no SDK types. Each has one default adapter and a conformance test suite.
- **One transaction per state change.** A handler writes its results, its next events and its own completion together, or none of them.
- **Fail loud.** Every error is classified and then retried, parked as dead with an alert, or surfaced to the owner as blocked. There is no `let _ =` on a `Result`, no `unwrap` outside tests, and no skipped row without a report.
- **No framework where a function will do.** No dependency-injection framework, no ORM and no event library. Wiring is explicit in one composition root per entrypoint.
- **Closed sets over open extensibility.** Derivation steps, expression functions and connector types are fixed enums and a fixed registry. Extending one is a code change with tests, never configuration that runs arbitrary code.
- **Make illegal states unrepresentable.** Use newtypes for IDs, enums for states and private constructors for guarded types, so the compiler rejects the mistake before a test has to.
- **Tests state intent.** A test name and a one-line comment say why the behaviour matters, so a test fails when the business rule changes and not only when the code does.

Conventions: `snake_case` for Rust and `camelCase` for TypeScript, as each ecosystem does. Libraries return typed errors with `thiserror`, and only a binary's `main` may use `anyhow`. One formatter and one linter per language, enforced in CI. Each `foo.rs` has its submodules in `foo/`, and `mod.rs` files are not used.

## Repository structure

Root files drive everything, and each folder under `packages/` is one package with its own toolchain. There are eight packages, named with plain nouns. Four are deployables and four are shared packages. Layers live inside packages as modules and are not packages themselves.

```
.
├── Cargo.toml                 workspace root: members, [workspace.dependencies], [workspace.lints]
├── Cargo.lock
├── rust-toolchain.toml        pinned toolchain, rustfmt and clippy
├── clippy.toml  deny.toml  rustfmt.toml  .taplo.toml  sgconfig.yml
├── .cargo/config.toml         alias: cargo x = run the xtask launcher
├── .sqlx/                     committed offline query cache
├── justfile                   imports the modules in packages/tooling/just/
├── package.json  bun.lock     Bun workspace for web and the Pulumi program only
├── biome.json
├── .tool-versions             pinned just, Bun, Pulumi, sops, age and ast-grep
├── CODEOWNERS
├── docs/                      ADRs, runbooks, design documents; checked by check-repo, not a package
├── .github/workflows/         CI (or the equivalent for the chosen host)
└── packages/                  flat: no nesting
    ├── web/        deployable   web app, generated API client, Playwright flows       TypeScript, SolidJS, Vite
    ├── api/        deployable   REST and MCP service, sessions, built-in auth          Rust, axum
    ├── worker/     deployable   worker roles, scheduler, operator commands, readers, connectors   Rust, tokio
    ├── infra/      deployable   Compose, Traefik, observability, Pulumi program        YAML, TypeScript
    ├── core/       shared       domain, ports, use cases both deployables need          Rust
    ├── adapters/   shared       PostgreSQL, object store, Polars engine, LLM gateways, mailer   Rust
    ├── contracts/  shared       event schemas, OpenAPI snapshot, message catalogs       JSON
    └── tooling/    shared       xtask launcher, just modules, ast-grep rules            Rust, just
```

Inside a package, the native tool's files sit beside the sources.

```
packages/worker/                         packages/web/
  Cargo.toml    manifest                   package.json  tsconfig.json  vite.config.ts
  Dockerfile    builds the image           src/   routes/  features/  shared/  api/  i18n/
  README.md                                e2e/   Playwright flows
  src/                                     tests/ component tests
    lib.rs  main.rs  claim_loop.rs
    handlers.rs  handlers/
    connectors.rs  connectors/
    formats.rs  formats/  normalize.rs
  tests/        integration tests
```

Import names. Each Rust package is a crate named `vitro-<folder>`, and its library is imported as `vitro_<folder>`. A package that uses another writes `use vitro_core::domain::Dataset;` and never a relative path across packages. The deployables are a library plus a thin binary: `src/lib.rs` holds the code and `src/main.rs` only builds settings and calls it, so tests import the library.

| Package folder | Crate | Import name | Contains |
| --- | --- | --- | --- |
| core/src/ | vitro-core | `vitro_core` | `domain`, `ports`, `usecases` |
| adapters/src/ | vitro-adapters | `vitro_adapters` | `postgres`, `objectstore`, `engine`, `llm`, `mail` |
| api/src/ | vitro-api | `vitro_api` | `routes`, `usecases`, `auth` |
| worker/src/ | vitro-worker | `vitro_worker` | `claim_loop`, `handlers`, `connectors`, `formats`, `normalize` |
| tooling/src/ | vitro-tooling | `vitro_tooling` | `commands` (the `cargo x` subcommands) |

The path comments in the code samples below start with the package name, so `// worker/claim_loop.rs` is `vitro_worker::claim_loop`, in the file `worker/src/claim_loop.rs`. Workspace members are listed explicitly in `Cargo.toml`, because `web`, `infra` and `contracts` are not crates. The crate root of each package is its `src/` folder, and Cargo exposes the crate as `vitro_<folder>` with no link tree or extra configuration.

## Package catalog

There are eight packages. A package is either a deployable, which builds an image or defines a deployment and runs as its own instance, or a shared package, which holds a concept used by two or more deployables. Code that one deployable needs lives inside it, and it moves into a shared package when a second deployable needs it. Phase numbers are the four phases of the MVP plan, and the build order is in the system design document.

| Package | Kind | Toolchain | Contains | Depends on | Phase |
| --- | --- | --- | --- | --- | --- |
| web | Deployable | TypeScript, SolidJS, Vite, Playwright | The web app, the generated API client with error mapping, message-key types and the end-to-end flows | contracts | 4 |
| api | Deployable | Rust, axum, utoipa | REST and MCP, sessions, presigned uploads, API use cases and the built-in authentication adapter. Writes the OpenAPI snapshot | core, adapters | 4 |
| worker | Deployable | Rust, tokio | The worker loop and handlers, the roles (connector, normalize, profile, derive, notify, scheduler, deleter), operator commands, the xlsx and csv readers, and the file\_upload and google\_sheets connectors with their Google client code. One image, run as a separate instance per role | core, adapters | 2 |
| infra | Deployable | Docker Compose, Traefik, Prometheus, Grafana, Loki, Pulumi (TypeScript) | The deployment of every instance, including PostgreSQL and RustFS: profiles core, obs, llm and mail, and cloud resources later | Nothing at build time | 1 |
| core | Shared | Rust, serde, schemars | `domain` (models, states, events, planner, specs, normalize rules), `ports` (traits, in-memory fakes, conformance suites) and `usecases` that both api and worker need | Nothing in the workspace | 1 to 3 |
| adapters | Shared | Rust, sqlx, an S3 client, Polars, reqwest, lettre | PostgreSQL repositories, event store, unit of work and migrations, the object store, the Polars compiler, runner and queries, the LLM gateways and the mailer. Each is used by both api and worker | core | 1 to 4 |
| contracts | Shared | JSON Schema, OpenAPI, JSON | Generated event schemas, the generated OpenAPI snapshot and the Vietnamese and English message catalogs. Data only | Nothing | 1 |
| tooling | Shared | Rust (xtask), just, ast-grep | The `cargo x` commands, the just modules and the ast-grep rules. Not shipped in any image | Nothing at build time | 1 |

Two details are easy to miss. Event payload types now live in core and are the source of the files in contracts, so core does not depend on contracts. Message catalogs are hand-written in contracts and read by adapters and web.

Why this grouping

- **Deployables come first.** web, api, worker and infra are the things you start, scale and release. Each has one Dockerfile, except infra, which defines how all the instances run together.
- **Each deployable keeps what only it uses.** The built-in authentication adapter is used only by the API, so it lives in api. The file readers, the connectors and the Google client code are used only by the worker, so they live in worker. The API image therefore carries no connector code and never handles connector credentials.
- **Shared packages exist because two deployables need the same concept.** core is the domain language, adapters holds the implementations both api and worker use, contracts holds what api, worker and web must agree on, and tooling holds the repo commands and rules.
- **Promotion works in both directions.** If the API later needs a file reader, the reader moves from worker into adapters. If only one deployable still uses something in a shared package, it moves back.
- **Worker roles stay in one package.** They share the loop, the handlers and most of the code, and each role runs as its own instance. They split into separate deployables only if their dependency profiles or release cadence diverge.
- **The trade-off is coarse rebuilds.** A change to adapters rebuilds and retests everything that depends on it. Cargo's incremental builds and separate `test` and `test-integration` recipes soften that, and splitting adapters is the first thing to revisit if it hurts.

## Package dependency rules

Dependencies point inward. Deployables depend on shared packages, shared packages never depend on deployables, and deployables never depend on each other. The Rust and TypeScript packages meet only through contracts.

&#91;embedded content: package graph · 4 deployables, 4 shared\]

| Package | May depend on | Must not depend on |
| --- | --- | --- |
| core | Only pure libraries: serde, schemars, thiserror, chrono, ulid, async-trait, secrecy. Inside it, `usecases` may use `ports` and `domain`, `ports` may use `domain`, and `domain` imports nothing from its siblings | adapters, deployables, and any crate that does I/O (sqlx, tokio runtime features, reqwest, polars, S3 clients) |
| adapters | `vitro_core`, but only `domain` and `ports`, plus third-party libraries and the message catalogs from contracts | `vitro_core::usecases` and deployables. One adapter module never imports another |
| api | `vitro_core`, `vitro_adapters` | `vitro_worker` and web |
| worker | `vitro_core`, `vitro_adapters` | `vitro_api` and web. One connector module never imports another |
| web | contracts (generated client and catalogs) | Any Rust package |
| infra | Nothing at build time. It refers to the other deployables' images at run time | Any code package |
| contracts | Nothing in the workspace | Any code package |
| tooling | May read the workspace (cargo metadata, the contracts folder) | Being a dependency of any code package |

Enforcement, from cheapest to strongest:

1. Cargo refuses a dependency cycle and refuses any `use` of a crate the manifest does not declare. Because core's manifest has no `sqlx` or `polars`, the domain cannot touch a database or a dataframe.
2. `cargo deny` bans crates by name. It keeps I/O crates out of core and keeps `anyhow` out of core and adapters, with a `wrappers` allow-list for the exceptions.
3. ast-grep rules (RS- IDs, listed under strict checks) enforce the layers inside a crate, which Cargo cannot see, and the rule that SQL appears only in `vitro_adapters::postgres`.
4. `cargo x check-repo` compares each package's declared dependencies with this table.
5. Each deployable's image builds from the workspace with only its declared crates compiled in, so the API binary cannot contain a connector.

Entity types in the domain are plain structs and enums with newtype IDs. Types that cross a boundary derive `Serialize`, `Deserialize` and `JsonSchema` only where the schema is part of a contract: derivation specs, event payloads and API schemas.

## Package contract and conventions

Every package satisfies the same contract, and `cargo x check-repo` enforces it in CI.

| Required item | Rule |
| --- | --- |
| Manifest | Rust: `Cargo.toml` named `vitro-<folder>` with `edition.workspace = true`, `publish = false` and `[lints] workspace = true`. TypeScript: `package.json` named `@vitro/<folder>`, `"private": true`, and workspace dependencies as `workspace:*` |
| Source folder | Rust sources in `src/`, with `lib.rs` (and `main.rs` for a deployable). The web app has `src/` too |
| Recipes | The package is reachable from the root through its language module in `packages/tooling/just/`. `just check` and `just test` cover it without extra wiring |
| Strict checks | Inherits the workspace lints (Rust) or extends the shared strict `tsconfig` (TypeScript). A package cannot override a lint level downward |
| Dockerfile | Deployables that run as a container only. It builds from the repo root |
| README | Purpose, what it may depend on, how to run its tests, and for the worker the credential scopes each connector needs |
| Tests | A `tests/` folder (integration) and `#[cfg(test)]` modules (unit) for Rust, or `tests/` and `e2e/` for web |
| Codegen | Generated files sit in a folder marked generated and are listed in `.gitattributes` as `linguist-generated` |

The generator `cargo x new-package <name> --kind deployable|shared` creates a package that already satisfies this contract and starts strict. Adding a package also updates the catalog and the dependency table, and `check-repo` fails if the two documents and the manifests disagree.

## Task runner: just and xtask

One `justfile` at the root imports modules from `packages/tooling/just/`, as Macro does. Recipes are thin, and each calls a native tool or a `cargo x` command.

| Recipe | What it does |
| --- | --- |
| `just setup` | Installs the pinned toolchain, runs `bun install --frozen-lockfile`, installs the git hook and runs code generation |
| `just dev` | Starts PostgreSQL, RustFS and Mailpit through infra, applies migrations, then runs api and worker with `cargo watch` and web with `vite` |
| `just check` | The fast tier, for every save or commit: formatting, clippy, ast-grep, Biome, oxlint and `tsc --noEmit`, limited to the packages the change touches |
| `just check full` | The full tier, for CI: everything in the fast tier on every package, plus `cargo deny`, the sqlx cache check, the contract snapshot check and `cargo x check-repo` |
| `just test` | Unit, property, contract and golden tests with fakes, through `cargo nextest` for Rust and Vitest for web |
| `just test-integration` | Tests that need containers (PostgreSQL and RustFS) |
| `just test-e2e` | Playwright flows against the Compose stack |
| `just contracts` | Regenerates `contracts/` from the Rust types (`cargo x contracts`), then the web client |
| `just migrate` | Applies migrations (`cargo x migrate`), the same command deployment runs |
| `just build` | Builds the images, tagged with the git SHA |
| `just deploy-pilot` | Runs the full check and the tests, builds and pushes the images, then updates the pilot VM |

Change-scoped routing. `cargo x changed` compares the working tree with the merge base and prints which packages are affected, using `cargo metadata` for the Rust dependency graph and the Bun workspace for TypeScript. The fast tier runs only for those packages. The full tier ignores the result and runs everything. Cargo already skips unchanged Rust work, so no task cache is needed.

The launcher is one crate in tooling (`vitro-tooling`, binary `x`), reached through the alias `cargo x` in `.cargo/config.toml`. Commands are modules under `src/commands/`: `changed`, `check_repo`, `contracts`, `migrate`, `new_package`, `env_example`, `docker_context`. A command that is only a wrapper stays in the justfile.

## Strict checks by default

Every package inherits the strictest settings its toolchain offers. A package cannot weaken these settings.

**Rust.** The compiler already checks types strictly, so strict means lints.

```toml
# Cargo.toml (workspace root)
[workspace.lints.rust]
unsafe_code = "forbid"
unused_must_use = "deny"
rust_2018_idioms = { level = "warn", priority = -1 }

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
dbg_macro = "deny"
print_stdout = "deny"
let_underscore_must_use = "deny"
```

```toml
# clippy.toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
disallowed-methods = [
  { path = "chrono::Utc::now", reason = "use the Clock port" },
  { path = "ulid::Ulid::new", reason = "use the IdGenerator port" },
  { path = "std::env::var", reason = "read configuration only in Settings::load" },
]
```

CI runs `cargo clippy --workspace --all-targets -- -D warnings -D clippy::disallowed_methods`. The two adapters that wrap the clock and the ID generator carry one `#[expect(clippy::disallowed_methods, reason = "...")]` each, so every exception is visible and has a reason.

**TypeScript.** The shared base `tsconfig` turns on `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noImplicitOverride` and `noFallthroughCasesInSwitch`, and `tsc --noEmit` is the typecheck. Biome and oxlint forbid `any` and non-null assertions. Branching on variants uses `ts-pattern` with `.exhaustive()`, and fallible code returns `neverthrow` results at feature boundaries.

**Structural rules.** ast-grep rules carry stable IDs, so a failure message points to one line in this table. The IDs are my proposal.

| ID | Rule |
| --- | --- |
| RS-001 | `vitro_core::domain` does not `use` `ports`, `usecases` or any crate outside the pure set |
| RS-002 | `vitro_core::ports` does not `use` `usecases` |
| RS-003 | `vitro_adapters` does not `use` `vitro_core::usecases`, and one adapter module does not `use` a sibling module |
| RS-004 | One connector module in the worker does not `use` another |
| RS-005 | No `mod.rs`: a module is `foo.rs` with its children in `foo/` |
| RS-006 | `sqlx::query` and `sqlx::query_as` (and their macros) appear only in `vitro_adapters::postgres` |
| RS-007 | `anyhow` is used only in `main.rs` files and in tooling |
| FE-001 | A feature folder does not import another feature folder, and shared code is imported from `shared/` |
| FE-002 | No `fetch` outside `src/api/` |
| FE-003 | User-visible text in JSX comes from the i18n catalog, not a string literal |
| FE-004 | A component does not destructure its props in the parameter list, because that loses reactivity |

A rule is added only when a mistake has happened or is likely, and each rule has a fixture that proves it catches a violation.

## Domain model

Entities are plain structs with newtype IDs, so a `DatasetId` cannot be passed where a `DerivationId` is expected. The compiler checks this, so no test is needed for it. Events and specs also derive serde and JSON Schema, because they are contracts.

```rust
// core/domain/models.rs
macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(#[schemars(with = "String")] Ulid);
    };
}
id_type!(TenantId);
id_type!(ConnectionId);
id_type!(StreamId);
id_type!(DatasetId);
id_type!(DerivationId);
id_type!(ColumnId); // stable across renames

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DatasetKind { Source, Derived }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DatasetState { Fresh, Stale, Running, Failed, Blocked }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputRef { pub dataset_id: DatasetId, pub version: u32 } // one edge of a version's lineage

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatasetVersion {
    pub dataset_id: DatasetId,
    pub number: u32,
    pub schema_version: u32,
    pub content_hash: String, // computed after sorting by the declared keys
    pub row_count: u64,
    pub object_prefix: String,
    pub inputs: Vec<InputRef>, // empty for source datasets
}

#[derive(Clone, Debug)]
pub struct Dataset {
    pub id: DatasetId,
    pub tenant_id: TenantId,
    pub kind: DatasetKind,
    pub state: DatasetState,
    pub current: Option<DatasetVersion>,
}
```

A dataset is fresh exactly when its recorded inputs equal the current input versions. That is a pure function, so the planner and the tests share it.

```rust
pub fn inputs_current(v: &DatasetVersion, current: &BTreeMap<DatasetId, u32>) -> bool {
    v.inputs.iter().all(|i| current.get(&i.dataset_id) == Some(&i.version))
}
```

Events share one envelope, and each type is a payload struct that implements one trait. Payload fields hold IDs, versions and URIs only, never rows or personal data. A unit test walks the generated JSON Schema of every registered payload and rejects any property of another kind.

```rust
// core/domain/events.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId, // ULID
    pub kind: String,
    pub version: u32,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
    pub correlation_id: String,
    pub causation_id: Option<EventId>,
    pub data: serde_json::Value, // validated against the payload type for (kind, version)
}

pub trait EventPayload: Serialize + DeserializeOwned + JsonSchema {
    const KIND: &'static str;
    const VERSION: u32;
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DatasetVersionPublished {
    pub dataset_id: DatasetId,
    pub version: u32,
    pub kind: DatasetKind,
    pub snapshot_uri: String,
}
impl EventPayload for DatasetVersionPublished {
    const KIND: &'static str = "DatasetVersionPublished";
    const VERSION: u32 = 1;
}
```

The payload structs are the source of truth. One macro lists them in `core::domain::events::catalog`, and `cargo x contracts` writes one JSON Schema per entry to `contracts/events/`. The schemas are generated from the types, never written by hand. A change to a payload that is not a compatible addition must bump its `VERSION`, and a check compares the new schema with the committed one.

State machine. Transitions are data, and one function enforces them. An invalid transition is a programming error, so it is permanent and raised loudly.

| From | To | Caused by |
| --- | --- | --- |
| fresh | stale | An upstream dataset published a new version |
| stale | running | The runner took the derivation's lease |
| stale | blocked | The spec check found a removed or retyped column |
| running | fresh | The run succeeded and its tests passed |
| running | failed | The run failed after retries |
| running | stale | An input published during the run |
| running | blocked | The spec check failed when the run started |
| failed | stale | A retry was requested, or an upstream changed |
| blocked | stale | The owner confirmed a mapping or edited the derivation |

```rust
// core/domain/states.rs
pub fn transition(from: DatasetState, to: DatasetState) -> Result<DatasetState, InvalidTransition> {
    use DatasetState::*;
    match (from, to) {
        (Fresh, Stale)
        | (Stale, Running | Blocked)
        | (Running, Fresh | Failed | Stale | Blocked)
        | (Failed, Stale)
        | (Blocked, Stale) => Ok(to),
        _ => Err(InvalidTransition { from, to }),
    }
}
```

Source datasets stay fresh once they have a version. A failed sync is recorded on the stream, not on the dataset, so a connector outage never makes good data look broken.

## Ports and adapters

Ports are traits in `core::ports`. They take and return domain types, so no SDK type crosses the boundary. They use `#[async_trait]` and are held as `Arc<dyn Port>`, because native `async fn` in traits does not support trait objects. Four examples show the style.

```rust
// core/ports/object_store.rs
#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn put(&self, key: &str, body: Bytes, content_type: Option<&str>) -> Result<(), PlatformError>;
    async fn get(&self, key: &str) -> Result<Bytes, PlatformError>;
    async fn exists(&self, key: &str) -> Result<bool, PlatformError>;
    async fn presign_put(&self, key: &str, expires: Duration) -> Result<String, PlatformError>;
    async fn delete_prefix(&self, prefix: &str) -> Result<u64, PlatformError>;
    fn uri(&self, key: &str) -> String; // the location the transform engine reads
}

// core/ports/events.rs
#[async_trait]
pub trait EventSink: Send {
    async fn append(&mut self, event: Event, consumers: &[&str], opts: AppendOptions) -> Result<(), PlatformError>;
}

#[async_trait]
pub trait DeliveryQueue: Send + Sync {
    async fn claim(&self, consumer: &str, lease: Duration, limit: u32) -> Result<Vec<Delivery>, PlatformError>;
    async fn retry(&self, d: &Delivery, run_after: DateTime<Utc>, reason: &str) -> Result<(), PlatformError>;
    async fn bury(&self, d: &Delivery, reason: &str) -> Result<(), PlatformError>;
}

// core/ports/llm.rs
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn profile_dataset(&self, req: ProfileRequest) -> Result<ProfileProposal, PlatformError>;
    async fn draft_derivation(&self, req: DraftRequest) -> Result<SpecDraft, PlatformError>;
    async fn answer(&self, req: AnswerRequest, tools: &dyn ToolSet) -> Result<Answer, PlatformError>;
}

// core/ports/engine.rs
#[async_trait]
pub trait TransformEngine: Send + Sync {
    async fn run(&self, spec: &DerivationSpec, inputs: &BTreeMap<InputAlias, TableRef>, out: &TableRef)
        -> Result<RunReport, PlatformError>;
    async fn query(&self, table: &TableRef, q: &StructuredQuery) -> Result<QueryResult, PlatformError>;
}
```

The other ports (`Mailer`, `SecretStore`, `AuthProvider`, `Clock`, `IdGenerator` and `Connector`) follow the same shape. The event store is split in two: `EventSink` is created per unit of work and shares its transaction, while `DeliveryQueue` is shared and opens its own short transaction for `claim`, `retry` and `bury`. `ObjectStore::get` returns all bytes, which is fine for pilot-size files. A streaming read is an open question below.

```rust
// core/ports/uow.rs
#[async_trait]
pub trait Uow: Send {
    fn datasets(&mut self) -> &mut dyn DatasetRepo;
    fn derivations(&mut self) -> &mut dyn DerivationRepo;
    fn connections(&mut self) -> &mut dyn ConnectionRepo;
    fn events(&mut self) -> &mut dyn EventSink; // bound to this transaction
    async fn complete(&mut self, d: &Delivery) -> Result<(), PlatformError>; // marks the delivery done in this transaction
    async fn commit(self: Box<Self>) -> Result<(), PlatformError>; // dropping without commit rolls back
}

#[async_trait]
pub trait UowFactory: Send + Sync {
    async fn begin(&self, tenant: Option<TenantId>) -> Result<Box<dyn Uow>, PlatformError>; // sets app.tenant_id for row-level security
}
```

Rules for every port

- **Keep it small.** The fewest methods that serve the use cases, and no multi-purpose methods such as `get_or_create`.
- **Translate at the edge.** An adapter converts library errors into `PlatformError` (a timeout becomes `Retryable`, a missing key becomes `NotFound`), so callers never match on a vendor error.
- **Ship the conformance suite with the port.** core exports one suite per port, plus in-memory fakes, under `vitro_core::ports::testing` behind a `testing` feature. A suite is a generic async function such as `object_store_conformance(&dyn ObjectStore)`. adapters runs the storage, engine, LLM and mailer suites against itself, and worker runs the connector suite for its connectors. For `ObjectStore`: a put and get round trip, a presigned PUT that works with a plain HTTP client, a prefix delete that returns the right count, and a missing key that returns `NotFound`. For the PostgreSQL queue: two claimers never get the same delivery, an expired lease can be claimed again, a `dedupe_key` collapses duplicates, and `run_after` is respected.
- **Composition root.** Each entrypoint builds its dependencies explicitly from its settings. Adapter choice is a configuration value, which is how the same code runs on a laptop and in the cloud.

```rust
// worker/main.rs
pub async fn build_deps(s: &WorkerSettings) -> Result<Deps, StartupError> {
    let pool = PgPoolOptions::new().max_connections(s.db.max_connections).connect(s.db.url.expose()).await?;
    let store: Arc<dyn ObjectStore> = match &s.object_store {
        ObjectStoreSettings::S3(c) => Arc::new(S3ObjectStore::new(c).await?),
        ObjectStoreSettings::Local { root } => Arc::new(LocalObjectStore::new(root)),
    };
    Ok(Deps {
        object_store: store,
        uow: Arc::new(PgUowFactory::new(pool.clone())),
        queue: Arc::new(PgDeliveryQueue::new(pool)),
        llm: build_llm(&s.llm)?,
        clock: Arc::new(SystemClock),
        ids: Arc::new(UlidGenerator),
    })
}
```

## Event handling and the worker loop

A worker is a generic loop over claimed deliveries, and each handler is registered for one consumer and one event type. Everything is async on tokio. Polars work is CPU-bound and runs on `spawn_blocking`, so it never stalls the loop.

&#91;embedded content: worker loop · 3 steps, 3 error outcomes\]

Claim. It runs in its own short transaction and commits at once, so nothing stays open while the handler works. An expired lease makes a delivery claimable again, which is how a crashed worker recovers.

```sql
WITH due AS (
  SELECT id FROM deliveries
  WHERE consumer = $1
    AND status = 'pending'
    AND run_after <= now()
    AND (locked_until IS NULL OR locked_until < now())
  ORDER BY run_after, id
  FOR UPDATE SKIP LOCKED
  LIMIT $3
)
UPDATE deliveries d
SET locked_until = now() + make_interval(secs => $2),
    attempts = attempts + 1
FROM due
WHERE d.id = due.id
RETURNING d.*;
```

The query runs through `sqlx::query_as!`, so the compiler checks its columns and types against the schema. The checked result is cached in `.sqlx/` for offline builds.

Handler contract. A handler gets a context and a typed payload and returns `Result<Done, PlatformError>`. `Done` has a private field, and only `ctx.finish(uow)` can build it, so a handler that forgets to complete its delivery does not compile. A long handler reads inside one transaction, does its slow work with no transaction open, and then writes everything in a second transaction that `finish` also uses to mark the delivery done.

```rust
// worker/handlers/derive.rs
pub async fn run_derivation(ctx: &Ctx, ev: DerivationRunRequested) -> Result<Done, PlatformError> {
    let mut uow = ctx.begin().await?; // transaction A: pin inputs, take the lease
    let run = start_run(uow.as_mut(), &ev, ctx.clock()).await?;
    uow.commit().await?;

    let report = ctx.engine().run(&run.spec, &run.inputs, &run.output).await?; // slow, no transaction open

    let mut uow = ctx.begin().await?; // transaction B: results, events, delivery done
    finish_run(uow.as_mut(), &run, &report, ctx.clock()).await?;
    ctx.finish(uow).await
}
```

```rust
// worker/claim_loop.rs
pub async fn process(deps: &Deps, registry: &Registry, delivery: Delivery) {
    let Some(handler) = registry.get(&delivery.consumer, &delivery.event.kind) else {
        deps.bury(&delivery, "no handler registered").await; // a wiring bug: fail loud
        return;
    };
    let ctx = Ctx::new(deps, &delivery);
    // spawn so that a panic in a handler becomes an error value instead of killing the role
    let outcome = tokio::spawn({ let h = handler.clone(); let ev = delivery.event.clone(); async move { h.handle(&ctx, &ev).await } }).await;
    match outcome {
        Ok(Ok(Done { .. })) => {}
        Ok(Err(e)) => match e.class() {
            ErrorClass::Retryable => deps.retry(&delivery, &e).await, // backoff honours retry_after
            ErrorClass::Permanent | ErrorClass::Blocked | ErrorClass::Programming => deps.bury(&delivery, &e.to_string()).await,
            _ => deps.retry_or_bury(&delivery, &e).await, // unexpected: retry until attempts run out
        },
        Err(join_error) => deps.retry_or_bury(&delivery, &join_error).await, // a panic
    }
}
```

The Rust code samples in this document are sketches written from my own knowledge. None of them has been compiled, so expect small fixes such as clones, lifetimes and exact crate signatures.

- **Idempotency.** Effects and completion commit together, so a delivery that crashed before commit has no effect and can be rerun, and one that committed is done. Effects outside PostgreSQL, such as object writes and emails, use deterministic keys, for example a version path derived from the derivation and its input versions.
- **Blocked** is handled inside the handler: it records the blocked state, notifies the owner and completes the delivery. A `Blocked` error that escapes a handler is a bug and is buried.
- **Backoff** is exponential with jitter and a cap, and it honours `retry_after` from rate-limited APIs.
- **Buried deliveries** stay in the table with a reason, raise an alert and can be re-queued with an operator command (`worker ops requeue`).
- **Wake-ups.** Poll about once a second. `LISTEN/NOTIFY` through sqlx's listener is only a hint.
- **Roles.** One codebase and one process per role (`--role connector`, `normalize`, `profile`, `derive`, `notify`, `scheduler` or `deleter`), so each role scales and fails independently.

## Connectors

A connector is a type that implements the three-function contract from the system design. It knows nothing about events, the database or other connectors. The connector worker hands it a context containing only what it may use. The trait is object-safe, so connectors live in one registry as `Arc<dyn Connector>`.

```rust
// core/ports/connector.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamSpec { pub key: String, pub name: String } // key is stable within the connection, e.g. a sheet tab ID

#[derive(Clone, Debug)]
pub struct ChangeInfo { pub changed: bool, pub token: String } // modified time, content hash or cursor

#[derive(Clone, Debug)]
pub struct RawBatch {
    pub objects: Vec<RawObject>, // keys under ctx.raw_prefix, with type and size
    pub state: serde_json::Value, // opaque to the core, stored on the stream
}

pub struct ConnectorContext {
    pub connection_id: ConnectionId,
    pub settings: serde_json::Value, // the connector parses it with its own settings type
    pub credential: Option<SecretString>, // decrypted just in time, never logged
    pub raw_prefix: String, // the only place fetch may write
    pub object_store: Arc<dyn ObjectStore>,
    pub clock: Arc<dyn Clock>,
}

#[async_trait]
pub trait Connector: Send + Sync {
    fn kind(&self) -> &'static str;
    fn settings_schema(&self) -> serde_json::Value; // JSON Schema of the settings, for the API and the form
    async fn discover(&self, ctx: &ConnectorContext) -> Result<Vec<StreamSpec>, PlatformError>;
    async fn check(&self, ctx: &ConnectorContext, stream: &StreamSpec, state: &serde_json::Value)
        -> Result<ChangeInfo, PlatformError>;
    async fn fetch(&self, ctx: &ConnectorContext, stream: &StreamSpec, state: &serde_json::Value)
        -> Result<RawBatch, PlatformError>;
}
```

Registry. Connectors are registered in one function in the worker, and a duplicate kind is an error at startup. There is no runtime plugin discovery: adding a connector is one visible line, and the compiler checks it. The cost is that a connector cannot be added without rebuilding the worker, which is the right trade for a closed set.

```rust
// worker/connectors.rs
pub fn load_connectors(deps: &Deps) -> Result<ConnectorRegistry, StartupError> {
    ConnectorRegistry::builder()
        .add(FileUpload::new())?
        .add(GoogleSheets::new(deps.http.clone()))?
        .build() // returns an error if two connectors share a kind
}
```

The handler. It is the only place that sequences check, fetch and event publishing, so every connector gets the same behaviour.

```rust
// worker/handlers/sync.rs
pub async fn sync_connection(ctx: &Ctx, ev: ConnectionSyncRequested) -> Result<Done, PlatformError> {
    let mut uow = ctx.begin().await?;
    let (conn, stream, spec, state) = load_for_sync(uow.as_mut(), &ev).await?;
    uow.commit().await?;

    let connector = ctx.connectors().get(&conn.connector_kind)?;
    let cctx = ctx.connector_context(&conn);
    let result = async {
        let change = connector.check(&cctx, &spec, &state).await?;
        let batch = if change.changed { Some(connector.fetch(&cctx, &spec, &state).await?) } else { None };
        Ok::<_, PlatformError>((change, batch))
    }.await;

    let mut uow = ctx.begin().await?;
    match result {
        Err(e) if e.class() == ErrorClass::Blocked => { // e.g. token revoked: only the owner can fix it
            mark_needs_reconnect(uow.as_mut(), &conn, &e).await?;
        }
        Err(e) => return Err(e),
        Ok((change, None)) => record_checked(uow.as_mut(), &stream, &change).await?, // unchanged: nothing is downloaded or published
        Ok((_, Some(batch))) => {
            record_batch(uow.as_mut(), &stream, &batch).await?;
            uow.events().append(raw_batch_landed(&ev, &batch), &["normalize"], Default::default()).await?;
        }
    }
    ctx.finish(uow).await
}
```

Errors a connector returns

| Situation | Returned | Result |
| --- | --- | --- |
| Network timeout or a 5xx response | `PlatformError::retryable(..)` | Retry with backoff |
| A rate limit with a retry-after hint | `PlatformError::retryable(..).after(duration)` | Retry at that time |
| Token revoked, or the file is no longer shared | `PlatformError::blocked(..)` | Connection marked needs-reconnect and the owner is notified |
| The file was deleted or the format is unsupported | `PlatformError::permanent(..)` | Buried with an alert |

Two connectors show how general the contract is. For `google_sheets`, `check` compares the Drive file's modified time with the stored token, and `fetch` reads the tab's values and writes one raw object. For `file_upload`, the browser has already put the object under the raw prefix, so `check` reports a change from the object's tag and `fetch` only verifies that it exists and returns its manifest.

Contract test suite. One generic function, `connector_contract(&dyn Connector, FakeExternal)`, run against every connector with a fake external system (wiremock for HTTP):

- `discover` returns the same stream keys on every call.
- A second `check` with the stored state reports no change when nothing changed.
- `fetch` is idempotent: the same state gives the same object keys and content hash.
- `fetch` writes only under `ctx.raw_prefix`, which a recording object store verifies.
- No credential value appears in captured logs or error messages. `SecretString` already hides it in `Debug` output, and the test checks the rest.
- Injected failures return the error classes in the table above.

## Derivations

A derivation has three parts that never mix: the spec, which is data, the engine, which compiles and runs it, and the planner, which decides when to run it. The Polars calls below are from my own knowledge and must be checked against the Polars crate. The crate's version numbers may not match the 2.0 numbering of the other Polars bindings, so the first spike must confirm which release carries the behaviour this design relies on.

The spec is a closed grammar of serde enums. Columns are referenced by ID, never by name, and there is no place to put code or SQL.

```rust
// core/domain/spec/models.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Fn { Add, Sub, Mul, Div, Eq, Gt, Lt, And, Or, Sum, Mean, Count, Min, Max, Lower, Strip, DateTrunc }
// closed set: adding a function is a code change with tests

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expr {
    Col { r#ref: ColumnId }, // a column ID, never a name
    Lit { value: Literal },
    Call { r#fn: Fn, args: Vec<Expr> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Literal { Null, Bool(bool), Int(i64), Float(f64), Str(String) }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Step {
    Filter { predicate: Expr },
    Select { columns: Vec<ColumnId> },
    Cast { column: ColumnId, to: ScalarType }, // the MVP plan lists cast as a step
    Derive { column: ColumnId, expr: Expr },
    GroupBy { keys: Vec<ColumnId>, aggregates: Vec<Aggregate> },
    Join { with: InputAlias, on: Vec<(ColumnId, ColumnId)>, how: JoinKind },
    Union { with: InputAlias },
    Sort { by: Vec<SortKey> },
    Limit { n: u64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DerivationSpec {
    pub inputs: BTreeMap<InputAlias, DatasetId>, // these are the edges of the dependency graph
    pub steps: Vec<Step>,
    pub output_sort: SortKeys, // required: the type rejects an empty list, because streaming results are unordered
}
```

Validation and output-schema inference are pure functions in the domain, so the API can reject a bad draft from the LLM before anything runs.

```rust
// core/domain/spec/validate.rs
/// Column IDs exist with compatible types, functions get the right argument types,
/// output_sort columns exist in the output, and the inputs create no cycle.
pub fn validate_spec(spec: &DerivationSpec, schemas: &BTreeMap<DatasetId, Schema>) -> Vec<SpecIssue>;
pub fn infer_output_schema(spec: &DerivationSpec, schemas: &BTreeMap<DatasetId, Schema>) -> Result<Schema, SpecInvalid>;
```

The compiler lives in the engine adapter. A `match` over the closed `Step` and `Fn` enums, so adding a variant without a compiler arm is a build error, not a test failure.

```rust
// adapters/engine/compile.rs
fn expr(e: &Expr, names: &NameMap) -> Result<pl::Expr, CompileError> {
    Ok(match e {
        Expr::Col { r#ref } => pl::col(names.get(r#ref)?),
        Expr::Lit { value } => literal(value),
        Expr::Call { r#fn, args } => {
            let args = args.iter().map(|a| expr(a, names)).collect::<Result<Vec<_>, _>>()?;
            function(*r#fn, args)? // exhaustive match over Fn
        }
    })
}

pub fn compile_spec(spec: &DerivationSpec, scans: &BTreeMap<InputAlias, LazyFrame>, names: &NameMap)
    -> Result<LazyFrame, CompileError> {
    let mut frame = scans[first_alias(spec)].clone();
    for step in &spec.steps {
        frame = compile_step(frame, step, scans, names)?; // exhaustive match over Step
    }
    Ok(frame.sort(sort_columns(&spec.output_sort, names), sort_options())) // explicit order, always
}

#[async_trait]
impl TransformEngine for PolarsEngine {
    async fn run(&self, spec: &DerivationSpec, inputs: &BTreeMap<InputAlias, TableRef>, out: &TableRef)
        -> Result<RunReport, PlatformError> {
        let (spec, inputs, out, this) = (spec.clone(), inputs.clone(), out.clone(), self.clone());
        tokio::task::spawn_blocking(move || this.run_blocking(&spec, &inputs, &out)) // CPU-bound, off the async threads
            .await
            .map_err(PlatformError::from_join)?
    }
}
// run_blocking: scan_parquet per input, compile_spec, sink_parquet with zstd (confirm the sink API),
// then summarize the output: row count, content hash, null counts.
```

Reading `s3://` locations directly from Polars needs the crate's cloud features and options. If that proves awkward against RustFS, the fallback is to download inputs to a temporary folder and upload the output. This is an item for the first spike.

A contract test checks that the compiled plan's schema equals `infer_output_schema` for generated specs, so validation and execution cannot drift apart.

The planner is a pure function. It never touches the database. An application function executes its actions: `MarkStale` becomes a checked state transition, and `RequestRun` appends a `DerivationRunRequested` event with the derivation ID as its `dedupe_key`, which collapses bursts into one pending run. The graph uses `BTreeMap` and `BTreeSet`, so iteration order is deterministic and tests do not depend on hash order.

```rust
// core/domain/planner.rs
pub struct Graph {
    pub inputs_of: BTreeMap<DatasetId, BTreeSet<DatasetId>>, // derived dataset -> datasets it reads
    pub blocked: BTreeSet<DatasetId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action { MarkStale(DatasetId), RequestRun(DatasetId) }

pub fn plan_after_publish(g: &Graph, states: &BTreeMap<DatasetId, DatasetState>, published: DatasetId) -> Vec<Action> {
    let stale = downstream(g, published); // every transitive consumer
    let mut actions: Vec<Action> = stale.iter()
        .filter(|d| states[d] != DatasetState::Stale)
        .map(|d| Action::MarkStale(*d))
        .collect();
    for d in consumers(g, published) { // direct consumers only, in sorted order
        let ready = g.inputs_of[&d].iter().all(|i| is_ready(*i, g, states, &stale));
        if ready && !g.blocked.contains(&d) {
            actions.push(Action::RequestRun(d));
        }
    }
    actions
}

fn is_ready(i: DatasetId, g: &Graph, states: &BTreeMap<DatasetId, DatasetState>, stale: &BTreeSet<DatasetId>) -> bool {
    let is_source = !g.inputs_of.contains_key(&i);
    !stale.contains(&i) && (is_source || states[&i] == DatasetState::Fresh)
}
```

When a run succeeds, `finish_run` writes the new version, sets the dataset fresh and appends `DatasetVersionPublished`, which brings the planner back for the next level. When a run fails, it sets the dataset failed and publishes nothing, so everything downstream stays stale. The planner's test invariants are in the testing section.

## Normalize and profiling

Normalization is a pipeline of small pure steps over a grid of cells, so each step can be tested with a tiny input. Format readers in the worker turn a file into a `RawGrid`: `calamine` reads xlsx and xls, the `csv` crate reads csv, and `encoding_rs` decodes legacy encodings such as Windows-1258 before anything else happens. Whether `encoding_rs` covers the older TCVN3 Vietnamese encoding is unverified, and a small custom table may be needed. Everything after the reader is domain code.

```rust
// worker/normalize.rs
pub fn normalize(raw: RawGrid, locale: &Locale, confirmed: Option<&SchemaVersion>, pii: &PiiPolicy)
    -> NormalizeResult {
    let grid = nfc(raw); // Vietnamese text compared in one normal form (unicode-normalization)
    let region = detect_table_region(&grid); // skips title rows, merged cells and notes
    let table = detect_header(region); // header row and column names
    let table = drop_subtotals(table); // total rows must not be counted twice
    let table = infer_types(table, locale); // numbers, dates, VND with no minor units
    let masked = mask_pii(table, pii); // detect, then drop, mask or pseudonymize
    let conflicts = compare_schema(masked.schema(), confirmed);
    NormalizeResult { masked, conflicts }
}
```

Unsafe data is unrepresentable. The types make the privacy rule a compile-time fact. A `RawGrid` and a `Table` may hold personal data. Only a `MaskedTable` can produce a `MaskedSample`, and the LLM port accepts nothing else. `MaskedTable` and `MaskedSample` have private fields, so only their own module can build them.

```rust
// core/ports/llm.rs
pub struct ProfileRequest {
    pub schema: Schema,
    pub sample: MaskedSample, // cannot be built from a RawGrid or a Table
    pub locale: Locale,
}

// core/domain/normalize/types.rs
pub fn make_sample(masked: &MaskedTable, n: usize) -> MaskedSample; // the only constructor
```

A `trybuild` compile-fail test proves the rule: it shows that building a `ProfileRequest` from a `Table` does not compile. The type checker catches the rest.

PII rules are pure functions that take a policy and, for pseudonymization, a per-tenant key passed in as a value. Detection uses column-name patterns, Vietnamese phone formats, emails, address keywords and ID-number patterns, checked on a sample of values. The owner confirms the detected columns, and each gets one policy: drop, mask or pseudonymize. A pseudonymous ID is a keyed hash (HMAC-SHA-256 through the `hmac` and `sha2` crates) of the value, so the same customer gets the same ID without the name being stored.

Schema conflicts are typed values. Comparing a new schema with the confirmed one returns a list, and the owner's choice for each becomes a `SchemaVersionConfirmed` event.

```rust
// core/domain/normalize/schema.rs
pub enum Conflict {
    MissingColumn(ColumnId),
    NewColumn(String),
    TypeMismatch { column: ColumnId, found: ScalarType, expected: ScalarType },
    LikelyRename { from: ColumnId, to: String, score: f32 },
    DuplicateKey(Vec<ColumnId>),
}
pub fn compare_schema(new: &Schema, confirmed: Option<&SchemaVersion>) -> Vec<Conflict>;
```

| Conflict | Default | Owner choices |
| --- | --- | --- |
| MissingColumn | Block | Map to another column, fill nulls, or accept a new schema version |
| NewColumn | Pause | Ignore it, or add it in a new version |
| TypeMismatch | Show the affected rows | Coerce, reject the rows, or change the type |
| LikelyRename | Suggest a mapping | Confirm or decline |
| DuplicateKey (on append) | Block | Upsert or reject |

`LikelyRename` uses a pure similarity function over the name, the type and the value distribution, so it is tested with Vietnamese names with and without diacritics.

Profiling is a use case around the LLM port. It builds a `ProfileRequest` from the masked sample, validates the model's answer against a JSON Schema, and stores it as a proposal. A proposal never changes data or schema by itself, because only the owner's confirmation, which is an event, does that.

## API layer and persistence

Routes are thin: parse the request, call a use case inside a unit of work, and map the result to a response type. Business rules live in use cases and the domain. Everything is async on tokio, end to end: axum, sqlx and the ports, so there is no sync and async split to manage. Heavy Polars runs stay in the worker, and API queries call the engine's `query` method on `spawn_blocking`.

- **App factory.** `build_router(state)` builds the axum `Router` from explicit dependencies, with no global state, so tests build an app around fakes and call it with `tower::ServiceExt::oneshot`.
- **Request context.** One extractor turns the session cookie into a `Principal { tenant_id, user_id, role }`. The tenant always comes from there, never from the body or path.
- **Separate types.** API request and response types in `vitro_api::routes::schemas` derive serde and utoipa's `ToSchema`, and map to and from domain objects with explicit `From` and `TryFrom` impls, so the API can evolve without changing the domain.
- **Errors.** `ApiError` wraps `PlatformError` and implements `IntoResponse`, returning `application/problem+json` with a stable `code` field.
- **Idempotency.** A middleware stores the response for each `Idempotency-Key` with a hash of the request. The same key and request replays the stored response, and the same key with a different request is rejected.
- **OpenAPI.** utoipa builds the document from the route annotations. `cargo x contracts` writes it to `contracts/openapi.json`, and CI fails if the committed file is out of date.
- **MCP.** The MCP endpoint uses an MCP server crate (`rmcp` is the official Rust SDK as far as I know). Check the current official repository before adopting it, because several forks exist.

```rust
// api/routes/derivations.rs
#[utoipa::path(post, path = "/derivations", request_body = CreateDerivationIn,
    responses((status = 201, body = DerivationOut)))]
async fn create_derivation(State(s): State<ApiState>, p: Principal, Json(body): Json<CreateDerivationIn>)
    -> Result<(StatusCode, Json<DerivationOut>), ApiError> {
    let mut uow = s.uow.begin(Some(p.tenant_id)).await?;
    let d = create_derivation_usecase(uow.as_mut(), &p, body.try_into_command()?, &*s.clock, &*s.ids).await?;
    uow.commit().await?;
    Ok((StatusCode::CREATED, Json(DerivationOut::from(d))))
}

// api/usecases/derivations.rs
pub async fn create_derivation_usecase(uow: &mut dyn Uow, p: &Principal, cmd: CreateDerivation,
    clock: &dyn Clock, ids: &dyn IdGenerator) -> Result<Derivation, PlatformError> {
    let schemas = uow.datasets().schemas_for(cmd.spec.inputs.values()).await?;
    let issues = validate_spec(&cmd.spec, &schemas); // pure
    if !issues.is_empty() { return Err(PlatformError::SpecInvalid(issues)); }
    let graph = uow.derivations().graph().await?.with_derivation(cmd.output_dataset_id, &cmd.spec);
    graph.assert_acyclic(MAX_DEPTH)?; // pure: cycles and depth limit
    let d = Derivation::new(ids.new_id(), p.tenant_id, cmd, clock.now());
    uow.derivations().add(&d).await?;
    uow.events().append(derivation_run_requested(&d, p), &["derive"], Default::default()).await?; // first build
    Ok(d)
}
```

Sessions and authentication. Passwords are hashed with Argon2id through the `argon2` crate. Sessions are server-side rows in PostgreSQL behind a `SessionStore` trait, and the cookie holds only an opaque random ID. I prefer a thin hand-written store over a session crate here, because the table, the expiry and the tenant check are part of the security design. State-changing requests also carry an anti-CSRF header on top of `SameSite`.

Persistence

- **Plain SQL with sqlx.** Repository methods hold SQL in `query!` and `query_as!` macros, which the compiler checks against the schema, and map rows to domain types with explicit functions. There is no ORM, because the interesting queries (the claim, the recursive downstream walk) are clearer as SQL. The offline cache `.sqlx/` is committed, and `cargo sqlx prepare --check` runs in the full tier.
- **Migrations.** Numbered, forward-only SQL files in `adapters/migrations/`, applied by `sqlx::migrate!` through one command, `worker ops migrate`. `just migrate` runs the same function for development, and deployment runs it as a one-off container from the worker image, so there is no second path. A test applies every migration to an empty database.
- **Row-level security.** Every business table has a `tenant_id` and a policy using `current_setting('app.tenant_id')`. The unit of work sets it with `set_config('app.tenant_id', $1, true)`, which is transaction-local, and a test proves that a missing tenant returns no rows. The `events` and `deliveries` tables are outside RLS because a claim crosses tenants. Handlers then work through `begin(Some(event.tenant_id))`, so all business data stays under RLS.
- **Pools.** One `PgPool` per process, and each unit of work holds one connection for its transaction.

```rust
// adapters/postgres/datasets.rs
impl DatasetRepo for PgDatasetRepo<'_> {
    async fn set_state(&mut self, id: DatasetId, to: DatasetState) -> Result<(), PlatformError> {
        let row = sqlx::query!("SELECT state FROM datasets WHERE id = $1 FOR UPDATE", id.to_string())
            .fetch_one(&mut *self.tx).await.map_err(db_err)?;
        let new = transition(DatasetState::parse(&row.state)?, to)?; // the domain decides what is allowed
        sqlx::query!("UPDATE datasets SET state = $1, state_changed_at = now() WHERE id = $2",
            new.as_str(), id.to_string())
            .execute(&mut *self.tx).await.map_err(db_err)?;
        Ok(())
    }
}
```

## Errors, logging and configuration

Errors are one enum in the domain. Each variant has exactly one outcome in the worker and one in the API, so nobody decides case by case. `class()` maps a variant to the worker outcome, and `status()` in the API crate maps it to the HTTP status.

| Variant | Meaning | Worker outcome | HTTP status | Example |
| --- | --- | --- | --- | --- |
| Retryable | May succeed later | Retry with backoff | 503, or 429 with a retry-after | A timeout, or a 5xx from Google |
| Permanent | Will never succeed as is | Buried with an alert | 500 | A corrupt Parquet file, an unsupported format |
| Blocked | Needs a person | The handler records it, notifies and completes | 409 | A revoked token, a spec that uses a removed column |
| Conflict | The request clashes with current state | Not used by workers | 409 | Confirming a schema that was already superseded |
| NotFound | Missing, or not visible to this tenant | Not used by workers | 404 | A dataset from another tenant, which never returns 403 |
| SpecInvalid | The input is invalid | Not used by workers | 422 | The list of spec issues |
| PermissionDenied | The role is not enough | Not used by workers | 403 | A member trying to delete a connection |
| InvalidTransition, Programming | A programming error | Buried with an alert | 500 | A state change the table does not allow |

There is no row for a handler contract error: a handler that forgets to complete its delivery does not compile.

```rust
// core/domain/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("retryable: {message}")]
    Retryable { message: String, retry_after: Option<Duration>, #[source] source: Option<BoxError> },
    #[error("permanent: {0}")]
    Permanent(String),
    #[error("blocked ({needs:?}): {message}")]
    Blocked { message: String, needs: Needs }, // Needs::Owner or Needs::Operator
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid spec")]
    SpecInvalid(Vec<SpecIssue>),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error(transparent)]
    InvalidTransition(#[from] InvalidTransition),
    #[error("programming error: {0}")]
    Programming(String),
}

impl PlatformError {
    pub fn class(&self) -> ErrorClass; // one outcome per variant
    pub fn code(&self) -> &'static str; // stable, machine-readable, used in API responses
}
```

- Libraries return `PlatformError` or their own `thiserror` enums. `anyhow` appears only in `main` and in tooling (RS-007).
- Never discard a `Result` (`let_underscore_must_use` is denied). Wrap a cause with `#[source]` or `#[from]` so it survives.
- Error messages carry IDs and reasons, never row values or credentials.
- Every skipped or rejected row is counted and reported in the run result, never dropped silently.

Logging. The `tracing` crate with a JSON formatter from `tracing-subscriber`. The worker opens a span per delivery and the API middleware opens one per request, each with `tenant_id`, `correlation_id`, `delivery_id` and `consumer` set once, so every event inside carries them. A `Masked<T>` wrapper prints as `***`, and secrets are `SecretString` as soon as they are read. State transitions are logged at info level with IDs only. `tracing-opentelemetry` exports the same spans.

Configuration. One serde settings struct per entrypoint (`ApiSettings`, `WorkerSettings`) with nested structs for the object store, LLM and database, loaded from environment variables only, in one function, `Settings::load`, which is the only place allowed to read the environment (clippy bans `std::env::var` elsewhere). A process fails at startup and lists every invalid or missing value. Which loader crate to use (`figment`, `config` or `envy`) is a spike item, and the choice must allow collecting all errors. Secrets come through the `SecretStore` port. The settings structs derive `JsonSchema`, and `cargo x env-example` generates `.env.example` from them, so the two cannot drift. SOPS decrypts the per-environment secret files, and `sops exec-env` injects them for the process.

Time. Every timestamp is `DateTime<Utc>`. A naive date-time type is banned through clippy's `disallowed-types` (`chrono::NaiveDateTime`) outside the parsing code.

## Web app

The web app is a deployable: a static single-page application served by Traefik as its own instance. It is one package that holds everything only the web app uses: the generated API client, the message-key types and the end-to-end flows. The stack below is a proposal for you to confirm.

```
packages/web/
  src/
    routes/                 one folder per page
    features/
      connections/          api.ts (thin wrappers over the client), queries.ts, components/
      datasets/
      derivations/
      chat/
      settings/
    shared/                 UI primitives, layout, formatting
    api/                    client generated from contracts/openapi.json, plus error mapping
    i18n/                   key types generated from contracts/i18n/{vi,en}.json
  e2e/                      Playwright flows and a few synthetic upload files
  tests/                    component tests
```

Stack: SolidJS with Vite (`vite-plugin-solid`) and TypeScript on the strict base config, `@solidjs/router` for routing, TanStack Query for Solid for server state, Kobalte for accessible UI primitives, `ts-pattern` for exhaustive matching, `neverthrow` for fallible code, Biome for format and lint plus oxlint for fast extra lints, Vitest with Solid's testing library for components, and Playwright for flows. The router and the UI primitives are proposals.

- **Data.** Every call goes through the generated client in `src/api/`, with no hand-written `fetch` (FE-002). Server state lives in TanStack Query, and there is no global client store until a real need appears. Dataset state (fresh, stale, running, failed, blocked) and freshness come from the API and render as badges that show the upstream cause.
- **Solid reactivity.** Components read props and signals where they are used, and do not destructure props in the parameter list, because that loses reactivity. Biome and oxlint have no Solid-specific rules as far as I know, so an ast-grep rule (FE-004) covers it. This is the main habit change from React.
- **Authentication.** The session cookie is HttpOnly and managed by the API, so the app never reads a token. State-changing requests also carry the anti-CSRF header.
- **Uploads.** The app asks the API for a presigned multipart URL, uploads straight to the object store, then calls the upload-complete endpoint. Progress and retry live in one feature module.
- **i18n.** Every user-visible string comes from the catalogs in contracts, and Vietnamese is the default. A test fails if a key is missing in either language or placeholders differ. Money, dates and numbers go through `Intl` with the workspace locale, and VND has no minor units.
- **Boundaries.** Feature folders do not import each other, and shared code goes in `shared/` (FE-001, enforced by ast-grep).
- **Client generation.** `just contracts` runs an OpenAPI-to-TypeScript generator (`openapi-typescript` with `openapi-fetch` is my candidate, unverified) over `contracts/openapi.json`, plus a small hand-written layer that maps problem-details responses to typed errors. If a second TypeScript deployable ever needs the client, `api/` is promoted into a shared package.
- **End to end.** One flow uploads a file, confirms the schema and watches a derived dataset turn fresh, against the Compose stack.

## Testing strategy

The design puts the risky logic in pure functions, so most tests are fast unit tests. Real PostgreSQL and RustFS are used only where the behaviour belongs to them. Rust tests run through `cargo nextest` (my proposal), and web tests through Vitest.

| Level | Covers | Lives in | Run by |
| --- | --- | --- | --- |
| Unit | Planner, spec validation, state machine, normalize rules, PII rules, web components | `#[cfg(test)]` modules and `tests/` of each package | `just test` |
| Property | Planner and spec compiler, with `proptest` | core, adapters | `just test` |
| Contract | Each port against each adapter, connectors against a fake external system, event schemas | Suites exported by core and run from adapters and worker, and the schema check in `cargo x contracts` | `just test`, plus `just test-integration` for the real adapters |
| Compile-fail | Unmasked data cannot reach the LLM port, and a handler cannot build `Done` itself, with `trybuild` | core and worker `tests/compile_fail/` | `just test` |
| Integration | Handlers with real PostgreSQL and RustFS through `testcontainers`: concurrent claims, crash recovery, row-level security | adapters and worker | `just test-integration` |
| Golden | Synthetic messy exports through the readers and normalize, compared with `insta` snapshots: merged cells, title rows, legacy encodings, Vietnamese text | `worker/tests/fixtures` | `just test` |
| End to end | Upload to derived dataset through the web app and the API | `web/e2e` against the Compose stack | `just test-e2e` |
| Load | Queue depth, cascade depth, memory on the largest pilot file | `cargo x` commands in tooling | On demand, recorded and not asserted |

Conventions. Tests use fakes, which are in-memory adapters from core that pass the same contract suites as the real ones, instead of mocks of ports. Builders such as `graph(..)` and `states(..)` live in core and are exported as `vitro_core::domain::testing` behind the `testing` feature. Golden data is synthetic, never real merchant data, and `web/e2e` keeps its own few small upload files. Each test has a name that states the rule and an `Intent:` comment that says why it matters. HTTP calls to Google are faked with `wiremock`.

```rust
// core/domain/planner.rs (tests)
#[test]
fn diamond_requests_the_join_only_after_both_branches_are_fresh() {
    // Intent: a derived dataset must never be built from one new and one old input.
    let g = graph(&[("B", &["A"]), ("C", &["A"]), ("D", &["B", "C"])]);
    let first = plan_after_publish(&g, &states(&[("A", Fresh), ("B", Fresh), ("C", Fresh), ("D", Fresh)]), id("A"));
    assert_eq!(set(first), set([stale("B"), stale("C"), stale("D"), run("B"), run("C")]));
    let after_b = plan_after_publish(&g, &states(&[("A", Fresh), ("B", Fresh), ("C", Stale), ("D", Stale)]), id("B"));
    assert!(!after_b.contains(&run("D"))); // C has not finished yet
    let after_c = plan_after_publish(&g, &states(&[("A", Fresh), ("B", Fresh), ("C", Fresh), ("D", Stale)]), id("C"));
    assert!(after_c.contains(&run("D")));
}

// worker/tests/golden.rs
#[test]
fn export_lands_as_the_table_the_owner_sees() {
    // Intent: real-world exports (title rows, subtotals, legacy encodings) must not change the numbers.
    insta::glob!("fixtures/*.xlsx", |path| {
        let case = GoldenCase::load(path);
        let result = normalize(read_grid(&case.file), &case.locale, None, &case.pii);
        insta::assert_yaml_snapshot!(result.records());
        assert!(result.conflicts.is_empty());
    });
}
```

Planner invariants, checked with `proptest` on random acyclic graphs and random states

- **Safety.** `RequestRun(d)` is never emitted when any input of `d` is stale, running, failed or blocked, or lies in the downstream set.
- **Scope.** `MarkStale` touches only the downstream set of the published dataset, and `RequestRun` only its direct, unblocked consumers.
- **Idempotence.** After the actions are applied, planning again for the same publish emits no new `MarkStale` and the same `RequestRun` set.
- **Progress.** Simulate every requested run succeeding and re-plan after each. Every downstream dataset ends fresh within as many rounds as the graph is deep.

Spec compiler properties: the compiled plan's schema equals the inferred schema, and shuffling the input rows leaves the sorted output and its content hash unchanged, which is the rule that protects against unordered streaming results.

CI: unit, property, compile-fail and fake-based contract tests run for every affected package on every push. Integration and end-to-end tests with containers run on pull requests. Formatting, clippy, ast-grep and the repo check run on both. The deployment section lists the exact commands.

## Extending the platform

Each kind of extension has one path: the packages to touch, and the packages that must stay unchanged. Most extensions add a module inside a deployable or a shared package, not a new package, and that is how the design stays extensible without a plugin framework.

Add a connector. It adds a module under worker and changes nothing else. It must not change events, core, normalize, derivations or the API.

1. Add `worker/src/connectors/<name>.rs` (the path `vitro_worker::connectors::<name>`, with children in `<name>/`) with a type implementing `Connector`, a settings struct and a credential type.
2. Add one `.add(..)` line to `load_connectors`, and add its client crates to the worker's `Cargo.toml`.
3. Record the minimum credential scopes and the personal data it can return in the worker README.
4. Pass `connector_contract` from core against a fake external system, including the error-class cases.
5. Add a file under `worker/tests/fixtures` if its raw format is new.

Add a derivation step or function. It touches core and adapters. It must not change the planner, events or handlers.

1. Add the `Step` variant, or the `Fn` member, in core's `domain/spec` with its validation and output-schema rules. The compiler now fails in adapters until step 2 is done.
2. Add its arm to the exhaustive `match` in adapters' engine.
3. Add a unit test for validation, a property test for schema agreement and a golden result.
4. Offer it to the LLM's spec grammar only after steps 1 to 3 pass.

Add or replace an adapter (object store, LLM, mailer, engine). It touches adapters and the composition roots that select it. It must not change core.

1. Implement the port in a module of adapters.
2. Run the port's conformance suite from core in adapters' tests.
3. Select it in the composition roots of api and worker with a configuration value.
4. If only one deployable will ever use it, as with authentication in api, keep it inside that deployable instead.

Add an event type. It must not change the worker loop or the claim query.

1. Add the payload struct in core with `EventPayload` and list it in the catalog. `cargo x contracts` writes its schema, and CI fails until the file is committed.
2. Add the consumer and event type to the subscriber table in core.
3. Register a handler in worker, and add a contract test that the payload holds only references.

Add an API endpoint.

1. Add the route and use case in api. If worker needs the same use case, promote it to core.
2. Run `just contracts` and commit the changed `contracts/openapi.json`.
3. Use the regenerated client in a web feature.

Add a table.

1. Write a migration in adapters with a `tenant_id` column and a row-level security policy.
2. Add the repository method with a checked `query!` and an explicit row mapper, then run `cargo sqlx prepare` and commit `.sqlx/`.
3. Add a test that proves another tenant sees no rows.

Promote code to a shared package when a second deployable needs it.

1. Move it to core if it is a concept (a type, a rule, a use case), or to adapters if it is an implementation that touches the outside world.
2. Update the dependency table, and let `cargo x check-repo` confirm the declared dependencies.
3. Move the tests with it, and run the affected suites in both deployables.

Split a worker role into its own deployable when its dependency profile or release cadence diverges. Create the package with the generator, move the role's handler modules into it, and leave the shared code in core and adapters.

Add a package. Do it only for a new deployable or a concept that two deployables share. Use `cargo x new-package`, satisfy the package contract, and update the catalog and the dependency table.

## Deployment, CI and developer workflow

Everything runs from the root through `just`, so the same recipes serve a laptop, CI and the pilot VM. Each deployable runs as its own instance, and infra defines how the instances run together.

Local development

- The prerequisites are `rustup` (which reads `rust-toolchain.toml`), Bun, `just`, Docker, and for deployment `sops`, `age` and Pulumi, with tool versions pinned in `.tool-versions`.
- `just setup` installs both ecosystems, installs the git hook and runs code generation.
- `just dev` starts PostgreSQL, RustFS and Mailpit through infra, applies migrations, and then runs api and worker with reload and web with Vite.
- infra also offers `up-obs` and `up-llm` recipes for the optional observability and local-model profiles.

One image per deployable

- api and worker have a multi-stage Dockerfile with the repo root as the build context. The build stage uses `cargo-chef` to cache dependency compilation, sets `SQLX_OFFLINE=true`, and builds only its own binary. The runtime stage is a slim image with the one binary.
- web builds with Vite and ships as static files behind Traefik.
- `cargo x docker-context <package>` computes the workspace subset a build needs from `cargo metadata`, so an image sees only its dependency closure.
- Images are tagged with the git SHA. The worker image runs as one instance per role, and operator commands run from it, for example `docker compose run worker ops requeue`.
- Because the binary links only the crates in its manifest, the API image has no connector code, and an undeclared import fails to compile.

Deploying the pilot

- `just deploy-pilot` runs the full check and the tests, builds the images, pushes them to an OCI registry, and updates the pilot VM through a Docker context over SSH: pull, run `worker ops migrate` as a one-off container, and restart the instances with `docker compose up -d`.
- Pulumi, in `infra/pulumi/` as a TypeScript program, defines the pilot VM and its network now and the cloud resources later. Compose stays the runtime on the VM. Whether Pulumi should also drive the container rollout is open.
- Rollback is a redeploy of the previous SHA. Migrations are forward-only, so a risky schema change ships in two releases (expand, then contract), and a restore from backup is the last resort.
- There are three environments: local (Compose core profile and dev servers), pilot (Compose on one VM) and, later, cloud (Pulumi plus Kubernetes). Configuration lives in env files per environment, with secrets encrypted by SOPS and age and never committed in plaintext.

CI

On every pull request, on any CI host:

1. Install the pinned tools, restore the cargo and `target/` caches, then `bun install --frozen-lockfile`.
2. `just check full`: formatting, clippy with `-D warnings`, ast-grep, `cargo deny`, the sqlx cache check, Biome, oxlint, `tsc --noEmit`, the contract snapshot check and `cargo x check-repo`.
3. `just test`, then `just test-integration` and `just test-e2e` with containers, for affected packages where `cargo x changed` allows.
4. The contract snapshot check regenerates `contracts/` and fails if the result differs from the committed files.

On the main branch, CI builds and tags the images and deploys to the pilot after a manual approval. Rust compile times are the main CI cost, so caching `target/` and `cargo-chef` layers matters. Remote compile caching (sccache) can come later.

Developer workflow

- `cargo x new-package` creates a package that already satisfies the contract and starts strict.
- A pre-commit hook runs the fast tier of `just check` on staged files. Development is trunk-based with small pull requests.
- Decisions are recorded as ADRs in the root `docs/` folder, starting from the decision table below.

## Decisions, open questions and sources

Fifteen decisions hold the design together, each with the alternative I rejected and the signal to revisit it.

| Decision | Choice | Alternative | Revisit when |
| --- | --- | --- | --- |
| Repo and task runner | A Cargo workspace with `just` and a `cargo x` launcher, following the macro-inc/macro setup, and Bun for the TypeScript side only | Turborepo, Make, Bazel | The TypeScript side grows into many packages, or Rust build times need remote caching beyond sccache |
| What a package is | A deployable that runs as its own instance, or a shared package for a concept used by two or more deployables. Eight in total, promoting code on second use | A package per layer, adapter and connector (28), or eleven packages grouped by toolchain and test profile | Two teams need independent releases, or a shared package's tests or dependencies start to diverge |
| Shared adapters | One adapters package for what both api and worker use. Single-deployable code such as authentication, file readers and connectors lives in its deployable | Adapter groups by test profile, or a package per adapter | Container tests and pure tests in adapters diverge enough to hurt rebuild times |
| Worker roles | One worker package, run as a separate instance per role | A deployable per role | Role dependency profiles or release cadence diverge |
| Connector packaging | Modules inside worker, registered in one compile-time function | Runtime plugins, or a crate per connector | A second deployable needs connectors, or one needs process isolation |
| Languages | Rust for every backend package and TypeScript for web and the Pulumi program | Python, Go | Hiring or iteration speed suffers, or a needed library exists only in Python |
| Toolchain pinning | `rust-toolchain.toml` and `.tool-versions` | A Nix flake, as Macro uses | Developers hit "works on my machine" differences |
| Boundary enforcement | Cargo's declared dependencies, `cargo deny`, and ast-grep rules with stable IDs | Convention only, or tests that scan imports | Rules multiply faster than people maintain them, or false positives appear |
| Contracts | Rust types are the source. Schemars and utoipa produce committed JSON Schema and OpenAPI, and the web client is generated from them | JSON Schema first with generated Rust types, protobuf | A non-Rust service needs to produce events |
| Database access | sqlx with checked plain SQL and a committed offline cache | SeaORM, Diesel | Query volume or team preference changes, or the offline cache becomes a merge burden |
| Async model | Async on tokio everywhere, with Polars on `spawn_blocking` | Sync threads with a blocking pool | Blocking work crowds out the async runtime in practice |
| Frontend | A SolidJS and Vite single-page app with the client and end-to-end flows inside it | React, SvelteKit, a separate SDK package | Solid's component ecosystem or hiring pool blocks progress, or server rendering becomes necessary |
| Infrastructure as code | Pulumi with TypeScript, as Macro does | OpenTofu with HCL | State hosting or licensing conflicts with the local-first goal, or HCL is easier for the team |
| Secrets | SOPS and age, with `sops exec-env` | Doppler, as Macro uses | Rotation or access auditing needs outgrow encrypted files |
| Spec execution | A typed spec compiled to Polars expressions | A spec compiled to SQL | Analysts need ad hoc SQL |

Open questions

1. Is eight the right granularity, or should the worker roles be separate deployables from the start?
2. Should api and worker ship as one backend image at first, to avoid version skew between them?
3. Should core and adapters merge into one shared library, or is the split between concepts and implementations worth keeping?
4. Do you accept `@solidjs/router`, Kobalte and TanStack Query for the web app?
5. Is `vitro` the final name for the crate prefix, the npm scope, the `VITRO_` environment prefix and the image names?
6. Which Rust version do we pin? Edition 2024 needs 1.85 or later as far as I know. Which versions of `just`, Bun and Pulumi do we pin?
7. Does the Polars crate carry the behaviour this design relies on (explicit output order, the parquet sink API), and can it scan `s3://` locations on RustFS, or do we download to a temporary folder?
8. Will build times be acceptable with sqlx macros and Polars in one workspace? What do we do first: `mold` or `lld`, sccache, `cargo-chef` layers, or splitting the xtask crate?
9. Is a self-managed Pulumi state backend acceptable, and which one? The Pulumi CLI is open source, but its default backend is a hosted service, which conflicts with the local-first goal. I have not checked the self-managed options. Which cloud provides the pilot VM?
10. I read "skip this tool" as dropping Doppler and keeping SOPS and age. Is that what you meant?
11. Which container registry and CI host will you use?
12. Should `ObjectStore` get a streaming read and write before the largest pilot files arrive?
13. Is the official MCP SDK for Rust (`rmcp`) mature enough to use, or should the MCP endpoint be a thin hand-written layer?
14. What are the maximum derivation depth and fan-in limits (`MAX_DEPTH`)?
15. Is polling every second enough, or should `LISTEN/NOTIFY` be used as a wake-up hint from the start?

## Sources

The tooling follows the [macro-inc/macro](https://github.com/macro-inc/macro) repository. I read files from its repository through raw links, because GitHub's tree pages blocked automated access, so I may have missed parts of its setup. Its CLAUDE.md and AGENTS.md came back empty. Every crate name, crate version and API call in this document, every Polars call, and every statement about Pulumi, sqlx, utoipa, `rmcp` and SolidJS libraries is from my own knowledge and was not checked, so confirm current versions and APIs before adopting them. The Rust samples have not been compiled. This document builds on System Design v2.
