//! SQLite bootstrap: schema, db-path resolution, upserts.
//!
//! One disposable file: deleting it only throws away cache, the next sync rebuilds it.
//! The schema is idempotent, so every entry point (CLI or GUI) can attach to a fresh
//! process. Stage-1 (list) and stage-2 (detail) writers only ever touch their own
//! columns, so a half-finished crawl stays browsable.
//!
//! This module never touches the network (docs/DEVELOPMENT.md house rules): `sync.rs`
//! is where HTTP and storage meet.

use crate::errors::AppError;
use crate::errors::AppResult;

use crate::models::ItemSummary;
use crate::models::ItemDetail;
use crate::models::Price;
use crate::models::Weapon;
use crate::models::Links;

use rusqlite::Connection;
use rusqlite::params_from_iter;
use rusqlite::types::Value;

use serde_json::to_string;

use serde::Serialize;

use std::env::var;
use std::fs::create_dir_all;
use std::fs::exists;
use std::time::UNIX_EPOCH;

/// The schema this build understands. Bumping it means the DB file is rebuilt.
pub const SCHEMA_VERSION: i64 = 1;

/// Default DB file name, used when WGT_DB does not point somewhere else.
pub const DB_FILE_NAME: &str = "wow-goldtool.sqlite";

/// Environment switch: tests and CI point the DB at a fixture.
pub const ENV_DB_KEY: &str = "WGT_DB";

/// Fallback data directory, relative to HOME: .local/share/wow-goldtool.
pub const FALLBACK_ROOT: &str = ".local";
pub const FALLBACK_SHARE: &str = "share";
pub const FALLBACK_SUBDIR: &str = "wow-goldtool";

/// Schema as one batch. SQLite's own statement splitter handles the newlines, and
/// every table is small enough that a rebuild is cheaper than migration.
pub const SCHEMA_SQL: &str = concat!(
    "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);",
    "CREATE TABLE item (",
    "id INTEGER PRIMARY KEY,",
    "name TEXT, name_lc TEXT,",
    "quality TEXT, quality_rank INTEGER,",
    "item_level INTEGER, required_level INTEGER,",
    "item_type TEXT, item_subtype TEXT, slot TEXT,",
    "item_set TEXT, unique_flag TEXT, flavor TEXT,",
    "icon_url TEXT,",
    "sockets TEXT, socket_bonus TEXT, classes TEXT, stats TEXT, effects TEXT,",
    "faction_prices TEXT,",
    "dmg_min INTEGER, dmg_max INTEGER, dps REAL, speed REAL,",
    "vendor_sell INTEGER,",
    "price_min INTEGER, price_market INTEGER, price_available INTEGER,",
    "price_realms INTEGER, price_region TEXT, price_scope TEXT,",
    "price_updated_at TEXT,",
    "api_url TEXT, html_url TEXT, tooltip_url TEXT,",
    "list_etag TEXT, detail_etag TEXT,",
    "list_synced_at INTEGER, detail_synced_at INTEGER,",
    "raw_json TEXT",
    ");",
    "CREATE TABLE watch (id INTEGER PRIMARY KEY, added_at INTEGER, note TEXT);",
    "CREATE TABLE sync_state (",
    "kind TEXT PRIMARY KEY, status TEXT,",
    "page INTEGER, done INTEGER, total INTEGER,",
    "message TEXT, updated_at INTEGER, error TEXT",
    ");",
    "CREATE INDEX item__name_lc ON item (name_lc);",
    "CREATE INDEX item__rank ON item (quality_rank, item_level);",
    "CREATE INDEX item__price ON item (price_min);",
    "CREATE INDEX item__detail ON item (detail_synced_at);",
);

/// How many VALUES markers the stage-1 upsert has (its SET list is the same minus the
/// primary key). bind_sql() checks the count so SQL and Rust cannot drift apart.
pub const LIST_VALUE_COUNT: usize = 12;

