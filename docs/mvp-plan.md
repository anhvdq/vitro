# MVP Plan: Data Integration and Derived-Data Platform for Small Businesses

Oct 10, 2026 · @Duc Quoc Anh Van

## Summary

The MVP lets a small-business owner upload Excel or CSV files or connect Google Sheets, get an AI-proposed understanding of the data, define derived datasets such as sales by day, and query them through chat, MCP and REST. The first ten pilots are Vietnamese merchants.

Positioning: a low-cost back-office analyst for owners without staff for reporting. It gives trusted numbers that trace back to source rows, not another dashboard builder.

Not in the MVP: direct marketplace and delivery APIs, an agent builder, workflow automation, auto-sent messages, and the personal-knowledge segment for individuals.

## Users and pilot

Ten Vietnamese small merchants run an eight-week pilot. A merchant qualifies if it has 1 to 10 staff, sells through two or more channels (shop, Shopee, Zalo, Instagram, delivery apps), and keeps data in Excel or Google Sheets or can export Excel from Sapo or KiotViet. The owner must be the decision-maker.

- **Recruiting (untested):** seller communities on Facebook and Zalo, and local accountants serving household businesses, given the 2026 household-business tax reform.
- **What they upload:** orders, products with costs, customers (pseudonymized), and receivables lists.
- **Pilot hook:** a monthly revenue-by-channel summary suited to tax declaration. It is a summary only, not tax advice, and the reform's details need checking with an accountant because my source is a vendor blog.

## Scope

The MVP covers two data sources, one derived-data engine and three ways to query it. Everything else waits for pilot evidence.

**In scope**

- CSV and Excel upload (static data, schema set at first upload, later uploads append or overwrite) and Google Sheets with scheduled polling
- AI profiling of each source, confirmed by the owner
- Schema versioning with explicit conflict handling
- Derivation builder with six templates, guided steps, natural-language drafting, preview and tests
- Chat over derived datasets with citations and freshness labels
- MCP server and read-only REST for derived datasets
- Vietnamese and English interface and AI output
- PII-aware upload that detects, drops or masks personal columns

**Out of scope**

- Direct connectors for Shopee, Grab, Uber Eats, Zalo and Instagram (add as approvals arrive)
- Agent crew or agent builder, workflow automation, and any message sending
- GraphQL
- The individual and personal-knowledge segment
- Australian pilots (the design supports them; pilots are Vietnam only)
- Real-time sync and incremental derivations (full refresh first)

## Core features

Seven features make up the MVP, each with a testable definition of done.

| Feature | What it does | Done when |
| --- | --- | --- |
| Upload | Reads CSV and Excel, including report-style exports with title rows, merged cells and subtotals | Header rows and total rows are detected correctly on 10 real exports |
| Google Sheets | Owner picks the file through Google Picker (drive.file scope), then the tab and header row. Polls on a schedule and snapshots only on change | Breaking drift pauses sync and alerts the owner |
| AI profiling | Proposes column meanings, grain, keys, quality issues and join candidates, all marked proposed until confirmed | At least 90% of columns correct on 20 messy files |
| Schema versions | New version on each confirmed change. Derivations reference column IDs, not names | Every conflict (missing column, new column, type mismatch, likely rename, duplicate key) ends in a clear owner choice |
| Derivations | Declarative JSON steps (filter, select, cast, derive, group, join, union, sort) run in Polars. The LLM drafts the spec and the owner previews it | Derived totals match the owner's own figures exactly on 10 test cases |
| Chat | Maps a question to a named metric or dataset query and answers with citations and freshness | 95% of 20 test questions per merchant match their records or are labelled provisional |
| MCP and REST | Tools list\_datasets, describe\_dataset and query\_dataset with structured filters, row limits, API keys and audit logs | A second AI tool reads a derived dataset with its snapshot ID and freshness |

Six pilot templates ship with the derivation builder:

1. Revenue by day, month and channel
2. Top products and slow movers
3. Gross margin by product, joined to a cost sheet
4. Receivables ageing (công nợ)
5. New versus repeat customers, using a pseudonymous ID
6. Channel summary, from a channel column the owner maintains

## Architecture and data model

Every upload or changed sheet becomes an immutable snapshot, and each derived dataset is rebuilt from snapshots by a tested derivation.

&#91;embedded content: architecture · 4 layers, 3 outputs\]

The LLM drafts profiles, derivation specs and chat wording from schema and masked samples. Code computes every number. Append adds rows to the current snapshot and overwrite replaces it, and both keep history for rollback. A sheet creates a new snapshot only when its content hash changes.

When a later upload or sheet read differs from the confirmed schema, the default is to stop and ask:

| Conflict | Default | Owner options |
| --- | --- | --- |
| Column missing | Block | Map to another column, fill nulls, or accept a new schema version |
| New column | Pause | Ignore it or add it in a new version |
| Type mismatch | Show affected rows | Coerce, reject the rows, or change the type |
| Likely rename | Suggest a mapping | Confirm or decline |
| Duplicate key on append | Block | Upsert or reject |

Derivations reference stable column IDs, so a rename never silently breaks them. A derivation whose inputs change in a breaking way is marked stale instead of running on bad data.

## Design rules

The LLM drafts and explains, and deterministic code computes every number. These rules hold across all features.

