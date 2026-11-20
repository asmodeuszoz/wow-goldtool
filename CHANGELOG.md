# Changelog

All notable changes to **WoW Goldtool** are documented here, following
[SemVer](https://semver.org/) and [Conventional Commits](https://www.conventionalcommits.org/).
`Added` / `Changed` / `Fixed` / `Removed` follow the [Keep a Changelog](https://keepachangelog.com/)
sections. The full narrative of *why* each change was made lives in
[docs/DEVLOG.md](./docs/DEVLOG.md); each entry below points at the matching log entry and git tag.

## v0.7.0 — the live-search door: one human ask, one request, a look not a sync · 2026-11-20 · tag `v0.7.0`

Added

- CLI verb `live-search TEXT` ([ADR 0009](./DECISIONS/0009-live-search-door.md)): the WHOLE
  verb tail joined with spaces is the query (`live-search Worn Shortsword` asks one name),
  ONE request to the grouped `/search/` door, payload `{status, retry_after, remaining, answer}`
  with `answer` a serde pas-through of `SearchResponse` (`groups` stays a `Value`, upstream's
  `+` inside its own `more` URLs rides verbatim). Names-only means an all-empty answer is a
  success (exit `0`), and the door writes NOTHING: no row, no `meta` key, no sample, no mark.
- A live-search panel in the UI: one field, one `Ask once` button, one request per click, a
  `group · id · name` table with a `view upstream` link per hit (attribution reaches the
  ANSWER, not only the footer), and a note printing `remaining` (honestly `null` when the
  host sends no burst header), the names-only fact, and whether any group said `has_more`.
- The fixture grew the grouped `/search/` answer from the measured transcript (six groups each
  `{has_more, more, results}`, names-only substring matching, the `+` form inside its own
  `more` URLs), and its `429` gate now never names the `search` stage — the door answers a
  `429` with exit `3` (like the `item` door) rather than retrying in place, so a gated search
  stage would turn the committed CI step red.
- CI grew the smoke sequence to fifteen steps: three live-search asks — one word, the two-word
  ask riding as `%20` on the wire (the fixture's GET log is the witness), and the names-only
  trap whose all-empty answer still passes at exit `0`.

Changed

- The build's dead-code set shrank as promised: `search_url`, `SearchResponse` and
  `ApiResponse.remaining` — compiled-but-unused through v0.6.0 — got their consumer, and `cargo
  build` now warns exactly `catalogue`, `open_db` and `pending_plan`'s private `PendingSql`
  (plus the incremental-cache hardlink notice on this filesystem).
- The version banner rides `0.7.0` (`Cargo.toml` and the `User-Agent` line naming the source),
  and the usage line says both search doors now: `search TEXT […]` (local, filtered) beside
  `live-search TEXT` (upstream's own answer).
- Nothing schema-wise: schema stays `3`. The door touches no DB — audited with an impossible
  `--db` path answering exit `0` with the answer while its directory is never created.

## v0.6.0 — price history + local icons: the offline catalogue starts paying · 2026-11-20 · tag `v0.6.0`

Added

- A second table, `price_history`, and one price sample per **fresh** detail write
  ([ADR 0007](./DECISIONS/0007-price-sampling.md)): `upsert_detail` appends a row
  after a fresh (200) write — and only when the payload carried prices, so a record
  not on the AH never becomes a blank chart point — while a `304` revalidation, a
  list write, a check-mark or a re-insert write **nothing**. The ledger is what the
  card draws (one bar per stored sample, newest-N window) and what `stats`, `status`
  and the sync panel print, so the sampling rule is auditable, not decorative.
  SQLite's FK clause turned out not to be the guard: measured 2026-11-20, an `INSERT`
  referencing a missing `item` id succeeds, so the order inside `upsert_detail` is
  the guard and `seq` is auto-assigned so an append can never collide with an older
  observation.
- The icon stage `sync --stage icons` ([ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md)):
  every answered record's OWN icon file is fetched once and copied into the DB as a
  `BLOB` — three states like the detail record's: copied + `icon_at`, checked-as-gone
  + `icon_missing_at` (one `404` on the icon FILE of an existing record, never asked
  again), and waiting. The pending rule requires a NON-EMPTY `icon_url`: live, 4 of
  30 detailed records answer `"icon_url": ""` (Peace Tea, Craftsman's Writ, Scroll
  of Imbue Iceknife, Lodestone), and under a NULL-only rule those would keep pending
  forever. The item card shows the local icon as a `data:` URI — nothing is hotlinked,
  which is the offline point — and CI's fixture now answers `/icon/<name>.jpg` with
  REAL `image/jpeg` bytes (1945 b and 1936 b, fetched once from the grounded asset
  host and embedded so the suite stays offline) plus one record whose icon FILE is
  gone upstream, the third state the stage must check once.
- Schema 3: three `ALTER`s append the icon columns, a `CREATE` adds `price_history`
  and its index, and the chain migrates **both** stale shapes in place — a file
  saying `schema_version = 1` (the user's own 19 258-row catalogue) gets v0.5.3's
  `detail_missing_at` first, then the v0.6.0 batch; a v2 file gets just the batch.
  A file **newer** than this build is still refused.

Changed

- `SCHEMA_VERSION` `2 → 3`; both fresh-DB stamps write the build's own version, so
  a fresh v3 file never refuses itself.
- `sync_state` gained the third stage (`kind = "icons"`); `status` prints three
  crawl rows, the icon pair (`copied`, `checked`) and the ledger count, and the
  sync panel's grid + "Pull all info" follow the chain list → detail → icons.
- The item card renders `stats` as name/value pairs (the one previously-unshaped
  field the live API has ever filled — measured) and gives the never-measured
  fields (`sockets`, `socket_bonus`, `classes`, `effects`) an honest raw row only
  when they carry something; the icon row and the trend bars join.

Fixed

- The icon queue's empty-url exclusion above (a NULL-only rule would never drain
  on the measured empty answers); and `ORDER BY` stays plain `id` here, because
  unlike the detail queue every icon candidate already carries a detail stamp, so
  a re-ask-first term would be a no-op — the docs now say what the code does.

[ADR 0007](./DECISIONS/0007-price-sampling.md) · [ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md) · [docs/DEVLOG.md](./DEVLOG.md)

## v0.5.3 — patch: a record the upstream dropped is a state · 2026-11-20 · tag `v0.5.3`

Fixed

- A detail record the upstream no longer has (`404`) aborted the whole crawl
  (`item 41 answered 404` → exit `3` on the live catalogue). The list walk lists
  those ids — `OLD…`, `Deprecated …`, `… (TEST)` names sit in the user's 19 k rows
  — so the one-click "Pull all info" from v0.5.2 could never finish. `classify` now
  answers `HttpKind::Missing`, the detail crawl marks the id
  (`detail_missing_at`, guarded by a row-exists check so a mark cannot invent a row)
  and counts it in the report (`missing`), then keeps walking; the stage-2 upsert
  clears that stamp, so "gone" and "fetched" cannot both be true.
- Pending is now "never fetched **and** never checked-missing" (`SQL_PENDING`), so a
  removed record costs one request per crawl instead of one per run. The arithmetic
  holds on the migrated scratch DB: `3000 = 10 fetched + 3 checked-gone + 2987
  pending`, and the next run did not re-ask the three marked ids.
- `SCHEMA_VERSION` moved `1 → 2` **with** a migration: a file saying `schema_version
  = 1` gets `ALTER TABLE item ADD COLUMN detail_missing_at INTEGER` in place and its
  `meta` stamp moves in the same call, because refusing it would throw away a 19 258-row
  catalogue. A file **newer** than this build is still refused. Both fresh-DB stamps now
  write the build's own version, so a fresh v2 file never refuses itself.

Added

- Coverage counts the third state (`missing` in `status`/`stats`), and the status card
  prints it (`checked as gone upstream`). The pull-all notes say what a completion
  means with removed records in it: fetched, gone, and what remains live.

Changed

- `item <id>` on a removed record answers the exit-code contract's `4` (`not-found`)
  and re-stamps the mark: a human-aimed ask may re-ask once per request, a crawl may
  not. A **list page** answering `404` stays a failure (exit `3`).
- The fixture now lists one id (`127555`) that has **no** detail record and answers
  upstream's own error body, so CI exercises the third state: `item 127555` → exit
  `4`, and the crawl after it asks nothing. The exit-code audit gained that probe:
  `2 · 2 · 3 · 4 · 4 · 4 · 0`.

See [ADR 0006](./docs/DECISIONS/0006-checked-missing-third-state.md) and
[docs/DEVLOG.md](./docs/DEVLOG.md).

## v0.5.2 — patch: one click finishes the catalogue · 2026-11-20 · tag `v0.5.2`

Fixed

- Pending is now "never fetched", not "no price" (`SQL_PENDING` in `src/stats.rs`).
  The old `OR price_min IS NULL` clause kept a fetched item that is simply **not
  on the auction house** pending forever: the detail stage walked it on every run,
  so the stage never terminated on the real catalogue and no "pull everything"
  flow could honestly promise nothing left to pull. `docs/DATABASE.md` keeps those
  two states apart on purpose; the pending rule now uses the same distinction.
- A sync verb launched from the GUI never reached the API at all: the marker
  audit stopped after the DB-open point with an empty fixture access log, while
  status polls kept completing around it. reqwest's blocking client builds a
  **private tokio runtime on an internal thread** and parks the calling thread
  waiting for that runtime's startup handshake; the verb body sat on a worker of
  tauri's tokio-backed async manager, so the handshake never landed and the verb
  stalled at `build_client` with zero requests issued. `cli_call` keeps the async
  wrapper from v0.5.1 (the page still answers immediately) and the verb body now
  hops to a fresh `std` thread — the runtime-free home the headless CLI has always
  used in the CI mirrors.

Added

- "Pull all info" replaces the "Sync pending details" button: one flow runs the
  list walk to its end, then the pending-detail queue to its end, so a single
  click retires all pending work. `runSync` reports an outcome
  (`complete`/`cancelled`/`stalled`/`failed`/`already-running`) so a cancelled
  chain stops at the stage in flight instead of jumping ahead — the next click
  resumes the list from `sync_state` and the pending queue recomputes what details
  remain. A stage that is complete with zero requests is reported as complete,
  not as "made no progress". The completion note is explicit about what "nothing
  left to pull" honestly means: every listed item has a detail record, a NULL
  price on a fetched item means not on the AH, and prices stay live, so later
  syncs only revalidate (every page answers `304` from the stored ETags).

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