pub const UPSERT_LIST_SQL: &str = concat!(
    "INSERT INTO item (id, name, name_lc, quality, quality_rank, item_level, required_level, api_url, html_url, tooltip_url, list_etag, list_synced_at)",
    " VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
    " ON CONFLICT (id) DO UPDATE SET name = ?, name_lc = ?, quality = ?, quality_rank = ?, item_level = ?, required_level = ?, api_url = ?, html_url = ?, tooltip_url = ?, list_etag = ?, list_synced_at = ?",
);

/// VALUES markers of the stage-2 upsert: it writes every summary column too (stage 1
/// may not have seen this id at all) plus the price, stat and detail columns.
pub const DETAIL_VALUE_COUNT: usize = 38;

pub const UPSERT_DETAIL_SQL: &str = concat!(
    "INSERT INTO item (id, name, name_lc, quality, quality_rank, item_level, required_level, item_type, item_subtype, slot, item_set, unique_flag, flavor, icon_url, sockets, socket_bonus, classes, stats, effects, faction_prices, dmg_min, dmg_max, dps, speed, vendor_sell, price_min, price_market, price_available, price_realms, price_region, price_scope, price_updated_at, api_url, html_url, tooltip_url, detail_etag, detail_synced_at, raw_json)",
    " VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    " ON CONFLICT (id) DO UPDATE SET name = ?, name_lc = ?, quality = ?, quality_rank = ?, item_level = ?, required_level = ?, item_type = ?, item_subtype = ?, slot = ?, item_set = ?, unique_flag = ?, flavor = ?, icon_url = ?, sockets = ?, socket_bonus = ?, classes = ?, stats = ?, effects = ?, faction_prices = ?, dmg_min = ?, dmg_max = ?, dps = ?, speed = ?, vendor_sell = ?, price_min = ?, price_market = ?, price_available = ?, price_realms = ?, price_region = ?, price_scope = ?, price_updated_at = ?, api_url = ?, html_url = ?, tooltip_url = ?, detail_etag = ?, detail_synced_at = ?, raw_json = ?",
);

/// Upserts over a primary key all share one shape: N VALUES markers, then the same
/// values minus the key in the SET list. These counts are what bind_sql() demands.
pub const META_VALUE_COUNT: usize = 2;
pub const SYNC_VALUE_COUNT: usize = 8;
pub const WATCH_VALUE_COUNT: usize = 3;

pub const SQL_SET_META: &str = "INSERT INTO meta (key, value) VALUES (?,?) ON CONFLICT (key) DO UPDATE SET value = ?";
pub const SQL_GET_META: &str = "SELECT value FROM meta WHERE key = ?";
pub const SQL_SYNC_GET: &str = "SELECT status, page, done, total, message, updated_at, error FROM sync_state WHERE kind = ?";
pub const SQL_SYNC_SET: &str = "INSERT INTO sync_state (kind, status, page, done, total, message, updated_at, error) VALUES (?,?,?,?,?,?,?,?) ON CONFLICT (kind) DO UPDATE SET status = ?, page = ?, done = ?, total = ?, message = ?, updated_at = ?, error = ?";
pub const SQL_WATCH_ADD: &str = "INSERT INTO watch (id, added_at, note) VALUES (?,?,?) ON CONFLICT (id) DO UPDATE SET added_at = ?, note = ?";
pub const SQL_WATCH_DROP: &str = "DELETE FROM watch WHERE id = ?";
pub const SQL_WATCH_LIST: &str = "SELECT id, note FROM watch ORDER BY id ASC";
pub const SQL_COUNT_ROWS: &str = "SELECT COUNT(1) FROM item";

/// One TEXT column of one item row, by id: the live lookup's ETag reuse reads the
/// stored copy when the API answers 304 (docs/API.md, docs/DATABASE.md).
pub const SQL_DETAIL_ETAG_OF: &str = "SELECT detail_etag FROM item WHERE id = ?";
pub const SQL_RAW_JSON_OF: &str = "SELECT raw_json FROM item WHERE id = ?";
pub const SQL_SYNCED_AT_OF: &str = "SELECT detail_synced_at FROM item WHERE id = ?";

/// Where the SQLite file lives and why: the CLI prints both so a user always knows
/// which file they are looking at.
#[derive(Serialize, Clone)]
pub struct DbTarget {
    pub path: String,
    pub source: String,
}

