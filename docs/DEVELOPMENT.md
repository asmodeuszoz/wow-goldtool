# Development guide

This document is the living map of the project: what we are building, why it is built
the way it is, and how to run it. Everything else in `docs/` hangs off this file.

## Product

**WoW Goldtool** answers one question quickly: *what does this item cost on the auction
house, and which items are worth hunting for?* To answer it offline it keeps the WoW
Forever item catalogue in a local SQLite database and syncs it from the free
[TheWoWDB](https://thewowdb.com/developers) API.

## Stack

| Layer | Choice | Why |
| ----- | ------ | --- |
| App shell / UI | [Tauri 2.0](https://v2.tauri.app) (`tauri` crate, pinned `2.12.7`) | small binary, system webview, HTML/CSS gives a real design surface |
| Frontend | hand-written `ui/index.html` + `ui/app.css` + `ui/app.js` | no Node toolchain, no bundler; the files are embedded at compile time |
| Commands | `#[tauri::command]` fns in `src/commands.rs` | one IPC hop (`window.__TAURI__.core.invoke`) |
| Storage | SQLite via `rusqlite` (bundled SQLite) | one disposable file, fast faceted queries |
| HTTP | `reqwest` blocking client (rustls) | rate limiting must stay fully under our control |
| Testing | `cargo test` + headless CLI | no display server needed in CI |

## Module map (`src/`)

```
main.rs        entry point: a verb means cli::run(argv), no verb means gui::run_gui()
cli.rs         the verbs and their JSON: dispatch(Invocation) + the CLI guard (exit codes)
commands.rs    the app's one Tauri command: cli_call(argv) -> envelope JSON, zero logic
gui.rs         the Builder shell: on_page_load log, generate_handler!, generate_context!
wowdb.rs       HTTP client: URL building, ETag revalidation, rate-limit pacing
models.rs      JSON <-> Rust structs for list summaries, item details, search hits
db.rs          SQLite bootstrap: schema, db-path resolution, upserts, meta table
query.rs       search + facets + sort + count, all executed as SQL over the local DB
sync.rs        the sync engine: list crawl, detail crawl (pending queue or a target list), resume, cancel, progress
stats.rs       price analytics (coverage, per-quality averages, extremes)
errors.rs      one AppError + kind -> exit-code/JSON-kind mapping shared by both doors
```

Outside `src/`: `build.rs` (calls `tauri_build::build()` — the write-half of the compile-time
embedding), `tauri.conf.json`, `ui/` (`index.html`, `app.css`, `app.js` — the embedded
frontend), `icons/icon.png` (generated), `tools/gen_icon.py` (what generated it), `tests/`
(the fixture server). There is **no `state.rs`**: ADR 0002 makes the `sync_state` row the
progress truth and both doors read it, so mirroring it in `AppState` would be a second
source of truth; cancel is a JS flag read at chunk granularity (ADR 0005).

As of v0.7.0 every module above exists, and `cargo build` still ends in warnings only —
the unused ones are exactly: `catalogue`, `open_db`, and `pending_plan`'s private `PendingSql`
(plus the hardlink notice, which is the DB-path doc reading `st_mtime`/`st_nlink` at build
time). The set SHRANK at v0.7.0: the live-search door consumed `search_url`, `SearchResponse`
and `ApiResponse.remaining` — the three names v0.6.0 shipped unused because the door docs/API.md
promised was never opened (ADR 0009 opened it, on measured answers). v0.6.0 consumed the readers
it grew: `history_of` + `PriceSample` + `SQL_HISTORY_OF`
and the icon trio `icon_blob_of`/`icon_at_of`/`icon_missing_at_of` all have their consumer
in the item card. The `NotFound` error kind stopped being dead code the moment
`item ID --cached` needed it.

Rules of thumb: `wowdb.rs` never touches the DB, `db*.rs` never touches the network,
`sync.rs` is the only place where the two meet, `commands.rs` contains no logic.

## Layout decision

The repo keeps **one Cargo project at the repo root** (`Cargo.toml`, `src/`, `ui/`,
`tauri.conf.json` next to each other) instead of the `src-tauri/` layout that
`create-tauri-app` generates — see [ADR 0001](./DECISIONS/0001-flat-cargo-layout-no-cli.md)
in `docs/DECISIONS/`. Consequences:

* `cargo build` in RustRover builds a runnable app: Tauri's `generate_context!` macro
  reads `tauri.conf.json` **at compile time** and embeds the config plus every file in
  `build.frontendDist` (`ui/`). No Tauri CLI, no `npm`, no `_assets.gen.rs` step.
* The app binary is a plain Cargo target, so `cargo run`, `cargo run --release` and the
  RustRover run/debug configurations all work unchanged.
* We do not use `tauri dev` (hot reload) — the frontend is three static files, and
  re-running the binary is fast enough. Packaging into `.deb`/`.AppImage` is documented
  as future work in the roadmap.

## Roadmap / milestones

Each milestone is one squashed logical change, one conventional commit series and one
git tag. The dated narrative lives in [docs/DEVLOG.md](./DEVLOG.md).

- [x] `v0.1.0` — repository scaffold, docs, licence, CI templates
- [x] `v0.2.0` — data layer **plus a working sync engine**: API client, JSON models, SQLite schema, headless CLI verbs, paced list/detail crawl, exit-code contract
- [x] `v0.3.0` — sync hardening: conditional GETs with stored ETags (per page in `meta`, per item in the row), stop→resume on the `sync_state` page cursor, cancel plumbing for the GUI's AppState
- [x] `v0.4.0` — Tauri shell + webview UI: one `cli_call(argv)` command, status/search/sync cards, chunked sync with cancel, `build.rs` + `tauri-build`, generated icon
- [x] `v0.5.0` — detail-sync targets (`--ids`, `--watch` driven from the UI), item card that reads the DB first, price-analytics card, watch toggles in hit rows
- [x] `v0.5.1` (patch) — the crawl stops freezing the page: `cli_call` is an async command, and a status ticker keeps the note and the card alive while a chunk is in flight
- [x] `v0.5.2` (patch) — pending = never fetched (fetched-but-unpriced stops being pending work), the GUI verb hop to a plain thread (a blocking-client build inside a tokio worker stalled the crawl at `build_client`), "Pull all info" chains both stages to their end
- [x] `v0.5.3` (patch) — a record upstream dropped is a checked third state (`detail_missing_at`, `HttpKind::Missing`, `missing` in the sync report and in coverage), pending = never fetched **and** never checked-missing, schema `1 → 2` migrated in place with `ALTER TABLE` so an existing catalogue keeps its rows
- [x] `v0.6.0` — price history (a second table, `price_history`: one sample per **fresh** detail write, never a `304`, ADR 0007), local icon copies via a THIRD sync stage (`icon_blob` + `icon_at` + `icon_missing_at`, ADR 0008), the item card renders the `stats` pairs (the one unshaped field grounded live), the local icon as a `data:` URI and the price trend; schema `3`, migrating v1→v2→v3 in place
- [x] `v0.7.0` — the live-search door (ADR 0009): CLI verb `live-search TEXT` — the whole verb tail as ONE query, ONE request to the grouped door, upstream's own answer printed as it came (`{status, retry_after, remaining, answer}`), names-only means all-empty groups still exit `0`; a UI panel asks ONCE per click with `view upstream` links; the door writes and reads NO DB (schema stays `3`), and it consumes `search_url`/`SearchResponse`/`ApiResponse.remaining` so the dead set shrinks to `catalogue`/`open_db`/`PendingSql`

## Build & run

```bash
cargo run                                  # GUI (needs a display server)
cargo run -- status                        # headless data pipeline, no display needed
cargo build                                # dev build
cargo build --release                      # release build, run ./target/release/wow-goldtool
```

The GUI is a foreground process: it blocks until the window is closed and exits 0. While it
runs it writes `gui:` lines to stderr (`gui: ipc verb status (0 words)`), because the webview
console is not forwarded on Linux — those lines are how the launch audit is read
(see [docs/DEVLOG.md](./DEVLOG.md)). While a crawl chunk is in flight the UI's ticker
polls `status` once a second, so in the audit the crawl's `gui: ipc verb sync` line
*alternates* with one-per-second status polls — that interleaving is the witness that
the webview runs during the chunk (fixed in v0.5.1: a bare-`fn` command runs inside
the IPC handler and parks the page behind the crawl, and fixed in v0.5.2 too: a blocking-
client build on the manager's tokio worker stalled the crawl at `build_client` before it
issued a single request). The verb body therefore runs on a fresh `std` thread (`verb_on_plain_thread`
in src/main.rs): the page keeps ticking while the crawl owns a runtime-free thread of its
own. "Pull all info" chains all three stages to their end in one flow (v0.6.0), and the launch audit
proves it against the fixture — the stage verbs interleave with the ticker's `status`
polls, all three stages' forced `429` retry in place, and the scratch DB ends at 0 pending
in every queue with all three `sync_state` rows `complete` (v0.6.0 chained
list → detail → icons; the icons stage copies the fixture's real JPEG bytes and checks the
one gone icon file). The GUI uses the same DB resolution as the CLI, so
`WGT_DB=/tmp/opencode/fixture.sqlite cargo run` audits the UI against the fixture DB instead
of the real one.

CLI subcommands (identical code path as the GUI, so they double as the app's own
integration test):

```bash
cargo run -- status                       # where the DB lives, coverage, catalogue build
cargo run -- sync --stage list            # resumes where a stopped run paused; finished
                                          # runs re-crawl page 1, every page revalidates
cargo run -- sync --stage detail --budget 500
cargo run -- sync --stage detail --ids 25,40,60   # exactly these records (also --watch)
cargo run -- search "Worn Shortsword"     # local DB query, prints JSON
cargo run -- item 25                      # live lookup; 304 + stored raw_json when
                                          # the record is already in the DB
cargo run -- item 25 --cached             # the same card with no request at all
cargo run -- sync --stage icons --ids 25,40   # exactly these icons (also --watch)
cargo run -- watch add 25 40              # the list the UI turns into sync targets
cargo run -- stats                        # DB coverage + the ledger count
```

Environment switches used by tests and CI:

| Variable | Meaning |
| -------- | ------- |
| `WGT_DB` | absolute path of the SQLite file (default: `~/.local/share/wow-goldtool/wow-goldtool.sqlite`) |
| `WGT_FIXTURE` | base URL of a fixture API server (tests never hammer the upstream API) |
| `WGT_LIMIT_BUDGET` | override the request budget (CI uses a tiny one) |

## Testing

- The real integration path is the headless CLI run against a **fixture server**
  (`tests/fixture_api.py`, committed at v0.2.0): two list pages, a detail record for any
  item id, one listed id (`127555`) that has **no** detail record — it answers `404` with
  the upstream's own error body, the shape v0.5.3's third state exists for — and one forced
  `429` + `Retry-After` per stage — except the `search` stage, which the gate never names
  from v0.7.0 on: the live-search door answers a `429` with exit `3` rather than retrying
  in place (ADR 0009), so a gated search stage would turn the committed CI step red. The
  crawler never sees the upstream API in CI. From v0.7.0
  it also answers the grouped `/search/?q=` shape the live probes grounded (six groups each
  `{has_more, more, results}`, names-only matching, `+` inside its own `more` URLs). From v0.6.0
  it also answers the ICON FILE shape: `/icon/<name>.jpg` answers `200 image/jpeg` with REAL
  bytes (the grounded `inv_mace_03.jpg` 1945 b and `inv_axe_11.jpg` 1936 b, fetched once from
  the asset host and embedded as base64 so the suite never asks a host for pixels), record
  `60` answers an `icon_url` whose FILE is `404` (the icon stage's third state, one check),
  and detail prices carry a per-process `RUN` offset — one fixture process is one observation
  window, so a second process' fresh write grows the `price_history` ledger by exactly one
  sample while a re-ask inside one process still writes none (ADR 0007's rule, measured).
  From v0.7.0 the sequence is fifteen steps with the icon stage and the live-search door
  inside, and it also covers a watch
  list, a targeted detail fetch, the same ids fetched again (which must answer `304` and write
  nothing) and the DB-only card read. Since v0.5.3 the local mirror adds the gone-id steps on
  the same DB: `item 127555` must answer **exit 4** (`not-found`) and re-stamp the mark, and
  the crawl that follows must answer `requests 0, complete true` — the removed record is not
  asked again. The committed CI sequence passes every step with the `404`s in it.
- `tests/api.rs` / `tests/pipeline.rs` (unit tests next to the code they test) are the
  plan, not code: v0.2.0 has no `#[test]` functions yet, so `cargo test` only proves
  everything compiles.
- CI (`.github/workflows/ci.yml`) runs `cargo build`, `cargo test`, `cargo run -- stats`
  on a seeded fixture DB and `cargo fmt`/`cargo lint` gates. The GUI is **compiled** by CI
  (one build unit) but not exercised: CI has no display server. The webview's only access to
  the app is the CLI's argv, so the CLI smoke sequence *is* the app's integration test
  (ADR 0004, ADR 0005); the GUI launch audit runs on a real desktop and is recorded in
  [docs/DEVLOG.md](./DEVLOG.md).

## House rules

- Conventional commits (`feat(sync): …`, `fix(db): …`, `docs(devlog): …`) with a body
  that explains *why*, not only *what*.
- One logical change per commit, and every commit must build.
- Version bumps happen in `Cargo.toml` — and only there. `tauri.conf.json` deliberately omits
  `version`, because `tauri-codegen` falls back to `env!("CARGO_PKG_VERSION")`
  (`context.rs:255`): two version fields would be one too many. Every bump still gets a
  `CHANGELOG.md` entry and a `vX.Y.Z` git tag.
- Rate limits are respected by construction, never by luck (see [docs/API.md](./API.md)).
- The visible attribution link to TheWoWDB stays in the UI. In v0.4.0 it is static HTML in
  `ui/index.html`, and no JS path removes it.
