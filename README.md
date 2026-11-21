# WoW Goldtool

A desktop app — **Rust + Tauri 2.0** — that downloads the item catalogue of **WoW
Forever** from the free [TheWoWDB](https://thewowdb.com/developers) JSON API and keeps
it in a local **SQLite** database, so browsing, filtering, searching and price-checking
items works offline.

Status: **v0.9.2 — old copies get their twin re-derived, with ZERO requests** (`refresh-twins`,
ADR 0013: the copy is the source of truth, so one ask re-derives every twin in the DB from
the bytes already on disk — 4 602 copies on the file that finished a crawl all sat in the
glued v0.9.0 shape, and this door is what clears them; the UI button is `Reword the copies`).
Before it: **v0.9.1 — the scope door ANSWERS and the twin keeps its word spaces** (the v0.9.0
tag shipped a `--where` whose bare `continue` skipped the parser's shared advance — measured
HUNG 2 h 01 m — and glued the live minified html into `Worn ShortswordItem Level 2Main Hand...`
twins; both fixed forward at `ee8c06d` + `5a36d19`, the tag never moved). Before it: **v0.9.0 —
the copies are READABLE and FINDABLE: every copied tooltip gets a derived readable twin** (ADR
0012): schema `5` writes `tooltip_text`/`tooltip_lc` **in the same write** as
the copy on every write path (the tooltip stage derives from the MERGED copy — overrides merge
LAST — `set-item tooltip_html=` derives beside the stated value, `add-item` births it), `search
TEXT --where name|description|all` + a UI where select read it (default `name`; upstream's own
door measured NAMES-only), and the card RENDERS the twin as a words row while the raw upstream
markup stays printed-as-text — the measured copy hotlinks pixels from `render.worldofwarcraft.com`
and `thewowdb.com/assets`, and rendering it would break the icon stage's rule. Before it:
**v0.8.1 — CI is green on all three targets, and the workflow claims only what it can prove**
(run #4: 5/5 jobs in 9 min 13 s): only actions that RESOLVE (`actions/checkout@v7`,
`dtolnay/rust-toolchain@stable`), the dev-library set measured off our own binary's `ldd`
(`libgtk-3-dev libwebkit2gtk-4.1-dev`), clippy informational and NO fmt gate — both because a
gate that cannot pass would be a measured lie about our own tree (ADR 0011); the scratch DB
path and the macOS matrix row (`aarch64-apple-darwin`) are measured runner facts too, and the
repo now has real `icons/icon.png` + `icons/icon.ico` because Windows builds name them. Before
it: v0.8.0 — the DB holds every forever item's description copy (FOURTH sync stage `--stage
tooltip`, its own three states/ETag/budget) and takes edits by hand (the manual layer's ONE
rule — overrides merge LAST — `set-item`/`add-item`/`delete-item`/`undelete-item`, tombstones,
reserved ids at `900 000 000+`, schema `4`),
the live-search door (v0.7.0), price history and local icon copies (v0.6.0), the checked-gone
state (v0.5.3), the one-click pull itself (v0.5.2), the crawl no longer freezing the page (v0.5.1),
and the GUI itself (v0.4.0).
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
| `v0.7.0` | the live-search door: CLI verb `live-search TEXT` + UI panel (`Ask once`), one human ask = ONE request to the grouped search endpoint, upstream's own answer printed as it came with `view upstream` links — and it writes nothing: no row, no sample, no mark (ADR 0009) | done |
| `v0.8.0` | the description copy + the manual layer: a FOURTH sync stage `--stage tooltip` holds each record's own tooltip `html` (three states, ETag, budget, resume — ≈ 19 258 requests ≈ 2.7 h), and CLI verbs `set-item` / `add-item` / `delete-item` / `undelete-item` plus a UI write panel let the user say it / make one / hide it / bring it back; overrides merge LAST on every crawl write, DELETE is a tombstone every query hides, user-made rows live at `900 000 000+` and answer their card from the DB with ZERO requests; schema `4` (ADR 0010) | done |
| `v0.8.1` | CI discipline: the workflow states only measured truths — run #4 **5/5 jobs green in 9 min 13 s** on linux/windows/macos, resolvable actions (`actions/checkout@v7`, `dtolnay/rust-toolchain@stable`) + plain apt (the package set measured off `ldd`: `libgtk-3-dev libwebkit2gtk-4.1-dev`), clippy informational + no fmt gate (both gates would contradict measured facts about our tree), real `icons/icon.png` + `icons/icon.ico` (tauri-build names `icons/icon.ico`; `bundle.icon` had pointed at a file that never existed), and the measured runner facts (`WGT_DB` relative, macOS arm64) — ADR 0011 | done |
| `v0.9.2` | the refresh door (ADR 0013): `refresh-twins` re-derives every twin from the copy the DB already holds — ZERO requests (ADR 0004's `3` is unreachable here), only differing/missing rows written, tombstones and manual rows ride, rows collected before any write, idempotent, exit `0` with a report for the nothing-to-do door; closes ADR 0012's honest gap and v0.9.1's residue (measured on the user's file: 4 602 copies, ZERO matching the corrected rule, ALL 4 602 equal to the glued v0.9.0 shape); the UI button `Reword the copies` rides the same verb; CI smoke 24 → 29 cargo steps + one plant line; schema stays `5`, no hop | done |
| `v0.9.1` | the v0.9.0 defects die here, forward-fixed under its own tag: `search --where` rides the parser's shared advance (measured HANG: bare `continue` on valid values skipped `index + 2`; both post-tag replays 2 h 01 m in state R; scratch `timeout 5` repro rc 124), and the twin derivation replaces every `<…>` span with ONE space (measured glue on the user file's 4 602 live copies `Worn ShortswordItem Level 2Main Hand...` — the fixture's own card keeps spaces at the boundaries, so fixture-shaped data cannot disprove live-shape claims), `fix(cli)` + `fix(db)` + this record | done |
| `v0.9.0` | the readable twin (ADR 0012): schema `5` adds `tooltip_text`/`tooltip_lc` derived from the copied tooltip IN THE SAME WRITE on EVERY write path (tooltip stage from the MERGED copy, `set-item tooltip_html=` beside the stated value, `add-item` at birth — the `name`→`name_lc` pair pattern v0.1.x started; the derivation is the measured three moves, ZERO entities seen in two live bodies so NO decoder is invented), `search TEXT --where name|description|all` (default `name`, bogus scope answers the usage door `2`, tombstones hidden in EVERY scope, DB-only ZERO requests) + a UI where select, and the card renders the twin as a words row and NEVER upstream's markup (its `<img>`s hotlink pixels from two measured hosts), CI smoke 21→24 steps, schema chain v1→v2→v3→v4→v5 | done |

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
* **Four crawl stages.** The API lists ~19 k items cheaply (name/quality/level) but prices and
  stats need one request per item, each answered record's own icon FILE needs one more, and — from
  v0.8.0 — its description copy lives behind a FOURTH door at all: the `forever/item` body carries
  no prose field whatsoever (measured key list), so `/api/v1/tooltip/forever/item/<id>` is where the
  text lives, and it costs the same order as the other stages (≈ 19 258 requests ≈ 2.7 h). Stage 1 is fast, stage 2 is staged, resumable and cancellable — see
  [ADR 0002](./docs/DECISIONS/0002-two-stage-sync.md) — and v0.6.0 adds a third, `sync --stage
  icons`, with the same budget, cancel and resume plumbing (
  [ADR 0008](./docs/DECISIONS/0008-icon-storage-third-stage.md)) and v0.8.0 adds the fourth,
  [ADR 0010](./docs/DECISIONS/0010-manual-layer-and-tooltip-copy.md) — `sync --stage tooltip`,
  same budget, same cancel, same resume. From v0.5.0
  stage 2 also takes **targets**: `--ids 25,40` and `--watch` fetch exactly what the user
  asked for, and the UI drives them in slices of 20 so Cancel stays meaningful between
  slices. Re-asking for an id that already has a record revalidates (`304`, nothing written).
* **The card answers offline — and answers for the rows the user MADE.** `item ID --cached` reads
  the stored `raw_json` and sends no request; the card prints when that record was fetched, draws
  the price **trend** from the `price_history` ledger, and shows the description copy as upstream's
  own markup printed as **escaped text** (rendering an upstream body would let upstream write the
  card). A row created by `add-item` lives in the reserved block (`900 000 000+`, measured max
  upstream id `288 340`) and answers its card from the DB's OWN columns on BOTH doors — upstream
  cannot know the id, so the live door short-circuits before building a client (ZERO requests).
* **The manual layer is one rule.** `manual_overrides` is one JSON array per row of `{field, value, at}`
  entries, and every write path — the list stage's upsert, the detail stage's, the tooltip stage's,
  and both card reads — applies it AFTER the payload binds, so a field the user stated survives the
  next crawl while the columns the crawl owns keep refreshing. DELETE is a mark (`deleted_at`), not
  an erase: the list stage UPSERTs every id upstream still lists, so a hard delete would be
  resurrected. `visible_clause()` is one clause pushed first, so search, its count and the card
  agree by construction — and no pending queue walks a tombstone or a manual row ([ADR 0010](./docs/DECISIONS/0010-manual-layer-and-tooltip-copy.md)).
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
