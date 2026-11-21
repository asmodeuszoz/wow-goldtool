//! Coverage and price analytics over the local DB.
//!
//! Every number here comes from one SQL aggregate, so the CLI, the UI and the test
//! suite read the same figures instead of three implementations of the same maths.
//! NULL prices are counted separately and never averaged in as zero.

use crate::errors::AppError;
use crate::errors::AppResult;

use crate::db::{icon_at_of, icon_b64_of, icon_missing_at_of};

use rusqlite::Connection;
use rusqlite::params_from_iter;
use rusqlite::types::Value;

use serde::Serialize;

/// How complete the local catalogue is, plus the price spread over everything
/// priced so far.
#[derive(Serialize, Clone)]
pub struct Coverage {
    pub items: i64,
    pub named: i64,
    pub priced: i64,
    pub detailed: i64,
    pub missing: i64,
    pub icons: i64,
    pub icon_missing: i64,
    pub tooltips: i64,
    pub tooltip_missing: i64,
    pub edited: i64,
    pub manual: i64,
    pub deleted: i64,
    pub history: i64,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub avg_price: Option<f64>,
}

/// Price statistics per quality label, the facets the UI charts.
#[derive(Serialize, Clone)]
pub struct QualityStat {
    pub quality: Option<String>,
    pub rank: Option<i64>,
    pub items: i64,
    pub priced: i64,
    pub avg_price: Option<f64>,
}

/// One entry of the extremes table: the cheapest and priciest things worth hunting.
#[derive(Serialize, Clone)]
pub struct Extreme {
    pub id: i64,
    pub name: String,
    pub quality: Option<String>,
    pub price_min: Option<i64>,
    /// v0.9.3 (ADR 0014): the same icon trio the search hits carry, so the two
    /// price lists render the DB's OWN icon bytes (ADR 0008's rule: upstream's
    /// `icon_url` is never hotlinked) and a row with no bytes still says WHICH
    /// state it is in - `--icons` fills the payload, the stamps always ride.
    pub icon_b64: Option<String>,
    pub icon_at: Option<i64>,
    pub icon_missing_at: Option<i64>,
}

/// One aggregate row over the whole catalogue. `AVG`/`MIN`/`MAX` over an empty
/// column is NULL, which is why those three fields are optional.
pub const SQL_COVERAGE: &str = concat!(
    "SELECT COUNT(1), ",
    "CAST(IFNULL(SUM(name IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(price_min IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(detail_synced_at IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(detail_missing_at IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(icon_at IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(icon_missing_at IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(tooltip_at IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(tooltip_missing_at IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(edited_at IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(manual_at IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(deleted_at IS NOT NULL), 0) AS INTEGER), ",
    "MIN(price_min), MAX(price_min), AVG(price_min) ",
    "FROM item",
);

/// Price observations stored locally, counted straight off the second table:
/// the status card prints it, and it is what proves the sampling rule (one fresh
/// detail write = one row, 304s write none) in the audits (ADR 0007).
pub const SQL_COUNT_SAMPLES: &str = "SELECT COUNT(1) FROM price_history";

/// The same aggregate split by quality label, cheapest rank first.
pub const SQL_QUALITY: &str = concat!(
    "SELECT quality, quality_rank, COUNT(1), CAST(IFNULL(SUM(price_min IS NOT NULL), 0) AS INTEGER), AVG(price_min) ",
    "FROM item GROUP BY quality_rank, quality ",
    "ORDER BY (quality_rank IS NULL) ASC, quality_rank ASC, quality ASC",
);

/// Cheapest priced items first: the things worth hunting for.
pub const SQL_CHEAP: &str = concat!(
    "SELECT id, name, quality, price_min FROM item WHERE price_min IS NOT NULL ",
    "ORDER BY price_min ASC, name ASC LIMIT ?",
);

/// Priciest priced items first.
pub const SQL_RICH: &str = concat!(
    "SELECT id, name, quality, price_min FROM item WHERE price_min IS NOT NULL ",
    "ORDER BY price_min DESC, name ASC LIMIT ?",
);

/// Item ids that have no detail record yet: what stage 2 of the sync will fetch.
/// Three states per id (docs/DATABASE.md): fetched (detail_synced_at set), checked
/// and gone upstream (detail_missing_at set after a 404), or never looked at.
/// Only the last one is pending, so a removed record costs one request per crawl
/// instead of one per run, and "nothing pending" stays literally true. v0.8.0 adds
/// two terms that keep the USER out of the crawl's way: a tombstoned row is deleted
/// ON PURPOSE (a queue that re-fetched it would answer 404 and stamp the third state
/// on a row the user buried), and a `manual_at` row is one the user MADE - upstream
/// that re-fetched it would answer 404 and stamp the third state on a row the user
/// buried), and a `manual_at` row is one the user MADE - upstream has no such id
/// (measured max upstream id on their file: 288 340, manual block starts at 900 000
/// 000), so a queue that walked it would ask for a record that cannot exist. An
/// explicit `--ids` ask still reaches both: targeting is a human decision, a queue
/// is not.
pub const SQL_PENDING: &str = concat!(
    "SELECT id FROM item WHERE detail_synced_at IS NULL AND detail_missing_at IS NULL ",
    "AND deleted_at IS NULL AND manual_at IS NULL ",
    "ORDER BY id ASC",
);

