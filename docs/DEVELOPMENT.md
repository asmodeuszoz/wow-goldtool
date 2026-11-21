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
models.rs      JSON <-> Rust structs for list summaries, item details, search hits, tooltip bodies
db.rs          SQLite bootstrap: schema, db-path resolution, upserts, meta table
query.rs       search + facets + sort + count, all executed as SQL over the local DB
sync.rs        the sync engine: FOUR stages (list crawl, detail crawl, icon copy, tooltip copy) - pending
               queue or a target list, resume, cancel, progress
stats.rs       the four pending queues + price analytics (coverage, per-quality averages, extremes)
errors.rs      one AppError + kind -> exit-code/JSON-kind mapping shared by both doors
```

Outside `src/`: `build.rs` (calls `tauri_build::build()` — the write-half of the compile-time
embedding), `tauri.conf.json`, `ui/` (`index.html`, `app.css`, `app.js` — the embedded
frontend), `icons/icon.png` (generated), `tools/gen_icon.py` (what generated it), `tests/`
(the fixture server). There is **no `state.rs`**: ADR 0002 makes the `sync_state` row the
progress truth and both doors read it, so mirroring it in `AppState` would be a second
source of truth; cancel is a JS flag read at chunk granularity (ADR 0005).

As of v0.8.0 every module above exists, and `cargo build` still ends in warnings only — exactly
FOUR of them: `catalogue`, `open_db`, `pending_plan`'s private `PendingSql` and rustc's
"multiple fields are never read" on **`Tooltip`** (`attribution`/`build`/`fields`/`icon_url`/
`id`/`kind`/`links`/`name`/`quality`/`version`: the measured key list of the tooltip door rides
the model so a key upstream adds cannot break a build, and the door's body only needs `html`;
ADR 0009's rule again — keep the measured mirror, read what the door needs). The set SHRANK at
v0.7.0 (the live-search door consumed `search_url`, `SearchResponse` and `ApiResponse.remaining`,
the three names v0.6.0 shipped unused because the door docs/API.md promised was never opened, on
measured answers), and v0.8.0 grew it by exactly ONE: the `Tooltip` mirror above. Every other new
name landed a consumer in the same release — the tooltip stage, its queue, its card row and the
four manual verbs all have one, and the audits read the numbers they print. v0.6.0 consumed the readers
it grew: `history_of` + `PriceSample` + `SQL_HISTORY_OF`
and the icon trio `icon_blob_of`/`icon_at_of`/`icon_missing_at_of` all have their consumer
in the item card. The `NotFound` error kind stopped being dead code the moment
`item ID --cached` needed it. v0.9.4 grew money's three helpers (`money_text`, `money_of`,
`money_of_avg` in `db.rs`) and the build measured **FIVE** dead-code names for one window of
that chain — `money_of_avg` had no consumer until feat(stats) landed its two (`Coverage.avg_price_money`
and `QualityStat.avg_price_money`) — and came back to the FOUR named above at the tag; the feat(db)
body says so, and the record keeps that window honest rather than editing a commit to hide it.

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
- [x] `v0.8.0` — the manual layer and the description copy (ADR 0010): a FOURTH sync stage `--stage tooltip` copies the tooltip door's own `html` (the `forever/item` body carries NO description — measured key list, docs/API.md), schema `4` with a chained v1→v2→v3→v4 migration; the manual layer is ONE rule (overrides merge LAST on every write path): CLI verbs `set-item` / `add-item` / `delete-item` / `undelete-item`, DELETE is a tombstone every query hides (`visible_clause`, pushed first), user-made rows live in the reserved block `900 000 000+` (`MANUAL_ID_BASE`, measured max upstream id `288 340`) and answer their card from the DB's own columns with ZERO requests, and all pending queues refuse tombstones and manual rows
- [x] `v0.9.0` — the readable twin (ADR 0012): schema `5` adds `tooltip_text`/`tooltip_lc` derived IN THE SAME WRITE as the copy on EVERY write path (the tooltip stage derives from the MERGED copy — ADR 0010's overrides merge LAST, so the twin always matches what the DB SAYS; `set-item ID tooltip_html=…` derives beside the stated value; `add-item` births it — the name→`name_lc` pattern v0.1.x started), and the derivation is exactly the measured three moves (two live tooltip bodies read whole: div/span/img tags, ZERO entities — no decoder is invented, entities would show verbatim): drop `<…>` spans, collapse whitespace, trim. The LOCAL search door grew the scope — `search TEXT --where name|description|all` (default `name` keeps every existing search exactly what it was; `description` reads `tooltip_lc` behind `IS NOT NULL`; a bogus scope answers the usage door `2`; `visible_clause` stays FIRST in every scope), and the card carries BOTH tooltip shapes while the UI renders only the twin — the copy's `<img>`s were measured hotlinking pixels from `render.worldofwarcraft.com` and `thewowdb.com/assets`, so upstream markup never renders (the icon stage's rule), CI smoke 21→24 steps, and `check_schema` chains v1→v2→v3→v4→v5 (its `--where` and its span-DROP both went wrong under live use — measured at v0.9.1, fixed forward there)
- [x] `v0.9.5` — the price homes (ADR 0016): the user's TEST said the truth of a price is FOUR numbers, not one — what a vendor SELLS (the column exists), what a vendor BUYS (only when the item can be bought at all — "cannot be bought" is a STATE, never a 0), the CURRENT auction-house min buyout (today's `price_min` answers the STORED copy's question: an aging payload's number is not a live price), and the historic "normal" the live trend answers from past data ("normal" ≠ "min": two questions, two numbers). The user scoped it: "how we'll get to those values can be ignored for now, but the columns should be allocated now" — so the schema hop **5 → 6** carves `vendor_buy` / `price_min_now` / `price_trend_normal` (names OURS: no upstream door ever answered these words — inventing a producer now would invent upstream shapes, rejected) and NOTHING writes or reads them yet: every door answers BYTE-IDENTICAL across the hop (13 measured pairs, both binaries over one fixture file: 12 byte-identical — search 1 941/763/755/3 363, stats 1 533/1 850/17 462, cards 4 336/1 103/3 868/4 022, refresh-twins 223 — and `status` identical once its own `db.path` echo is pinned, the paths cannot match because the OLD binary refuses the migrated file). `check_schema` chains EVERY stale shape v1..v5 up to 6 (the staircase v0.9.0's comment promised since its own hop); measured on the USER's own file: 19 258 rows / 4 231 vendor_sell / 4 285 samples intact, trio NULL on ALL, idempotent second open re-migrates NOTHING (second status byte-identical), pinned v0.8.1 binary refuses the 6-file honest (exit 4 "wants 4", file untouched). NULL in the trio = **"no producer has ever written it"** — the fetched/not-there third state arrives WITH the first producer, never before (ADR 0011: a checked state for a door that does not exist is a plan, not a measurement); money (ADR 0015) waits for copper, `money_of(NULL)` answers `null` and ZERO rows carry values, so ZERO money exists to answer now. CI gains NO step (31 + one plant line verified by step-line count; naive grep counts the comment's own mention — honest filter used); smoke file BORN at 6, ALTER chain measured on scratch copies post-tag. `price_min` vs `price_min_now`: TWO columns on purpose, stored vs live are two questions. Schema `6` — the FIRST hop since v0.9.0's own; the first release whose dead-code count never left FOUR (`SQL_MIGRATE_TO_V6` had its consumer in the same commit, unlike v0.9.4's transient fifth)
- [x] `v0.9.4` — the money words (ADR 0015): prices read like the GAME writes them, and the unit is PROVEN from the DB's own tooltip copies (`money_text(vendor_sell)` equals the row's own copy for 4 230 of 4 231 rows — 810 `45267` ↔ `Sell Price: 4g 52s 67c`, 733 `100` ↔ `1s`, 36 `7` ↔ `7c`), so the copies also settle the zero rule (a zero denomination is DROPPED: `25000` ↔ `2g 50s`, `20000` ↔ `2g`, middle-zero `20042` ↔ `2g 42c` on row 14138, the only such row in 4 231 while `price_min` has 19 and `price_market` 17). ONE Rust formatter (`money_text`/`money_of`/`money_of_avg` in `db.rs` beside the price accessors) rides as a SIBLING beside every copper number — `Hit`, `Coverage`, `QualityStat`, `Extreme`, `PriceSample` and the card's three top-level fragments — never inside `item` (models.rs mirrors the wire, ADR 0014's ground for `icon_b64`), derived at ANSWER time so nothing is stored: **no column, no schema hop (`schema_version` stays `5`), no refresh door, NO SQL touched** (`SQL_COVERAGE`/`SQL_QUALITY`/`SQL_CHEAP`/`SQL_RICH`/`SQL_PRICE_SAMPLE`/`SQL_HISTORY_OF`/`HIT_COLUMNS` unchanged, read slots stay `0..14`), the copper keeps being what sorts and scripts read. `money_of_avg` FLOORS (`AVG(price_min) 54 391.705896` → `5g 43s 91c`, rounded would state `5g 43s 92c`, a price no observation reached); `price_available`/`price_realms` are auction COUNTS and never become money (row 25 answers `available 12` for 12 offers of 15 000 copper); absent stays absent (`null` beside `null`, never `0c` — measured: 0 rows at copper 0 in any copper column). The UI's ONE `money_cell` puts money in the cell and the exact copper in the hover (hits table, both price lists, the card's price rows and its trend hovers, the status grid, the by-quality bars, the stats note; `.moneycell` `tabular-nums`). Money needs NO flag — it costs ≈ 56 bytes per priced row against ADR 0014's ≈ 3.1 K chars per icon. Honest residue recorded, not swept: item 25's column (`500`) versus its own copy (`Sell Price: 7c`) is upstream's disagreement, the fixture reproduces it (all THREE mirror rows that state words disagree with their column), and money reads the COLUMN; smoke stays **31 cargo steps + one plant line** (money rides, adds nothing), schema stays `5`
- [x] `v0.9.3` — the icon doors (ADR 0014): `--icons` (no-value flag, rides `bare_flag` beside `--watch`/`--cached`) on the list doors `search`/`stats` puts the icon on each row from the DB's OWN bytes through the ONE `b64_text` encoder, which moved from `cli.rs` into `db.rs` beside the bytes it encodes (measured byte-identical across the move); `Hit` and `Extreme` grew the trio `icon_b64`/`icon_at`/`icon_missing_at` (keys always present, payload only when asked, so a no-bytes row says WHICH state it is in); rows collected before the per-row icon reads; ZERO requests leave either door (ADR 0004's `3` unreachable — 0/2/4 only); the UI draws an icon column on the hits and both price lists and answers no-bytes rows with the placeholder glyph `◇` plus the stamp's reason in the hover; ADR 0008's rule again: `icon_url` rides the shape but is never an image src (measured on the user's DB: 4 602 detailed rows all carry an `icon_url`, only 4 030 carry bytes); honest non-gains — the card already rendered an `<img>` since v0.6.0, the watch list has NO panel (measured in `ui/index.html`), the live-search table has no local copy (ADR 0009's door reads/writes NO DB); smoke 29 → 31 cargo steps, schema stays `5` (the doors write nothing)
- [x] `v0.9.2` — the refresh door (ADR 0013): `refresh-twins` re-derives every twin from the copy the DB already holds, ZERO requests (ADR 0004's `3` unreachable, doors are 0/4 only), only differing/missing rows written, tombstones and manual rows ride, rows collected before any write, idempotent, exit 0 with a report when there is nothing to do; it closes ADR 0012's honest gap and v0.9.1's glued-twin residue (measured on the user's file: 4 602 copies, ZERO matching the corrected rule, ALL 4 602 equal to the glued v0.9.0 shape); the UI button `Reword the copies` rides the same verb, CI smoke 24 → 29 cargo steps + one plant line, schema stays `5`
- [x] `v0.9.1` — the v0.9.0 defects, both found by MEASUREMENT not review: `search --where` HUNG 2 h 01 m in both replays (bare `continue` on valid values skipped the parser's shared `index + 2` advance; scratch `timeout 5` repro rc 124) and the twin glued words because live tooltip html is MINIFIED (span → ONE space rule; the fixture's boundary spaces hid it — measured lesson: fixture-shaped data cannot disprove claims about the live shape); `fix(cli)` `ee8c06d` + `fix(db)` `5a36d19`, tag `v0.9.1`

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
own. "Pull all info" chains all FOUR stages to their end in one flow (v0.6.0 grew three of them,
v0.8.0 chains the fourth leg), and the launch audit proves it against the fixture — the stage
verbs interleave with the ticker's `status` polls, all four stages' forced `429` retry in place,
and the scratch DB ends at 0 pending in every queue with all four `sync_state` rows `complete`
(v0.6.0 chained list → detail → icons; the icons stage copies the fixture's real JPEG bytes and
checks the one gone icon file; v0.8.0 adds the tooltip stage, which copies the fixture's own
stand-in `html` — its own words, never upstream's prose — and checks the record whose tooltip is
gone). The four GUI audits agree: the shipped page boots FOUR verbs (`status, watch, stats,
status`) because the new write panel fires nothing by itself, and a temporary probe proved the
four write doors plus a manual row's live card reach the SAME ipc guard and ask the fixture
ZERO times. The GUI uses the same DB resolution as the CLI, so
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
cargo run -- search sword --where description   # the readable twin (v0.9.0, ADR 0012); also `all`
cargo run -- refresh-twins                # re-derive every twin from its copy, ZERO requests (v0.9.2, ADR 0013)
cargo run -- search sword --icons                        # the DB's OWN icon bytes on the hits (v0.9.3, ADR 0014)
cargo run -- stats --icons                               # the same trio on both price lists (v0.9.3, ADR 0014)
cargo run -- stats                                       # money words ride EVERY copper number this
                                                         # answer carries - no flag, no new verb
                                                         # (v0.9.4, ADR 0015: derived at answer time,
                                                         #  the copper stays what sorts and scripts read)
cargo run -- item 25                      # live lookup; 304 + stored raw_json when
                                          # the record is already in the DB
cargo run -- item 25 --cached             # the same card with no request at all
cargo run -- sync --stage icons --ids 25,40   # exactly these icons (also --watch)
cargo run -- sync --stage tooltip --ids 25,40 # exactly these description copies (also --watch)
cargo run -- set-item 25 vendor_sell=606      # the manual layer: one whitelist field, one row
cargo run -- add-item "name=CI Test Mug" quality=Rare  # the DB picks the id (reserved block)
cargo run -- delete-item 60                   # DELETE is a tombstone, every query hides it
cargo run -- undelete-item 60                 # and everything the row had comes back
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
  From v0.8.0 it also answers the TOOLTIP COPY shape: `/api/v1/tooltip/…` answers `200` with
  `html` in upstream's OWN class names but its OWN stand-in words (the probe measured upstream's
  1404 b of markup; the fixture never copies upstream's prose), revalidates the door like the
  detail door (the probe measured `ETag W/"2b111cbcd3bbb4d7078f6d05"` and `cache-control public,
  max-age=300`), and answers `404` with upstream's own JSON BODY — measured 2026-11-20, 79 b:
  `{"error":"not_found","message":"No item 999999999999 in forever.","status":404}` — for ids it
  has no tooltip for, so one ANSWERED record has a real twin for the tooltip stage's third state
  (`12809`; the tooltip gate also names its stage, so the stage retries a `429` in place like the
  other three). From v0.9.0 the sequence is TWENTY-FOUR steps: the same 21 (the `add-item` step now
  ALSO states a description, so its twin is born with it) plus the local door's scope pair
  `search sword --where description` / `search sword --where all` (both DB-only — the fixture
  log's GET tally cannot move) and the manual row's DB-only card `item 900000000 --cached`, which
  shows the twin its stated fields earned at birth. From v0.8.0 the sequence is TWENTY-ONE steps with
  the fourth stage and the four manual doors inside (a budgeted tooltip pass, a drained one, then
  `set-item`/`add-item`/`delete-item`/`undelete-item` on the same DB), and it also covers a watch
  list, a targeted detail fetch, the same ids fetched again (which must answer `304` and write
  nothing) and the DB-only card read. Since v0.5.3 the local mirror adds the gone-id steps on
  the same DB: `item 127555` must answer **exit 4** (`not-found`) and re-stamp the mark, and
  the crawl that follows must answer `requests 0, complete true` — the removed record is not
  asked again. The committed CI sequence passes every step with the `404`s in it.
- The mirror replays the committed sequence verbatim (fresh DB, fresh fixture port) and the
  exit-code audit re-runs every door family against it: at the v0.8.0 tag, **21/21 steps exit 0**
  and **57 doors** land where ADR 0004 says — the eighteen of v0.6.0 byte-for-byte, the seven of
  v0.7.0, the tooltip stage's five (`0,0,0,2,3`), the manual layer's nine (`0,2,2,2,2,4,2,0,0`),
  three queue doors, the edit doors, the tombstone's five (`0,4,4,4,4`), the restore's four and
  one fresh-write door against a SECOND fixture process. No DB-only door ever reached `3`: the
  manual verbs send no request, so they cannot be rate limited. From v0.9.0 the audit grows the
  two scope doors (both rc `0`) and a bogus `--where` landing at the usage door (`2`); each release's
  OWN replay numbers land in the verification record committed AFTER the tag — the tag never moves
  to carry them.
- `tests/api.rs` / `tests/pipeline.rs` (unit tests next to the code they test) are the
  plan, not code: v0.2.0 has no `#[test]` functions yet, so `cargo test` only proves
  everything compiles.
