//! Local queries over the SQLite DB: substring search, filters, sorting and counting.
//!
//! Two rules from docs/DATABASE.md shape everything here: filter values are always
//! bound parameters (never pasted into SQL), and LIKE wildcards coming from user text
//! are escaped, so searching for "100%" cannot match every row. NULL prices sort last
//! instead of pretending to be cheap, which is what a gold-hunting tool needs.

use crate::errors::AppError;
use crate::errors::AppResult;

use crate::db::{icon_at_of, icon_b64_of, icon_missing_at_of, money_of};

use serde::Serialize;

use rusqlite::Connection;
use rusqlite::params_from_iter;
use rusqlite::types::Value;

/// One row of a local search: the fields the UI shows and `search` prints as JSON.
#[derive(Serialize, Clone)]
pub struct Hit {
    pub id: i64,
    pub name: String,
    pub quality: Option<String>,
    pub quality_rank: Option<i64>,
    pub price_min: Option<i64>,
    pub price_market: Option<i64>,
    pub price_available: Option<i64>,
    /// v0.9.4 (ADR 0015): the money words of the TWO copper numbers above, written
    /// by the ONE formatter in `db.rs` at ANSWER TIME - the copper stays the number
    /// scripts and price sorts read, and `price_available` stays a COUNT (measured:
    /// row 25 answers `price_available 12` for 12 auctions at 15000 copper, and a
    /// count is not money). A row the AH has no price for answers `null` beside
    /// `null` - never `0c`, which would read as "this item costs nothing".
    pub price_min_money: Option<String>,
    pub price_market_money: Option<String>,
    pub html_url: Option<String>,
    pub icon_url: Option<String>,
    /// v0.9.3 (ADR 0014): the DB's OWN icon bytes as base64, filled ONLY when the
    /// door was asked with --icons - measured avg 2 327 bytes per copy, about 3.1 K
    /// chars once encoded, so an unbounded hit list would be huge and --limit bounds
    /// it. The two stamps say WHICH no-icon state a row is in (checked as gone
    /// upstream vs never fetched), and `icon_url` above is upstream's address, which
    /// ADR 0008's rule says is never hotlinked - only the bytes in the DB get shown.
    pub icon_b64: Option<String>,
    pub icon_at: Option<i64>,
    pub icon_missing_at: Option<i64>,
}

