//! The sync engine: the crawl stages (list, detail, and - v0.6.0 - the icon copy),
//! resume bookkeeping and progress.
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

use crate::wowdb::{ApiResponse, BinaryResponse, Client, HttpKind, RateBudget, classify, get_bytes, get_json, icon_target, item_url, page_url, recipe_file_url, tooltip_url, FALLBACK_RETRY_MS};

use crate::models::{ItemDetail, ListPage, Tooltip, RecipeFile};

use crate::db::{copy_icon, detail_etag_of, icon_url_of, mark_icon_missing_at, mark_missing_at, mark_price_missing_at, mark_tooltip_missing_at, meta_get, meta_set, sync_state_get, sync_state_set, tooltip_etag_of, unix_now, upsert_detail, upsert_summary, write_price_now, write_recipe, write_tooltip, SyncState};

use crate::stats::{icon_pending_ids, pending_ids, price_pending_ids, tooltip_pending_ids, coverage, Coverage};

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

/// Pause the loop for the documented Retry-After and slow the schedule down. Takes the
/// header value, not the response: the detail crawl reads text and the icon crawl reads
/// bytes, and both honour the same pause (v0.6.0).
pub fn honour_retry_after(budget: &mut RateBudget, retry_after: Option<u32>) {
    let wait: u64 = match retry_after {
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
                honour_retry_after(&mut budget, response.retry_after);
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
                    honour_retry_after(&mut budget, response.retry_after);
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

/// What the icon stage's third state adds to a run's message (v0.6.0): the record
/// EXISTS, only its icon file answered 404 - a different fact than "gone upstream".
pub fn icon_missing_note(missing: i64) -> String {
    if missing == 0 {
        return "".to_string();
    }
    return format!("; {} answered 404 for the icon file and are checked as no-icon-upstream", missing);
}

/// Where the icon stage points its queue: pending icon work (stats.rs::SQL_PENDING_ICON)
/// or exactly the ids a caller targeted. Shares wanted_of's slice mechanics.
pub fn icon_wanted_of(conn: &Connection, wanted: Vec<i64>, id_limit: Option<i64>) -> AppResult<Vec<i64>> {
    if wanted.is_empty() {
        return icon_pending_ids(conn, id_limit);
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

/// Where the tooltip stage points its queue (stats.rs::SQL_PENDING_TOOLTIP) or
/// exactly the ids a caller targeted. Shares wanted_of's slice mechanics.
pub fn tooltip_wanted_of(conn: &Connection, wanted: Vec<i64>, id_limit: Option<i64>) -> AppResult<Vec<i64>> {
    if wanted.is_empty() {
        return tooltip_pending_ids(conn, id_limit);
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

/// Stage 4 of a pull (v0.8.0, ADR 0010): fetch the TOOLTIP COPY of records whose
/// detail answered, once each - the pending rule (copied? checked gone? never looked
/// at?) is the twin of the detail and icon rules, so a full crawl costs ONE request
/// per tooltip, ever, and the whole catalogue costs about 19 258 requests at the
/// measured 2 per second (≈ 2.7 h, same order as the detail and icon stages).
/// Why a fourth stage at all: the user asked for DESCRIPTIONS, and the forever/item
/// body carries none (measured key list, docs/API.md) - the text lives behind this
/// door, whose body answers `html` with the card's own markup. The write applies the
/// manual layer (ADR 0010's override rule), so a user-stated tooltip_html survives
/// this stage's copy, and ADR 0007's price sample stays on the DETAIL write only: a
/// tooltip re-ask is not a second observation of the auction house in this build.
/// 429 retries in place (the crawl stays a success), a 404 upstream's own JSON marks
/// the third state and keeps crawling, anything else aborts. `--ids`/`--watch`
/// targeting is honored so a user can copy just the tooltips of their watch list.
pub fn sync_tooltips(client: Client, conn: &Connection, mut budget: RateBudget, id_limit: Option<i64>, cancel: &CancelFlag, wanted: Vec<i64>) -> AppResult<SyncReport> {
    let started: i64 = unix_now();
    let mut report: SyncReport = SyncReport::make("tooltip".to_string());
    let ids: Vec<i64> = match tooltip_wanted_of(conn, wanted.clone(), id_limit) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let total: i64 = ids.len() as i64;
    for id in ids.clone() {
        if cancel.cancelled {
            let left: i64 = if wanted.is_empty() {
                match tooltip_pending_ids(conn, None) {
                    Err(problem) => return Err(problem),
                    Ok(value) => value.len() as i64,
                }
            } else {
                ids.len() as i64 - report.items
            };
            report.message = format!("{}{}",
                format!("tooltip crawl cancelled by user request: {} copies fetched, {} left pending", report.written, left),
                tooltip_missing_note(report.missing));
            return finish(conn, report, started, 0, total, "cancelled".to_string());
        }
        let mut attempts: i32 = 0;
        while attempts < 4 {
            budget.wait_and_count();
            report.requests = report.requests + 1;
            let url: String = tooltip_url(id);
            let send_etag: String = match tooltip_etag_of(conn, id) {
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
                    honour_retry_after(&mut budget, response.retry_after);
                    attempts = attempts + 1;
                    continue;
                },
                HttpKind::Missing => {
                    // upstream answered its own 404 JSON for this tooltip (docs/API.md
                    // measures the body): the third state of THIS stage. Mark and move
                    // on - the record itself exists (its detail answered).
                    report.missing = report.missing + 1;
                    match mark_tooltip_missing_at(conn, id, unix_now()) {
                        Err(problem) => return Err(problem),
                        Ok(_) => {},
                    }
                    break;
                },
                HttpKind::Failed => {
                    return Err(AppError::network(format!("tooltip of item {} answered {}", id, response.status)));
                },
                HttpKind::NotModified => report.unchanged = report.unchanged + 1,
                HttpKind::Success => {},
            }
            if response.unchanged {
                report.items = report.items + 1;
                break;
            }
            let parsed: Tooltip = match from_str(&response.body) {
                Ok(value) => value,
                Err(problem) => return Err(AppError::decode(format!("tooltip of item {} is not the JSON we expect: {}", id, problem))),
            };
            match write_tooltip(conn, id, parsed.html.clone(), response.etag.clone(), unix_now()) {
                Err(problem) => return Err(problem),
                Ok(written) => report.written = report.written + written as i64,
            }
            report.items = report.items + 1;
            break;
        }
    }
    let remaining: i64 = if wanted.is_empty() {
        match tooltip_pending_ids(conn, None) {
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
                format!("tooltip crawl copied {} copies", report.written),
                tooltip_missing_note(report.missing))
        } else {
            format!("{}{}",
                format!("tooltip crawl visited every targeted record: {} handled", report.items),
                tooltip_missing_note(report.missing))
        };
        return finish(conn, report, started, 0, total, "complete".to_string());
    }
    report.message = if wanted.is_empty() {
        format!("{}{}",
            format!("tooltip crawl paused: {} of {} copies fetched, {} left pending", report.written, total, remaining),
            tooltip_missing_note(report.missing))
    } else {
        format!("{}{}",
            format!("tooltip crawl stopped inside the target list: {} of {} handled, {} left in the list", report.items, total, remaining),
            tooltip_missing_note(report.missing))
    };
    return finish(conn, report, started, 0, total, "stopped".to_string())
}

/// The recipes fetch stage (v0.10.0, ADR 0021): pull the dataset's OWN classic
/// recipe files from the second source docs/API.md's second-source section records
/// (MIT text measured at the repo) - 8 files, one request each, conditional GETs
/// like every other door (the file ETags ride the meta table, so a re-pull pays
/// 304s and the rows keep what the first pull wrote). The dataset speaks per-file
/// `profession` (measured set: 8 classic professions - NO Jewelcrafting there,
/// recorded honestly in the ADR and the tab note). There is no per-record third
/// state here: a file that 404s is the SOURCE moving, not a record being gone, so
/// it ABORTS the run (rc 3) like the Failed kind does - and the door-side truth
/// (a crafted id the DB never saw, a reagent row whose price nobody answered) is
/// what the reader reports instead. `--budget` bounds FILES per run; the cursor is
/// the file count already done, so a stopped pull resumes at the next file.
pub const RECIPE_FILE_COUNT: i64 = 8;

pub fn recipe_file_at(index: i64) -> String {
    match index {
        0 => return "alchemy_recipes.json".to_string(),
        1 => return "blacksmithing_recipes.json".to_string(),
        2 => return "cooking_recipes.json".to_string(),
        3 => return "enchanting_recipes.json".to_string(),
        4 => return "engineering_recipes.json".to_string(),
        5 => return "first_aid_recipes.json".to_string(),
        6 => return "leatherworking_recipes.json".to_string(),
        _ => return "tailoring_recipes.json".to_string(),
    };
}

pub fn recipe_etag_key(file: String) -> String {
    return format!("recipe_etag_{}", file);
}

pub fn sync_recipes(client: Client, conn: &Connection, mut budget: RateBudget, file_limit: Option<i64>, cancel: &CancelFlag) -> AppResult<SyncReport> {
    let started: i64 = unix_now();
    let mut report: SyncReport = SyncReport::make("recipes".to_string());
    let saved: SyncState = match sync_state_get(conn, "recipes".to_string()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let cursor_at: i64 = match saved.page {
        None => 0,
        Some(done) => if saved.status == "complete" { 0 } else { done },
    };
    report.resumed_from = if cursor_at > 0 { Some(cursor_at) } else { None };
    let bound: i64 = match file_limit {
        None => RECIPE_FILE_COUNT,
        Some(want) => if want < 0 { 0 } else { want },
    };
    let total: i64 = RECIPE_FILE_COUNT;
    let mut index: i64 = cursor_at;
    while index < total && index < cursor_at + bound {
        if cancel.cancelled {
            report.message = format!("recipe pull cancelled by user request: {} files fetched, {} entries seen, {} rows written", report.pages, report.items, report.written);
            return finish(conn, report, started, index, total, "cancelled".to_string());
        }
        let file: String = recipe_file_at(index);
        let mut attempts: i32 = 0;
        while attempts < 4 {
            budget.wait_and_count();
            report.requests = report.requests + 1;
            let url: String = recipe_file_url(file.clone());
            let send_etag: String = match meta_get(conn, recipe_etag_key(file.clone())) {
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
                    honour_retry_after(&mut budget, response.retry_after);
                    attempts = attempts + 1;
                    continue;
                },
                HttpKind::Missing => {
                    return Err(AppError::network(format!("recipe file {} answered {} - the second source moved, the recorded set cannot be pulled", file, response.status)));
                },
                HttpKind::Failed => {
                    return Err(AppError::network(format!("recipe file {} answered {}", file, response.status)));
                },
                HttpKind::NotModified => {
                    report.unchanged = report.unchanged + 1;
                    break;
                },
                HttpKind::Success => {},
            }
            let parsed: RecipeFile = match from_str(&response.body) {
                Ok(value) => value,
                Err(problem) => return Err(AppError::decode(format!("recipe file {} is not the JSON this source's record describes: {}", file, problem))),
            };
            for entry in parsed.recipes.clone() {
                let written: i64 = match write_recipe(conn, parsed.profession.clone(), entry) {
                    Err(problem) => return Err(problem),
                    Ok(count) => count as i64,
                };
                report.written = report.written + written;
                report.items = report.items + 1;
            }
            if response.etag != "" {
                match meta_set(conn, recipe_etag_key(file), response.etag.clone()) {
                    Err(problem) => return Err(problem),
                    Ok(_) => {},
                }
            }
            break;
        }
        index = index + 1;
        report.pages = report.pages + 1;
    }
    report.complete = index >= total;
    if report.complete {
        report.message = format!("recipe pull complete: {} of {} files fetched ({} answered the same copy), {} entries seen, {} rows written", report.pages, total, report.unchanged, report.items, report.written);
        return finish(conn, report, started, total, total, "complete".to_string());
    }
    report.message = format!("recipe pull stopped: {} of {} files fetched, {} entries seen, {} rows written", report.pages, total, report.items, report.written);
    return finish(conn, report, started, index, total, "stopped".to_string());
}

/// Stage 3 of a pull (v0.6.0, ADR 0008): copy the icon FILES of records that answered,
/// once each - the pending rule (copied? checked? never looked at?) is the icon twin of
/// the detail rule, so a full crawl costs ONE request per icon, ever. Bytes ride the
/// DB as a BLOB; the card reads them back. Note where the requests go: icon files live
/// on the asset hosts the detail payloads name (docs/API.md measures both), NOT on the
/// rate-limited API endpoints - but the schedule and the Retry-After courtesy still
/// apply: one host, one politeness, and the GUI runs this stage from the same budget.
/// A 404 on the ICON of an existing record is the third state (mark, keep crawling);
/// any other failure aborts. `--ids`/`--watch` targeting is honored so a user can copy
/// just the icons of their watch list.
pub fn sync_icons(client: Client, conn: &Connection, mut budget: RateBudget, id_limit: Option<i64>, cancel: &CancelFlag, wanted: Vec<i64>) -> AppResult<SyncReport> {
    let started: i64 = unix_now();
    let mut report: SyncReport = SyncReport::make("icons".to_string());
    let ids: Vec<i64> = match icon_wanted_of(conn, wanted.clone(), id_limit) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let total: i64 = ids.len() as i64;
    for id in ids.clone() {
        if cancel.cancelled {
            let left: i64 = if wanted.is_empty() {
                match icon_pending_ids(conn, None) {
                    Err(problem) => return Err(problem),
                    Ok(value) => value.len() as i64,
                }
            } else {
                ids.len() as i64 - report.items
            };
            report.message = format!("{}{}",
                format!("icon crawl cancelled by user request: {} icons copied, {} left pending", report.written, left),
                icon_missing_note(report.missing));
            return finish(conn, report, started, 0, total, "cancelled".to_string());
        }
        let stored_url: Option<String> = match icon_url_of(conn, id) {
            Err(problem) => return Err(problem),
            Ok(found) => found,
        };
        let target: Option<String> = icon_target(stored_url.clone());
        match target {
            None => {
                // a targeted visit of a record whose detail answered WITHOUT an icon_url
                // (or with a shape icon_target refuses to guess): there is nothing to
                // fetch, and calling that "no icon upstream" would be a lie about a
                // fact the upstream never stated. Nothing requested, nothing marked.
                report.items = report.items + 1;
                report.unchanged = report.unchanged + 1;
                continue;
            },
            Some(url) => {
                let mut attempts: i32 = 0;
                while attempts < 4 {
                    budget.wait_and_count();
                    report.requests = report.requests + 1;
                    let response: BinaryResponse = match get_bytes(client.clone(), url.clone()) {
                        Err(problem) => return Err(problem),
                        Ok(value) => value,
                    };
                    match classify(response.status) {
                        HttpKind::RateLimited => {
                            report.rate_limited = report.rate_limited + 1;
                            honour_retry_after(&mut budget, response.retry_after);
                            attempts = attempts + 1;
                            continue;
                        },
                        HttpKind::Missing => {
                            // the RECORD exists (its detail answered) - the icon FILE
                            // does not: the third state of THIS stage. Mark and move on.
                            report.missing = report.missing + 1;
                            match mark_icon_missing_at(conn, id, unix_now()) {
                                Err(problem) => return Err(problem),
                                Ok(_) => {},
                            }
                            report.items = report.items + 1;
                            break;
                        },
                        HttpKind::Failed => {
                            return Err(AppError::network(format!("icon of item {} answered {}", id, response.status)));
                        },
                        HttpKind::NotModified => {
                            // no If-None-Match is ever sent for icons, so this cannot
                            // happen against a well-behaved server; count and move on.
                            report.unchanged = report.unchanged + 1;
                            report.items = report.items + 1;
                            break;
                        },
                        HttpKind::Success => {},
                    }
                    match copy_icon(conn, id, response.bytes.clone(), unix_now()) {
                        Err(problem) => return Err(problem),
                        Ok(copied) => report.written = report.written + copied as i64,
                    }
                    report.items = report.items + 1;
                    break;
                }
            },
        }
    }
    let remaining: i64 = if wanted.is_empty() {
        match icon_pending_ids(conn, None) {
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
                format!("icon crawl copied {} icons", report.written),
                icon_missing_note(report.missing))
        } else {
            format!("{}{}",
                format!("icon crawl visited every targeted record: {} handled", report.items),
                icon_missing_note(report.missing))
        };
        return finish(conn, report, started, 0, total, "complete".to_string());
    }
    report.message = if wanted.is_empty() {
        format!("{}{}",
            format!("icon crawl paused: {} of {} icons copied, {} left pending", report.written, total, remaining),
            icon_missing_note(report.missing))
    } else {
        format!("{}{}",
            format!("icon crawl stopped inside the target list: {} of {} handled, {} left in the list", report.items, total, remaining),
            icon_missing_note(report.missing))
    };
    return finish(conn, report, started, 0, total, "partial".to_string());
}

/// Catalogue coverage, used by `status` and the UI footer.
pub fn catalogue(conn: &Connection) -> AppResult<Coverage> {
    return coverage(conn);
}

/// "N answered 404 for the tooltip and are checked as no-tooltip-upstream", the
/// icon/detail note's twin (a zero count adds nothing, so the message stays clean).
pub fn tooltip_missing_note(missing: i64) -> String {
    if missing == 0 {
        return "".to_string()
    }
    return format!("; {} answered 404 for the tooltip and are checked as no-tooltip-upstream", missing)
}

/// Stage 5 of a pull (v0.9.6, ADR 0017 "the live min buyout"): REFRESH the
/// auction-house minimum for records that already hold a tooltip copy. The door
/// is the tooltip door itself - docs/API.md measures its `fields.price` and the
/// developers page states tooltips cache ~5 min "because they carry a live
/// Auction House price"; measured 2026-11-21 on the live door: id 25 answered
/// `min 149` at `updated_at 22:25:10`, ids 202/237/763 answered 5000/3500/4425
/// while their STORED copies carried NO price at all, and id 200 answered
/// `fields.price` as literal NULL - the door's own third state. This is why the
/// write exists: price_min (the stored copy's number, ADR 0007 sampled it into
/// the ledger) AGES, and price_min_now is what the door says NOW - ADR 0016's
/// two columns, now with the FIRST producer for one of them. The pending rule
/// is the twin of the icon and tooltip rules: refreshed? checked as "no price
/// right now"? never looked at? - so a full pass costs ONE request per copied
/// record, ever, excludes manual rows and tombstones, and the queue walks
/// id ASC like every stage. The stage sends NO If-None-Match even though the
/// tooltip stage stored an ETag: the fields.price this ask needs was NEVER
/// parsed before (the copy stage reads `html` only), so a conditional answer
/// would mean "unchanged since a read that never saw this field" - the door's
/// own ~5-min server cache is what keeps the pass polite. Fresh writes sample
/// NOTHING: ADR 0007's price sample stays on the DETAIL write (the sampling
/// contract is that release's, and the trend that would eat denser samples is
/// later work). The stage counts a null `fields.price` (measured) as the third
/// state; the door's 404 answers the same state for parity with its twin,
/// though the pending rule only points at ids whose copy EXISTS - a 404 here
/// would be "the door lost a copy", never measured, honestly marked and kept
/// walking like the tooltip stage does.
pub fn sync_prices(client: Client, conn: &Connection, mut budget: RateBudget, id_limit: Option<i64>, cancel: &CancelFlag, wanted: Vec<i64>) -> AppResult<SyncReport> {
    let started: i64 = unix_now();
    let mut report: SyncReport = SyncReport::make("price".to_string());
    let ids: Vec<i64> = match price_wanted_of(conn, wanted.clone(), id_limit) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let total: i64 = ids.len() as i64;
    for id in ids.clone() {
        if cancel.cancelled {
            let left: i64 = if wanted.is_empty() {
                match price_pending_ids(conn, None) {
                    Err(problem) => return Err(problem),
                    Ok(value) => value.len() as i64,
                }
            } else {
                ids.len() as i64 - report.items
            };
            report.message = format!("{}{}",
                format!("price crawl cancelled by user request: {} refreshed, {} left pending", report.written, left),
                price_now_missing_note(report.missing));
            return finish(conn, report, started, 0, total, "cancelled".to_string());
        }
        let mut attempts: i32 = 0;
        while attempts < 4 {
            budget.wait_and_count();
            report.requests = report.requests + 1;
            let url: String = tooltip_url(id);
            // no ETag sent - see the doc above: a 304 here would save nothing.
            let response: ApiResponse = match get_json(client.clone(), url.clone(), "".to_string()) {
                Err(problem) => return Err(problem),
                Ok(value) => value,
            };
            match classify(response.status) {
                HttpKind::RateLimited => {
                    report.rate_limited = report.rate_limited + 1;
                    honour_retry_after(&mut budget, response.retry_after);
                    attempts = attempts + 1;
                    continue;
                },
                HttpKind::Missing => {
                    // the door's own 404 body (docs/API.md: upstream's JSON words,
                    // measured on ids WITHOUT a copy) - same third state as the
                    // null price, marked and walked past.
                    report.missing = report.missing + 1;
                    match mark_price_missing_at(conn, id, unix_now()) {
                        Err(problem) => return Err(problem),
                        Ok(_) => {},
                    }
                    break;
                },
                HttpKind::Failed => {
                    return Err(AppError::network(format!("price probe of item {} answered {}", id, response.status)));
                },
                HttpKind::NotModified => report.unchanged = report.unchanged + 1,
                HttpKind::Success => {},
            }
            if response.unchanged {
                report.items = report.items + 1;
                break;
            }
            let parsed: Tooltip = match from_str(&response.body) {
                Ok(value) => value,
                Err(problem) => return Err(AppError::decode(format!("tooltip of item {} is not the JSON we expect: {}", id, problem))),
            };
            // the live price block, read THROUGH the model mirror: the wire's
            // `fields` object lands in TooltipFields directly (serde ignores the
            // keys we have no reader for - upstream cannot add a key that breaks
            // a build), so nothing here hand-parses the copy.
            let live: Option<i64> = match parsed.fields.clone() {
                None => None,
                Some(fields) => match fields.price.clone() {
                    None => None,
                    Some(block) => match block.min.clone() {
                        None => None,
                        Some(number) => Some(number),
                    },
                },
            };
            match live {
                None => {
                    // measured third state, live id 200 today: `fields.price` is
                    // literal null - the door LOOKED and the AH offers NOTHING now.
                    // COUNTED like the 404 arm: the first mechanism run measured the
                    // stamp on disk but a lying `missing 0` in the report (the stamp
                    // existed, the counter did not move) - fixed the same breath.
                    report.missing = report.missing + 1;
                    match mark_price_missing_at(conn, id, unix_now()) {
                        Err(problem) => return Err(problem),
                        Ok(_) => {},
                    }
                },
                Some(copper) => {
                    match write_price_now(conn, id, copper, unix_now()) {
                        Err(problem) => return Err(problem),
                        Ok(written) => report.written = report.written + written as i64,
                    }
                },
            }
            report.items = report.items + 1;
            break;
        }
    }
    let remaining: i64 = if wanted.is_empty() {
        match price_pending_ids(conn, None) {
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
                format!("price crawl refreshed {} live prices", report.written),
                price_now_missing_note(report.missing))
        } else {
            format!("{}{}",
                format!("price crawl visited every targeted record: {} handled", report.items),
                price_now_missing_note(report.missing))
        };
        return finish(conn, report, started, 0, total, "complete".to_string());
    }
    report.message = if wanted.is_empty() {
        format!("{}{}",
            format!("price crawl paused: {} of {} live prices refreshed, {} left pending", report.written, total, remaining),
            price_now_missing_note(report.missing))
    } else {
        format!("{}{}",
            format!("price crawl stopped inside the target list: {} of {} handled, {} left in the list", report.items, total, remaining),
            price_now_missing_note(report.missing))
    };
    return finish(conn, report, started, 0, total, "stopped".to_string())
}

/// Where the price stage points its queue (stats.rs::SQL_PENDING_PRICE, the twin
/// of the tooltip and icon rules) or exactly the ids a caller targeted. Shares
/// wanted_of's slice mechanics.
pub fn price_wanted_of(conn: &Connection, wanted: Vec<i64>, id_limit: Option<i64>) -> AppResult<Vec<i64>> {
    if wanted.is_empty() {
        return price_pending_ids(conn, id_limit);
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

/// "N looked and the auction house offered NOTHING right now, and are checked",
/// the price twin of the tooltip/icon missing notes (a zero count adds nothing).
pub fn price_now_missing_note(missing: i64) -> String {
    if missing == 0 {
        return "".to_string()
    }
    return format!("; {} answered no auction price now and are checked as price-less-at-ask-time", missing)
}