- CI (`.github/workflows/ci.yml`) runs `cargo build` (dev + release, on three platform
  targets), `cargo test` and the 24-step CLI smoke sequence (the three v0.9.0 additions: the local
  door's two scope steps + the manual row's DB-only card) on a seeded fixture DB. It
  **checks the tree out** (`actions/checkout@v7`) and installs the toolchain
  (`dtolnay/rust-toolchain@stable`, rustup-based) plus the gtk/webkit2gtk dev libraries the
  Tauri shell links against on Linux, by plain `apt-get` — the earlier workflow named three
  actions that do not exist (`gh api orgs/…` answers HTTP 404 for two of the orgs) and never
  checked anything out, which is why the first public run died in 7 s with
  "Unable to resolve action". A SECOND measurement round (GitHub run #3) caught three more
  lies with its own lines: `runner.tmpDir` is not a context property — it evaluated to the
  empty string, so the smoke sequence died at its FIRST step opening `/fixture.sqlite` (exit
  4); `macos-latest` IS arm64 (the run's toolchain step answers `host: aarch64-apple-darwin`),
  so the x86 mac std was never installed there; and the Windows build demanded `icons/icon.ico`
  by name — which is REPO work: `icons/icon.png` + `icons/icon.ico` landed as real assets drawn
  by hand (the `bundle.icon` line in tauri.conf.json already pointed at the png, a file that
  never existed). See [ADR 0011](./DECISIONS/0011-ci-asserts-only-what-is-true.md). GitHub run
  #4 (`59b8379`): **5/5 jobs green in 9 min 13 s** (quality 2 m 04 s · build macos 2 m 27 s ·
  test 4 m 19 s · build linux 5 m 46 s · build windows 9 m 12 s).
  What CI does NOT do, and why: no `cargo fmt --check` gate (measured on this tree: it answers a
  diff, because the house style is hand-written — a fmt-clean tree would be its own deliberate
  milestone) and no `-D warnings` lint gate (this build's four dead-code warnings ARE the plan
  this file names; `cargo clippy --all-targets` is measured at 0 errors / 600 warnings, so it runs
  as an informational step). The GUI is **compiled** by CI but not exercised: CI has no display
  server, so the webview's only access to the app is the CLI's argv, which makes the smoke
  sequence the app's integration test (ADR 0004, ADR 0005); the GUI launch audit runs on a real
  desktop and is recorded in [docs/DEVLOG.md](./DEVLOG.md).

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
