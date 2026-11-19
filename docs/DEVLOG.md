# Development log

Dated, reverse-chronological notes on what was tried, measured and decided while building
WoW Goldtool. The changelog records *what* shipped; this file records *why*.

---

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
