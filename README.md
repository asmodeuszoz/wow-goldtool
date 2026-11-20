# WoW Goldtool

A desktop app — **Rust + Tauri 2.0** — that downloads the item catalogue of **WoW
Forever** from the free [TheWoWDB](https://thewowdb.com/developers) JSON API and keeps
it in a local **SQLite** database, so browsing, filtering, searching and price-checking
items works offline.

Status: **v0.6.0 — price history + local icon copies** (a second table keeps one price
per **fresh** detail write — `304` revalidations write nothing — so the item card draws a
real trend; every answered record's own icon file is copied into the DB once as a `BLOB`
and the card shows it as a `data:` URI, nothing hotlinked; schema `3`, migrating both
stale shapes in place). Before it: the checked-gone state (v0.5.3), the one-click pull
itself (v0.5.2), the crawl no longer freezing the page (v0.5.1), and the GUI itself (v0.4.0).
See the milestone list below and the dated narrative in [docs/DEVLOG.md](./docs/DEVLOG.md).

```
cargo build          # works from RustRover straight away, no Tauri CLI needed
cargo run            # the app: opens a window (needs a display server)
cargo run -- status  # headless: DB location, coverage, sync state — no display needed
```

## Why this app exists

WoW (and the forever-style clients) keep a huge item catalogue with live auction-house
prices. Answering "what does this cost, and is it worth hunting?" in a browser means
waiting for a website. This app pulls the catalogue once, caches it, and answers from
SQLite — including the sort orders the upstream API cannot give you (cheapest first,
highest damage per second, most expensive vendor sell…).

## Feature roadmap

| Milestone | What lands | State |
| --------- | ---------- | ----- |
| `v0.1.0` | repository scaffold, docs, licence, CI templates | done |
| `v0.2.0` | data layer + working sync: TheWoWDB client, JSON models, SQLite schema, headless CLI (`sync`, `stats`, `search`, `item`, `watch`, `status`), paced two-stage crawl, exit-code contract | done |
| `v0.3.0` | sync hardening: conditional GETs with stored ETags (per page in `meta`, per item in the row), stop→resume on the `sync_state` page cursor, cancel plumbing, live lookup answered from the DB on `304` | done |
| `v0.4.0` | Tauri shell + webview UI: status card, local search (text, quality, sort, `priced only`), chunked sync with cancel; `build.rs` + `tauri-build`, generated icon | done |
| `v0.5.0` | detail-sync targets (`--ids`, `--watch`) driven from the UI, item card that reads the DB first, price-analytics card, watch toggles | done |
| `v0.5.1` | patch: a crawl no longer freezes the page (async `cli_call`), mid-chunk status ticker | done |
| `v0.5.2` | patch: pending = never fetched (the two-state rule), the GUI verb hop off tokio (a blocking-client build inside a tokio worker stalled the crawl at `build_client`), "Pull all info" runs both stages to their end | done |
| `v0.5.3` | patch: a record upstream dropped is a checked third state (`detail_missing_at`, `HttpKind::Missing`, `missing` in the report and in coverage), pending = never fetched and never checked-missing, schema `1 → 2` migrated in place (`ALTER TABLE`) so an existing catalogue keeps its rows | done |
| `v0.6.0` | price history (a second table: one price per item cannot show a trend), local icon copies, the item card rendering `stats` (the only unshaped field grounded live so far) | done |

## Design notes

* **One Cargo project at the repo root.** Tauri's `generate_context!` macro reads
  `tauri.conf.json` at compile time, so plain `cargo build` embeds config and UI assets.
  No `src-tauri/` directory and no Node/Tauri-CLI toolchain is required to build or run
  the app — see [ADR 0001](./docs/DECISIONS/0001-flat-cargo-layout-no-cli.md). What cargo
  *does* need is a `build.rs` calling `tauri_build::build()`: that is the write-half of the
  same compile-time embedding, and without it `window.__TAURI__` never exists. `tauri-build`
  drops JSON schemas into `/gen/`, which is gitignored.
* **One command across the IPC hop.** The webview sends the CLI's argv
  (`window.__TAURI__.core.invoke('cli_call', { argv })`) and gets the same JSON back, so a
  click and a `cargo run --` line cannot drift apart — see
  [ADR 0005](./docs/DECISIONS/0005-gui-ipc-commands.md), which also records that app commands
  are not ACL-checked by construction while no app ACL manifest ships.
* **Two crawl stages plus an icon stage.** The API lists ~19 k items cheaply (name/quality/level)
  but prices and stats need one request per item, and each answered record's own icon FILE
  needs one more. Stage 1 is fast, stage 2 is staged, resumable and cancellable — see
  [ADR 0002](./docs/DECISIONS/0002-two-stage-sync.md) — and v0.6.0 adds a third, `sync --stage
  icons`, with the same budget, cancel and resume plumbing (
  [ADR 0008](./docs/DECISIONS/0008-icon-storage-third-stage.md)). From v0.5.0
  stage 2 also takes **targets**: `--ids 25,40` and `--watch` fetch exactly what the user
  asked for, and the UI drives them in slices of 20 so Cancel stays meaningful between
  slices. Re-asking for an id that already has a record revalidates (`304`, nothing written).
* **The card answers offline.** `item ID --cached` reads the stored `raw_json` and sends
  no request; the card prints when that record was fetched, so the user can tell today's
  prices from last month's, and draws the price **trend** from the `price_history` ledger.
  Item icons are deliberately *not* hotlinked: v0.6.0 copies each icon's bytes into the
  DB as a `BLOB` and the card shows that local copy as a `data:` URI — `icon_url` alone
  is a remote path, and the app's point is answering without the network.
* **Rate limits are respected by construction** (burst budget + 2 req/s pacing + `429`
  backoff + `ETag` reuse). See [docs/API.md](./docs/API.md).

## Data attribution

Data shown in the app is served by [TheWoWDB](https://thewowdb.com/) — the app renders a
visible link back to the source (and to each record's `links.html`) as their
[developer terms](https://thewowdb.com/developers/#terms) require. Game data and names
belong to Blizzard Entertainment; TheWoWDB is a fan site and is not affiliated with
Blizzard. This project does not resell the raw dataset.

## License

See [LICENSE](./LICENSE) (MIT, plus a note about data ownership). Community project, not
affiliated with Blizzard Entertainment or TheWoWDB.
