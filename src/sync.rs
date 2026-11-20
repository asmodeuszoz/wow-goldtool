//! The sync engine: the two crawl stages, resume bookkeeping and progress.
//!
//! This is the only module where HTTP and SQLite meet (docs/DEVELOPMENT.md house
//! rules): `wowdb.rs` never touches the DB and `db.rs` never touches the network.
//!
//! Pacing is by construction, never by luck (docs/API.md): a burst may run freely as
//! long as it stays under the headroom the API reports, past that the schedule is two
//! requests per second, and a 429 pauses the loop for the documented Retry-After
//! without failing the run. Nothing here retries a hard failure: a half-finished
//! crawl is resumable, and the next run picks up where `sync_state` says it stopped.

use crate::errors::AppError;
use crate::errors::AppResult;

use serde::Serialize;

use crate::wowdb::{Client, ApiResponse, HttpKind, RateBudget, classify, get_json, item_url, page_url, FALLBACK_RETRY_MS};

use crate::models::{ItemDetail, ListPage};

use crate::db::{sync_state_set, unix_now, upsert_detail, upsert_summary, SyncState};

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
    pub seconds: i64,
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
            seconds: 0,
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
pub fn finish(conn: &Connection, report: SyncReport, started: i64, total: i64, status: String) -> AppResult<SyncReport> {
    let mut out: SyncReport = report;
    out.seconds = unix_now() - started;
    let stage: String = out.stage.clone();
    let note: String = out.message.clone();
    match record(conn, stage, status, out.pages, out.items, total, note) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(out),
    }
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

/// Stage 1: crawl the item list into SQLite. `page_limit` bounds the run so tests
/// and CI can crawl a fixture cheaply (the CLI's `--budget`).
pub fn sync_list(client: Client, conn: &Connection, mut budget: RateBudget, page_limit: Option<i64>) -> AppResult<SyncReport> {
    let started: i64 = unix_now();
    let mut report: SyncReport = SyncReport::make("list".to_string());
    let mut page: i64 = 1;
    let mut total: i64 = 0;
    let mut more: bool = true;
    while more {
        match page_limit {
            Some(cap) => {
                if report.pages >= cap {
                    report.message = format!("stopped after {} list pages because of the budget", report.pages);
                    return finish(conn, report, started, total, "stopped".to_string());
                }
            },
            None => {},
        }
        if page > PAGE_CEILING {
            report.message = format!("the crawl hit the page ceiling at page {}", page);
            return finish(conn, report, started, total, "ceiling".to_string());
        }
        let url: String = page_url(page as u32, PER_PAGE);
        budget.wait_and_count();
        report.requests = report.requests + 1;
        let response: ApiResponse = match get_json(client.clone(), url.clone(), "".to_string()) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        match classify(response.status) {
            HttpKind::RateLimited => {
                report.rate_limited = report.rate_limited + 1;
                honour_retry_after(&mut budget, response);
                continue;
            },
            HttpKind::Failed => {
                return Err(AppError::network(format!("list page {} answered {}", page, response.status)));
            },
            HttpKind::NotModified => report.unchanged = report.unchanged + 1,
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
            },
            None => {},
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
    report.message = format!("list crawl complete: {} pages, {} records seen", report.pages, report.items);
    return finish(conn, report, started, total, "complete".to_string());
}

/// Stage 2: fetch the detail record of items that have none yet. This is the slow
/// stage (roughly 19,000 requests at two per second), so it is opt-in and bounded.
pub fn sync_detail(client: Client, conn: &Connection, mut budget: RateBudget, id_limit: Option<i64>) -> AppResult<SyncReport> {
    let started: i64 = unix_now();
    let mut report: SyncReport = SyncReport::make("detail".to_string());
    let ids: Vec<i64> = match pending_ids(conn, id_limit) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let total: i64 = ids.len() as i64;
    for id in ids {
        let mut attempts: i32 = 0;
        while attempts < 4 {
            budget.wait_and_count();
            report.requests = report.requests + 1;
            let url: String = item_url(id);
            let response: ApiResponse = match get_json(client.clone(), url.clone(), "".to_string()) {
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
    let remaining: i64 = match pending_ids(conn, None) {
        Err(problem) => return Err(problem),
        Ok(value) => value.len() as i64,
    };
    report.complete = remaining == 0;
    if report.complete {
        report.message = format!("detail crawl fetched {} records", report.items);
        return finish(conn, report, started, total, "complete".to_string());
    }
    report.message = format!("detail crawl paused: {} of {} records fetched, {} left pending", report.items, total, remaining);
    return finish(conn, report, started, total, "partial".to_string());
}

/// Catalogue coverage, used by `status` and the UI footer.
pub fn catalogue(conn: &Connection) -> AppResult<Coverage> {
    return coverage(conn);
}
