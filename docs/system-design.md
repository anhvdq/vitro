# System Design v2: Open-Source, Local-First Data Platform

Oct 10, 2026 · @Duc Quoc Anh Van

## Summary

The platform is rewritten as open-source containers that run on a laptop with one `docker compose up` and deploy unchanged to a VM or Kubernetes in any cloud. PostgreSQL is both the metadata store and the event queue, RustFS replaces S3, Polars 2.0 replaces DuckDB, and the MVP uses simple built-in authentication. Six decisions shape it.

1. **Generic concepts.** Everything is a Connector, Connection, Stream, Dataset, Dataset version or Derivation, so Google Sheets is one connector among many to come, and adding a connector changes no core code.
2. **Ports and adapters.** Code depends on small interfaces for the event bus, object store, LLM, mailer, secrets and authentication. Each ships with one open-source default, and managed-cloud adapters are optional later.
3. **PostgreSQL is the event queue.** Events and their per-consumer deliveries are rows written in the same transaction as the state change and claimed with `FOR UPDATE SKIP LOCKED`. There is no broker and no outbox relay, so state and events cannot diverge. PostgreSQL becomes the single point of failure for both state and events, so backups and failover matter more.
4. **Multi-level derivation is a state graph woken by events.** A source change marks every downstream dataset stale in one transaction, and each derivation runs once all of its inputs are fresh, so diamonds run once and failures stop at the failed node.
5. **Polars 2.0 runs a typed JSON spec directly, not SQL.** No user or LLM text is ever executed as code or SQL, and every output sorts by declared keys because Polars 2.0's streaming engine does not guarantee row order.
6. **Hosting is an open choice.** Because everything is containers, the whole stack can run in Vietnam, for example on EC2 in AWS's Hanoi Local Zone, instead of splitting data across borders. Counsel still decides.

The trade-off of self-hosting: no per-request cloud bills, but you now own patching, backups, upgrades and monitoring.

## Assumptions and design principles

The scale and team assumptions are: ten pilot merchants growing to low hundreds, files up to tens of megabytes, and one to three engineers.

- **Local first:** every developer and every CI run starts the full stack with one command and needs no cloud account.
- **Same artifacts everywhere:** one container image per service, configured only through environment variables.
- **Generic by default:** model the domain with Connector, Connection, Stream, Dataset and Derivation (see Core concepts) instead of naming things after the first data source.
- **Ports and adapters:** the code depends on interfaces (`ObjectStore`, `EventStore`, `LlmClient`, `Mailer`, `SecretStore`, `AuthProvider`, and the `Connector` contract). Build only the open-source default adapter for each now.
- **Simple authentication first:** the MVP uses built-in email and password with server-side sessions, and the `AuthProvider` port leaves room for an identity provider later.
- **Permissive licenses by default:** Apache 2.0, MIT and MPL components are preferred. AGPL components (Grafana, Loki and Tempo) are fine to self-host unmodified, but confirm before modifying or redistributing them.
- **No commercial-only parts:** Polars Cloud and the enterprise editions of open-core products are out.
- **Unchanged from before:** multi-tenant SaaS with a tenant ID everywhere, no personal data in events or logs, the LLM sees only schema, descriptions and masked samples, and the same out-of-scope list (marketplace and delivery-app connectors, agent builder, workflow automation, real-time streaming).

**Maturity risk:** one report dates the Polars 2.0 release to 6 October 2026, four days ago, after a September release candidate. Pin an exact version and expect early patch releases. RustFS is reported to have reached 1.0 and implements a tested subset of the S3 API, so run your own upload and read tests before trusting it.

## Core concepts

The design uses a small set of generic concepts so that new data sources and new kinds of derivation plug in without changing the core. File upload and Google Sheets are simply the first two connectors.

| Concept | Meaning | In the MVP | Added later |
| --- | --- | --- | --- |
| Connector | A code plugin that knows how to fetch from one kind of external system | `file_upload`, `google_sheets` | Marketplace, delivery and messaging APIs, databases |
| Connection | A tenant's configured instance of a connector, with settings, a credential and a sync schedule | One per upload group or Google file | One per shop account, API key or database |
| Stream | A named table-like feed inside a connection | A sheet tab, an uploaded file | An API endpoint such as orders or products |
| Dataset | A named table with a stable ID and a schema. Kind `source` is fed by one stream, and kind `derived` is made by one derivation | Both kinds | Other kinds, such as a metric dataset |
| Dataset version | An immutable snapshot of a dataset, with schema version, content hash, row count and lineage | Parquet files plus a manifest | Table formats such as Iceberg |
| Derivation | A typed spec that reads one or more datasets and produces one derived dataset | Filter, select, cast, derive, group by, join, union, sort, limit | More step types, reusable metrics |
| Credential | A secret or token used by a connection, encrypted and never logged | Google refresh token | API keys and OAuth tokens for other connectors |

