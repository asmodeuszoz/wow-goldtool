use crate::cli::{dispatch, exit_for, fragment, json_opt_seconds, object, parse_invocation, Invocation};
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
/// `async` on purpose (v0.5.1): the macro's blocking wrapper runs the verb
/// INSIDE the webview's IPC handler - its last acts are `kind.block(result,
/// resolver)` and `return true` (tauri-macros/src/command/wrapper.rs:425/:428)
/// - so a bare fn charges one crawl chunk (2/s pacing, plus any Retry-After
/// wait on a 429) to the whole page: hover stops hit-testing, clicks queue
/// behind the verb for as long as the crawl runs. An `async fn` routes to
/// `respond_async_serialized` instead (wrapper.rs:~372): tauri hands the verb
/// to its tokio-backed async manager and the handler returns true right away,
/// so the crawl runs while the page stays interactive. Same argv, same
/// cli::dispatch: only the wrapper kind flips (sync -> async).
#[tauri::command]
pub async fn cli_call(argv: Vec<String>) -> String {
    let inv: Invocation = parse_invocation(argv);
    eprintln!("gui: ipc verb {} ({} words)", inv.verb, inv.words.len());
    let result: AppResult<String> = dispatch(inv);
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
