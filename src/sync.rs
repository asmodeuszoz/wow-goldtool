//! The sync engine: the two crawl stages, resume bookkeeping and progress.
//!
//! This is the only module where HTTP and SQLite meet (docs/DEVELOPMENT.md house
//! rules): `wowdb.rs` never touches the DB and `db.rs` never touches the network.
//!
//! Pacing is by construction, never by luck (docs/API.md): a burst may run freely
//! only while it stays under the headroom the API reports, past that the schedule is
//! two requests per second, and a 429 pauses the loop for the documented Retry-After
//! without failing the run, retried in place up to three times.
//!
//! Reuse across runs is conditional GETs. Each list page keeps its ETag plus
//! continuation hints (has_more, count) in the meta table, so a re-crawl sends
//! If-None-Match and pays 304 plus a DB read instead of a fresh payload: a 304 still
//! costs one request (the burst charges it), what it saves is bandwidth and DB churn.
//! A stopped or cancelled list run resumes at the page after the one sync_state says
//! it last fetched; a finished one re-crawls from page 1, where revalidation makes
//! that cheap. Every loop checks a cancel flag: the CLI hands it a fresh one today
//! (Ctrl-C leaves the DB consistent and the run resumable), the GUI raises it at
//! v0.4.0 through AppState.

use crate::errors::AppError;
use crate::errors::AppResult;

use serde::Serialize;

use crate::wowdb::{Client, ApiResponse, HttpKind, RateBudget, classify, get_json, item_url, page_url, FALLBACK_RETRY_MS};

use crate::models::{ItemDetail, ListPage};

use crate::db::{detail_etag_of, mark_missing_at, meta_get, meta_set, sync_state_get, sync_state_set, unix_now, upsert_detail, upsert_summary, SyncState};

use crate::stats::{pending_ids, coverage, Coverage};

use rusqlite::Connection;

use serde_json::from_str;

use std::thread::sleep;
use std::time::Duration;

/// How many records one list page asks for. The API clamps larger values anyway
/// (docs/API.md), and asking for more than the clamp would waste budget.
pub const PER_PAGE: u16 = 100;

/// Hard page ceiling: the API refuses pages beyond it (docs/API.md), and the crawl
/// must terminate even if the catalogue grows.
pub const PAGE_CEILING: i64 = 200;

/// What one crawl run did: printed as JSON by the CLI and shown as the sync banner
/// in the UI.
#[derive(Serialize, Clone)]
pub struct SyncReport {
    pub stage: String,
    pub requests: i64,
    pub pages: i64,
    pub items: i64,
    pub written: i64,
    pub unchanged: i64,
    pub rate_limited: i64,
    pub missing: i64,
    pub seconds: i64,
    pub resumed_from: Option<i64>,
    pub complete: bool,
    pub message: String,
}

impl SyncReport {
    pub fn make(stage: String) -> SyncReport {
        return SyncReport {
            stage: stage,
            requests: 0,
            pages: 0,
            items: 0,
            written: 0,
            unchanged: 0,
            rate_limited: 0,
            missing: 0,
            seconds: 0,
            resumed_from: None,
            complete: false,
            message: "".to_string(),
        };
    }
}

/// Record the run in `sync_state` so the next run and the UI both know where the
/// catalogue stopped. `total` is the catalogue size the API reported, not a guess.
pub fn record(conn: &Connection, stage: String, status: String, page: i64, done: i64, total: i64, message: String) -> AppResult<()> {
    let state: SyncState = SyncState {
        kind: stage,
        status: status,
        page: Some(page),
        done: Some(done),
        total: Some(total),
        message: Some(message),
        updated_at: Some(unix_now()),
        error: None,
    };
    match sync_state_set(conn, state) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(()),
    }
}

/// Stamp the elapsed time into the report and write the `sync_state` row in one go.
/// `cursor` is where the crawl stands in absolute page numbers: `pages` counts this
/// run only, and a resumed run started past page 1, so the two are not the same.
pub fn finish(conn: &Connection, report: SyncReport, started: i64, cursor: i64, total: i64, status: String) -> AppResult<SyncReport> {
    let mut out: SyncReport = report;
    out.seconds = unix_now() - started;
    let stage: String = out.stage.clone();
    let note: String = out.message.clone();
    match record(conn, stage, status, cursor, out.items, total, note) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(out),
    }
}