**Connector contract.** A connector implements three functions, and everything after the raw data lands is shared.

- `discover(connection)` lists the streams on offer, such as the tabs in a sheet.
- `check(connection, stream, state)` returns cheap change information, such as a modified time, a content hash or a cursor, so unchanged data costs no download.
- `fetch(connection, stream, state)` writes raw objects to the raw bucket and returns a manifest and the new state.

The MVP sync modes are `snapshot` (replace) and `append`. Cursor-based incremental sync can be added later without changing the contract, because the core treats a connector's state as opaque.

**Shared by every connector:** normalization (parsing, PII masking, schema checks), versioning, profiling, derivations, the query API and MCP. A new connector adds one plugin and its settings schema, and no new event types. Connectors run inside the connector worker as Rust modules registered in one compile-time list, and a connector with heavy dependencies can later run as its own container behind the same contract.

## Architecture overview

Clients call one thin API service, and every other step is a worker claiming an event from a queue in PostgreSQL, with PostgreSQL and the object store as the only state.

&#91;embedded content: architecture · 4 layers, 5 workers, 2 stores\]

Each worker claims its deliveries from PostgreSQL, reads what it needs from the object store, then writes its results and the next events in one transaction. The workers are connector (fetches raw data), normalize (parses, masks and versions it), profile, derive (plans and runs derivations) and notify. There is no broker, no outbox relay and no central orchestrator. Interactive queries bypass the queue and read derived Parquet directly. The scheduler is a small separate container that requests syncs for connections that are due, and locally every box runs as a container on one Compose network.

## Local and cloud deployment

Every concern has an open-source component that runs as a container locally and the same container in the cloud. PostgreSQL carries the event queue as well as the metadata, so there is no separate broker. MinIO is not the object store because its community edition was archived in April 2026, so RustFS (Apache 2.0, reported generally available as 1.0) takes that role.

| Concern | Component | Local and cloud |
| --- | --- | --- |
| Event queue | PostgreSQL tables (`events`, `deliveries`) claimed with `SKIP LOCKED` | Part of the same PostgreSQL container, so nothing extra to run. In the cloud, the same image or any managed PostgreSQL |
| Object storage | RustFS (S3 API) | One container, or plain files in dev. In the cloud, the same container, or any S3-compatible service by changing the endpoint. RustFS uses path-style addressing |
| Metadata database | PostgreSQL | One container. In the cloud, the same image or any managed PostgreSQL |
| Query and transform engine | Polars 2.0 in worker and API processes | A Rust library (the Polars crate), so no extra service |
| Workers and API | Rust containers: connector, normalize, profile, derive and notify workers, and an axum API | Compose services. Compose on a VM for pilots, Kubernetes later |
| Ingress and rate limits | Traefik | Compose service with local TLS. Same in the cloud, or as the Kubernetes ingress |
| Authentication | Built-in email and password with server-side sessions in PostgreSQL | Part of the API, so nothing extra to run. An identity provider can replace it later behind the `AuthProvider` port |
| Scheduler | Scheduler container that requests syncs for due connections | Compose service |
| LLM access | Gateway interface, Ollama locally, a hosted API or vLLM in the cloud | Ollama in an optional profile. Hosted calls need the masking rules |
| Email | Any SMTP server, Mailpit locally | Mailpit container. Any SMTP provider in production |
| Observability | OpenTelemetry, Prometheus, Grafana, Loki | Optional compose profile. Same stack in the cloud |
| Secrets | SOPS with age, OpenBao later | Encrypted env file. Kubernetes secrets or OpenBao in the cloud |
| Provisioning | Pulumi (TypeScript) | Not needed locally. Provisions VMs, networks and DNS in the cloud |

**Compose profiles:** `core` (Traefik, API, workers, PostgreSQL, RustFS, web app), plus optional `obs`, `llm` and `mail`. Measure laptop memory early instead of assuming how light the stack is.

## Event model

The nine domain events and the envelope are generic: nothing in them names a particular connector. Each event is a row in an `events` table, and delivery to each consumer is a row in a `deliveries` table. The envelope is `id` (ULID), `type`, `version`, `tenant_id`, `occurred_at`, `correlation_id`, `causation_id` and `data`, and `data` holds references only.

