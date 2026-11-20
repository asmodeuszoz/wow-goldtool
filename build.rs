//! Cargo runs this before compiling the app: `tauri_build::build()` writes the global-API
//! script list (which is what makes `window.__TAURI__` exist in the webview), the ACL
//! manifests and the embedded-asset cache into `$OUT_DIR`, and `generate_context!` reads
//! those back. Without this call `withGlobalTauri` silently has no effect - the empty
//! build.rs we first tried only fixed the `OUT_DIR` codegen needs. ADR 0001 keeps its
//! meaning: this is cargo's own build script, not the Tauri CLI, and it reads
//! tauri.conf.json plus build.frontendDist from the repo root.

use tauri_build::build;

fn main() {
    build();
}
