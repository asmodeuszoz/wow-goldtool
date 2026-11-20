// The entry point dispatches to the headless CLI or the Tauri GUI: a verb on
// argv means the CLI, no verb at all means the app (docs/DECISIONS/0004). One
// binary, one code path, so CI and the GUI cannot drift apart.

mod errors;
mod models;
mod wowdb;
mod db;
mod query;
mod stats;
mod sync;
mod cli;
mod commands;
mod gui;

use std::process::exit;
use std::env::args;

/// The words after the program name: empty means the GUI was asked for.
pub fn words_of_argv() -> Vec<String> {
    let mut out: Vec<String> = [].to_vec();
    for word in args() {
        out.push(word);
    }
    if out.is_empty() {
        return out;
    }
    out.remove(0);
    return out;
}

/// The GUI's verb hop (v0.5.2): one verb body runs on a plain std thread.
/// reqwest's blocking client carries a private tokio runtime, and building
/// it on a thread that belongs to tauri's async-manager tokio runtime
/// deadlocks - the v0.5.2 audit saw a sync verb stall at build_client with
/// zero fixture hits while the webview kept polling fine (the DB never was
/// the lock). A fresh std thread is runtime-free: exactly the home of the
/// headless CLI. The join occupies the tokio worker that hosts the async
/// command while the verb runs; the webview has its own thread and stays
/// interactive. The helper sits at the crate root because that is the one
/// place both doors can reach it from without an import cycle: cli_call
/// (commands.rs) calls it, main() does not need it.
pub fn verb_on_plain_thread(inv: cli::Invocation) -> errors::AppResult<String> {
    let verb_thread = std::thread::spawn(|| {
        return cli::dispatch(inv);
    });
    return match verb_thread.join() {
        Err(_) => {
            eprintln!("gui: the verb thread panicked mid-verb");
            return Err(errors::AppError::decode("the verb's thread panicked mid-verb: see the log above".to_string()));
        },
        Ok(found) => found,
    };
}

pub fn main() {
    let words: Vec<String> = words_of_argv();
    if words.is_empty() {
        // v0.4.0: the GUI shell. The window comes from tauri.conf.json (app.windows)
        // and run_gui blocks until the window closes (docs/DECISIONS/0001).
        exit(gui::run_gui());
    }
    // exit() rather than a numeric main: this std models the exit status as its
    // own type, and asking for a plain i32 here is rejected.
    exit(cli::run(words));
}
