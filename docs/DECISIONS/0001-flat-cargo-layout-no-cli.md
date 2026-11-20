# ADR 0001 · Flat Cargo layout, no Tauri CLI

- **Status:** accepted
- **Date:** 2026-11-19
- **Discussion:** [docs/DEVLOG.md](../DEVLOG.md) · commits tagged `v0.1.0`

## Context

The reference Tauri 2.0 project layout is:

```
src-tauri/Cargo.toml + src-tauri/tauri.conf.json + tauri.conf.json > build > frontendDist
```

and the intended workflow is `tauri dev` / `tauri build` from `@tauri-apps/cli`, which
generates a config context and an **asset map** (`_assets.gen.rs`, a compile-time `phf`
map) before invoking `cargo`.

Constraints that collide with that workflow:

1. The project is a **RustRover** project: it must compile and run from the IDE with
   plain `cargo build` / `cargo run` on the default run configuration, with nothing
   generated beforehand.
2. The repo owner wants a normal GitHub-style history (buildable commits, tags), so a
   build that silently depends on a CLI step that is not in the repo is unacceptable.
3. A Node/`npm` toolchain and internet access at build time are not assumptions we want.

## Options considered

| Option | Trade-off |
| ------ | --------- |
| Keep `src-tauri/` + require `@tauri-apps/cli` | Canonical, but RustRover's default build fails until the CLI has generated the context/assets; adds Node to the build |
| `src-tauri/` + hand-written `_assets.gen.rs` checked in | Works with plain `cargo build`, but the file is machine-generated noise in the repo and drifts from the real config |
| **Flat single-Crate layout at the repo root** (`Cargo.toml`, `src/`, `ui/`, `tauri.conf.json`) | `tauri::generate_context!()` reads `tauri.conf.json` relative to `CARGO_MANIFEST_DIR` **at compile time**, and `tauri-codegen` reads `build.frontendDist` **from the filesystem during compilation** and embeds the assets itself. No codegen step, no CLI, no Node |
| Pure-Rust `Window` builder API (`Window::new()…create()`) | Also CLI-free, but those builder types are `unstable_struct!`-gated behind the `unstable` Cargo feature — an explicitly unstable public API we would be pinning to a minor version |

## Decision

Take the **flat layout**. Concretely:

* `Cargo.toml` at the repo root, single package, `src/` as the module root.
* `tauri.conf.json` at the repo root — `build.frontendDist = "./ui"`, one window entry,
  `withGlobalTauri: true`, `security.csp: null` (the UI is fully local and offline).
* `ui/index.html`, `ui/app.css`, `ui/app.js` are static files embedded into the binary at
  compile time; `WebviewUrl` defaults to `App("index.html")`, i.e. `tauri://localhost`.
* Commands are registered with the stable core API: `#[tauri::command]` +
  `Builder::invoke_handler(tauri::generate_handler![…])` + `Builder::manage(…)`.
* `bundle.active = false`: the app ships as a plain Cargo executable. Installers remain a
  roadmap item.

## Amendment · v0.4.0 (2026-11-19)

Building the shell forced three more facts, all consistent with the decision above:

* `tauri-build` is a **build**-dependency and `build.rs` calls `tauri_build::build()`. That is
  cargo's own build script, not the Tauri CLI: ADR 0001's "no CLI" is about `@tauri-apps/cli`
  plus Node. The call is what writes the global-API script list, the ACL manifests and the
  capabilities file into `OUT_DIR` for `generate_context!` to read back — without it
  `withGlobalTauri` is inert and `window.__TAURI__` never exists (see ADR 0005).
* `tauri-build` writes `gen/schemas/*.json` next to `Cargo.toml` at build time. The flat layout
  has no `src-tauri/` to hide such artifacts in, so `.gitignore` carries `/gen/`.
* `bundle.icon` is read at compile time (`CachedIcon`), so `icons/icon.png` must exist before
  the first GUI build. `tools/gen_icon.py` generates it deterministically and the PNG is
  committed, so a fresh clone still builds.

The shipped `tauri.conf.json` also carries no `app.security` section: the UI is fully local, so
no CSP header and no ACL manifest are emitted at all (which is why app commands are reachable —
ADR 0005 spells out the gate at `tauri/src/webview/mod.rs:2080`).

## Consequences

**Gained**

* `cargo build`/`cargo run` in RustRover builds and runs the real app — no prerequisite
  step, no external toolchain, repo state alone determines the build.
* `cargo test`, `cargo run --release` and CI all exercise the same code paths.
* Frontend editing stays trivial: three files, no bundler, no dev server.

**Given up**

* `tauri dev` hot reload (irrelevant for three static files) and `tauri build` installers
  (`.deb`/`.AppImage`/`.rpm`) are not produced by this layout. Documented as future work;
  if packaging is needed later, the CLI can be reintroduced with a `src-tauri`-compatible
  shim rather than by moving the whole project.
* The `bundle.*` metadata (icons, categories) stays declarative-only until packaging lands.

## Pinning

`tauri = "2.12.7"` — the API surface we rely on (`generate_context!`, `generate_handler!`,
`invoke_handler`, `manage`, `State`) is stable inside that minor line, and pinning means a
future minor release cannot silently break the app between two of our commits.