/// What a local search can filter on. Every part is optional, and the parts are
/// ANDed together.
#[derive(Clone, Serialize)]
pub struct Filter {
    pub text: Option<String>,
    pub scope: String,
    pub quality: Option<String>,
    pub item_type: Option<String>,
    pub max_required_level: Option<i64>,
    pub priced_only: bool,
    pub sort: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Filter with nothing set: everything in the DB, in the default order.
pub fn empty_filter() -> Filter {
    return Filter {
        text: None,
        scope: "name".to_string(),
        quality: None,
        item_type: None,
        max_required_level: None,
        priced_only: false,
        sort: "price".to_string(),
        limit: None,
        offset: None,
    };
}

/// LIKE's escape character for this project. `#` is used instead of a backslash so
/// the pattern survives one round of Rust string literals and one of SQLite's.
pub const LIKE_ESCAPE: &str = "#";

/// Escape the wildcards in user-supplied text.
pub fn escape_like(text: String) -> String {
    let mut out: String = "".to_string();
    for ch in text.chars() {
        if ch == '#' {
            out = out + LIKE_ESCAPE + LIKE_ESCAPE;
        }
        if ch == '%' {
            out = out + LIKE_ESCAPE + "%";
        }
        if ch == '_' {
            out = out + LIKE_ESCAPE + "_";
        }
        if ch != '#' && ch != '%' && ch != '_' {
            out.push(ch);
        }
    }
    return out;
}

/// Columns a search reads back, in the order `search` binds them.
pub const HIT_COLUMNS: &str = "id, name, quality, quality_rank, price_min, price_market, price_available, html_url, icon_url";

/// Default ordering: cheapest first, NULL prices last, ties broken by name so the
/// order is stable across runs.
pub const DEFAULT_ORDER: &str = "ORDER BY (price_min IS NULL) ASC, price_min ASC, name ASC";

/// Order by quality label rank (common .. heirloom), NULL ranks last.
pub const QUALITY_ORDER: &str = "ORDER BY (quality_rank IS NULL) ASC, quality_rank ASC, name COLLATE NOCASE ASC";

/// Order by name, A first.
pub const NAME_ORDER: &str = "ORDER BY name COLLATE NOCASE ASC";

/// Order by item level, high level first, unknown levels last.
pub const LEVEL_ORDER: &str = "ORDER BY (item_level IS NULL) ASC, item_level DESC";

/// Plain id order: the crawl order.
pub const ID_ORDER: &str = "ORDER BY id ASC";

/// Order by the LIVE min buyout (v0.9.9, ADR 0020): the price the price stage
/// saw at its ask time, so this order reads what the AH offers NOW, not what
// the DB once stored. Rows with NULL here — never refreshed OR checked price-less
// at ask time — sort LAST exactly like every other price sort (the two kinds of
// NULL stay apart in their own stamps, the sort does not care and does not guess);
// ties broken by name so the order is stable across runs.
pub const NOW_ORDER: &str = "ORDER BY (price_min_now IS NULL) ASC, price_min_now ASC, name ASC";

/// Sort keys the CLI and the UI pass around; unknown keys fall back to the default.
pub fn order_for(sort: String) -> String {
    if sort == "quality" {
        return QUALITY_ORDER.to_string();
    }
    if sort == "name" {
        return NAME_ORDER.to_string();
    }
    if sort == "level" {
        return LEVEL_ORDER.to_string();
    }
    if sort == "id" {
        return ID_ORDER.to_string();
    }
    if sort == "price-now" {
        return NOW_ORDER.to_string();
    }
    return DEFAULT_ORDER.to_string();
}

/// SQL fragments for each filter part. The bound values are pushed by `where_parts`
/// in the same order these clauses appear.
/// Where a text search looks (v0.9.0, ADR 0012): the upstream search door was
/// measured NAMES-ONLY (docs/API.md: "tooltip" answers empty groups even though
/// the tooltip carries the word), so the local door grew the honest superset -
/// names (the default: what the UI always did), descriptions (the readable twin
/// of the copied tooltips) or anywhere. Tombstoned rows stay invisible either
/// way: `visible_clause` is pushed FIRST, before any scope clause.
pub fn text_clauses(scope: String) -> String {
    match scope.as_str() {
        "description" => {
            return format!("(tooltip_lc IS NOT NULL AND tooltip_lc LIKE ? ESCAPE '{}')", LIKE_ESCAPE);
        },
        "all" => {
            let like: String = format!("LIKE ? ESCAPE '{}'", LIKE_ESCAPE);
            return format!("(name_lc {} OR (tooltip_lc IS NOT NULL AND tooltip_lc {}))", like, like);
        },
        _ => {},
    }
    return format!("name_lc LIKE ? ESCAPE '{}'", LIKE_ESCAPE);
}

/// How many needles the scope clause of `text_clauses` binds (one, one, two).
pub fn text_needles(scope: String) -> usize {
    match scope.as_str() {
        "all" => return 2,
        _ => {},
    }
    return 1;
}

pub fn quality_clause() -> String {
    return "quality = ?".to_string();
}

pub fn type_clause() -> String {
    return "item_type = ?".to_string();
}

pub fn level_clause() -> String {
    return "required_level <= ?".to_string();
}

pub fn priced_clause() -> String {
    return "price_min IS NOT NULL".to_string();
}

/// A prepared query: the SQL text plus the values bound into its placeholders.
pub struct Plan {
    pub sql: String,
    pub params: Vec<Value>,
}

/// Turn a Filter into the WHERE chain and the bound values for it.
pub fn where_parts(filter: Filter) -> Plan {
    let mut clauses: Vec<String> = [].to_vec();
    let mut params: Vec<Value> = [].to_vec();
    clauses.push(visible_clause());
    match filter.text {
        Some(text) => {
            clauses.push(text_clauses(filter.scope.clone()));
            let needle: String = format!("%{}%", escape_like(text.to_lowercase()));
            let want: usize = text_needles(filter.scope.clone());
            let mut seen: usize = 0;
            while seen < want {
                params.push(Value::Text(needle.clone()));
                seen = seen + 1
            }
        },
        None => {},
    }
    match filter.quality {
        Some(text) => {
            clauses.push(quality_clause());
            params.push(Value::Text(text));
        },
        None => {},
    }
    match filter.item_type {
        Some(text) => {
            clauses.push(type_clause());
            params.push(Value::Text(text));
        },
        None => {},
    }
    match filter.max_required_level {
        Some(value) => {
            clauses.push(level_clause());
            params.push(Value::Integer(value));
        },
        None => {},
    }
    if filter.priced_only {
        clauses.push(priced_clause());
    }
    if clauses.is_empty() {
        return Plan { sql: "".to_string(), params: params };
    }
    let joined: String = clauses.join(" AND ");
    return Plan { sql: format!("WHERE {} ", joined), params: params };
}

/// One full SELECT for a search: the WHERE chain, the ORDER BY and paging.
pub fn search_plan(filter: Filter) -> Plan {
    let head: Plan = where_parts(filter.clone());
    let mut sql: String = format!("SELECT {} FROM item ", HIT_COLUMNS);
    sql = format!("{}{}", sql, head.sql);
    sql = format!("{}{}{}", sql, order_for(filter.sort), " ");
    let mut params: Vec<Value> = [].to_vec();
    params.extend(head.params);
    match filter.limit {
        Some(value) => {
            sql = sql + "LIMIT ? ";
            params.push(Value::Integer(value));
        },
        None => {},
    }
    match filter.offset {
        Some(value) => {
            sql = sql + "OFFSET ? ";
            params.push(Value::Integer(value));
        },
        None => {},
    }
    return Plan { sql: sql, params: params };
}

/// Rows the user deleted are invisible to EVERY local query (ADR 0010's tombstone):
/// one clause, pushed first, so `search`, its `--limit` count and the card agree.
pub fn visible_clause() -> String {
    return "deleted_at IS NULL".to_string();
}

/// How many rows a filter matches: the same WHERE chain as `search_plan`, so the
/// UI can show "showing 40 of 1250" without a second, drifting query.
pub fn count_plan(filter: Filter) -> Plan {
    let head: Plan = where_parts(filter.clone());
    let sql: String = format!("SELECT COUNT(1) FROM item {}", head.sql);
    return Plan { sql: sql, params: head.params };
}

/// Run a search and read every hit. `with_icons` is the door's `--icons` flag
/// (v0.9.3, ADR 0014): it asks for the DB's own icon bytes per hit and sends
/// NOTHING to the network - the rows are COLLECTED first, then each row's three
/// icon columns are read beside it, so a hit list cannot straddle a write.
/// `HIT_COLUMNS` grows NOTHING for the money pair (v0.9.4, ADR 0015): money is
/// DERIVED from the copper this read already carries, so there is no money column
/// to keep in step and the sort keys stay the copper columns.
pub fn search(conn: &Connection, filter: Filter, with_icons: bool) -> AppResult<Vec<Hit>> {
    let plan: Plan = search_plan(filter);
    let mut stmt = match conn.prepare(&plan.sql) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mapped = match stmt.query_map(params_from_iter(plan.params), |row|
        Ok(Hit {
            id: row.get(0)?,
            name: row.get(1)?,
            quality: row.get(2)?,
            quality_rank: row.get(3)?,
            price_min: row.get(4)?,
            price_market: row.get(5)?,
            price_available: row.get(6)?,
            price_min_money: money_of(row.get(4)?),
            price_market_money: money_of(row.get(5)?),
            html_url: row.get(7)?,
            icon_url: row.get(8)?,
            icon_b64: None,
            icon_at: None,
            icon_missing_at: None,
        })
    ) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut hits: Vec<Hit> = [].to_vec();
    for item in mapped {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(value) => hits.push(value),
        }
    }
    match with_icons {
        false => {},
        true => {
            let mut index: usize = 0;
            while index < hits.len() {
                let one_id: i64 = hits[index].id;
                hits[index].icon_b64 = match icon_b64_of(conn, one_id) {
                    Err(problem) => return Err(problem),
                    Ok(found) => found,
                };
                hits[index].icon_at = match icon_at_of(conn, one_id) {
                    Err(problem) => return Err(problem),
                    Ok(found) => found,
                };
                hits[index].icon_missing_at = match icon_missing_at_of(conn, one_id) {
                    Err(problem) => return Err(problem),
                    Ok(found) => found,
                };
                index = index + 1;
            }
        },
    }
    return Ok(hits);
}

/// Count the rows a filter matches.
pub fn count_matching(conn: &Connection, filter: Filter) -> AppResult<i64> {
    let plan: Plan = count_plan(filter);
    match conn.query_row(&plan.sql, params_from_iter(plan.params), |row| row.get(0)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => return Ok(value),
    }
}