/// What the checked-missing third state (v0.5.3) adds to a run's message: those
/// ids cost ONE request per crawl, not one per run, so a catalogue with removed
/// records still finishes.
pub fn missing_note(missing: i64) -> String {
    if missing == 0 {
        return "".to_string();
    }
    return format!("; {} answered 404 and are checked as removed", missing);
}

/// Turn one list-page body into a `ListPage`, or a decode error naming the URL that
/// failed so the CLI output points at the culprit.
pub fn parse_page(body: String, url: String) -> AppResult<ListPage> {
    match from_str(&body) {
        Ok(value) => return Ok(value),
        Err(problem) => return Err(AppError::decode(format!("list page {} is not the JSON we expect: {}", url, problem))),
    }
}

/// Pause the loop for the documented Retry-After and slow the schedule down.
pub fn honour_retry_after(budget: &mut RateBudget, response: ApiResponse) {
    let wait: u64 = match response.retry_after {
        Some(seconds) => seconds as u64,
        None => FALLBACK_RETRY_MS / 1000,
    };
    sleep(Duration::from_secs(wait));
    budget.penalize(FALLBACK_RETRY_MS);
}

/// Key for one list page's reuse notes in the meta table: its ETag plus the two
/// facts the crawl needs to walk past a 304 without a body (has_more, count).
pub fn meta_page_key(page: i64, field: String) -> String {
    return format!("list-page-{}-{}", page, field);
}

pub fn more_of(hint: Option<String>) -> bool {
    match hint {
        None => return false,
        Some(text) => return text == "1",
    }
}

pub fn count_of(hint: Option<String>) -> Option<i64> {
    match hint {
        None => return None,
        Some(text) => return match text.trim().parse() {
            Err(_) => None,
            Ok(value) => Some(value),
        },
    }
}

/// Cancel plumbing: the sync loops ask this before every request. The CLI hands out
/// a fresh flag (nothing in a headless run raises it; Ctrl-C exits mid-request and
/// the DB stays consistent, so the next run resumes anyway). The GUI will keep one
/// in AppState and set it from the UI at v0.4.0 (docs/DEVELOPMENT.md state.rs plan).
pub struct CancelFlag {
    pub cancelled: bool,
}

impl CancelFlag {
    pub fn make() -> CancelFlag {
        return CancelFlag { cancelled: false };
    }
}

/// Where a paused list run restarts. sync_state's page column is the last page the
/// crawl fetched, so stopped and cancelled runs resume the page after it; everything
/// else (complete, ceiling, idle, unknown statuses) re-crawls from page 1, where the
/// stored ETags make the walk mostly 304s.
pub fn list_resume_at(state: SyncState) -> i64 {
    if state.status != "stopped" {
        if state.status != "cancelled" {
            return 1;
        }
    }
    match state.page {
        None => return 1,
        Some(found) => {
            if found >= 1 {
                return found + 1;
            }
            return 1;
        },
    }
}