/// Resolve the DB path. Three resolutions, first hit wins:
///
/// 1. WGT_DB env var (tests and CI point this at a fixture),
/// 2. the Tauri appDataDir once the GUI is wired (passed in as app_data_dir),
/// 3. HOME/.local/share/wow-goldtool, created here because nothing else will.
pub fn db_path(app_data_dir: Option<String>) -> DbTarget {
    let configured = var(ENV_DB_KEY);
    match configured {
        Ok(value) => return DbTarget { path: value, source: "env".to_string() },
        Err(_) => {},
    }
    match app_data_dir {
        Some(dir) => {
            let full: String = format!("{}{}{}", dir, "/", DB_FILE_NAME);
            return DbTarget { path: full, source: "app-data-dir".to_string() };
        },
        None => {},
    }
    let home: String = match var("HOME") {
        Ok(value) => value,
        Err(_) => ".".to_string(),
    };
    let dir: String = format!("{}{}{}{}{}{}{}", home, "/", FALLBACK_ROOT, "/", FALLBACK_SHARE, "/", FALLBACK_SUBDIR);
    match create_dir_all(&dir) {
        Err(problem) => return DbTarget { path: DB_FILE_NAME.to_string(), source: format!("{}", problem) },
        Ok(_) => {},
    }
    let full: String = format!("{}{}{}", dir, "/", DB_FILE_NAME);
    return DbTarget { path: full, source: "data-dir".to_string() };
}

/// Unix seconds, used for every sync bookkeeping timestamp.
pub fn unix_now() -> i64 {
    match UNIX_EPOCH.elapsed() {
        Ok(span) => return span.as_secs().try_into().unwrap(),
        Err(_) => return 0,
    }
}

/// Quality label to sort rank: 1 common .. 7 artifact/heirloom. Unknown labels keep
/// NULL so the UI can still show the raw label.
pub fn rank_of(quality: Option<String>) -> Option<i64> {
    match quality {
        None => return None,
        Some(text) => {
            if text == "Common" {
                return Some(1);
            }
            if text == "Uncommon" {
                return Some(2);
            }
            if text == "Rare" {
                return Some(3);
            }
            if text == "Epic" {
                return Some(4);
            }
            if text == "Legendary" {
                return Some(5);
            }
            if text == "Artifact" {
                return Some(6);
            }
            if text == "Heirloom" {
                return Some(7);
            }
            return None;
        },
    }
}

/// The SQLite value for a text column: absent means NULL, never the empty string.
pub fn text_value(opt: Option<String>) -> Value {
    match opt {
        None => return Value::Null,
        Some(text) => return Value::Text(text),
    }
}

pub fn int_value(opt: Option<i64>) -> Value {
    match opt {
        None => return Value::Null,
        Some(value) => return Value::Integer(value),
    }
}

/// Levels come off the API as 32-bit numbers; SQLite only speaks i64 here.
pub fn level_value(opt: Option<i32>) -> Value {
    match opt {
        None => return Value::Null,
        Some(value) => return Value::Integer(value as i64),
    }
}

pub fn num_value(opt: Option<f64>) -> Value {
    match opt {
        None => return Value::Null,
        Some(value) => return Value::Real(value),
    }
}

/// JSON-shaped columns (sockets, classes, stats, effects, faction prices) are stored
/// as their compact text form, so a future schema bump can still read them.
pub fn json_value(opt: Option<serde_json::Value>) -> Value {
    match opt {
        None => return Value::Null,
        Some(payload) => match to_string(&payload) {
            Ok(text) => return Value::Text(text),
            Err(_) => return Value::Null,
        },
    }
}

/// VALUES list plus the SET list: every value once for the insert, then the same
/// values again minus the primary key for the update. Keeping the two halves in one
/// place is what makes the param-count check below meaningful.
pub fn upsert_params(head: Vec<Value>) -> Vec<Value> {
    let mut params: Vec<Value> = [].to_vec();
    params.extend(head.clone());
    let mut again: Vec<Value> = head;
    again.remove(0);
    params.extend(again);
    return params;
}

