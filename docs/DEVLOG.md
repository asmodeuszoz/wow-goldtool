# Development log

Dated, reverse-chronological notes on what was tried, measured and decided while building
WoW Goldtool. The changelog records *what* shipped; this file records *why*.

---

## 2026-11-19 · v0.4.0 — the shell: `build.rs`, `window.__TAURI__`, chunked cancel

### The question that shaped the release

v0.3.0 proved the pipeline headless. The next question is the one the IDE asks first: does
`cargo run` open a window at all? Everything in this release is downstream of reading the
crate sources in the registry cache instead of tutorials — every tutorial assumes `create-tauri-app`
plus `tauri dev`, which [ADR 0001](./DECISIONS/0001-flat-cargo-layout-no-cli.md) forbids.

### The trap that cost the most time

`cargo build` failed with `error: OUT_DIR env var is not set, do you have a build script?`,
raised by `tauri-codegen/src/embedded_assets.rs:335` while expanding `generate_context!`. The
obvious fix — an empty `build.rs` — compiles, opens the window, loads the page **and leaves
`window.__TAURI__` undefined**, so every button in the UI fails silently. The reason is one
line up the pipeline: `tauri-codegen/src/context.rs:412` injects the global-API scripts only
when `tauri_utils::plugin::read_global_api_scripts(out_dir)` finds a list file that
`tauri_build::build()` writes. So the fix is not "a build.rs", it is "the build.rs that runs
`tauri-build`", which is why `tauri-build = "2.7.6"` is a build-dependency and why
`tauri.conf.json`'s `withGlobalTauri` means anything at all. A GUI feature that can fail
silently is a design smell, and it is why the audit below ends in Rust log lines.

### ACL: measured, not assumed

The gate is `tauri/src/webview/mod.rs:2080`:

```rust
if (plugin_command.is_some() || has_app_acl_manifest || !is_local)
  && request.cmd != crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND
  && invoke.acl.is_none()
```

Ours is none of the three: `cli_call` is an app command, the page is a local origin, and
`has_app_acl_manifest` is false — `acl/mod.rs:353` defines it as `acl.contains_key("__app-acl__")`
and the build's own artifact `gen/schemas/acl-manifests.json` carries 10 plugin keys with no
`__app-acl__` in it. The app command is reachable **by construction**, and the evidence is a
file the build itself produced. The trap that follows is recorded in ADR 0005: the moment an
app-side ACL manifest lands, `has_app_acl_manifest` flips and a missing grant becomes a hard
failure.

### Cancel without `AppState`, because v0.3.0 already made it possible

ADR 0002 says the progress truth is the `sync_state` row, and v0.3.0 made `sync_state.page` an
absolute cursor with resume legal for `stopped` and `cancelled`. That makes chunked sync free:
the UI issues `sync --stage <stage> --budget 10` (list) or `--budget 25` (detail) in a row until
the report says `complete`, each chunk ends with a `sync_state` write, and a JS flag read at the
next chunk boundary *is* cancel. There is no `state.rs`: nothing mirrors the cursor in Rust, so
nothing can drift. The cost is honest — cancel waits for the chunk, and a `304` still costs one
request of rate budget (the v0.3.0 note), so a chunk of list pages is cheap in bandwidth, not
in budget.

### The icon is generated, so the blob is explainable

Codegen reads `bundle.icon` at compile time (`CachedIcon`), so `cargo build` fails without the
PNG. `tools/gen_icon.py` draws it in PIL from fixed constants — navy plate, gold coin, rising
price line — supersampled 1024 → 256 with LANCZOS. Nothing is downloaded or pasted: the script
sits next to the committed PNG, so a fresh clone builds and a reviewer can see how the icon was
made.

### The version source, once more

`tauri-codegen/src/context.rs:255` computes `package_version` as `config.version` **else**
`env!("CARGO_PKG_VERSION")`. The shipped conf therefore omits `version` entirely: `Cargo.toml`
is the only version source, and the house rule that used to say "bump both files" was amended
rather than broken silently.

### Launch audit — display server: KDE Plasma on Wayland (`wayland-1`), XWayland `:1`

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| `cargo build` with the new files | still one buildable unit | four errors, each fixed at the source of the message: `OUT_DIR` (build.rs + `tauri-build`), `cannot find macro __tauri_command_name_cli_call` and `__cmd__cli_call` (→ `generate_handler![crate::commands::cli_call]`, because `handler.rs:166` rewrites the last path segment), `E0382` partial move of `inv` (→ match on `inv.fixture.clone()`) |
| `WGT_DB=/tmp/opencode/fixture.sqlite cargo run` | window opens, page loads, JS reaches Rust | `gui: wow-goldtool shell starting…` → `gui: the webview asked for tauri://localhost (window main)` → `gui: ipc verb status (0 words)` — the page-load log proves the asset map served `index.html`, the ipc line proves the awaited `core.invoke` round-tripped |
| webview console | not a channel | `console.log` added to `ui/app.js` never appeared; grepping `tauri`/`wry`/`tauri-runtime-wry` for console forwarding found nothing on Linux, so the observable audit channel is Rust's `gui:` lines and the UI states its own rendered-field count in the status card |
| CI mirror + exit codes + v0.3.0 audits after the `cli.rs` refactor | unchanged | smoke mirror all `exit=0` (it is a zsh script: run with `zsh`, `bash` mis-parses the `$=step` split), exit-code audit `2 · 2 · 3 · 4 · 4 · 0`, `resumed_from: 2`, `unchanged: 2`, `item 25 → 304 reused true` |
| `window.__TAURI__` presence | proven by the round trip above | the empty-`build.rs` trap above is exactly the state that produces no `gui: ipc …` line |

### Deliberately not done in this tag

No packaging yet (`bundle.icon` and `identifier` are in place for it). No `state.rs`. No
detail-sync target selection from the UI yet — the buttons run the whole stage with chunk
budgets, and choosing watchlist/ids is v0.5.0. The webview's first paint was not reviewed
pixel-by-pixel: KDE is Wayland, `grim`/`kscreen-shot` are absent and X11 `import` cannot see
Wayland windows, so the visual check is a `cargo run` for the user to run.

### Follow-ups parked for later

* `.deb`/`.AppImage` once packaging is worth doing — the identifier and icon groundwork is done.
* Whether a second window (`app.windows` with `templates`) is worth the config surface.
* `tests/api.rs` / `tests/pipeline.rs` unit tests are still a plan, not code.

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