| Event | Producer | Consumers | Key data |
| --- | --- | --- | --- |
| ConnectionSyncRequested | Scheduler (when due), API (manual sync or upload-complete) | Connector worker | connection\_id, stream\_id, trigger |
| RawBatchLanded | Connector worker | Normalize worker | connection\_id, stream\_id, raw manifest URI, connector state |
| SchemaConflictDetected | Normalize worker | Notifier | stream\_id, conflict types |
| DatasetVersionPublished | Normalize worker, Derive worker | Derivation planner, Profile worker (new source schemas), Notifier | dataset\_id, version, kind, snapshot\_uri |
| DatasetProfileProposed | Profile worker | UI reads the stored profile | dataset\_id, profile\_uri |
| SchemaVersionConfirmed | API | Normalize worker (resumes), Derivation planner | dataset\_id, schema\_version |
| DerivationRunRequested | Derivation planner, API | Derive worker (runner) | derivation\_id, spec\_version |
| DerivationRunFailed | Derive worker (runner) | Notifier | run\_id, failed test names |
| DataDeletionRequested | API | Deleter | tenant\_id, scope |

**Delivery rules** (a design built on well-known PostgreSQL features, described in the sources below)

- **One transaction:** a service writes its state change, one `events` row and one `deliveries` row per subscribed consumer in a single PostgreSQL transaction. Which consumers subscribe to which event type lives in code and is checked in tests. No outbox relay is needed, because the queue sits in the same database as the state.
- **Claiming:** a worker claims due deliveries with `SELECT ... FOR UPDATE SKIP LOCKED`, sets a lease (`locked_until`) and an attempt count, and commits at once so no transaction stays open during the work. It then handles the event and, in a new transaction, writes its results and marks the delivery done. If a worker crashes, its lease expires and the delivery becomes claimable again.
- **Wake-ups:** poll about once a second as the reliable baseline. `LISTEN/NOTIFY` is an optional latency hint only, because a notification sent while no worker is listening is lost, and `LISTEN` does not work through a transaction-mode connection pooler.
- **Retries and dead letters:** a failure retries with backoff by setting `run_after`. After a maximum number of attempts the delivery's status becomes `dead`, which raises an alert and can be re-queued from the UI or a CLI.
- **Idempotent effects:** a worker's state change and its delivery completion commit together, so a retry never repeats them. Side effects outside PostgreSQL, such as writing objects or sending email, use deterministic keys like the version path of a snapshot.
- **No ordering guarantee under retries:** snapshots and dataset versions carry increasing numbers, and consumers ignore stale ones.
- **Replay:** events stay in the `events` table, so a replay inserts new deliveries for a chosen consumer and time range. Partition `events` by month and drop old partitions, and delete finished deliveries after a short retention to limit table bloat.
- **Contracts:** one JSON Schema per event in the repo, checked on publish and in consumer contract tests. Additive changes only, and a new `version` for breaking ones.
- **Debounce:** the derivation planner inserts its delivery a few seconds into the future with a unique key on consumer and dedupe key, so a burst collapses into one run.
- **Scale limit:** this suits pilot-scale volumes. A jobs table is a work queue, not an event streaming platform, so watch queue depth and revisit if event volume or fan-out grows sharply.
- **Build or adopt:** write a small module behind the `EventStore` port. Existing PostgreSQL queue libraries and the pgmq extension cover the claiming mechanics, but I have not checked whether any of them handles per-consumer fan-out and replay, so confirm that before adopting one.

## Core flows

An upload becomes a derived dataset through seven event hops, and the owner's schema confirmation is the only human step in the chain.

&#91;embedded content: event flow · 8 steps, 7 hops\]

Other paths reuse the same pieces:

- **Scheduled connectors:** the scheduler emits `ConnectionSyncRequested` for each connection that is due. The connector worker calls the connector's `check`, and only a change leads to `fetch` and `RawBatchLanded`. Normalization then writes a new dataset version and joins the chain at the profiling step. Breaking schema drift emits `SchemaConflictDetected`, pauses that stream and alerts the owner while the last valid version keeps serving.
- **Later uploads:** the same flow, with the upload-complete call as the trigger. Normalize compares the new file's schema to the confirmed version. A match appends or replaces as the owner chose, and a mismatch emits `SchemaConflictDetected` and waits for `SchemaVersionConfirmed`.
- **Derived levels:** each `DatasetVersionPublished` from a derivation re-enters the planner, which is how the cascade continues. The next section covers it.
- **Chat and MCP queries:** synchronous. The API maps the question to a named tool, runs a structured Polars query on the current derived version, and returns the answer with version and freshness. No event is needed unless the call changes state.

## Multi-level derivation

Derived datasets can read other derived datasets, so one source change can cascade through several levels. The cascade is event-driven, but correctness comes from state in PostgreSQL: events only wake the derivation planner, and the planner decides what to run by reading the dependency graph.