/// The icon stage's queue (v0.6.0, ADR 0008): pending means **a detail record
/// answered AND no icon copied yet AND not already checked as no-icon-upstream
/// AND there IS a url to fetch**. The icon_url terms keep the rule honest: a
/// record whose detail answered WITHOUT a url, or with the EMPTY one (both
/// measured live 2026-11-20: Peace Tea, Craftsman's Writ, Scroll of Imbue
/// Iceknife and Lodestone answer "icon_url": ""), has NOTHING to fetch - under a
/// rule without them pending would never drain and "nothing pending" would lie,
/// the same trap v0.5.2 fixed for prices. Unlike the detail queue this one needs
/// no re-ask-first CASE: every candidate already carries a detail stamp, so the
/// queue is ordered by id.
pub const SQL_PENDING_ICON: &str = concat!(
    "SELECT id FROM item WHERE detail_synced_at IS NOT NULL AND icon_at IS NULL ",
    "AND icon_missing_at IS NULL AND icon_url IS NOT NULL AND icon_url <> '' ",
    "AND deleted_at IS NULL AND manual_at IS NULL ",
    "ORDER BY id ASC",
);

/// The tooltip stage's queue (v0.8.0, ADR 0010): pending means **a detail record
/// answered AND no tooltip copy yet AND not already checked as no-tooltip-upstream
/// AND not a tombstone AND not a user-made row**. A detail stamp is required because
/// the catalogue's ids come from the list stage, and the door the tooltip lives behind
/// answers upstream's own 404 JSON for an id it has not (measured 2026-11-20:
/// `{"error":"not_found","message":"No item 999999999999 in forever.","status":404}`)
/// - so asking the tooltip of an id upstream lacks costs a request AND stamps a state
/// about a fact upstream never stated. The manual/tombstone terms are the v0.8.0 rule
/// that a crawl never walks a row the user made or buried (same reason as SQL_PENDING
/// and SQL_PENDING_ICON).
pub const SQL_PENDING_TOOLTIP: &str = concat!(
    "SELECT id FROM item WHERE detail_synced_at IS NOT NULL AND tooltip_at IS NULL ",
    "AND tooltip_missing_at IS NULL AND deleted_at IS NULL AND manual_at IS NULL ",
    "ORDER BY id ASC",
);

struct PendingSql {
    pub sql: String,
    pub params: Vec<Value>,
}

/// The limit is a bound parameter so a CLI number never becomes SQL text; the queue
/// SQL is a parameter too now: the detail queue and the icon queue share the paging
/// mechanics (v0.6.0).
pub fn pending_plan(sql: String, limit: Option<i64>) -> PendingSql {
    let more: String = if limit.is_some() { " LIMIT ?".to_string() } else { "".to_string() };
    let mut params: Vec<Value> = [].to_vec();
    match limit {
        Some(value) => params.push(Value::Integer(value)),
        None => {},
    }
    return PendingSql { sql: format!("{}{}", sql, more), params: params };
}

/// Coverage of the whole catalogue (grown at v0.8.0): the item aggregate answers in
/// ONE read - SUM() columns carry CASTs because rusqlite reads rows by DECLARED column
/// type and a bare aggregate expression has none (docs/DATABASE.md) - and the price-history
/// count is a SECOND read off the second table: different rows, same Coverage answer,
/// so the CLI, the UI and the audits cannot drift apart. v0.8.0's five new counts carry
/// the manual layer (edited = rows the user stated something about, manual = rows the
/// user MADE, deleted = the tombstones the queries hide) plus the tooltip door's two
/// states, so a completeness answer shows what a pull-all reached AND what a user did
/// on top of it; the price columns moved from read slots 7-9 to 12-14, which is exactly
/// the drift the expected-count guard catches.
pub fn coverage(conn: &Connection) -> AppResult<Coverage> {
    let found: Coverage = match conn.query_row(SQL_COVERAGE, [], |row| {
        Ok(Coverage {
            items: row.get(0)?,
            named: row.get(1)?,
            priced: row.get(2)?,
            detailed: row.get(3)?,
            missing: row.get(4)?,
            icons: row.get(5)?,
            icon_missing: row.get(6)?,
            tooltips: row.get(7)?,
            tooltip_missing: row.get(8)?,
            edited: row.get(9)?,
            manual: row.get(10)?,
            deleted: row.get(11)?,
            history: 0,
            min_price: row.get(12)?,
            max_price: row.get(13)?,
            avg_price: row.get(14)?,
        })
    }) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let samples: i64 = match conn.query_row(SQL_COUNT_SAMPLES, [], |row| row.get(0)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut out: Coverage = found;
    out.history = samples;
    return Ok(out);
}

/// Price statistics grouped by quality label.
pub fn by_quality(conn: &Connection) -> AppResult<Vec<QualityStat>> {
    let mut stmt = match conn.prepare(SQL_QUALITY) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mapped = match stmt.query_map([], |row| {
        Ok(QualityStat {
            quality: row.get(0)?,
            rank: row.get(1)?,
            items: row.get(2)?,
            priced: row.get(3)?,
            avg_price: row.get(4)?,
        })
    }) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut out: Vec<QualityStat> = [].to_vec();
    for item in mapped {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(value) => out.push(value),
        }
    }
    return Ok(out);
}

