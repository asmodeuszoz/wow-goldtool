# ADR 0003 · Local storage: SQLite via `rusqlite` (bundled)

- **Status:** accepted
- **Date:** 2026-11-19
- **Discussion:** [docs/DATABASE.md](../DATABASE.md) · [ADR 0002](./0002-two-stage-sync.md)

## Context

We must cache ~19 k item records and answer, offline and fast:

* substring name search (`"shortsword"`),
* typed filters (quality, item type, slot, level range, "has price", watched only),
* sorts the upstream API cannot express (`price_min` asc/desc, `dps` desc, `ilvl` desc),
* facet counts for the filter chips,
* coverage/price analytics,
* and per-row detail lookups — all while a sync thread writes batches to the same file.

## Options considered

| Option | Verdict |
| ------ | ------- |
| JSON files + in-memory structs | Fast for one-shot reads, but filters/facets/sorts/coverage queries become hand-rolled Rust that must be kept in sync with the cache format; no resumable partial state |
| `sqlite3` crate (raw bindings) | Works, but every query is `queryf`/`Query<T>` plumbing we would wrap anyway |
| `sqlite_orm` | Typed, compile-checked queries, but it needs SQLite headers/system lib and generates large templates for a 30-column record; heavier than needed |
| **`rusqlite` with the `bundled` SQLite amalgamation** | Ergonomic `execute`/`query` + `params`, no system `libsqlite3` dependency, vendored source compiles offline |

## Decision

Use `rusqlite = "0.43"` with `features = ["bundled"]`, one SQLite file (disposable),
schema documented in [docs/DATABASE.md](../DATABASE.md):

* one wide `item` row per catalogue item — summary columns written by stage 1, price/stat
  columns by stage 2, `raw_json` kept for forward-compatibility;
* `meta` for singletons (schema version, catalogue build, attribution, timestamps);
* `watch` for the user's star list (also the "sync details for these" target);
* `sync_state` for resumable sync bookkeeping;
* indexes exactly for the query shapes the UI issues (`name_lc`, `quality_rank`,
  `item_level`, `price_min`, `detail_synced_at`).

Discipline rules baked into `query.rs`:

* every user-supplied value is a **bound parameter**, never string-interpolated;
* the search term is escaped for `LIKE` (`%`, `_`, `\`) and every pattern carries
  `ESCAPE '\'`;
* NULL prices are pushed last in ascending sorts via `price_min IS NULL` as the primary
  key, so "unknown price" never masquerades as "free".

## Consequences

* **Gained:** the DB is greppable with any SQLite tool (`sqlite3 .read`/`.print`), CI can
  seed it from fixtures, and `cargo run -- stats` is the same queries the UI runs.
* **Given up:** no WAL/VFS complexity (default journal), so a reader may briefly hit a
  lock error while a batch is written — handled with a bounded retry in the command layer,
  documented in `docs/DATABASE.md`.
* `bundled` avoids assuming a system `libsqlite3`; it needs a C compiler (`gcc`/`clang`),
  which the probe on this machine confirmed.
