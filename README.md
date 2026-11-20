# WoW Goldtool

A desktop app — **Rust + Tauri 2.0** — that downloads the item catalogue of **WoW
Forever** from the free [TheWoWDB](https://thewowdb.com/developers) JSON API and keeps
it in a local **SQLite** database, so browsing, filtering, searching and price-checking
items works offline.

Status: **v0.5.1 — the app stays alive while it crawls** (async IPC command:
hover, clicks and the status card all work mid-chunk; the sync note counts the
chunk's seconds). Before the patch: watch lists, targeted detail sync, an offline
item card, price analytics. The GUI shell landed at v0.4.0.
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
| `v0.6.0` | price history (a second table: one price per item cannot show a trend), local icon copies | planned |

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
* **Two-stage sync.** The API lists ~19 k items cheaply (name/quality/level) but prices,
  stats and icons need one request per item. Stage 1 is fast, stage 2 is staged, resumable
  and cancellable — see [ADR 0002](./docs/DECISIONS/0002-two-stage-sync.md). From v0.5.0
  stage 2 also takes **targets**: `--ids 25,40` and `--watch` fetch exactly what the user
  asked for, and the UI drives them in slices of 20 so Cancel stays meaningful between
  slices. Re-asking for an id that already has a record revalidates (`304`, nothing written).
* **The card answers offline.** `item ID --cached` reads the stored `raw_json` and sends
  no request; the card prints when that record was fetched, so the user can tell today's
  prices from last month's. Item icons are deliberately *not* hotlinked: `icon_url` is a
  remote path, and the app's point is answering without the network.
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
