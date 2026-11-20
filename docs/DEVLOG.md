# Development log

Dated, reverse-chronological notes on what was tried, measured and decided while building
WoW Goldtool. The changelog records *what* shipped; this file records *why*.

---

## 2026-11-20 · v0.5.2 — patch: pending's real rule, the client-build stall, the one-click catalogue

### The ask

"instead of the 'sync pending details' make it into a pull all info… basically
run the 'Re-check online' for every item that doesn't have all its information
in the DB yet… one big run and then never have to touch it again." A button that
retires pending work is only honest if pending means *work that exists*.

### Pending: the rule had to carry the truth

`SQL_PENDING` asked `price_min IS NULL OR detail_synced_at IS NULL`. The first
clause was never a pending rule: `docs/DATABASE.md` distinguishes fetched-with-
NULL-price (the item exists, it is **not on the AH**) from never-fetched on
purpose, and `price_min` is read field-by-field out of an optional object, so a
fetched record can legitimately carry a NULL price. Keeping such an item pending
means the detail stage walks it again forever — the stage never terminates on the
real catalogue, and no pull-all flow can promise "nothing pending". The user's DB
(read-only query: 19258 items, 7 with details, 0 fetched-and-unpriced at this
hour) cannot expose the trap yet, because only 7 records have ever been fetched:
the clause is latent, and it becomes loud the moment a real catalogue item answers
without AH data. It is fixed **before** the flow that depends on it ships, not
after that flow has lied. `detail_synced_at` already encodes "has a detail record",
so pending is now that one predicate.

### The GUI never reached the API

Driving a full `runPullAll()` from the boot probe made the next failure loud: the
stage verb entered Rust and then nothing happened, and the fixture access log stayed
empty for the whole session. Markers at each verb step pinned the stop point: the
verb logged the DB-open marker and never reached the request site — it stalled inside
`build_client()`. The DB was never the lock: status polls completed normally between
those marker lines, and the page stayed interactive throughout.

Mechanism read from the crates: reqwest's blocking `Client::new` does not build a value
on the spot. `ClientHandle::new` (`blocking/client.rs:1424` spawns a thread named
`reqwest-internal-sync-runtime` that builds its **own** single-threaded tokio runtime
(`runtime::Builder::new_current_thread()` at `:1428`) and `rt.block_on`s the request loop
(`:1468`); the thread that asked for the client then parks while waiting for that
thread's startup handshake (`wait::timeout(spawn_rx, None)` at `:1476`), and that wait
polls the future and `thread::park()s` the caller (`blocking/wait.rs:58`). Headless that
is exactly the CLI's shape, and every CI mirror has always worked; in the GUI the verb
body sits on a tokio worker of tauri's async manager (v0.5.1's `async fn cli_call`), and
parking there while the client's private runtime has to start is the nested-runtime case:
measured as the stall at `build_client`, zero requests issued.

Fix: the command stays `async` — v0.5.1's wrapper decision is untouched, the IPC handler
still answers immediately and the page keeps ticking — and the verb body hops to a fresh
`std` thread, the runtime-free home the headless CLI proved good. `join` occupies one
tokio worker, which the webview never borrows. A verb that panics on that thread becomes
an `AppError` instead of leaking `Result`'s `String` payload, so the envelope stays the
one shape the UI knows. The helper sits at the crate root because that is where both doors
reach it without an import cycle.

### The flow itself

