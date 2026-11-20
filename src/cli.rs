//! Headless subcommands: the same functions the GUI calls, so CI and the app's own
//! integration test exercise one code path (docs/DECISIONS/0004).
//!
//! Output contract, because CI parses it: JSON on stdout only, human text on stderr,
//! and the exit code is the kind of failure (0 ok · 2 usage · 3 network or rate limit
//! · 4 database). Nothing here may print progress to stdout: it would break the JSON.

use crate::errors::AppError;
use crate::errors::AppResult;
use crate::errors::ErrorKind;

use crate::db::{count_items, db_path, detail_etag_of, open_db_at, raw_json_of, sync_state_get, upsert_detail, watch_add, watch_drop, watch_list, DbTarget, SyncState, unix_now};

use crate::query::{Filter, Hit, empty_filter, search, count_matching};
use crate::stats::{coverage, by_quality, cheapest, priciest, Coverage, QualityStat, Extreme};
use crate::sync::{sync_detail, sync_list, CancelFlag, SyncReport};
use crate::wowdb::{build_client, get_json, item_url, ApiResponse, HttpKind, RateBudget, classify, Client, ENV_FIXTURE_KEY};
use crate::models::{ItemDetail};

use rusqlite::Connection;

use serde_json::from_str;
use serde_json::to_string;

use std::env::set_var;

/// What the CLI can answer. Keeping the list here means an unknown verb becomes a
/// usage error that names what the user actually typed.
pub fn usage_text() -> String {
    return "usage: status | sync --stage list|detail [--budget N] | search TEXT [--quality X --type Y --max-level N --priced --sort price|name|quality|level|id --limit N --offset N] | stats [--limit N] | item ID | watch add ID [note] | watch list | watch drop ID | status [--db PATH] [--fixture URL]".to_string();
}

/// The verbs of one invocation plus its flags. Unknown flags are kept rather than
/// dropped, so the usage error can quote the typo.
pub struct Invocation {
    pub verb: String,
    pub words: Vec<String>,
    pub stage: String,
    pub sort: String,
    pub quality: Option<String>,
    pub type_name: Option<String>,
    pub max_level: Option<i64>,
    pub priced: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub budget: Option<i64>,
    pub db: Option<String>,
    pub fixture: Option<String>,
    pub bad: String,
}

/// Flags that need a value after them.
pub fn takes_value(flag: String) -> bool {
    if flag == "--db" { return true }
    if flag == "--fixture" { return true }
    if flag == "--stage" { return true }
    if flag == "--budget" { return true }
    if flag == "--limit" { return true }
    if flag == "--offset" { return true }
    if flag == "--sort" { return true }
    if flag == "--quality" { return true }
    if flag == "--type" { return true }
    if flag == "--max-level" { return true }
    return false;
}

/// Flags that stand alone.
pub fn bare_flag(flag: String) -> bool {
    if flag == "--priced" { return true }
    return false;
}

/// A CLI number: a bare whole number, because these values index pages, ids and
/// limits. Anything else is a usage error, never a silent 0.
pub fn as_int(token: String) -> Option<i64> {
    match token.parse() {
        Err(_) => return None,
        Ok(value) => return Some(value),
    }
}

