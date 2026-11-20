# Development log

Dated, reverse-chronological notes on what was tried, measured and decided while building
WoW Goldtool. The changelog records *what* shipped; this file records *why*.

---

## 2026-11-19 · v0.3.0 — reuse, resume, cancel: the second crawl

### The question that shaped the release

The v0.2.0 crawler only ever crawled forward from scratch. Re-opening the app would
re-download everything it already knew. The upstream had told us how to avoid that
(`ETag` + `If-None-Match` → `304` **with no body**) — so the release is built around
what a body-less response forces us to admit.

### What a 304 forces, and what the v0.2 code would have done

The old loop fell through to `parse_page(body)` for every non-429 status: a `304` would
have died as a decode error on an empty body. Every reuse path now branches **before**
parsing. Because a list page has no DB row to re-read, each page also caches its
`has_more` and `count` in `meta` (`list-page-<n>-etag/-more/-count`) — the crawl needs
exactly those two facts to decide "walk on?" and keep `total` honest across 304s.

The detail side is opposite: the record is already on disk, so `raw_json` is the stored
copy and `item 25` revalidates the row instead of echoing a fresh payload. A `304`
**without** a stored copy is a real inconsistency — reported as an honest decode error,
never papered over. And a fresh `200` from a live lookup is now written through the
same upsert the sync stage uses, so its ETag can be reused next run.

### Honest accounting, twice

- A `304` still costs one request against the burst: revalidation buys bandwidth and
  DB churn, **not** rate budget. Written into docs/API.md so nobody reads "reuse" as
  "19 k fewer requests".
- `pages` counts a run; the `sync_state` page column must be an **absolute** cursor for
  resume to mean anything. `finish()` took an explicit `cursor` parameter — the two
  numbers were quietly the same value in v0.2 only because every run started at page 1.
- The report grew `resumed_from` and leans on `unchanged`: a run of two 304 pages
  honestly reads `complete: true, items: 0, unchanged: 2`.

### Resume only for pauses, not for completions

`stopped` and `cancelled` resume at stored page + 1. `complete`/`ceiling`/`idle` restart
at page 1 — a completed crawl has nothing to resume from, and the ETag walk makes that
cheap. Unknown status strings fall to the same branch, so a hand-edited DB cannot wedge
the crawler.

### Cancel without signal juggling

No atomics, no FFI `sigaction`: a plain `CancelFlag` struct that the loops check before
every request. The GUI will own one in AppState (v0.4.0) and raise it from the UI; the
CLI hands out a fresh one because Ctrl-C already leaves the DB consistent (every write
is its own statement) and the next run resumes by construction. A cancelled run records
status `cancelled` and exits **0** — the user asked for it, nothing failed.

### Fixture audit — every claim executed, never asserted

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| list `--budget 1` | stopped, 1 page, forced 429 retried | `requests 2, pages 1, rate_limited 1, complete false` |
| list (resume) | `resumed_from 2`, page 1 not refetched | `requests 1, resumed_from 2, complete true` |
| list (full re-sync) | both pages `304`, nothing written | `unchanged 2, written 0, items 0` |
| detail | 429 + retry + 5 records | `requests 6, written 5, rate_limited 1` |
| `item 25` | `304`, record served from stored `raw_json` | `status 304 reused true` |
| exit-code audit | 2 usage · 3 network · 4 DB · 0 healthy | all six cases re-ran green |

The fixture grew conditional-GET behaviour itself (stable per-endpoint ETags, real 304
responses with no body): a server that cannot answer `If-None-Match` would have made the
release untestable, so the test asset learned the protocol at the same commit.

## 2026-11-19 · v0.2.0 — data layer, sync engine, contract checks

### One buildable unit, eight modules

The data layer and the sync engine landed together because they are one compile unit:
`main.rs` declares every module, so a half-written tree does not build. Splitting further
would have broken the "every commit must build" rule, so the code ships as one
`feat(core)` commit and the docs/CI split into separate commits after it.

### rustc as the arbiter, twice