&#91;embedded content: dependency graph · 5 datasets, 3 levels\]

**How the cascade runs**

1. **Publish.** A normalize or derivation run publishes a dataset version and emits `DatasetVersionPublished`. In the picture, `orders` v8 has just been published.
2. **Mark stale.** The planner receives the event. In one transaction it finds every derived dataset downstream of `orders` with a recursive query over `derivation_inputs`, and marks them all stale. That is why `monthly_summary` is already stale before anything has run.
3. **Queue what is ready.** For each derivation that reads `orders` directly, the planner checks that every input is a source dataset or a fresh derived dataset. `revenue_daily` and `margin_by_product` qualify, so each gets one pending `DerivationRunRequested` delivery. `monthly_summary` does not, because its inputs are stale.
4. **Run in parallel.** The runner claims each delivery, resolves the latest published version of every input, runs the derivation, tests the result and writes the new version.
5. **Publish and repeat.** Each successful run marks its dataset fresh and emits `DatasetVersionPublished`, and the planner applies the same rule to that dataset's direct downstream. When both inputs are fresh, `monthly_summary` is queued once and runs on a consistent pair of inputs.

```
on DatasetVersionPublished(dataset) or SchemaVersionConfirmed(dataset):
    begin transaction
        mark every derived dataset downstream of `dataset` as stale   -- recursive query over derivation_inputs
        for each derivation d that reads `dataset` directly:
            if d is not blocked and every input of d is a source or a fresh derived dataset:
                insert one pending DerivationRunRequested delivery for d   -- unique per derivation
    commit

on derivation run succeeded:
    publish the new version, mark the dataset fresh,
    then apply the rule above to its direct downstream
```

**Dataset states**

| State | Meaning | Leaves when |
| --- | --- | --- |
| fresh | The current version was built from the current versions of all inputs | An input publishes a new version |
| stale | An input or an ancestor changed, and a rebuild is waiting or queued | The derivation starts running |
| running | A run holds the derivation's lease | The run succeeds, fails or is blocked |
| failed | The last run failed after retries. The previous version keeps serving, labelled stale | A retry or a spec fix succeeds |
| blocked | The derivation cannot run, for example because an input column it uses was removed in a new schema version | The owner confirms a mapping or edits the derivation |

**Rules**

- **Diamonds run once:** a derivation with several stale ancestors waits until all of its inputs are fresh, so it never builds from a mix of new and old inputs. The trade-off is that the final result appears only after the slowest branch finishes.
- **Inputs are pinned and recorded:** each run writes the exact input versions into the new version's lineage, and a dataset is fresh only when its recorded inputs equal the current ones. The output path comes from the derivation ID and a hash of the input versions, so a retried run writes the same path.
- **Changes during a run:** if an input publishes while a derivation runs, the planner marks the derivation stale again and queues one follow-up run after the current one finishes.
- **Bursts collapse:** at most one pending run per derivation, and the run resolves the latest inputs when it starts, so extra events add no extra runs.
- **Failures stop at the failed node:** downstream datasets stay stale and keep serving their last good versions, with a stale label and the upstream cause. One alert goes to the notifier, and a successful retry continues the cascade by itself.
- **Schema changes upstream:** a confirmed schema version triggers a spec check of every derivation that reads the dataset. Those whose column IDs still exist with compatible types proceed, and the rest become blocked, which stops the cascade there until the owner decides.
- **No cycles:** saving a derivation runs a graph check and rejects a cycle. A depth limit and a fan-in limit keep cascades bounded.
- **Level triggered, not event triggered:** the planner re-reads state instead of trusting the event payload, so duplicate, late or out-of-order events cannot corrupt it. A periodic sweep re-runs the planner for any stale dataset with no pending run, which catches a lost wake-up.
- **One run at a time per derivation:** a unique constraint on the derivation's active run stops two workers from building the same output.
- **Manual rebuild:** the API can request a run of one derivation, optionally with its downstream.

## Data stores and model

The object store holds data and PostgreSQL holds metadata, as before. DynamoDB is gone because PostgreSQL now holds the event queue and idempotency state, in the same transaction as the state they protect.

```
s3://raw/{tenant}/{connection}/{stream}/{batch_id}/...      short retention, deleted after normalize
s3://data/{tenant}/{dataset}/v{n}/part-*.parquet            dataset versions, source and derived
s3://data/{tenant}/{dataset}/manifest.json                  schema, content hash, lineage, test results
```

