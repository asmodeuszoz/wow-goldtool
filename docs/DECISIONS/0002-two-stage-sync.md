# ADR 0002 · Two-stage sync

- **Status:** accepted
- **Date:** 2026-11-19
- **Discussion:** [docs/DEVLOG.md](../DEVLOG.md) · [docs/API.md](../API.md)

## Context

Measured on 2026-11-19 against the live API:

* The `forever/item` **list** endpoint carries, per item: `id`, `name`, `quality`,
  `item_level`, `required_level`, `links.*`. It is paginated (`per_page ≤ 100`,
  `page ≤ 200`) and the full catalogue is **≈ 19 257 items in 193 pages**.
* Prices, stats, sockets, weapon numbers, flavour text, the item set, classes and the
  **icon** exist only on the **per-item detail** endpoint (`/forever/item/<id>`).
* Rate model per IP: **120 request burst, refilled at 2 req/s**; `429` + `Retry-After`
  beyond that.

Cost table therefore:

| Pass | Requests | Wall clock at 2/s |
| ---- | -------- | ----------------- |
| list crawl | ≈ 193 | ≈ 2 min |
| full detail crawl | ≈ 19 257 | ≈ 2.7 h |

## Decision

Split the sync into two explicit, independently runnable stages, and make the expensive
one opt-in, resumable and cancellable.

* **Stage 1 — `list`.** Default action on a fresh database. Fills the summary columns and
  `list_etag`, writes `sync_state.list`. Cheap, and after it the app can already search,
  filter and sort (name, quality, levels) — just not by price.
* **Stage 2 — `detail`.** Fills prices, stats, icons, `raw_json`, `detail_etag`.
  Available targets: `all`, `ids`, `watchlist`, `query` (the current UI filter),
  `stale` (detail older than a threshold). Runs in a worker thread, re-entrant:
  it only processes items whose `detail_synced_at` is `NULL` or older than the threshold,
  so an interrupted pass resumes where it stopped.
* **Conditional GETs.** `If-None-Match` with the stored `list_etag`/`detail_etag`; a `304`
  updates the timestamp but neither re-parses nor re-writes the row, and it still costs
  rate budget.
* **Budget, not luck.** Pacing keeps 30 requests of headroom below the 120 burst, then
  sleeps to a 2 req/s schedule. A `429` honours `Retry-After` (default 2 s), refills the
  budget low and **continues**; it never aborts a sync. Max two workers against one host.

## Alternatives rejected

| Alternative | Why not |
| ----------- | ------- |
| One "sync everything" button | 2.7 h with no visible progress and no cancel is a UX dead end |
| Fetch details only during search hits | Prices would be invisible to sorting — the app's whole purpose is price ordering |
| Scrape/parallel-fetch faster than 2/s | Disrespects a published rate limit and gets us `429`-banned; explicitly against the terms |
| Store everything as JSON files | Loses the indexed, sortable, filterable queries the UI needs (see [ADR 0003](./0003-sqlite-rusqlite-bundled.md)) |

## Consequences

* NULL price columns mean "detail not fetched yet"; "fetched, not on the AH" is
  representable and distinct. The UI and `cargo run -- stats` surface the difference, so
  users understand why a "sort by price" result is incomplete on a fresh database.
* `docs/DATABASE.md` carries `detail_synced_at` and the "missing data" queries this design
  depends on.
* A full detail pass is a deliberate, signposted action in the UI ("sync all details —
  about 2.7 h"), never a side effect of browsing.
