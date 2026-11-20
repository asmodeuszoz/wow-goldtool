# Changelog

All notable changes to **WoW Goldtool** are documented here, following
[SemVer](https://semver.org/) and [Conventional Commits](https://www.conventionalcommits.org/).
`Added` / `Changed` / `Fixed` / `Removed` follow the [Keep a Changelog](https://keepachangelog.com/)
sections. The full narrative of *why* each change was made lives in
[docs/DEVLOG.md](./docs/DEVLOG.md); each entry below points at the matching log entry and git tag.

## v0.5.1 — patch: the crawl no longer freezes the webview · 2026-11-19 · tag `v0.5.1`

Fixed

- A running sync chunk no longer parks the whole app. A bare-`fn` command
  gets tauri-macros' blocking wrapper — its last acts are
  `kind.block(result, resolver)` then `return true` (`wrapper.rs:425`/`:428`)
  — so one chunk (2/s pacing, plus a `Retry-After` sleep on a 429: minutes in
  the worst case) ran synchronously **inside** the webview's IPC handler,
  which is the page's message loop: hover stopped highlighting, clicks queued
  behind the crawl for minutes. `cli_call` is now an `async` command
  (`respond_async_serialized`, tauri's tokio-backed async manager — tokio is
  a required dependency of tauri 2.x, so no feature switch was needed): same
  argv, same dispatch, same envelope, only the wrapper kind flips.
- Mid-chunk the page now says so: a ticker re-reads the DB once a second
  (coverage grows per page in list, per item in detail) and prints the
  chunk's elapsed time in the sync note; the cursor line honestly stays at
  the last chunk end, because progress truth lives in `sync_state` (ADR 0002).

Added

- The launch audit gained a witness: during a crawl, `gui: ipc verb sync`
  alternates with one-per-second `gui: ipc verb status` polls — that
  interleaving is what proves the webview runs mid-chunk
  (see [docs/DEVLOG.md](./DEVLOG.md)).

## v0.5.0 — detail-sync targets, price analytics, item card · 2026-11-19 · tag `v0.5.0`

Added

- Targeted detail sync: `sync --stage detail --ids 25,40,60` and `sync --stage detail --watch`
  visit exactly those records instead of the pending queue, which is what turns the watch list
  from something that prints into something that acts. The UI drives it with two buttons
  ("Details: watch list", "Details: shown hits") that slice the ids 20 per verb call, so
  Cancel between slices stays meaningful and one slice stays far below the 120-request burst.
- `item ID --cached`: a DB-only card read — no request at all. A record that was never
  fetched is `not-found` (the first use of that error kind, which retires its dead-code
  warning) and the UI turns the word into an offer instead of an empty card.
- UI cards: an item card that answers offline first (`Read the DB` vs `Re-check online`,
  printing when the record was fetched), a price-analytics card that renders
  `stats --limit 6` as per-quality bars in the search-chip palette plus cheapest/priciest
  extremes, and watch toggles inside the hit rows (the id travels in `data-id`, one
  delegated handler serves the whole table) with a `Watch these hits` batch action.
- Detail fetches send the stored `detail_etag` when one exists, so re-asking for the same
  ids answers `304` and rewrites nothing — the audit reads `written 0, unchanged 2`.
- The CI smoke block grew by six steps (watch add → targeted fetch → the same ids again
  answering 304s → watch-list target → DB-only card read), all eleven green in the local
  mirror before the commit.

Fixed

- Three single-column lookups in `db.rs` inferred a bare `String` getter for a nullable
  column, so a NULL came back as rusqlite's `Invalid column type Null` **DB error** (exit 4)
  instead of `None`. The pending stage never read those columns before fetching; the
  targeted stage reads `detail_etag_of` for every id, so the latent wrongness surfaced on the
  first audit run. The getters now ask for the `Option` type they always claimed to return.
- Targeted runs now answer their own question: `remaining` counts the target list ("left in
  the list", not "left pending") and `complete` means the target set was fully visited.
- `boot()` reads status, the watch list and the analytics at startup, so no card paints
  empty; the second status read is the launch audit's witness — four `gui: ipc verb …`
  lines prove the analytics render ran to the end rather than throwing after its await.

## v0.4.0 — Tauri shell + webview UI · 2026-11-19 · tag `v0.4.0`

Added

- The GUI shell lands: `tauri.conf.json` (window declared in `app.windows`, `withGlobalTauri`,
  `frontendDist: "ui"`, `bundle.icon`, no `version` key — see below), `src/gui.rs` (the
  `Builder` + `generate_context!` shell whose `run_gui()` blocks until the window closes) and
  `src/commands.rs` (the app's **one** command, `cli_call(argv)`, returning the envelope
  `{"ok":true,"result":…}` / `{"ok":false,"error":{kind,exit,message,retry_after}}`).
- `ui/index.html`, `ui/app.css`, `ui/app.js`: a status card (DB path, coverage, crawl state),
  a search card (text, quality, sort order, `priced only`) rendering the `Hit` fields with
  quality chips and a `view` link per record, and a sync card whose Cancel button works at
  chunk granularity. The attribution footer is static HTML — no code path can hide the link
  TheWoWDB's developer terms require.
- `src/cli.rs` grew `dispatch(Invocation) -> AppResult<String>`: the CLI door (`run`) and the
  GUI door (`cli_call`) send verbs through the same function, so ADR 0004's single code path
  reaches into the UI (see [ADR 0005](./docs/DECISIONS/0005-gui-ipc-commands.md)).
- Chunked sync from the UI: `sync --stage … --budget 10` (list) / `--budget 25` (detail) in a
  row until `report.complete`; Cancel raises a JS flag read at the next chunk boundary, and
  every chunk boundary is a `sync_state` write, so stopping is always resume-safe (ADR 0002).
- `build.rs` calling `tauri_build::build()` plus `tauri-build = "2.7.6"` as a **build**-dependency:
  this is what writes the global-API script list into `OUT_DIR`, i.e. what makes
  `window.__TAURI__.core.invoke` exist. It is cargo's own build script, not the Tauri CLI —
  [ADR 0001](./docs/DECISIONS/0001-flat-cargo-layout-no-cli.md) carries an amendment.
- `icons/icon.png` is generated by `tools/gen_icon.py` (codegen reads `bundle.icon` at compile
  time, so the file must exist before the first GUI build); `.gitignore` gains `/gen/` for the
  JSON schemas `tauri-build` writes next to `Cargo.toml`.
- `gui:` log lines on stderr (`gui: ipc verb status (0 words)` …) are the app's audit channel —
  nothing in `tauri`/`wry` forwards the webview console on Linux.
- `Cargo.toml` version `0.4.0`. `tauri.conf.json` deliberately omits `version`:
  `tauri-codegen` falls back to `env!("CARGO_PKG_VERSION")`, so `Cargo.toml` is the single
  version source and the "bump both files" house rule was amended, not broken twice.

Fixed

- The `cli.rs` refactor re-audited green rather than assumed so: exit-code audit `2 · 2 · 3 · 4 · 4 · 0`,
  the CI smoke mirror all `exit=0`, and the v0.3.0 contract (`resumed_from: 2`, `unchanged: 2`,
  `item 25` answered `304 reused true` from stored `raw_json`) unchanged.

## v0.3.0 — sync hardening: reuse, resume, cancel · 2026-11-19 · tag `v0.3.0`

Added

- Conditional GETs everywhere the crawler revisits: each list page keeps its ETag plus
  `has_more`/`count` continuation hints in `meta` (a `304` has no body — the crawl walks
  on using the hints), and the `sync_state` page column is an absolute cursor.
- Stop→resume: a list run that stopped by budget or cancel resumes at the page after the
  one `sync_state` says it last fetched; a finished run re-crawls from page 1, where the
  stored ETags make that a walk of `304`s. Honest reports: `resumed_from`, `unchanged`.
- Live `item` lookups revalidate instead of re-download: stored `detail_etag` goes out as
  `If-None-Match`, a `304` is answered from `raw_json` on disk, a fresh `200` is written
  through the same upsert the sync stage uses. Output gained `"reused"`.
- Cancel plumbing: both crawl loops check a `CancelFlag` before every request; the CLI
  hands out a fresh one (Ctrl-C stays resume-safe), the GUI raises it via AppState at
  v0.4.0. A cancelled run records status `cancelled` and exits 0.
- The fixture server now speaks conditional GETs (stable per-endpoint ETags, real `304`
  no-body), and the CI smoke sequence exercises the whole v0.3.0 contract: budget stop
  → resume → full re-sync (all `304`) → detail crawl → `item` answered from the DB.

## v0.2.0 — data layer + working sync · 2026-11-19 · tag `v0.2.0`

Added

- `src/` headless core, one buildable unit: `errors.rs`, `models.rs`, `wowdb.rs`,
  `db.rs`, `query.rs`, `stats.rs`, `sync.rs`, `cli.rs` plus `main.rs` argv dispatch —
  the six CLI verbs of [ADR 0004](./docs/DECISIONS/0004-headless-cli-mode.md).
- SQLite storage ([docs/DATABASE.md](./docs/DATABASE.md)): wide `item` row plus
  `meta`/`watch`/`sync_state`, list + detail upserts, `--db PATH` overriding `WGT_DB`.
- HTTP client with pacing by construction: 90-request headroom under the 120-request
  burst, 2 req/s schedule, `429` honours `Retry-After` and never aborts.
- Both sync stages verified end-to-end against a fixture server, incl. retry-in-place
  for a `429`'d item and an honest `complete`/`partial` report derived from the pending
  count.
- CI smoke test is now live (was commented in the scaffold tag): fixture server
  `tests/fixture_api.py` + `sync --stage list` + `sync --stage detail` + `stats`.
- `Cargo.toml` version 0.2.0 with the pinned dependency line unchanged. `tauri.conf.json`
  joins the tree at v0.4.0 (GUI shell), so its version bump lands there — see the DEVLOG
  note on the amended house rule.

## v0.1.0 — scaffold · 2026-11-19 · tag `v0.1.0`

Added

- Repository layout: `src/` (Rust), `ui/` (frontend assets), `docs/`, `.github/`.
- Project documentation: [DEVELOPMENT](./docs/DEVELOPMENT.md) (architecture, roadmap, house
  rules), [API](./docs/API.md) (TheWoWDB notes: paging, ETag revalidation, rate limits),
  [DATABASE](./docs/DATABASE.md) (SQLite schema + query recipes), [DECISIONS](./docs/DECISIONS/)
  (ADR-style records).
- `README.md` with the roadmap and the required data attribution, `CHANGELOG.md`,
  `LICENSE` (MIT + data note), `.gitignore` for Rust/Tauri/IDE artefacts.
- GitHub-style repository metadata: issue + PR templates, CI workflow (`build`, `test`,
  `stats` on a seeded fixture DB).
- Default `hello world` binary kept untouched (base crate compiles as-is).
