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
main.rs        entry point: dispatches to cli::run(argv) or gui::run()
cli.rs         headless subcommands: sync | stats | search | item | watch | status
gui.rs         Tauri Builder: context, state, invoke_handler, window creation
state.rs       AppState: DB path, sync-request flag, progress, cancel flag
wowdb.rs       HTTP client: URL building, ETag revalidation, rate-limit pacing
models.rs      JSON <-> Rust structs for list summaries, item details, search hits
db.rs          SQLite bootstrap: schema, db-path resolution, upserts, meta table
query.rs       search + facets + sort + count, all executed as SQL over the local DB
sync.rs        the sync engine: list crawl, detail crawl, resume, cancel, progress
stats.rs       price analytics (coverage, per-quality averages, extremes)
commands.rs    Tauri command surface: thin serialisation over the modules above
```

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
- [ ] `v0.2.0` — data layer: API client, JSON models, SQLite schema, CLI `sync`/`stats`
- [ ] `v0.3.0` — sync engine hardening: rate-limit pacing, ETag reuse, resume/cancel
- [ ] `v0.4.0` — Tauri shell + webview UI (search, filters, sorting, item cards)
- [ ] `v0.5.0` — detail-sync targets (watchlist/ids), price analytics, UI polish

## Build & run

```bash
cargo run                                  # GUI (needs a display server)
cargo run -- status                        # headless data pipeline, no display needed
cargo build                                # dev build
cargo build --release                      # release build, run ./target/release/wow-goldtool
```

CLI subcommands (identical code path as the GUI, so they double as the app's own
integration test):

```bash
cargo run -- status                       # where the DB lives, coverage, catalogue build
cargo run -- sync --stage list            # crawl the whole item list into SQLite
cargo run -- sync --stage detail --limit 500
cargo run -- search "Worn Shortsword"     # local DB query, prints JSON
cargo run -- item 25                      # live lookup of one item id
cargo run -- stats                        # DB coverage + price analytics
```

Environment switches used by tests and CI:

| Variable | Meaning |
| -------- | ------- |
| `WGT_DB` | absolute path of the SQLite file (default: `~/.local/share/wow-goldtool/wow-goldtool.sqlite`) |
| `WGT_FIXTURE` | base URL of a fixture API server (tests never hammer the upstream API) |
| `WGT_LIMIT_BUDGET` | override the request budget (CI uses a tiny one) |

## Testing

- Unit tests sit next to the code they test (`wowdb.rs` parses recorded API payloads).
- `tests/api.rs` runs the crawler against a **fixture server** so the suite is
  reproducible, fast and never rate-limited by the upstream host.
- `tests/pipeline.rs` seeds a small SQLite DB from fixtures, then asserts search, facets
  and stats behaviour.
- CI (`.github/workflows/ci.yml`) runs `cargo build`, `cargo test`, `cargo run -- stats`
  on a seeded fixture DB and `cargo fmt`/`cargo lint` gates.

## House rules

- Conventional commits (`feat(sync): …`, `fix(db): …`, `docs(devlog): …`) with a body
  that explains *why*, not only *what*.
- One logical change per commit, and every commit must build.
- Version bumps happen in `Cargo.toml` **and** `tauri.conf.json` together, get a
  `CHANGELOG.md` entry and a `vX.Y.Z` git tag.
- Rate limits are respected by construction, never by luck (see [docs/API.md](./API.md)).
- The visible attribution link to TheWoWDB stays in the UI.