The session had a display quirk: tool output and echoed payloads mangled `::` into `.`
and angle-bracket generics into parentheses. Never patched Rust from displayed text;
byte counters in python verified file contents and rustc decided what was right. A
multi-file "apply rustc suggestions" script also overwrote files **inside `~/.cargo`**
(`rusqlite`'s and `serde_json`'s extracted sources); the fix restricted the applier to the
diagnostic's primary file, and the corrupted crates were restored byte-identical from the
registry cache tarballs.

### Two bugs the compiler cannot see

- **SQL arity drift**: the detail upsert declared 38 columns but its VALUES run had 40
  markers — rusqlite fails at *runtime*, rustc stays silent. A checker script cross-verified
  column names, VALUES markers, SET assignments and the `*_VALUE_COUNT` consts; 40 fixed to
  38.
- **WHERE-chain builder**: it emitted `a AND b AND ` with no keyword — SQLite answered
  `near 'LIKE': syntax error`. Now the builder owns its keyword: `format!("WHERE {} ",
  joined)`.

Verified along the way: bundled rusqlite 0.43 really does accept the
`CREATE INDEX name ON table (cols)` spelling that docs/DATABASE.md documents — the open
question from v0.1.0 is closed.

### The fixture caught a lie in the report

`sync --stage detail` against the fixture reported `done: 4 / total: 5` next to
`"complete": true`: a 429 hit the `continue` of the per-item loop, which dropped that item
from the run, and `complete` was hardcoded. Fixture forced this by answering the **first
request of each stage** with 429 + Retry-After. Fixed the semantics: retry the same item
in place (up to 3 retries after each Retry-After), leave it pending if that still fails,
and derive `complete` from the remaining pending count — otherwise report `partial`.
Re-run: `requests: 6, written: 5, rate_limited: 1, complete: true`, exit 0.

### Exit-code audit (ADR 0004)

| Scenario                     | Observed | Expected |
| ---------------------------- | -------- | -------- |
| unknown verb / unknown flag  | 2 / 2    | 2        |
| unreachable base URL         | 3        | 3        |
| corrupt DB via `--db`        | 4        | 4        |
| unwritable DB path           | 4        | 4        |
| healthy `status`             | 0        | 0        |

### House rule amended mid-flight: the version bump

"Version bumps happen in `Cargo.toml` **and** `tauri.conf.json` together" cannot run yet:
`tauri.conf.json` joins the tree at v0.4.0 (it is what `generate_context!` reads). The
rule now means "every version field that exists today"; `Cargo.toml` bumps to 0.2.0, and
the conf-file half of the bump arrives with the GUI shell.

## 2026-11-19 · v0.1.0 — scaffold, measurements, decisions

### Setting the stage

The project started life as a default RustRover binary (`hello world`, git history empty,
branch `master`). Before writing any application code I pinned down the two things that
decide the whole architecture: **what the data source can actually do** and **what the
toolchain can actually build**.

### Toolchain probe (before writing a line of app code)

| Check | Result |
| ----- | ------ |
| `rustc` / `cargo` | 1.94.0 (2026 snapshot) — comfortably newer than Tauri's 1.81 floor |
| Node | v22 present, but deliberately **not** used (see below) |
| `tauri` crate resolves | 2.12.7 (`cargo add tauri@2` locked 427 packages, incl. webkit2gtk/gtk stack) |
| `tauri` + webkit2gtk + gtk compile on this box | ✅ probe crate built in 61 s |
| `rusqlite` / `libsqlite3-sys` | already in the cargo cache, `bundled` feature available |
| `reqwest` 0.13 | default TLS is rustls (`aws-lc-rs`), so no OpenSSL linkage assumption needed |
| Desktop | KDE Plasma on Wayland + XWayland (`WAYLAND_DISPLAY=wayland-1`, `DISPLAY=:1`) |

The probe crate is thrown away; its only purpose was to prove the GUI layer can build
before I spend effort on it. Lesson learned early and written into the house rules:
**probe the toolchain, then design**.

