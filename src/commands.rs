use crate::cli::{exit_for, fragment, json_opt_seconds, object, parse_invocation, Invocation};
use crate::errors::{AppError, AppResult, ErrorKind};

use serde_json::to_string;

/// A String encoded as a JSON string value: the envelope needs quoted strings, and
/// serde is what keeps quotes and escapes out of hand-written JSON.
pub fn json_quoted(text: String) -> String {
    match to_string(&text) {
        Err(_) => return "\"\"".to_string(),
        Ok(encoded) => return encoded,
    }
}

/// The error kind as the name the UI shows: the same words docs/DECISIONS/0004
/// defines for the exit codes, as strings.
pub fn kind_name(problem: AppError) -> String {
    match problem.kind {
        ErrorKind::Usage => return "usage".to_string(),
        ErrorKind::Network => return "network".to_string(),
        ErrorKind::RateLimited => return "rate-limited".to_string(),
        ErrorKind::Db => return "database".to_string(),
        ErrorKind::Decode => return "decode".to_string(),
        ErrorKind::NotFound => return "not-found".to_string(),
    }
}

/// The GUI's one command: argv in (exactly the words `cargo run --` would take),
/// envelope JSON out: {"ok": true, "result": <verb json>} or
/// {"ok": false, "error": {"kind", "exit", "message", "retry_after"}}.
/// `async` on purpose (v0.5.1), plus a plain-thread verb hop (v0.5.2):
/// the macro's blocking wrapper runs a bare fn's verb INSIDE the webview's
/// IPC handler (its last acts: `kind.block(result, resolver)` then `return
/// true`, tauri-macros wrapper.rs:425/:428), and that handler is the page's
/// message loop - one crawl chunk then parks hover hit-testing, clicks and
/// later IPC behind it: the reported "hover won't highlight, clicks take
/// minutes". An `async fn` routes to `respond_async_serialized` (wrapper.rs
/// :~372): the handler returns immediately and the page keeps running.
/// The VERB BODY, though, must not run on a tokio manager thread either:
/// reqwest's blocking client carries a private tokio runtime, and nested
/// runtimes deadlock - the v0.5.2 audit showed a GUI sync verb stalled at
/// build_client (zero fixture hits logged, while status polls between it
/// completed fine: the DB never was the lock). So the verb runs on a fresh
/// std thread: the same runtime-free home as the headless CLI. `join` costs
/// one tokio worker; the webview has its own thread and never sees it.
#[tauri::command]
pub async fn cli_call(argv: Vec<String>) -> String {
    let inv: Invocation = parse_invocation(argv);
    eprintln!("gui: ipc verb {} ({} words)", inv.verb, inv.words.len());
    // The verb itself runs on a fresh plain thread (main.rs helper): reqwest's
    // blocking client would otherwise be built on a tokio worker of tauri's
    // async manager and deadlock there (the v0.5.2 audit: verb reached the DB
    // open, then stalled at build_client, zero fixture hits, while later
    // status polls kept completing).
    let result: AppResult<String> = crate::verb_on_plain_thread(inv);
    match result {
        Ok(text) => {
            let mut parts: Vec<String> = [].to_vec();
            parts.push(fragment("ok".to_string(), "true".to_string()));
            parts.push(fragment("result".to_string(), text));
            return object(parts);
        },
        Err(problem) => {
            let mut inner: Vec<String> = [].to_vec();
            inner.push(fragment("kind".to_string(), json_quoted(kind_name(problem.clone()))));
            inner.push(fragment("exit".to_string(), format!("{}", exit_for(problem.clone()))));
            inner.push(fragment("message".to_string(), json_quoted(problem.to_text())));
            inner.push(fragment("retry_after".to_string(), json_opt_seconds(problem.retry_after.clone())));
            let mut parts: Vec<String> = [].to_vec();
            parts.push(fragment("ok".to_string(), "false".to_string()));
            parts.push(fragment("error".to_string(), object(inner)));
            return object(parts);
        },
    }
}
