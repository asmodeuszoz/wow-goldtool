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

pub fn main() {
    let words: Vec<String> = words_of_argv();
    if words.is_empty() {
        // The Tauri shell lands here in v0.4.0; until then this explains itself
        // instead of pretending the GUI works.
        println!("wow-goldtool: no verb given. Run cargo run -- status to inspect the data layer, or see docs/DEVELOPMENT.md.");
        return;
    }
    // exit() rather than a numeric main: this std models the exit status as its
    // own type, and asking for a plain i32 here is rejected.
    exit(cli::run(words));
}
