# ADR 0005 · One IPC command, ACL by construction, chunk-granular cancel

- **Status:** accepted
- **Date:** 2026-11-19
- **Discussion:** [docs/DEVLOG.md](../DEVLOG.md) · commits tagged `v0.4.0`

## Context

`v0.4.0` has to put a UI in front of the CLI verbs of
[ADR 0004](./0004-headless-cli-mode.md) while keeping
[ADR 0001](./0001-flat-cargo-layout-no-cli.md) (plain `cargo`, no Tauri CLI, no Node).
Four questions have no answer outside the crate sources, because every tutorial assumes
the CLI layout that ADR 0001 rejects. Each fact below is quoted from the crates in the
cargo registry cache (`file:line`), not from prose docs.

| Question | Fact found in the crate source | Consequence |
| -------- | ------------------------------ | ----------- |
| Does the page load at all? | `tauri-macros/src/lib.rs:200` — conf is read **relative to `CARGO_MANIFEST_DIR` at compile time**; `tauri-utils/src/config.rs` `WindowConfig.url` deserializes `"index.html"` through `WebviewUrlDeserializer::Path` → `WebviewUrl::App(PathBuf)` | the window is declared in `tauri.conf.json` (`app.windows`), the page is served from the embedded asset map |
| Is `window.__TAURI__` injected without the CLI? | `tauri-codegen/src/context.rs:412` injects the global-API scripts **only** when `config.app.with_global_tauri` **and** `tauri_utils::plugin::read_global_api_scripts(&out_dir)` is `Some`; `tauri-utils/src/plugin.rs:63` returns `None` when `GLOBAL_API_SCRIPT_FILE_LIST_PATH` is not in `OUT_DIR`; that file is written by `tauri-build::build()` | a `build.rs` calling `tauri_build::build()` is **required**; `withGlobalTauri: true` on its own does nothing at all (an empty `build.rs` fixes the `OUT_DIR` codegen read but leaves the webview without `__TAURI__` — a silent trap) |
| Are the app's own commands ACL-checked? | `tauri/src/webview/mod.rs:2080` gates the invoke on `plugin_command.is_some() \|\| has_app_acl_manifest \|\| !is_local`; `acl/mod.rs:353` defines `has_app_acl` as `acl.contains_key("__app-acl__")`, and the build's own `gen/schemas/acl-manifests.json` carries 10 plugin keys with **no** `__app-acl__` | a local page reaching an app command is **not** ACL-checked by construction: no `*.permission.toml` is needed today, and the reason is visible in the shipped build artifacts |
| Does `Builder.run` block? | `tauri/src/app.rs:2616` — `Builder::run(ctx)` is `self.build(ctx)?.run(\|_, _\| {})` | `run_gui()` blocks until the window is closed, then `main` exits 0 — the GUI is a normal foreground process, so RustRover's run button behaves |
| How is a command from another module registered? | `tauri-macros/src/command/handler.rs:166` rewrites the last path segment to `__tauri_command_name_<cmd>`, a macro emitted next to `#[tauri::command]` | register as `generate_handler![crate::commands::cli_call]`; `use crate::commands::cli_call;` alone does not bring the macro along |

## Options considered for the UI ↔ Rust bridge

| Option | Trade-off |
| ------ | --------- |
| one command per CLI verb | six Rust wrappers, six JSON shapes the UI must learn; the UI and the CLI can drift apart, which is exactly what ADR 0004 refuses to allow |
| crawl state in `AppState` + a `cancel()` command | progress would live twice — a Rust mirror of the `sync_state` row and the DB — and ADR 0002 says the DB is the truth |
| **one `cli_call(argv)` + a JS chunk loop bounded by `--budget`** | one wrapper; the argv the CLI parses *is* the argv the UI sends; cancel is a JS flag read at the next chunk boundary |

## Decision

* `src/commands.rs` holds exactly one `#[tauri::command] cli_call(argv: Vec<String>) -> String`.
  It returns an envelope: `{"ok": true, "result": <verb json>}` or
  `{"ok": false, "error": {"kind","exit","message","retry_after"}}`. It contains no logic
  besides handing the argv to `cli::dispatch` and wrapping the result.
* `src/cli.rs` grew `dispatch(Invocation) -> AppResult<String>`, the single place both doors
  send verbs through; `run()` is the CLI door (`guard(dispatch(inv))`), `cli_call` the GUI
  door. The `--fixture` switch is applied inside `dispatch`, so both doors can aim at the
  fixture server.