/// Word by word, the first non-flag word is the verb and the rest are positionals.
pub fn parse_invocation(args: Vec<String>) -> Invocation {
    let mut out: Invocation = Invocation {
        verb: "".to_string(),
        words: [].to_vec(),
        stage: "list".to_string(),
        sort: "price".to_string(),
        quality: None,
        type_name: None,
        max_level: None,
        priced: false,
        limit: None,
        offset: None,
        budget: None,
        db: None,
        fixture: None,
        bad: "".to_string(),
    };
    let mut index: usize = 0;
    while index < args.len() {
        let token: String = args[index].clone();
        if token.starts_with("--") {
            if bare_flag(token.clone()) {
                out.priced = true;
                index = index + 1;
                continue;
            }
            if !takes_value(token.clone()) {
                out.bad = format!("unknown flag {} (run help to see the verbs)", token);
                return out;
            }
            if index + 1 >= args.len() {
                out.bad = format!("{} needs a value", token);
                return out;
            }
            let value: String = args[index + 1].clone();
            if token == "--db" {
                out.db = Some(value.clone());
            }
            if token == "--fixture" {
                out.fixture = Some(value.clone());
            }
            if token == "--stage" {
                out.stage = value.clone();
            }
            if token == "--sort" {
                out.sort = value.clone();
            }
            if token == "--quality" {
                out.quality = Some(value.clone());
            }
            if token == "--type" {
                out.type_name = Some(value.clone());
            }
            if token == "--budget" || token == "--limit" || token == "--offset" || token == "--max-level" {
                match as_int(value.clone()) {
                    None => {
                        out.bad = format!("{} wants a whole number, got {}", token, value);
                        return out;
                    },
                    Some(number) => {
                        if token == "--budget" {
                            out.budget = Some(number);
                        }
                        if token == "--limit" {
                            out.limit = Some(number);
                        }
                        if token == "--offset" {
                            out.offset = Some(number);
                        }
                        if token == "--max-level" {
                            out.max_level = Some(number);
                        }
                    },
                }
            }
            index = index + 2;
            continue;
        }
        if out.verb == "" {
            out.verb = token;
        } else {
            out.words.push(token);
        }
        index = index + 1;
    }
    return out;
}

/// Where the DB this invocation points at, and why that path was chosen: the CLI
/// prints both, so a user always knows which file they are looking at.
pub fn describe(inv: &Invocation) -> DbTarget {
    match inv.db.clone() {
        Some(path) => return DbTarget { path: path.clone(), source: "cli --db flag".to_string() },
        None => return db_path(None),
    }
}

/// Attach to the DB this invocation points at, schema rules included.
pub fn open_at(inv: &Invocation) -> AppResult<Connection> {
    let target: DbTarget = describe(inv);
    return open_db_at(target.path.clone());
}

/// The exit-code contract (docs/DECISIONS/0004): the kind of failure decides the
/// code, so CI can tell a rate limit apart from a broken DB file.
pub fn exit_for(problem: AppError) -> i32 {
    match problem.kind {
        ErrorKind::Usage => return 2,
        ErrorKind::Network => return 3,
        ErrorKind::RateLimited => return 3,
        _ => return 4,
    }
}

/// One already-encoded value for the object the commands assemble. Encoding through
/// serde is what keeps quotes and escaping out of hand-written JSON.
pub fn json_db_target(target: DbTarget) -> String {
    match to_string(&target) {
        Err(_) => return "{}".to_string(),
        Ok(text) => return text,
    }
}

pub fn json_state(state: SyncState) -> String {
    match to_string(&state) {
        Err(_) => return "{}".to_string(),
        Ok(text) => return text,
    }
}

pub fn json_coverage(found: Coverage) -> String {
    match to_string(&found) {
        Err(_) => return "{}".to_string(),
        Ok(text) => return text,
    }
}

pub fn json_rows(rows: Vec<QualityStat>) -> String {
    match to_string(&rows) {
        Err(_) => return "[]".to_string(),
        Ok(text) => return text,
    }
}

pub fn json_extremes(rows: Vec<Extreme>) -> String {
    match to_string(&rows) {
        Err(_) => return "[]".to_string(),
        Ok(text) => return text,
    }
}

pub fn json_hits(rows: Vec<Hit>) -> String {
    match to_string(&rows) {
        Err(_) => return "[]".to_string(),
        Ok(text) => return text,
    }
}

pub fn json_report(report: SyncReport) -> String {
    match to_string(&report) {
        Err(_) => return "{}".to_string(),
        Ok(text) => return text,
    }
}

pub fn json_item(detail: ItemDetail) -> String {
    match to_string(&detail) {
        Err(_) => return "{}".to_string(),
        Ok(text) => return text,
    }
}