| Table | Holds |
| --- | --- |
| tenants, users, memberships | Accounts, Argon2id password hashes and roles |
| sessions, invites | Server-side sessions with expiry, and one-time invite and password-reset tokens |
| credentials | Encrypted secrets and tokens used by connections |
| connections, streams | Connector type, settings, sync schedule and state, and the streams discovered or chosen |
| schema\_versions | Column IDs, names, types, descriptions, confirmed or proposed, version number |
| datasets, dataset\_versions | Kind (source or derived), state (fresh, stale, running, failed, blocked), current pointer, object URI, row count, content hash, input versions as lineage |
| derivations, derivation\_inputs, derivation\_runs | Spec with version, the input edges that form the graph, run status, test results and timings |
| api\_keys | Hashed keys, dataset scopes, last used |
| events, deliveries | Events and per-consumer deliveries with status, lease, attempts and run\_after |
| audit\_log | Every API and MCP read and every confirmed schema change |

- **Stable content hashes:** because Polars 2.0 may reorder rows, sort every output by its declared keys before writing, so an unchanged result hashes identically and does not create a spurious version.
- **Tenant isolation:** the tenant ID is in every row and object key, with PostgreSQL row-level security as a backstop. Most self-hosted S3 stores cannot issue per-tenant session policies the way AWS can, so isolation depends on the application layer and one access key per service. Verify what the chosen store supports.
- **Minimizing personal data:** raw batches are deleted after normalize or on a short TTL, snapshots are stored masked, and the deleter handles erasure across the object store, PostgreSQL and backups.
- **Table format:** plain Parquet plus a manifest for the MVP. Polars 2.0 can write Iceberg, which is the later option if you need table-level time travel.

## Compute and derivation execution

Polars 2.0 runs inside the worker and API processes as a library. The reports I found say its default engine is now the streaming engine, which processes data in parallel chunks and does not guarantee row order in joins and group-bys unless you opt in. That drives most of the choices below.

- **Spec compiler:** each step in a derivation's JSON spec (filter, select, cast, derive, group by, join, union, sort, limit) is a serde enum variant that compiles to lazy Polars expressions over column IDs mapped to names. The expression grammar is a whitelist of column references, literals, arithmetic, comparisons and a fixed function set.
- **No code or SQL from users or the LLM:** no UDFs, no `sql_expr`, no SQL context fed with spec text, and no file access from the spec. Data is read only through the platform loader, which knows the tenant's object prefix. Polars 2.0 also has a SQL engine, but compiling to expressions keeps the attack surface smaller and the plan testable.
- **Explicit order:** the spec requires sort keys on every output. Where order matters inside a step, the compiler requests order-preserving behavior, and the output is sorted before writing.
- **Writing results:** write with `sink_parquet` so a result need not be fully materialized, with zstd compression, to the next version path. The runner resolves the latest published version of every input once, at the start, and records those versions as lineage, so a run can be reproduced.
- **Tests before publish:** compute row counts, null counts per column and totals reconciliation in Polars. Emit `DatasetVersionPublished` only if they pass. A failure emits `DerivationRunFailed` and leaves the current pointer unchanged, and downstream datasets stay stale.
- **Memory:** reports conflict on out-of-core support. One says spill-to-disk is on by default for supported operations, and another says full out-of-core is planned for a later release. Treat workers as memory-bound until you load-test, and size container limits from your largest pilot file.
- **Interactive queries:** structured filters, group-bys and aggregates become a lazy scan of the derived Parquet with projection and predicate pushdown and a row limit. Run each query in a child process with a timeout and memory cap, because a running Polars query is hard to cancel inside the API process.
- **Normalize:** read Excel with the calamine crate for both data and layout cues such as merged cells (its merged-cell support is unchecked), then run the report-style header and subtotal detection. Decode legacy encodings with encoding\_rs before Polars sees the bytes, and normalize Vietnamese text to NFC.
- **Scaling:** run more worker replicas or a larger container. Polars' distributed engine and Polars Cloud are commercial, so they are not part of this design. Because the engine sits behind an interface, DuckDB could return later for ad hoc SQL if analysts need it.
- **Upgrade discipline:** pin the exact Polars version, run the 2.0 upgrade guide's checks in CI, and replay a fixed set of real pilot files on every upgrade to catch changes in results.

## AI layer

The model sits behind a gateway interface, so the same code runs against a local open-weight model on a laptop or a hosted model in production.