* `ui/app.js` issues a crawl as chunks: `["sync", "--stage", stage, "--budget", …]` — 10
  requests per chunk for `list`, 25 for `detail` — and loops until the report says
  `complete`. Cancel sets a JS flag the loop reads before the next chunk.
* `build.rs` calls `tauri_build::build()`; `tauri-build = "2.7.6"` is a **build**-dependency.
* `tauri.conf.json` ships no `app.security` and no `version` —
  `tauri-codegen/src/context.rs:255` falls back to `env!("CARGO_PKG_VERSION")`, so `Cargo.toml`
  is the single version source.

## Amendment (v0.5.1, 2026-11-19)

* `cli_call` became an `async fn` command. The wrapper kind is the whole
  question: a bare `fn` gets the macro's blocking body, whose last acts are
  `kind.block(result, resolver)` then `return true`
  (`tauri-macros/src/command/wrapper.rs:425`/`:428`) — the verb runs
  synchronously **inside** the webview's IPC handler, which *is* the page's
  message loop: hit-testing (CSS hover), click dispatch and later IPC
  messages only run when the verb is done. One crawl chunk — 2/s pacing
  plus a `Retry-After` sleep on a 429, i.e. minutes in the worst case —
  charged to the page: that is the reported "hover won't highlight,
  clicks take several minutes". An `async fn` gets `body_async`: the
  future is handed to `respond_async_serialized`
  (`wrapper.rs:~372`), tauri's tokio-backed async manager
  (tokio `rt-multi-thread` is a required dependency of tauri 2.x, not
  a feature switch), and the handler returns immediately. Same argv,
  same `cli.rs` dispatch, same envelope — only the wrapper kind flips.
* Rejected for mid-crawl feedback: a progress channel from JS
  (`emit`/`event.listen`) — those are plugin commands and ACL-gated, so
  the ACL-by-construction exemption above would stop covering them
  (an `app:core:event` grant would be needed). Progress truth stays in
  the DB (ADR 0002): the UI polls the `status` verb once a second while
  a chunk is in flight and prints elapsed seconds, because
  `sync_state` moves only at chunk ends (`finish()` in src/sync.rs).

**Amended in v0.5.2 (the async command needs a thread hop for the verb body)**

* The async wrapper of v0.5.1 is necessary but not sufficient. reqwest's blocking client
  builds a **private tokio runtime on an internal thread** and parks the calling thread
  waiting for that runtime's startup handshake (`blocking/client.rs:1424` spawns
  `reqwest-internal-sync-runtime`, `:1428` builds `new_current_thread()`, `:1476` waits on
  the startup `oneshot`, whose body `thread::park()s` the caller at `blocking/wait.rs:58`).
  The async manager hands the verb body to a tokio worker, so the client cannot be built
  there: the audit saw the verb reach the DB-open point and stall at `build_client()` with an
  empty fixture access log, while `status` polls kept completing around it (the DB was never
  the lock).
* Chosen shape: `cli_call` stays `async` (the page still answers immediately), and
  `verb_on_plain_thread` (src/main.rs, crate root so both doors reach it) runs the verb body
  on a fresh `std` thread — the same runtime-free home the headless CLI runs its verbs in.
  `join` costs one tokio worker, which the webview never borrows. A verb that panics on that
  thread is converted to an `AppError`, so the envelope keeps its single shape.
* Rejected in v0.5.2: switching the client to async reqwest (an engine change this patch
  deliberately avoids — ADR 0004's one-code-path rule keeps CLI and GUI on the same client),
  and moving progress reporting into `emit`/`event.listen` (the ACL reason recorded above
  still applies).

## Consequences

**Gained**

* One code path from a click to a verb: a UI bug and a CLI bug are the same bug, and the
  CLI audits of ADR 0004 keep covering the app.
* Cancel is resumable by construction — the chunk boundary ends a `sync_state` write, so
  the next chunk continues from the stored cursor (ADR 0002).
* The audit channel is Rust's `gui:` log lines: nothing in `tauri`/`wry` forwards the webview
  console on Linux, which the launch audit confirmed by running it.

**Given up / traps**

* Cancel granularity is one chunk: at most 10 list pages or 25 detail records keep running
  after the click. Stopping at a request boundary is what keeps the DB consistent.
* `--fixture` is reachable from the GUI, so a fixture run is user-reachable. Honest, and the
  status card prints which DB file is in play; documented rather than hidden.
* If an app-side ACL manifest ever lands, `has_app_acl_manifest` flips to `true` and the
  missing grant for `cli_call` becomes a hard failure — the trap to know before adding one.
* `AppState` stays unused: there is no `state.rs`, deliberately (progress truth = DB).