### The hard constraint that shaped the layout

The app must compile straight out of RustRover. That killed the usual
`create-tauri-app` + `tauri dev` route, which needs the Tauri CLI and a Node toolchain to
generate the config context and the asset map. Reading the actual crate sources in the
cargo registry cache answered the question precisely:

* `tauri-macros` documents `generate_context!` as reading `tauri.conf.json` **relative to
  `CARGO_MANIFEST_DIR`, at compile time**;
* `tauri-codegen/src/context.rs` then reads `config.build.frontend_dist` **from the
  filesystem during compilation** and generates a `phf_map!` of assets by itself
  (`EmbeddedAssets::new(phf_map! { … })`).

So the plain `cargo build` path already embeds config + UI assets. The Tauri CLI is only
needed for hot reload and installer bundling. That made the flat layout
([ADR 0001](./DECISIONS/0001-flat-cargo-layout-no-cli.md)) strictly better for this
project than the `src-tauri/` convention.

### API measurement day (TheWoWDB, all figures below are measured, not assumed)

* `GET /api/v1/forever/item/25` → full record for *Worn Shortsword*: `price.available 11`,
  `price.market 400`, `price.min 290` copper, `vendor_sell 7`, `weapon{1-3 dmg, 1.9 s,
  dps 1.1}`, `quality "common"`, faction price buckets present but empty for that item.
* `GET /api/v1/forever/search/?q=Worn Shortsword` → grouped hits with per-kind `has_more`
  and a `more` URL.
* Paging: `per_page` is clamped server-side to **100** (asked 200, got `per_page: 100`);
  `page` runs to 200.
* Total item count is **not** reported on large lists — only `has_more` + `links.next`.
  Binary search over pages located the last page carrying data: **page 193, 57 records,
  `has_more:false`** ⇒ ≈ **19 257 items**. Ids are sparse (first 25, last 288 340), so
  id-space iteration is useless: crawl by page, not by id.
* Docs confirm caching rules (`ETag`, ~1 h public cache, tooltips 5 min, `304` on
  `If-None-Match`) and the rate model: **120 req burst per IP, refilled at 2/s**, `429` +
  `Retry-After` + `X-RateLimit-Remaining` past that.

Cost arithmetic written down before designing the sync, because it decides the design:
193 list requests ≈ 2 minutes, a full detail pass ≈ 19.3 k requests ≈ **2.7 hours**. A
single "sync everything" button is therefore unusable as designed → stage the sync, make
it resumable, cancellable and opt-in ([ADR 0002](./DECISIONS/0002-two-stage-sync.md)).

### Decisions recorded as ADRs

| ADR | Verdict in one line |
| --- | ------------------- |
| [0001 flat Cargo layout, no Tauri CLI](./DECISIONS/0001-flat-cargo-layout-no-cli.md) | RustRover must build the app with plain `cargo build`; `generate_context!` already does the embedding |
| [0002 two-stage sync](./DECISIONS/0002-two-stage-sync.md) | list crawl is cheap, per-item detail crawl is 2.7 h → stage, resume, cancel, opt-in |
| [0003 rusqlite over the DB](./DECISIONS/0003-sqlite-rusqlite-bundled.md) | one disposable file with sort orders the upstream API cannot express |
| [0004 headless CLI in the same binary](./DECISIONS/0004-headless-cli-mode.md) | the data pipeline must be testable without a display server |

### Deliberately not done yet

No application code in this tag. `src/main.rs` is still the generator's `hello world`, so
`cargo build` passes and CI would be green: the scaffold is documentation-only.

### Follow-ups parked for later

* Packaging: `.deb`/`.AppImage` generation without the CLI (or a documented
  `src-tauri`-compatible shim) — roadmap item after `v0.5.0`.
* Retail version (`/api/v1/retail/...`) as a second catalogue tab; `forever` is pinned for
  now (see [docs/API.md](./API.md)).
* Whether item tooltips (`/api/v1/tooltip/...`) should be fetched lazily on demand instead
  of reconstructed from the detail record.