- **Interface:** `profile_dataset`, `draft_derivation` and `answer_with_tools`, with model and prompt versions in configuration, output validated against JSON Schemas, retries, and a per-tenant token budget. LiteLLM is an optional adapter if you want one client for many providers.
- **Local default:** Ollama in the optional `llm` compose profile, serving an open-weight model. This keeps development offline and free. How well small local models handle Vietnamese schema descriptions is unverified, so gate them on the golden set below.
- **Hosted option:** a hosted model API, or Bedrock if you deploy on AWS. When I checked earlier, newer Claude models on Bedrock from Singapore, Sydney and Jakarta supported only global cross-region routing, so assume prompts may be processed outside the region. Self-hosted vLLM is the option for tenants who cannot send even masked samples abroad.
- **Boundaries unchanged:** the model sees schema, descriptions, masked samples and aggregates only. It never runs queries and never sees raw rows. Chat is tool calling over named tools, each a deterministic Polars query with row limits, and every answer states the dataset version and freshness.
- **Logging:** keep prompts and responses out of logs beyond hashes and metadata.
- **Evaluation:** a golden set of Vietnamese and English questions per template, plus profiling accuracy against owner-confirmed schemas, run on every model or prompt change. Use it to decide whether a local model is good enough.

## APIs, authentication and MCP

One modular axum service serves the web app, REST clients and the MCP endpoint behind Traefik, with built-in sessions handling sign-in.

- **Auth:** built-in email and password. Passwords are hashed with Argon2id, and a sign-in creates a server-side session in PostgreSQL, sent as an `HttpOnly`, `Secure`, `SameSite` cookie. The tenant comes from the session, never from the request body.
- **Accounts are invite-only for the pilot:** you create the tenant and send a one-time invite link, and the user sets a password. Password reset uses single-use, expiring email tokens. There is no social login, two-factor authentication or self-service signup in the MVP.
- **Later:** the `AuthProvider` port lets an identity provider such as Keycloak, Authentik or Ory replace the built-in provider without touching the rest of the API.
- **Ingress:** Traefik terminates TLS, applies rate limits, including on the sign-in endpoints, and routes to the API and the web app. The same setup works locally with a local certificate.
- **REST:** resources `/connections`, `/streams`, `/datasets`, `/derivations`, `/runs` and `/query`, described by an OpenAPI spec, with an `Idempotency-Key` header on every POST.
- **Uploads:** the API returns a presigned multipart URL for the object store, and the browser uploads directly. The browser then calls an upload-complete endpoint, which checks the object exists and inserts a `ConnectionSyncRequested` event for the `file_upload` connection. This avoids relying on store-specific bucket notifications, so it works on any S3-compatible store. RustFS's compatibility notes list presigned PUT URLs and multipart upload as tested and recommend enabling CORS for web clients, so test the full browser upload flow early.
- **Status in the UI:** poll `/runs` and the dataset state (fresh, stale, running, failed, blocked) first, then add server-sent events if polling feels slow.
- **MCP:** a remote server over streamable HTTP, mounted in the same service, exposing `list_datasets`, `describe_dataset` and `query_dataset`. Queries are structured filters, group-bys and aggregates with row limits, never raw SQL.
- **MCP access:** per-dataset API keys, stored hashed, in the first version, and OAuth for client apps later. Keys are read-only and rate-limited at Traefik, and every read is audited.
- **Responses** always include the dataset version, snapshot ID, last-synced time and whether the dataset is currently stale.

## Security, privacy and hosting

Portable containers make the Vietnam residency question easier: the whole stack can sit in Vietnam, and only masked schema and samples go to a hosted LLM. Counsel still decides what the law requires (this is not legal advice).

- **Network:** only Traefik is exposed. PostgreSQL and the object store sit on an internal network.
- **Accounts:** Argon2id password hashes, expiring server-side sessions, single-use invite and reset tokens, and rate-limited sign-in. Add two-factor authentication and an identity provider before opening self-service signup.
- **Encryption:** disk encryption on volumes, TLS at the edge and between services outside a single host, and application-level envelope encryption for stored credentials, with the key from the secret store.
- **Connector credentials:** stored only as `credentials` rows, referenced by connections and never logged. For Google, scope is limited to files the owner picks with Google Picker (`drive.file`), and the credential is revoked on disconnect. Every new connector must declare the minimum scopes it needs.
- **Service credentials:** one object-store key and one PostgreSQL role per service, with least privilege.
- **Supply chain:** pin image digests, scan images in CI and keep a patch routine. You now own that work, which a managed cloud service would have done for you.
- **Logs and audit:** structured logs with redaction, plus the application audit log.
- **Deletion:** `DataDeletionRequested` drives erasure across the object store, PostgreSQL and backups, and the backup retention period must be stated to merchants.

**Where to host.** AWS opened a Local Zone in Hanoi in June 2026 (ap-southeast-1-han-1a), announced with EC2 on C7i, M7i and R7i, EBS with local snapshots, ECS, EKS, VPC and Application Load Balancer. That is enough to run this stack as containers.

