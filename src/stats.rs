//! Coverage and price analytics over the local DB.
//!
//! Every number here comes from one SQL aggregate, so the CLI, the UI and the test
//! suite read the same figures instead of three implementations of the same maths.
//! NULL prices are counted separately and never averaged in as zero.

use crate::errors::AppError;
use crate::errors::AppResult;

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
}

/// One aggregate row over the whole catalogue. `AVG`/`MIN`/`MAX` over an empty
/// column is NULL, which is why those three fields are optional.
pub const SQL_COVERAGE: &str = concat!(
    "SELECT COUNT(1), ",
    "CAST(IFNULL(SUM(name IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(price_min IS NOT NULL), 0) AS INTEGER), ",
    "CAST(IFNULL(SUM(detail_synced_at IS NOT NULL), 0) AS INTEGER), ",
    "MIN(price_min), MAX(price_min), AVG(price_min) ",
    "FROM item",
);

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
/// The rule is "never fetched", not "no price": an item that was fetched and has
/// a NULL price is fetched-but-not-on-the-AH (docs/DATABASE.md's distinction),
/// and under the old OR price_min rule such items stayed pending forever - the
/// detail run never terminated on the real catalogue, and a pull-all run could
/// never honestly promise "nothing pending" (v0.5.2 patch).
pub const SQL_PENDING: &str = "SELECT id FROM item WHERE detail_synced_at IS NULL ORDER BY id ASC";

struct PendingSql {
    pub sql: String,
    pub params: Vec<Value>,
}

/// The limit is a bound parameter so a CLI number never becomes SQL text.
pub fn pending_plan(limit: Option<i64>) -> PendingSql {
    let more: String = if limit.is_some() { " LIMIT ?".to_string() } else { "".to_string() };
    let mut params: Vec<Value> = [].to_vec();
    match limit {
        Some(value) => params.push(Value::Integer(value)),
        None => {},
    }
    return PendingSql { sql: format!("{}{}", SQL_PENDING, more), params: params };
}

/// Coverage of the whole catalogue.
pub fn coverage(conn: &Connection) -> AppResult<Coverage> {
    match conn.query_row(SQL_COVERAGE, [], |row| {
        Ok(Coverage {
            items: row.get(0)?,
            named: row.get(1)?,
            priced: row.get(2)?,
            detailed: row.get(3)?,
            min_price: row.get(4)?,
            max_price: row.get(5)?,
            avg_price: row.get(6)?,
        })
    }) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => return Ok(value),
    }
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
pub fn cheapest(conn: &Connection, limit: i64) -> AppResult<Vec<Extreme>> {
    return extremes(conn, SQL_CHEAP.to_string(), limit);
}

/// The `limit` priciest priced items.
pub fn priciest(conn: &Connection, limit: i64) -> AppResult<Vec<Extreme>> {
    return extremes(conn, SQL_RICH.to_string(), limit);
}

pub fn extremes(conn: &Connection, sql: String, limit: i64) -> AppResult<Vec<Extreme>> {
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
    return Ok(out);
}

/// Item ids the detail stage still has to fetch, oldest id first.
pub fn pending_ids(conn: &Connection, limit: Option<i64>) -> AppResult<Vec<i64>> {
    let plan: PendingSql = pending_plan(limit);
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