/// A list of ids as a JSON array: `watch list` prints this, the UI reads it.
pub fn json_ids(ids: Vec<i64>) -> String {
    match to_string(&ids) {
        Err(_) => return "[]".to_string(),
        Ok(text) => return text,
    }
}

/// An optional number as JSON: absent means `null`, never a made-up 0, because "the
/// API did not say" and "the API said zero" are different facts.
pub fn json_opt_seconds(opt: Option<u32>) -> String {
    match opt {
        None => return "null".to_string(),
        Some(value) => return format!("{}", value),
    }
}

/// The item ids of a watch command: everything after the verb, all numeric. A
/// non-numeric token is a usage error naming the token, never a silent drop.
pub fn collect_ids(words: Vec<String>) -> AppResult<Vec<i64>> {
    let mut out: Vec<i64> = [].to_vec();
    let mut index: usize = 1;
    while index < words.len() {
        let token: String = words[index].clone();
        match as_int(token.clone()) {
            None => return Err(AppError::usage(format!("watch wants numeric item ids, got {}", token))),
            Some(number) => out.push(number),
        }
        index = index + 1;
    }
    return Ok(out);
}

/// The filter a search actually applied, echoed back so a script can log what it
/// asked for and the UI can reuse the same shape.
pub fn json_filter(filter: Filter) -> String {
    match to_string(&filter) {
        Err(_) => return "{}".to_string(),
        Ok(text) => return text,
    }
}

/// `key: value` fragments joined into one JSON object.
pub fn object(parts: Vec<String>) -> String {
    return format!("{}{}{}", "{", parts.join(", "), "}");
}

pub fn fragment(key: String, value: String) -> String {
    return format!("{}{}", format!("{}{}{}", "\"", key, "\": "), value);
}