/// Run one statement with a bound parameter list. Values are always bound, never
/// interpolated into the SQL (docs/DATABASE.md rules), and the expected count is
/// checked so editing a SQL constant without its Rust twin fails loudly.
pub fn bind_sql(conn: &Connection, sql: &str, params: Vec<Value>, expected: usize) -> AppResult<usize> {
    if params.len() != expected {
        return Err(AppError::db(format!("sqlite param mismatch: {} bound, {} wanted", params.len(), expected)));
    }
    match conn.execute(sql, params_from_iter(params)) {
        Ok(count) => return Ok(count),
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
    }
}

/// Open the DB file and make sure the schema is on disk. Every entry point (CLI
/// subcommand, Tauri command) goes through this.
pub fn open_db(app_data_dir: Option<String>) -> AppResult<Connection> {
    let target: DbTarget = db_path(app_data_dir);
    let already_there: bool = match exists(&target.path) {
        Ok(value) => value,
        Err(_) => false,
    };
    match Connection::open(&target.path) {
        Err(problem) => return Err(AppError::db(format!("cannot open {}: {}", target.path, problem))),
        Ok(conn) => {
            if already_there {
                match check_schema(&conn) {
                    Err(problem) => return Err(problem),
                    Ok(_) => return Ok(conn),
                }
            }
            match conn.execute_batch(SCHEMA_SQL) {
                Err(problem) => return Err(AppError::db(format!("schema rejected: {}", problem))),
                Ok(_) => {},
            }
            match meta_set(&conn, "schema_version".to_string(), "1".to_string()) {
                Err(problem) => return Err(problem),
                Ok(_) => return Ok(conn),
            }
        },
    }
}

/// Open the DB file at an explicit path: the CLI's `--db` flag and the test
/// harness hand this a path, the GUI hands it app_data_dir. Same schema
/// rules as `open_db`, just without the path resolution.
pub fn open_db_at(path: String) -> AppResult<Connection> {
    let already_there: bool = match exists(&path) {
        Ok(value) => value,
        Err(_) => false,
    };
    match Connection::open(&path) {
        Err(problem) => return Err(AppError::db(format!("cannot open {}: {}", path, problem))),
        Ok(conn) => {
            if already_there {
                match check_schema(&conn) {
                    Err(problem) => return Err(problem),
                    Ok(_) => return Ok(conn),
                }
            }
            match conn.execute_batch(SCHEMA_SQL) {
                Err(problem) => return Err(AppError::db(format!("schema rejected: {}", problem))),
                Ok(_) => {},
            }
            match meta_set(&conn, "schema_version".to_string(), "1".to_string()) {
                Err(problem) => return Err(problem),
                Ok(_) => return Ok(conn),
            }
        },
    }
}

/// Refuse a foreign or stale-schema file rather than half-applying a new schema.
pub fn check_schema(conn: &Connection) -> AppResult<()> {
    let found: Option<String> = match meta_get(conn, "schema_version".to_string()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match found {
        None => return Err(AppError::db("that file is not a wow-goldtool database; delete it to rebuild".to_string())),
        Some(text) => {
            let version: i64 = match text.parse() {
                Err(_) => 0,
                Ok(value) => value,
            };
            if version != SCHEMA_VERSION {
                return Err(AppError::db(format!("database on disk uses schema {}, this build wants {}; delete the file to rebuild it", version, SCHEMA_VERSION)));
            }
            return Ok(());
        },
    };
}

pub fn meta_get(conn: &Connection, key: String) -> AppResult<Option<String>> {
    let mut stmt = match conn.prepare(SQL_GET_META) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Text(key));
    let mut rows = match stmt.query(params_from_iter(single)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    match rows.next() {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(None) => return Ok(None),
        Ok(Some(row)) => return match row.get(0) {
            Err(problem) => Err(AppError::db(format!("{}", problem))),
            Ok(value) => Ok(Some(value)),
        },
    };
}

pub fn meta_set(conn: &Connection, key: String, value: String) -> AppResult<usize> {
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Text(key));
    head.push(Value::Text(value));
    return bind_sql(conn, SQL_SET_META, upsert_params(head), META_VALUE_COUNT * 2 - 1);
}