- **LLM boundary:** the model proposes descriptions, mappings, derivation specs and chat wording. It never does arithmetic or runs queries directly.
- **What the LLM sees:** schema, descriptions and masked samples only, never raw personal data. Sending raw rows to an offshore model API may count as a cross-border transfer, so confirm with counsel.
- **PII handling:** detect names, phones and addresses on upload and let the owner drop, mask or pseudonymize them before storage. Customer analysis uses a pseudonymous ID.
- **Vietnam compliance (verify with counsel, not legal advice):** the Personal Data Protection Law has applied since 1 January 2026. As a processor of merchants' customer data, the platform likely does not qualify for the small-business grace period. Decide the hosting region and sign a data processing agreement with each pilot before real data arrives.
- **Provenance and freshness:** every answer shows the snapshot ID, last-synced time and whether the data is provisional or settled, with drill-down to source rows.
- **Locale:** detect and confirm decimal and thousands separators and day-month order, treat VND as having no minor units, normalize Vietnamese text to NFC, handle UTF-8 BOM and legacy encodings, and store time in UTC.
- **Fail loud:** sync failures, schema drift and rejected rows are always shown to the owner, never skipped silently.

## Timeline

The proposed plan builds in four phases over ten weeks, starts the eight-week pilot in week 7, and runs the kill check in week 10. The durations are my proposal, so adjust them to team capacity.

&#91;embedded content: timeline · 14 weeks, 1 gate\]

1. **Phase 1 (weeks 1 to 4):** upload, report-style Excel parser, PII-aware mapping, schema versions, AI profiling
2. **Phase 2 (weeks 4 to 7):** derivations, six templates, tests, preview
3. **Phase 3 (weeks 6 to 8):** Vietnamese interface and chat
4. **Phase 4 (weeks 8 to 10):** Google Sheets sync with drift handling, MCP and REST

Recruiting and sample-file collection run in weeks 1 to 4 alongside Phase 1.

## Success and kill criteria

The pilot succeeds if at least 5 of 10 merchants use the product weekly for four weeks and at least 3 commit to pay. Fewer than 3 active at week 4 is the signal to rethink.

| Area | Target |
| --- | --- |
| Recruiting | 10 pilots within 4 weeks |
| First session | At least 7 of 10 upload a real file and confirm a schema |
| Correctness | At least 7 of 10 get a monthly revenue summary matching their own records within 10 minutes |
| AI profiling | At least 90% of columns correct on 20 real files |
| Retention | At least 5 of 10 return weekly for 4 consecutive weeks |
| Willingness to pay | At least 3 pre-commit, testing 2 to 3 price points anchored on Sapo's 550,000đ/month Omnichannel plan |
| PII | At least 95% detection on a Vietnamese test corpus, zero incidents |
| Schema drift | 100% detected in the test suite |

**Pivot options if the kill signal fires:** an accountant-led channel selling to many clients, or returning to a marketplace-connector wedge.

## Risks and unverified assumptions

The problem looks validated, but no evidence yet shows merchants will pay for this exact product.

| Risk | Why it matters | Mitigation |
| --- | --- | --- |
| Data protection | Vietnam's 2026 law, processor status and cross-border transfer rules | PII masking, LLM sees no raw personal data, counsel review, hosting decision, data processing agreements |
| Incumbents | Sapo and MISA add AI assistants inside their systems, and tools such as Fabi.ai already offer connect, ask and build | Target Excel and Sheets-only merchants and cross-source joins, and trial the competitors first |
| Willingness to pay | Sapo's survey shows cautious 2026 sentiment | Price test in the pilot, accountant channel |
| Data entry burden | Owners without staff must still get data into files | Import templates and Excel export from Sapo or KiotViet, direct connectors later |
| Messy files | Merged cells, subtotals and renamed columns break naive parsing | Report-style parser, schema contracts, drift alerts |
| Crowded horizontal engine | Generic data tools are plentiful | Vertical templates and Vietnamese-first messaging |

**Unverified:** API approval terms for Shopee, Grab and Zalo, the service-account sharing shortcut for Google Sheets, the accountant channel, the tax-reform details, and why the AI spreadsheet Rows was discontinued in May 2026.

## Open decisions

Seven decisions block or shape the build.

- [ ] Hosting region and legal review for Vietnamese data, before the first real file
- [ ] Who writes and reviews the Vietnamese interface copy and AI prompts
- [ ] Pilot recruiting network for merchants and accountants
- [ ] Price points to test in the pilot
- [ ] Google access path: Picker with the drive.file scope, or sharing with a service account for early pilots
- [ ] Trial Fabi.ai, Basedash, Julius, Airtable Omni, Relevance AI and Coupler.io on one messy Vietnamese Excel file (monthly revenue, a saved derived dataset, exposure to another AI tool)
- [ ] Timing of Australia entry and the accountant channel

## Sources

Figures come from search results gathered during this planning conversation, not from individually opened pages, so verify before relying on them.

- [Google Sheets API scopes](https://developers.google.com/workspace/sheets/api/scopes)
- [Vietnam PDPL summary (Tilleke & Gibbins)](https://www.tilleke.com/?p=68117)
- [Vietnam household business tax reform guide (vendor blog)](https://beancount.io/blog/2026/08/12/vietnam-household-business-tax-reform-2026-guide)
- [Sapo: AI and Vietnamese sellers in 2026](https://www.sapo.vn/bao-chi-noi-ve-sapo/AI-tac-dong-truc-tiep-kha-nang-tang-truong-cua-nha-ban-hang-Viet-Nam-2026-a2789.html)
- [Sapo: omnichannel overview](https://sapo.vn/sapo-la-gi.html)
- [Xero AI analytics launch](https://www.xero.com/au/media-releases/xero-introduces-enterprise-grade-analytics/)
- [Fabi.ai: AI Analyst Agent](https://fabi.ai/product/analyst-agent)
- [Rows vs Airtable (Rows discontinued)](https://automationatlas.io/guides/rows-vs-airtable-2026-comparison/)
- [Coupler.io: Google Sheets MCP](https://www.coupler.io/mcp/google-sheets)