/// The `limit` cheapest priced items.
pub fn cheapest(conn: &Connection, limit: i64, with_icons: bool) -> AppResult<Vec<Extreme>> {
    return extremes(conn, SQL_CHEAP.to_string(), limit, with_icons);
}

/// The `limit` priciest priced items.
pub fn priciest(conn: &Connection, limit: i64, with_icons: bool) -> AppResult<Vec<Extreme>> {
    return extremes(conn, SQL_RICH.to_string(), limit, with_icons);
}

pub fn extremes(conn: &Connection, sql: String, limit: i64, with_icons: bool) -> AppResult<Vec<Extreme>> {
    let mut params: Vec<Value> = [].to_vec();
    params.push(Value::Integer(limit));
    let mut stmt = match conn.prepare(&sql) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mapped = match stmt.query_map(params_from_iter(params), |row| {
        Ok(Extreme {
            id: row.get(0)?,
            name: row.get(1)?,
            quality: row.get(2)?,
            price_min: row.get(3)?,
            icon_b64: None,
            icon_at: None,
            icon_missing_at: None,
        })
    }) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut out: Vec<Extreme> = [].to_vec();
    for item in mapped {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(value) => out.push(value),
        }
    }
    match with_icons {
        false => {},
        true => {
            let mut index: usize = 0;
            while index < out.len() {
                let one_id: i64 = out[index].id;
                out[index].icon_b64 = match icon_b64_of(conn, one_id) {
                    Err(problem) => return Err(problem),
                    Ok(found) => found,
                };
                out[index].icon_at = match icon_at_of(conn, one_id) {
                    Err(problem) => return Err(problem),
                    Ok(found) => found,
                };
                out[index].icon_missing_at = match icon_missing_at_of(conn, one_id) {
                    Err(problem) => return Err(problem),
                    Ok(found) => found,
                };
                index = index + 1;
            }
        },
    }
    return Ok(out);
}

/// Item ids whose ICON file the icon stage still has to copy, oldest id first.
pub fn icon_pending_ids(conn: &Connection, limit: Option<i64>) -> AppResult<Vec<i64>> {
    let plan: PendingSql = pending_plan(SQL_PENDING_ICON.to_string(), limit);
    let mut stmt = match conn.prepare(&plan.sql) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let rows = match stmt.query_map(params_from_iter(plan.params), |row| row.get(0)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut out: Vec<i64> = [].to_vec();
    for item in rows {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(value) => out.push(value),
        }
    }
    return Ok(out);
}

/// Item ids the detail stage still has to fetch, oldest id first.
pub fn pending_ids(conn: &Connection, limit: Option<i64>) -> AppResult<Vec<i64>> {
    let plan: PendingSql = pending_plan(SQL_PENDING.to_string(), limit);
    let mut stmt = match conn.prepare(&plan.sql) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let rows = match stmt.query_map(params_from_iter(plan.params), |row| row.get(0)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut out: Vec<i64> = [].to_vec();
    for item in rows {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(value) => out.push(value),
        }
    }
    return Ok(out);
}

/// The tooltip stage's queue (v0.8.0, ADR 0010): see SQL_PENDING_TOOLTIP.
pub fn tooltip_pending_ids(conn: &Connection, limit: Option<i64>) -> AppResult<Vec<i64>> {
    let plan: PendingSql = pending_plan(SQL_PENDING_TOOLTIP.to_string(), limit);
    let mut stmt = match conn.prepare(&plan.sql) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let rows = match stmt.query_map(params_from_iter(plan.params), |row| row.get(0)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut out: Vec<i64> = [].to_vec();
    for item in rows {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(value) => out.push(value),
        }
    }
    return Ok(out);
}