| Option | Where data lives | Trade-offs |
| --- | --- | --- |
| A. Compose on one EC2 VM in the Hanoi Local Zone | Vietnam | Single zone and newly launched. Narrow instance choice. You run everything yourself |
| B. Compose or Kubernetes at a Vietnam-based provider | Vietnam | Same containers. Provider choice, service levels and compliance still to assess |
| C. Singapore or another region | Outside Vietnam | Easiest operations. Needs a cross-border transfer analysis |

Recommendation: develop locally, run pilots as Compose on one VM in option A or B, and take backups into the same country unless counsel approves otherwise. Moving between options is a redeploy, not a redesign.

## Deployment and runtime view

This view shows what runs where on a pilot host and how a release gets there. It reuses the components from the architecture section: the same Compose file runs on a laptop and on the pilot VM, and only the host, the secrets and the image tags change.

&#91;embedded content: deployment topology · 1 VM, 1 public entry, 7 worker roles, 2 stores\]

Only Traefik is published to the internet. The other containers sit on the Compose network, and the outbound calls (Google Sheets, a hosted model API, an SMTP provider) are made by the workers and the api when a connector, the AI layer or the notify worker needs them.

**Where each environment runs**

| Environment | Where it runs | How it starts |
| --- | --- | --- |
| Laptop | PostgreSQL, RustFS and Mailpit in containers, with api, worker and web as local processes that reload on change | `just dev` |
| CI | The built images, with the end-to-end tests run against them | The `just test-e2e` recipe after the image builds |
| Pilot VM | Every instance as a Compose service, from images tagged with the git SHA | A deploy after a manual approval on the main branch |
| Later | The same images on Kubernetes, with Traefik as the ingress | Not built for the MVP |

**Runtime rules**

- **State lives in two places.** PostgreSQL and RustFS each keep their data on a volume. Every other container is stateless and is replaced on each release, so backups and restore drills cover those two volumes.
- **One image, many instances.** Each worker role is the same image started with a different `--role`, so a role can be restarted or scaled alone. The scheduler runs as a single instance.
- **Start order.** PostgreSQL and RustFS start first, then the migration, then the api and the workers. A restarted worker claims deliveries again once their leases expire, so a restart does not lose work.
- **Optional profiles.** `obs`, `llm` and `mail` are off by default. Start them for dashboards, a local model or a mail catcher. Hosted model calls need the masking rules.
- **Same images later.** Moving to Kubernetes changes the manifests, not the images: Traefik becomes the ingress and each worker role can become its own deployment.

**How a release reaches the pilot VM**

&#91;embedded content: release flow · 7 steps, 1 approval gate\]

The approval gate sits before anything leaves CI for the pilot, and the migration runs as a one-off container before the instances restart, so the old and new code never both run against an unmigrated schema.

## Reliability, observability and cost

Fail loud: every failure becomes a visible state and an alert, never a silent skip. Without managed services, backups and restores become your job, so test them.

**Reliability**

- One PostgreSQL holds both state and the event queue, so it is the single point of failure on a pilot VM. Accept that for pilots, and move to a replicated PostgreSQL with automatic failover before merchants depend on the product.
- PostgreSQL point-in-time recovery with WAL-G or pgBackRest, object-store replication or scheduled copies, and a restore drill before the first real file.
- Retries with backoff and jitter on Google and LLM calls, with a circuit breaker, and worker timeouts shorter than the delivery lease.
- Recovery targets (RPO and RTO) to be set with pilot merchants, not assumed now.

**Observability**

- OpenTelemetry in every service, with the `correlation_id` carried through events, sent to a collector.
- Prometheus alerts on dead deliveries, the age of the oldest pending delivery, derivation failure rate, the age of the oldest stale dataset and sync staleness against a freshness target (proposed: changes visible within 15 minutes at the default poll interval).
- Grafana dashboards for uploads, runs, test failures and time to first derived dataset, which feed the pilot success criteria.

**Cost drivers**

- No per-request cloud bills. Costs are the VM or VMs, disks, bandwidth, LLM tokens if hosted, and engineering time for operations, which is the hidden one.
- Keep pilots cheap with one VM sized from a load test, and delay Kubernetes until a single VM is outgrown.

## Decisions, build order and open questions

Nine decisions hold the design together, each with the alternative I rejected and the signal to revisit it.