`btn-detail` becomes `btn-pull` ("Pull all info") and `runPullAll()` chains the two
stages to their ends: the list walk resumes from `sync_state` and runs to completion,
then the pending queue is drained to completion. Orchestration only — ADR 0005's one-
command door, the chunk budgets, the cancel flag and the DB's progress truth are
untouched. For the chain to know a stage finished, `runSync` returns an outcome
(`complete`/`cancelled`/`stalled`/`failed`/`already-running`): Cancel stops the stage in
flight and the flow stops with it rather than jumping ahead, because the next click
resumes the list from the stored cursor and the pending queue recomputes which details
remain. The same change re-orders the chunk's two checks (`complete` before `requests
=== 0`): a stage with nothing left to fetch answers `complete: true, requests: 0`, which
is a legal completion, not a stalled crawl — the old order called it "made no progress".

The completion note refuses to overpromise. "Nothing left to pull" means every listed
item has a detail record; a NULL price on a fetched item is *not on the AH*, not missing
work; and prices are live data, so later syncs still make sense — they revalidate cheaply
(a finished walk restarts at page 1 and every page answers `304` from the stored ETags).

### Audit

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| pull-all launch probe (shipped code + boot probe, fresh fixture server + fresh DB) | both stages crawl, page alive throughout | `sync` verb → two ticker `status` polls → second `sync` verb → three more polls |
| fixture access log for that run | one forced `429` per stage, retried in place, then payloads | page 1 `429` then `200`, page 2 `200`, item 25 `429` then `200`, items 40/60/12809/127555 `200` |
| scratch DB after the pull | everything fetched, nothing pending | 5 items, 5 detailed, **0 pending**, both `sync_state` rows `complete` |
| boot audit on the shipped binary (probe removed) | nothing crawls by itself | exactly four verbs `status/watch/stats/status`, zero fixture hits |
| CI mirror | eleven steps | 11/11 `exit=0`, and both stages report `rate_limited: 1` with the wait inside the run (`seconds: 1`) |
| exit-code audit | unchanged | `2 · 2 · 3 · 4 · 4 · 0` |
| stop → resume → resync | the cursor contract stays | `--budget 1` stops (`complete: false`), the next run reads `resumed_from: 2` without refetching page 1, and a finished walk restarts at page 1 with both pages `304` (`unchanged: 2`, `written: 0`) |
| clean build | warnings only | the dead-code set already planned for v0.6.0 (`PendingSql` privacy, `SearchResponse`, `search_url`, `remaining`, `open_db`, `catalogue`) plus the filesystem's hardlink notice |

### Deliberately not done in this tag

No engine change for pull-all: it is UI orchestration over the existing stages, so ADR
0005 and the CI mirrors stay untouched. No new progress channel — the rejected `emit`/
`event.listen` note in ADR 0005 still applies, and the ticker remains the mid-chunk
feedback. No price history, no icon copies, no item-card rendering of `sockets`/`stats`/
`classes` (that stays v0.6.0).

---

## 2026-11-19 · v0.5.1 — patch: the crawl no longer owns the page

### The report

During a sync list run the whole app was frozen: hovering a button stopped
highlighting, clicking anything took several minutes. The measurements agree
on the length: one GUI chunk is 3.5 s (fixture) / ~5 s (real) at the 2/s
pacing — and a 429 with a `Retry-After` sleep inside the chunk stretches it
to minutes. The "several minutes" is that wait, charged to the page rather
than to the crawl.

### The cause, read from the crate source

`#[tauri::command]`'s wrapper kind decides it. A bare `fn` gets the macro's
blocking body, and that body's last acts are `kind.block(result, resolver)`
(`tauri-macros/src/command/wrapper.rs:425`) then `return true` (`:428`): the
verb runs synchronously **inside** the webview's IPC handler — and that
handler is the page's message loop. So hit-testing (CSS hover), click
dispatch and any later IPC message can only run when the verb returns. A
hover that stays dead for exactly one chunk is that stall.

### The fix: the command itself

`cli_call` becomes `pub async fn`. `body_async` hands the future to
`respond_async_serialized` (`wrapper.rs:~372`): tauri's tokio-backed async
manager — tokio `rt-multi-thread` is a required dependency of tauri 2.x,
not a feature switch, so the manager exists in this build — and the IPC
handler returns immediately. Same argv, same dispatch in `cli.rs`, same
envelope: only the wrapper kind flips (sync → async). The crawl still runs
synchronously inside its own future (pacing blocks one tokio worker and
none of the webview); tokio logged no complaint about the blocking calls,
stderr stayed all `gui:` lines.

Cancel granularity deliberately unchanged (chunk ends, ADR 0005): this
release is about the page, not the crawl — and "always honour `Retry-After`"
is policy, so the wait still delays Cancel; what must not delay is the page.

### Liveness made visible

The UI gains a ticker: while a chunk's await is pending, every second the
status card re-reads the DB (coverage grows per page in list, per item in
detail) and the sync note carries the chunk's elapsed time. Honest about
position on purpose: `sync_state` moves only at chunk ends (`finish()` in
src/sync.rs), and ADR 0002 keeps progress truth in the DB — so the note
prints elapsed seconds, not a mid-chunk cursor that would be a lie.

### Audit

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| boot audit after the fix | unchanged from v0.5.0 | four verbs `status/watch/stats/status` |
| probe launch: 12-id targeted chunk, timestamps added for the run (probe removed before commit) | status polls **inside** the chunk await | `sync` verb at t=…268, ticker polls t=…269…293 one per second — the Retry-After stretch with the page alive |
| tokio slow-task warnings | none | stderr still only `gui:` lines |
| exit-code audit | unchanged | `2 · 2 · 3 · 4 · 4 · 0` |
| CI mirror | unchanged | 11/11 `exit=0` |

**Amendment (v0.5.2)**: the probe row above attributes the visible stretch to a
`Retry-After` wait inside the chunk. The v0.5.2 audit shows that probe's crawl never
issued a request from the GUI at all — it stalled at `build_client`, so no wait was ever
reached from the webview. The liveness witness itself stands (the `status` polls
interleave while the verb runs, and that is what the fix is about), and the wrapper
decision it recorded is unchanged; only the cause of the stretch was attributed wrong.