/// status: where the DB is, how complete it is and what the last crawl did.
pub fn cmd_status(inv: &Invocation) -> AppResult<String> {
    let target: DbTarget = describe(inv);
    let conn: Connection = match open_db_at(target.path.clone()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let seen: i64 = match count_items(&conn) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let found: Coverage = match coverage(&conn) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let list_state: SyncState = match sync_state_get(&conn, "list".to_string()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let detail_state: SyncState = match sync_state_get(&conn, "detail".to_string()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("db".to_string(), json_db_target(target)));
    parts.push(fragment("items".to_string(), format!("{}", seen)));
    parts.push(fragment("coverage".to_string(), json_coverage(found)));
    parts.push(fragment("list".to_string(), json_state(list_state)));
    parts.push(fragment("detail".to_string(), json_state(detail_state)));
    return Ok(object(parts));
}

/// sync: run one crawl stage. `--budget` bounds the run so CI and a test fixture can
/// crawl cheaply; the report says where the crawl stopped either way.
pub fn cmd_sync(inv: &Invocation) -> AppResult<String> {
    if inv.stage != "list" {
        if inv.stage != "detail" {
            return Err(AppError::usage(format!("sync wants --stage list or --stage detail, got {}", inv.stage)));
        }
    }
    let conn: Connection = match open_at(inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let client: Client = match build_client() {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let budget: RateBudget = RateBudget::new();
    let mut cancel: CancelFlag = CancelFlag::make();
    if inv.stage == "list" {
        eprintln!("crawling the item list: about 193 pages, one request per page");
        let report: SyncReport = match sync_list(client.clone(), &conn, budget, inv.budget, &cancel) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        eprintln!("list stage: {} pages, {} records seen, {} written", report.pages, report.items, report.written);
        return Ok(json_report(report));
    }
    eprintln!("fetching item details: this is the slow stage, about two records per second");
    let report: SyncReport = match sync_detail(client.clone(), &conn, budget, inv.budget, &cancel) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    eprintln!("detail stage: {} records fetched", report.items);
    return Ok(json_report(report));
}

/// search: a local query over the SQLite DB, with the same WHERE chain used for the
/// count, so the UI can show "showing 40 of 1250" without two drifting queries.
pub fn filter_for(inv: &Invocation) -> Filter {
    let mut out: Filter = empty_filter();
    if inv.words.len() > 0 {
        out.text = Some(inv.words[0].clone());
    }
    out.quality = inv.quality.clone();
    out.item_type = inv.type_name.clone();
    out.max_required_level = inv.max_level;
    out.priced_only = inv.priced;
    out.sort = inv.sort.clone();
    out.limit = inv.limit;
    out.offset = inv.offset;
    return out;
}

pub fn cmd_search(inv: &Invocation) -> AppResult<String> {
    let conn: Connection = match open_at(inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let filter: Filter = filter_for(inv);
    let applied: String = json_filter(filter.clone());
    let total: i64 = match count_matching(&conn, filter.clone()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let hits: Vec<Hit> = match search(&conn, filter) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("filter".to_string(), applied));
    parts.push(fragment("total".to_string(), format!("{}", total)));
    parts.push(fragment("hits".to_string(), json_hits(hits)));
    return Ok(object(parts));
}

/// stats: coverage plus the price facets the UI charts.
pub fn cmd_stats(inv: &Invocation) -> AppResult<String> {
    let conn: Connection = match open_at(inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let found: Coverage = match coverage(&conn) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let rows: Vec<QualityStat> = match by_quality(&conn) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let limit: i64 = match inv.limit {
        None => 10,
        Some(value) => value,
    };
    let cheap: Vec<Extreme> = match cheapest(&conn, limit) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let rich: Vec<Extreme> = match priciest(&conn, limit) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("coverage".to_string(), json_coverage(found)));
    parts.push(fragment("by_quality".to_string(), json_rows(rows)));
    parts.push(fragment("cheapest".to_string(), json_extremes(cheap)));
    parts.push(fragment("priciest".to_string(), json_extremes(rich)));
    parts.push(fragment("limit".to_string(), format!("{}", limit)));
    return Ok(object(parts));
}

/// item: one live lookup. A copy already in the DB is revalidated, not re-downloaded:
/// the stored detail ETag goes out as If-None-Match, a 304 comes back with no body,
/// and the record answered from raw_json on disk (docs/API.md, docs/DATABASE.md). A
/// fresh 200 is written through the same upsert the sync stage uses, so the next run
/// can revalidate it.
pub fn cmd_item(inv: &Invocation) -> AppResult<String> {
    if inv.words.len() < 1 {
        return Err(AppError::usage("item wants an item id, for example: item 25".to_string()));
    }
    let id: Option<i64> = as_int(inv.words[0].clone());
    match id {
        None => return Err(AppError::usage(format!("item wants a numeric id, got {}", inv.words[0]))),
        Some(number) => {
            let conn: Connection = match open_at(inv) {
                Err(problem) => return Err(problem),
                Ok(value) => value,
            };
            let stored_etag: Option<String> = match detail_etag_of(&conn, number) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            let send_etag: String = match stored_etag {
                None => "".to_string(),
                Some(text) => text,
            };
            let client: Client = match build_client() {
                Err(problem) => return Err(problem),
                Ok(value) => value,
            };
            let response: ApiResponse = match get_json(client, item_url(number), send_etag) {
                Err(problem) => return Err(problem),
                Ok(value) => value,
            };
            match classify(response.status) {
                HttpKind::Success => {},
                HttpKind::NotModified => {},
                HttpKind::RateLimited => return Err(AppError::rate_limited("the API asked us to back off".to_string(), response.retry_after)),
                HttpKind::Failed => return Err(AppError::http(response.status, response.body)),
            }
            let payload: String = match response.status {
                304 => match raw_json_of(&conn, number) {
                    Err(problem) => return Err(problem),
                    Ok(None) => return Err(AppError::decode(format!("item {} answered 304 but the DB has no stored copy of it", number))),
                    Ok(Some(text)) => text,
                },
                _ => response.body.clone(),
            };
            let detail: ItemDetail = match from_str(&payload) {
                Err(_) => return Err(AppError::decode(format!("item {} is not the JSON we expect", number))),
                Ok(value) => value,
            };
            if response.status == 200 {
                match upsert_detail(&conn, detail.clone(), response.etag.clone(), response.body.clone(), unix_now()) {
                    Err(problem) => return Err(problem),
                    Ok(_) => {},
                }
            }
            let reused: bool = response.status == 304;
            let mut parts: Vec<String> = [].to_vec();
            parts.push(fragment("status".to_string(), format!("{}", response.status)));
            parts.push(fragment("retry_after".to_string(), json_opt_seconds(response.retry_after)));
            parts.push(fragment("reused".to_string(), match reused { true => "true".to_string(), false => "false".to_string() }));
            parts.push(fragment("item".to_string(), json_item(detail)));
            return Ok(object(parts));
        },
    }
}

/// watch: the ids the user cares about, so the GUI and later sync runs can pin them.
pub fn cmd_watch(inv: &Invocation) -> AppResult<String> {
    if inv.words.len() < 1 {
        return Err(AppError::usage("watch wants add ID [note], list or drop ID".to_string()));
    }
    let conn: Connection = match open_at(inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let action: String = inv.words[0].clone();
    if action == "list" {
        let ids: Vec<i64> = match watch_list(&conn) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        return Ok(json_ids(ids));
    }
    let ids: Vec<i64> = match collect_ids(inv.words.clone()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    if ids.is_empty() {
        return Err(AppError::usage(format!("watch {} wants at least one item id", action)));
    }
    let encoded: String = json_ids(ids.clone());
    if action == "add" {
        let now: i64 = unix_now();
        for one in ids {
            match watch_add(&conn, one, None) {
                Err(problem) => return Err(problem),
                Ok(_) => {},
            }
        }
        let mut parts: Vec<String> = [].to_vec();
        parts.push(fragment("added".to_string(), encoded));
        parts.push(fragment("at".to_string(), format!("{}", now)));
        return Ok(object(parts));
    }
    if action == "drop" {
        for one in ids {
            match watch_drop(&conn, one) {
                Err(problem) => return Err(problem),
                Ok(_) => {},
            }
        }
        return Ok(encoded);
    }
    return Err(AppError::usage(format!("watch wants add, list or drop, got {}", action)));
}

/// One verb's result through the same output guard: JSON on stdout, the message
/// and the reason on stderr, and the kind of failure as the exit code.
pub fn guard(body: AppResult<String>) -> i32 {
    match body {
        Err(problem) => {
            eprintln!("{}", problem.to_text());
            return exit_for(problem);
        },
        Ok(text) => {
            println!("{}", text);
            return 0;
        },
    }
}

/// Every verb through the same guard: JSON out, message and code to stderr.
pub fn run(args: Vec<String>) -> i32 {
    let inv: Invocation = parse_invocation(args);
    match inv.fixture.clone() {
        Some(url) => unsafe { set_var(ENV_FIXTURE_KEY, url); },
        None => {},
    }
    if inv.bad != "" {
        eprintln!("{}", inv.bad);
        eprintln!("{}", usage_text());
        return 2;
    }
    if inv.verb == "" {
        eprintln!("{}", usage_text());
        return 2;
    }
    if inv.verb == "status" {
        return guard(cmd_status(&inv));
    }
    if inv.verb == "sync" {
        return guard(cmd_sync(&inv));
    }
    if inv.verb == "search" {
        return guard(cmd_search(&inv));
    }
    if inv.verb == "stats" {
        return guard(cmd_stats(&inv));
    }
    if inv.verb == "item" {
        return guard(cmd_item(&inv));
    }
    if inv.verb == "watch" {
        return guard(cmd_watch(&inv));
    }
    if inv.verb == "help" {
        eprintln!("{}", usage_text());
        return 0;
    }
    return guard(Err(AppError::usage(format!("unknown verb {} (run help to see the verbs)", inv.verb))));
}