| Decision | Choice | Alternative | Revisit when |
| --- | --- | --- | --- |
| Event queue | PostgreSQL tables claimed with SKIP LOCKED | NATS JetStream, pgmq, pgqueuer, RabbitMQ, Kafka | Event volume or fan-out outgrows a database queue, or you need streaming replay at scale |
| Object store | RustFS (Apache 2.0) | SeaweedFS (Apache 2.0), Garage (AGPL, single binary), Ceph, archived MinIO | Upload or read tests expose gaps in its tested subset of the S3 API, or you need cluster durability beyond a pilot |
| Engine | Polars 2.0 on typed specs | DuckDB with SQL | Analysts need ad hoc SQL, or Polars 2.0 proves unstable |
| Metadata store | PostgreSQL | Managed PostgreSQL | You want cloud-managed backups and failover |
| Authentication | Built-in email and password with sessions | Keycloak, Authentik, Ory, a hosted identity provider | Merchants need single sign-on, social login or two-factor, or accountants need delegated access |
| Ingress | Traefik | Caddy, nginx | Rate-limit or MCP needs outgrow it |
| Orchestration | Event-driven stale marking and per-derivation runs by the planner | Temporal, Argo Workflows, Dagster or Airflow-style schedulers | Derivations need compensation or human approval steps, or the graph outgrows a database-driven planner |
| Deployment | Compose on one VM, then Kubernetes | Kubernetes from day one | One VM is outgrown |
| Observability | OpenTelemetry, Prometheus, Grafana, Loki | SigNoz | AGPL licensing is a problem |

**Build order** (six steps across the four MVP phases: steps 1 and 2 are Phase 1, step 3 is Phase 2, step 4 is Phase 3, steps 5 and 6 are Phase 4)

1. Foundation: Compose stack, ports and adapters, CI, events and deliveries tables with the claim loop, observability, and built-in authentication
2. Connector framework with the `file_upload` connector, the normalize worker, dataset versions and profiling
3. Derivations with tests, then the planner and the multi-level cascade
4. API, web app and chat
5. The `google_sheets` connector, which is the first test of the connector contract, and the MCP server
6. Hardening: backup and restore drills, deletion workflow, load test with real pilot files, then the first VM deployment

**Open questions**

- [ ] Where do pilots run: Hanoi Local Zone, a Vietnam-based provider, or Singapore, and does counsel agree?
- [ ] Does RustFS pass the full browser upload flow (presigned multipart and CORS) and run stably on the pilot VM?
- [ ] Does the PostgreSQL queue hold up in a load test at pilot-scale event volume, including a deep derivation cascade?
- [ ] Is Polars 2.0 stable enough on real pilot files, and is the release date right?
- [ ] Does a local model handle Vietnamese well enough, or do pilots use a hosted model with masking?
- [ ] Is AGPL acceptable for the observability stack, or should it be SigNoz?
- [ ] Is invite-only email and password enough for pilots, or does any merchant need single sign-on or two-factor?
- [ ] Is waiting for all inputs the right policy for diamonds, and what depth and fan-in limits should derivation graphs have?

## Sources

Facts come from search results gathered while writing, not individually opened pages. Statements marked as my own knowledge are unverified.

- [Polars 2.0.0 release candidate notes](https://github.com/pola-rs/polars/releases/tag/py-2.0.0-rc.1)
- [RuntimeWire: Polars 2.0 makes streaming the default](https://runtimewire.com/article/polars-2-ritchie-vink-streaming-out-of-core)
- [Polars 2.0 upgrade guide (third-party)](https://www.kunalganglani.com/blog/polars-2-0-upgrade-guide)
- [Polars 2.0 overview (third-party)](https://www.stork.ai/blog/polars-20-will-break-your-code)
- [MinIO archived: self-hosted alternatives](https://pinggy.io/blog/minio_archived_self_hosted_s3_alternatives/)
- [Open edX: replacing MinIO with Garage](https://discuss.openedx.org/t/replacing-minio-in-tutor-minio-why-we-chose-garage-and-what-comes-next/18868)
- [MinIO alternatives in 2026 (RustFS, Garage, SeaweedFS)](https://dev.to/ethan-carter/best-minio-alternatives-in-2026-6-options-that-actually-work-17p2)
- [RustFS 1.0 reaches GA (Linuxiac)](https://linuxiac.com/?p=219580)
- [RustFS S3 compatibility matrix](https://docs.rustfs.com/en/reference/s3-compatibility)
- [RustFS S3 API guide](https://docs.rustfs.com/administration/protocols/s3)
- [Postgres as a message queue with SKIP LOCKED](https://env.dev/guides/postgres-as-a-queue)
- [You don't need a job queue, Postgres already has SKIP LOCKED (Prisma)](https://www.prisma.io/blog/you-dont-need-a-job-queue-postgres-already-has-skip-locked)
- [Task queues on PostgreSQL: pgqueuer (PyCon Sweden 2025)](https://pretalx.com/pycon-sweden-2025/talk/8ASAP7)
- [AWS: Local Zone in Hanoi announcement](https://aws.amazon.com/about-aws/whats-new/2026/06/aws-local-zones-hanoi-vietnam/)