Rejected for mid-crawl feedback: a JS progress channel (`emit`/`event.listen`)
— those are plugin commands and ACL-gated; the exemption recorded in ADR 0005
would stop covering them (an `app:core:event` grant would be needed), and
progress truth is already the DB.

---

## 2026-11-19 · v0.5.0 — targets, analytics, the card: what the buttons finally answer

### The question this release answers

v0.4.0's sync buttons could only crawl "everything": the list stage, and the whole pending
queue. The watch list had existed since v0.2.0 as something that only **printed**. The
release starts where the roadmap arithmetic already insisted: 19 k details cost 2.7 hours,
so the only useful question is "which details do I actually need?" — and that question
belongs to the user, not the crawler.

### A latent bug found by the first run that read those columns

`sync --ids 25,40` exited 4 on the first audit, and the message was rusqlite's:
`Invalid column type Null at index: 0, name: detail_etag`. Three single-column lookups in
`db.rs` were written `Ok(value) => Ok(Some(value))`, which makes rusqlite infer a **bare
String** for the getter, so a NULL column is an error instead of a value. The signatures
always claimed `Option<String>`; nothing had ever read a NULL through them, because the
pending stage fetched an item before it ever asked for that item's ETag. The targeted stage
reads `detail_etag_of` for every id, so the latent wrongness surfaced immediately. Asking
for the Option type makes NULL a `None` — which is what the type always promised.

### Chunking targeted ids in JS, not in the engine

The engine's `--budget` truncates from the head of the list, so a second chunk of the same
`--ids` would re-visit what the first chunk already fetched and the UI's chunk loop would
never reach the tail. The UI slices instead (20 ids per call), which also keeps Cancel
meaningful between chunks and one slice far under the 120-request burst. `--budget` still
bounds the pending crawl, where it means "items per run".

### Honest accounting, twice more

A targeted run answers its own question: `remaining` is the target list, not the pending
queue ("left in the list" ≠ "left pending"), and `complete` means the target set was fully
visited. The detail stage now sends the stored `detail_etag` when one exists, so re-asking
for the same ids is a `304` — the audit reads `requests 2, written 0, unchanged 2` on the
second pass over 25,40. docs/API.md counts both sides the same way (a `304` costs rate
budget, saves bandwidth and DB churn) and the sentence about "pending items have no ETag
yet" is now scoped to what it actually is: the pending stage.

### The card reads the DB, and offline stays the point

`item ID --cached` sends no request: the card answers from `raw_json` and prints
`detail_synced_at` so the user can tell today's prices from last month's. A record never
fetched is `not-found`, not an empty card — the UI turns that word into an offer, and the
first use of the `NotFound` error kind retired its dead-code warning. No item icons are
rendered: `icon_url` is a remote path, and hotlinking it would trade the app's whole
purpose (answering offline) for decoration. Quality colour carries identity instead; a
local icon copy is roadmap work.

### Analytics before any crawl

`stats --limit 6` is one verb, and its JSON (coverage, by-quality, extremes) renders as
bars in the same palette as the search chips, cheapest/priciest extremes, and one sentence
about the spread. The question "is this worth hunting" is answerable before the detail
stage has seen a single item.

### boot() as the audit's witness

Startup reads status, the watch list and the analytics, so no card paints empty — and the
Rust log shows four verbs (`status`, `watch`, `stats`, `status`). The fourth line is the
witness that the analytics **render** ran to the end: a JS throw after the awaited verb
would leave three lines, not four. CI still cannot open a window, so the eleven-step CLI
mirror (six new steps: `watch add`, a targeted fetch, the same ids again answering 304s, a
watch-list target, the DB-only card) is what guards this release.

### Audit table

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| `sync --ids 25,40` (first pass) | both written | `requests 2, written 2, unchanged 0, complete true` |
| `sync --ids 25,40` (second pass) | revalidation, nothing rewritten | `requests 2, written 0, unchanged 2` |
| `sync --watch` after `watch add 25 40` | target = the watch list | `visited every targeted record: 2 fetched` |
| `--ids 25,x` | usage | exit 2 |
| `item 999 --cached` | not fetched yet, not a crash | `not-found`, exit 4 |
| GUI launch | window + four verbs from `boot()` | `gui: ipc verb status/watch/stats/status` |
| CI mirror | eleven steps | all `exit=0` |
| exit-code audit | unchanged | `2 · 2 · 3 · 4 · 4 · 0` |

### Deliberately not done in this tag

No price history yet: the DB stores one price per item, so a trend line needs a second
table (roadmap). No icon download. No item-card rendering of `sockets`/`stats`/`classes`
values (they arrive as unshaped JSON, and inventing a shape is not our call to make).

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