pub fn detail_etag_of(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let mut stmt = match conn.prepare(SQL_DETAIL_ETAG_OF) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Integer(id));
    let mut rows = match stmt.query(params_from_iter(single)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    match rows.next() {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(None) => return Ok(None),
        Ok(Some(row)) => return match row.get(0) {
            Err(problem) => Err(AppError::db(format!("{}", problem))),
            Ok(found) => Ok(found),
        },
    }
}

pub fn raw_json_of(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let mut stmt = match conn.prepare(SQL_RAW_JSON_OF) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Integer(id));
    let mut rows = match stmt.query(params_from_iter(single)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    match rows.next() {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(None) => return Ok(None),
        Ok(Some(row)) => return match row.get(0) {
            Err(problem) => Err(AppError::db(format!("{}", problem))),
            Ok(value) => Ok(Some(value)),
        },
    }
}

#[derive(Serialize, Clone)]
pub struct SyncState {
    pub kind: String,
    pub status: String,
    pub page: Option<i64>,
    pub done: Option<i64>,
    pub total: Option<i64>,
    pub message: Option<String>,
    pub updated_at: Option<i64>,
    pub error: Option<String>,
}

pub fn sync_state_get(conn: &Connection, kind: String) -> AppResult<SyncState> {
    let mut stmt = match conn.prepare(SQL_SYNC_GET) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut params: Vec<Value> = [].to_vec();
    params.push(Value::from(Some(kind.clone())));
    let mut rows = match stmt.query(params_from_iter(params)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    match rows.next() {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(None) => return Ok(SyncState {
            kind: kind,
            status: "idle".to_string(),
            page: None,
            done: None,
            total: None,
            message: None,
            updated_at: None,
            error: None,
        }),
        Ok(Some(row)) => {
            let status: Option<String> = match row.get(0) {
                Err(_) => None,
                Ok(value) => value,
            };
            let page: Option<i64> = match row.get(1) {
                Err(_) => None,
                Ok(value) => value,
            };
            let done: Option<i64> = match row.get(2) {
                Err(_) => None,
                Ok(value) => value,
            };
            let total: Option<i64> = match row.get(3) {
                Err(_) => None,
                Ok(value) => value,
            };
            let message: Option<String> = match row.get(4) {
                Err(_) => None,
                Ok(value) => value,
            };
            let updated_at: Option<i64> = match row.get(5) {
                Err(_) => None,
                Ok(value) => value,
            };
            let problem: Option<String> = match row.get(6) {
                Err(_) => None,
                Ok(value) => value,
            };
            return Ok(SyncState {
                kind: kind,
                status: match status {
                    Some(text) => text,
                    None => "idle".to_string(),
                },
                page: page,
                done: done,
                total: total,
                message: message,
                updated_at: updated_at,
                error: problem,
            });
        },
    }
}

pub fn sync_state_set(conn: &Connection, state: SyncState) -> AppResult<usize> {
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Text(state.kind));
    head.push(text_value(Some(state.status)));
    head.push(int_value(state.page));
    head.push(int_value(state.done));
    head.push(int_value(state.total));
    head.push(text_value(state.message));
    head.push(int_value(state.updated_at));
    head.push(text_value(state.error));
    return bind_sql(conn, SQL_SYNC_SET, upsert_params(head), SYNC_VALUE_COUNT * 2 - 1);
}

pub fn watch_add(conn: &Connection, id: i64, note: Option<String>) -> AppResult<usize> {
    let now: i64 = unix_now();
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(id));
    head.push(int_value(Some(now)));
    head.push(text_value(note));
    return bind_sql(conn, SQL_WATCH_ADD, upsert_params(head), WATCH_VALUE_COUNT * 2 - 1);
}

pub fn watch_drop(conn: &Connection, id: i64) -> AppResult<usize> {
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Integer(id));
    return bind_sql(conn, SQL_WATCH_DROP, single, 1);
}