/// Stage 1: crawl the item list into SQLite. `page_limit` bounds the run so tests
/// and CI can crawl a fixture cheaply (the CLI's `--budget`). A stopped or cancelled
/// run resumes at the page after the stored one; a finished run re-crawls from page
/// 1 and every page sends If-None-Match from its stored ETag.
pub fn sync_list(client: Client, conn: &Connection, mut budget: RateBudget, page_limit: Option<i64>, cancel: &CancelFlag) -> AppResult<SyncReport> {
    let started: i64 = unix_now();
    let mut report: SyncReport = SyncReport::make("list".to_string());
    let saved: SyncState = match sync_state_get(conn, "list".to_string()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let start_page: i64 = list_resume_at(saved);
    if start_page > 1 {
        report.resumed_from = Some(start_page);
    }
    let mut page: i64 = start_page;
    let mut total: i64 = 0;
    let mut more: bool = true;
    while more {
        if cancel.cancelled {
            report.message = format!("sync cancelled by user request at page {}", page);
            return finish(conn, report, started, page - 1, total, "cancelled".to_string());
        }
        match page_limit {
            Some(cap) => {
                if report.pages >= cap {
                    report.message = format!("stopped after {} list pages because of the budget", report.pages);
                    return finish(conn, report, started, page - 1, total, "stopped".to_string());
                }
            },
            None => {},
        }
        if page > PAGE_CEILING {
            report.message = format!("the crawl hit the page ceiling at page {}", page);
            return finish(conn, report, started, page - 1, total, "ceiling".to_string());
        }
        let url: String = page_url(page as u32, PER_PAGE);
        let stored_etag: String = match meta_get(conn, meta_page_key(page, "etag".to_string())) {
            Err(problem) => return Err(problem),
            Ok(found) => match found {
                None => "".to_string(),
                Some(text) => text,
            },
        };
        budget.wait_and_count();
        report.requests = report.requests + 1;
        let response: ApiResponse = match get_json(client.clone(), url.clone(), stored_etag.clone()) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        match classify(response.status) {
            HttpKind::RateLimited => {
                report.rate_limited = report.rate_limited + 1;
                honour_retry_after(&mut budget, response);
                continue;
            },
            HttpKind::Missing => {
                return Err(AppError::network(format!("list page {} answered {}", page, response.status)));
            },
            HttpKind::Failed => {
                return Err(AppError::network(format!("list page {} answered {}", page, response.status)));
            },
            HttpKind::NotModified => {
                report.unchanged = report.unchanged + 1;
                report.pages = report.pages + 1;
                let hint_more: bool = match meta_get(conn, meta_page_key(page, "more".to_string())) {
                    Err(problem) => return Err(problem),
                    Ok(found) => more_of(found),
                };
                let hint_count: Option<i64> = match meta_get(conn, meta_page_key(page, "count".to_string())) {
                    Err(problem) => return Err(problem),
                    Ok(found) => count_of(found),
                };
                match hint_count {
                    Some(carried) => {
                        if carried > total {
                            total = carried;
                        }
                    },
                    None => {},
                }
                more = hint_more;
                page = page + 1;
                continue;
            },
            HttpKind::Success => {},
        }
        let parsed: ListPage = match parse_page(response.body, url) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        let seen: usize = parsed.results.len();
        report.pages = report.pages + 1;
        report.items = report.items + seen as i64;
        for summary in parsed.results {
            match upsert_summary(conn, summary, response.etag.clone(), unix_now()) {
                Err(problem) => return Err(problem),
                Ok(written) => report.written = report.written + written as i64,
            }
        }
        match parsed.count {
            Some(value) => {
                if value as i64 > total {
                    total = value as i64;
                }
                match meta_set(conn, meta_page_key(page, "count".to_string()), format!("{}", value)) {
                    Err(problem) => return Err(problem),
                    Ok(_) => {},
                }
            },
            None => {},
        }
        let carried_more: bool = match parsed.has_more {
            Some(flag) => flag,
            None => false,
        };
        match meta_set(conn, meta_page_key(page, "more".to_string()), match carried_more { true => "1".to_string(), false => "0".to_string() }) {
            Err(problem) => return Err(problem),
            Ok(_) => {},
        }
        if response.etag != "" {
            match meta_set(conn, meta_page_key(page, "etag".to_string()), response.etag.clone()) {
                Err(problem) => return Err(problem),
                Ok(_) => {},
            }
        }
        match parsed.has_more {
            Some(flag) => more = flag,
            None => more = false,
        }
        if seen == 0 {
            more = false;
        }
        page = page + 1;
    }
    report.complete = true;
    if report.resumed_from != None {
        report.message = format!("list crawl resumed from page {}: complete, {} pages, {} records seen", start_page, report.pages, report.items);
    } else {
        report.message = format!("list crawl complete: {} pages, {} records seen", report.pages, report.items);
    }
    return finish(conn, report, started, page - 1, total, "complete".to_string());
}

/// Stage 2: fetch the detail record of items that have none yet (pending), or exactly
/// the ids a caller asked for (`--ids` / the watch list). This is the slow stage, so it
/// is opt-in, bounded and targeted. A targeted re-visit of an item that already has a
/// record sends its stored ETag and gets a `304`, so re-asking for the same ids costs
/// rate budget but neither bandwidth nor a rewrite - the reuse note in docs/API.md
/// counts both sides the same way here.
pub fn wanted_of(conn: &Connection, wanted: Vec<i64>, id_limit: Option<i64>) -> AppResult<Vec<i64>> {
    if wanted.is_empty() {
        return pending_ids(conn, id_limit);
    }
    let mut out: Vec<i64> = [].to_vec();
    let cap: i64 = match id_limit {
        None => wanted.len() as i64,
        Some(number) => number,
    };
    for one in wanted {
        if out.len() as i64 >= cap {
            break;
        }
        out.push(one);
    }
    return Ok(out);
}

pub fn sync_detail(client: Client, conn: &Connection, mut budget: RateBudget, id_limit: Option<i64>, cancel: &CancelFlag, wanted: Vec<i64>) -> AppResult<SyncReport> {
    let started: i64 = unix_now();
    let mut report: SyncReport = SyncReport::make("detail".to_string());
    let ids: Vec<i64> = match wanted_of(conn, wanted.clone(), id_limit) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let total: i64 = ids.len() as i64;
    for id in ids.clone() {
        if cancel.cancelled {
            let left: i64 = if wanted.is_empty() {
                match pending_ids(conn, None) {
                    Err(problem) => return Err(problem),
                    Ok(value) => value.len() as i64,
                }
            } else {
                ids.len() as i64 - report.items
            };
            report.message = format!("{}{}",
                format!("detail sync cancelled by user request: {} records fetched, {} left pending", report.items, left),
                missing_note(report.missing));
            return finish(conn, report, started, 0, total, "cancelled".to_string());
        }
        let mut attempts: i32 = 0;
        while attempts < 4 {
            budget.wait_and_count();
            report.requests = report.requests + 1;
            let url: String = item_url(id);
            let send_etag: String = match detail_etag_of(conn, id) {
                Err(problem) => return Err(problem),
                Ok(None) => "".to_string(),
                Ok(Some(found)) => found,
            };
            let response: ApiResponse = match get_json(client.clone(), url.clone(), send_etag) {
                Err(problem) => return Err(problem),
                Ok(value) => value,
            };
            match classify(response.status) {
                HttpKind::RateLimited => {
                    report.rate_limited = report.rate_limited + 1;
                    honour_retry_after(&mut budget, response);
                    attempts = attempts + 1;
                    continue;
                },
                HttpKind::Missing => {
                    // the record is gone upstream: remember the check, keep crawling.
                    report.missing = report.missing + 1;
                    match mark_missing_at(conn, id, unix_now()) {
                        Err(problem) => return Err(problem),
                        Ok(_) => {},
                    }
                    break;
                },
                HttpKind::Failed => {
                    return Err(AppError::network(format!("item {} answered {}", id, response.status)));
                },
                HttpKind::NotModified => report.unchanged = report.unchanged + 1,
                HttpKind::Success => {},
            }
            if response.unchanged {
                report.items = report.items + 1;
                break;
            }
            let parsed: ItemDetail = match from_str(&response.body) {
                Ok(value) => value,
                Err(problem) => return Err(AppError::decode(format!("item {} is not the JSON we expect: {}", id, problem))),
            };
            match upsert_detail(conn, parsed, response.etag.clone(), response.body, unix_now()) {
                Err(problem) => return Err(problem),
                Ok(written) => report.written = report.written + written as i64,
            }
            report.items = report.items + 1;
            break;
        }
    }
    let remaining: i64 = if wanted.is_empty() {
        match pending_ids(conn, None) {
            Err(problem) => return Err(problem),
            Ok(value) => value.len() as i64,
        }
    } else {
        ids.len() as i64 - report.items
    };
    report.complete = remaining == 0;
    if report.complete {
        report.message = if wanted.is_empty() {
            format!("{}{}",
                format!("detail crawl fetched {} records", report.items),
                missing_note(report.missing))
        } else {
            format!("{}{}",
                format!("detail crawl visited every targeted record: {} fetched", report.items),
                missing_note(report.missing))
        };
        return finish(conn, report, started, 0, total, "complete".to_string());
    }
    report.message = if wanted.is_empty() {
        format!("{}{}",
            format!("detail crawl paused: {} of {} records fetched, {} left pending", report.items, total, remaining),
            missing_note(report.missing))
    } else {
        format!("{}{}",
            format!("detail crawl stopped inside the target list: {} of {} fetched, {} left in the list", report.items, total, remaining),
            missing_note(report.missing))
    };
    return finish(conn, report, started, 0, total, "partial".to_string());
}

/// Catalogue coverage, used by `status` and the UI footer.
pub fn catalogue(conn: &Connection) -> AppResult<Coverage> {
    return coverage(conn);
}
