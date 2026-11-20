//! Tauri GUI shell. ADR 0001: `generate_context!` reads tauri.conf.json relative to
//! CARGO_MANIFEST_DIR at compile time, so plain cargo embeds the config and every
//! file in build.frontendDist (ui/) - no Tauri CLI, no Node. The window itself is
//! declared in the config (app.windows), and the webview talks back through the
//! single command in commands.rs. There is no state.rs: the progress truth lives in
//! the DB's sync_state table (ADR 0002), which both doors already read, and
//! cancelling works at chunk granularity from the UI loop.

use tauri::Builder;

/// Builds the configured application and blocks until its window is closed (the
/// docs call this a shorthand for build() + App.run()). Tauri's own error type is
/// not an AppError, so the CLI's 2/3/4 kind-map does not apply: startup failures
/// print and exit 1, closed-then-clean exits exit 0.
pub fn run_gui() -> i32 {
    eprintln!("gui: wow-goldtool shell starting; window declared by tauri.conf.json");
    let outcome = Builder::default()
        .on_page_load(|view, payload| {
            eprintln!("gui: the webview asked for {} (window {})", payload.url(), view.label());
        })
        .invoke_handler(tauri::generate_handler![crate::commands::cli_call])
        .run(tauri::generate_context!());
    match outcome {
        Ok(_) => return 0,
        Err(problem) => {
            eprintln!("gui: the shell failed to start: {}", problem);
            return 1;
        },
    }
}