/// When the detail record was last fetched (NULL means never). The item card prints it
/// so a user can tell today's prices from last month's without leaving the app.
pub fn synced_at_of(conn: &Connection, id: i64) -> AppResult<Option<i64>> {
    let mut stmt = match conn.prepare(SQL_SYNCED_AT_OF) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Integer(id));
    let mut rows = match stmt.query(params_from_iter(single)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    match rows.next() {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(None) => return Ok(None),
        Ok(Some(row)) => return match row.get(0) {
            Err(problem) => Err(AppError::db(format!("{}", problem))),
            Ok(found) => Ok(found),
        },
    }
}

pub fn watch_list(conn: &Connection) -> AppResult<Vec<i64>> {
    let mut stmt = match conn.prepare(SQL_WATCH_LIST) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let rows = match stmt.query_map([], |row| row.get(0)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut ids: Vec<i64> = [].to_vec();
    for item in rows {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(value) => ids.push(value),
        }
    }
    return Ok(ids);
}

pub fn count_items(conn: &Connection) -> AppResult<i64> {
    match conn.query_row(SQL_COUNT_ROWS, [], |row| row.get(0)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => return Ok(value),
    }
}

/// Read one field out of an optional struct: the API omits plenty of them, and an
/// omitted field must land as SQL NULL rather than as an empty string.
pub fn field_api(links: Option<Links>) -> Option<String> {
    match links {
        None => return None,
        Some(value) => return value.api,
    }
}

pub fn field_html(links: Option<Links>) -> Option<String> {
    match links {
        None => return None,
        Some(value) => return value.html,
    }
}

pub fn field_tooltip(links: Option<Links>) -> Option<String> {
    match links {
        None => return None,
        Some(value) => return value.tooltip,
    }
}

pub fn price_min(price: Option<Price>) -> Option<i64> {
    match price {
        None => return None,
        Some(value) => return value.min,
    }
}

pub fn price_market(price: Option<Price>) -> Option<i64> {
    match price {
        None => return None,
        Some(value) => return value.market,
    }
}

pub fn price_available(price: Option<Price>) -> Option<i64> {
    match price {
        None => return None,
        Some(value) => return value.available,
    }
}

pub fn price_realms(price: Option<Price>) -> Option<i64> {
    match price {
        None => return None,
        Some(value) => return value.realms,
    }
}

pub fn price_region(price: Option<Price>) -> Option<String> {
    match price {
        None => return None,
        Some(value) => return value.region,
    }
}

pub fn price_scope(price: Option<Price>) -> Option<String> {
    match price {
        None => return None,
        Some(value) => return value.scope,
    }
}

pub fn price_updated_at(price: Option<Price>) -> Option<String> {
    match price {
        None => return None,
        Some(value) => return value.updated_at,
    }
}

pub fn weapon_min(weapon: Option<Weapon>) -> Option<i64> {
    match weapon {
        None => return None,
        Some(value) => return value.damage_min,
    }
}

pub fn weapon_max(weapon: Option<Weapon>) -> Option<i64> {
    match weapon {
        None => return None,
        Some(value) => return value.damage_max,
    }
}

pub fn weapon_dps(weapon: Option<Weapon>) -> Option<f64> {
    match weapon {
        None => return None,
        Some(value) => return value.dps,
    }
}

pub fn weapon_speed(weapon: Option<Weapon>) -> Option<f64> {
    match weapon {
        None => return None,
        Some(value) => return value.speed,
    }
}

/// Empty strings from the API (or a missing ETag) become NULL, so "no value" and
/// "empty value" do not split into two representations in the DB.
pub fn optional_text(text: String) -> Option<String> {
    if text == "" {
        return None;
    }
    return Some(text);
}

/// Stage 1 write: one row of a list page. Only summary columns are touched, so the
/// stage-2 write can fill price and detail columns later without fighting this write.
pub fn upsert_summary(conn: &Connection, summary: ItemSummary, etag: String, synced_at: i64) -> AppResult<usize> {
    let name_lc: String = summary.name.to_lowercase();
    let rank: Option<i64> = rank_of(summary.quality.clone());
    let links: Option<Links> = summary.links;
    let api_url: Option<String> = field_api(links.clone());
    let html_url: Option<String> = field_html(links.clone());
    let tooltip_url: Option<String> = field_tooltip(links);
    let stamp: Option<i64> = Some(synced_at);
    let etag_text: Option<String> = optional_text(etag);
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(summary.id));
    head.push(text_value(Some(summary.name)));
    head.push(text_value(Some(name_lc)));
    head.push(text_value(summary.quality));
    head.push(int_value(rank));
    head.push(level_value(summary.item_level));
    head.push(level_value(summary.required_level));
    head.push(text_value(api_url));
    head.push(text_value(html_url));
    head.push(text_value(tooltip_url));
    head.push(text_value(etag_text));
    head.push(int_value(stamp));
    return bind_sql(conn, UPSERT_LIST_SQL, upsert_params(head), LIST_VALUE_COUNT * 2 - 1);
}

/// Stage 2 write: one detail record. `raw_json` keeps the verbatim payload so a future
/// schema bump can read fields this struct does not model yet.
pub fn upsert_detail(conn: &Connection, detail: ItemDetail, etag: String, raw_json: String, synced_at: i64) -> AppResult<usize> {
    let name_lc: String = detail.name.to_lowercase();
    let rank: Option<i64> = rank_of(detail.quality.clone());
    let links: Option<Links> = detail.links;
    let api_url: Option<String> = field_api(links.clone());
    let html_url: Option<String> = field_html(links.clone());
    let tooltip_url: Option<String> = field_tooltip(links);
    // Price and Weapon are read field by field, so each read takes a cheap copy.
    let price: Option<Price> = detail.price;
    let p_min: Option<i64> = price_min(price.clone());
    let p_market: Option<i64> = price_market(price.clone());
    let p_available: Option<i64> = price_available(price.clone());
    let p_realms: Option<i64> = price_realms(price.clone());
    let p_region: Option<String> = price_region(price.clone());
    let p_scope: Option<String> = price_scope(price.clone());
    let p_updated: Option<String> = price_updated_at(price);
    let weapon: Option<Weapon> = detail.weapon;
    let w_min: Option<i64> = weapon_min(weapon.clone());
    let w_max: Option<i64> = weapon_max(weapon.clone());
    let w_dps: Option<f64> = weapon_dps(weapon.clone());
    let w_speed: Option<f64> = weapon_speed(weapon);
    let etag_text: Option<String> = optional_text(etag);
    let stamp: Option<i64> = Some(synced_at);
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(detail.id));
    head.push(text_value(Some(detail.name)));
    head.push(text_value(Some(name_lc)));
    head.push(text_value(detail.quality));
    head.push(int_value(rank));
    head.push(level_value(detail.item_level));
    head.push(level_value(detail.required_level));
    head.push(text_value(detail.item_type));
    head.push(text_value(detail.item_subtype));
    head.push(text_value(detail.slot));
    head.push(text_value(detail.item_set));
    head.push(text_value(detail.unique));
    head.push(text_value(detail.flavor));
    head.push(text_value(detail.icon_url));
    head.push(json_value(detail.sockets));
    head.push(text_value(detail.socket_bonus));
    head.push(json_value(detail.classes));
    head.push(json_value(detail.stats));
    head.push(json_value(detail.effects));
    head.push(json_value(detail.faction_prices));
    head.push(int_value(w_min));
    head.push(int_value(w_max));
    head.push(num_value(w_dps));
    head.push(num_value(w_speed));
    head.push(int_value(detail.vendor_sell));
    head.push(int_value(p_min));
    head.push(int_value(p_market));
    head.push(int_value(p_available));
    head.push(int_value(p_realms));
    head.push(text_value(p_region));
    head.push(text_value(p_scope));
    head.push(text_value(p_updated));
    head.push(text_value(api_url));
    head.push(text_value(html_url));
    head.push(text_value(tooltip_url));
    head.push(text_value(etag_text));
    head.push(int_value(stamp));
    head.push(text_value(Some(raw_json)));
    return bind_sql(conn, UPSERT_DETAIL_SQL, upsert_params(head), DETAIL_VALUE_COUNT * 2 - 1);
}
