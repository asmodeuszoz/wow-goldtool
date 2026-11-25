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
use crate::models::ManualEdit;
use crate::models::RecipeEntry;
use crate::models::Reagent;

use rusqlite::Connection;
use rusqlite::params_from_iter;
use rusqlite::types::Value;

use serde_json::to_string;
use serde_json::from_str;

use serde::Serialize;

use std::env::var;
use std::fs::create_dir_all;
use std::fs::exists;
use std::time::UNIX_EPOCH;

/// The schema this build understands. v0.6.0 adds three columns, one table and one
/// index; the bump is paired with an in-place migration for BOTH older shapes - v1
/// files chain through v2 to this shape - because the user's own catalogue sits on
/// disk at v1 and refusing it would throw away 19 258 rows. A file NEWER than this
/// build is still refused: we cannot know what it holds.
pub const SCHEMA_VERSION: i64 = 8;

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
    "item_set TEXT, unique_flag TEXT, flavor TEXT, tooltip_html TEXT,",
    "icon_url TEXT,",
    "sockets TEXT, socket_bonus TEXT, classes TEXT, stats TEXT, effects TEXT,",
    "faction_prices TEXT,",
    "dmg_min INTEGER, dmg_max INTEGER, dps REAL, speed REAL,",
    "vendor_sell INTEGER,",
    "price_min INTEGER, price_market INTEGER, price_available INTEGER,",
    "price_realms INTEGER, price_region TEXT, price_scope TEXT,",
    "price_updated_at TEXT,",
    "api_url TEXT, html_url TEXT, tooltip_url TEXT, tooltip_etag TEXT,",
    "list_etag TEXT, detail_etag TEXT,",
    "list_synced_at INTEGER, detail_synced_at INTEGER, tooltip_at INTEGER,",
    "detail_missing_at INTEGER, tooltip_missing_at INTEGER,",
    "tooltip_text TEXT, tooltip_lc TEXT,",
    "icon_blob BLOB, icon_at INTEGER, icon_missing_at INTEGER,",
    "manual_at INTEGER, manual_overrides TEXT, edited_at INTEGER, deleted_at INTEGER,",
    "vendor_buy INTEGER, price_min_now INTEGER, price_trend_normal INTEGER,",
    "price_now_at INTEGER, price_now_missing_at INTEGER,",
    "raw_json TEXT",
    ");",
    "CREATE TABLE watch (id INTEGER PRIMARY KEY, added_at INTEGER, note TEXT);",
    "CREATE TABLE sync_state (",
    "kind TEXT PRIMARY KEY, status TEXT,",
    "page INTEGER, done INTEGER, total INTEGER,",
    "message TEXT, updated_at INTEGER, error TEXT",
    ");",
    "CREATE TABLE price_history (seq INTEGER PRIMARY KEY, item_id INTEGER, sampled_at INTEGER, price_min INTEGER, price_market INTEGER);",
    // The v8 recipe table (ADR 0021): one row per craft-recipe the fetch stage
    // wrote, keyed by its spell id, joined to item only BY VALUE at the door
    // (spell ids and item ids are different namespaces - the measured forever
    // beta uses spell_id 1283400 for what classic item 728 shows; only the
    // crafted_item_id column names an item). reagents_json keeps the dataset's
    // reagent array as bytes - the same "the DB gets the bytes" rule raw_json
    // rides - and the door resolves each reagent item_id through the item rows
    // it already owns. This text must stay byte-identical to the CREATE inside
    // SQL_MIGRATE_TO_V8 below: one table, two doors.
    "CREATE TABLE recipe (spell_id INTEGER PRIMARY KEY, profession TEXT, recipe_name TEXT, crafted_item_id INTEGER, crafted_item_name TEXT, skill_required INTEGER, skill_tier TEXT, trivial_low INTEGER, trivial_high INTEGER, reagents_json TEXT, learned_json TEXT, acquisition TEXT, acquisition_note TEXT, source TEXT);",
    "CREATE INDEX recipe__crafted ON recipe (crafted_item_id);",
    "CREATE INDEX item__name_lc ON item (name_lc);",
    "CREATE INDEX price_history__item ON price_history (item_id);",
    "CREATE INDEX item__detail ON item (detail_synced_at);",
    "CREATE INDEX item__rank ON item (quality_rank, item_level);",
    "CREATE INDEX item__price ON item (price_min);",
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
/// may not have seen this id at all) plus the price, stat and detail columns - and it
/// clears detail_missing_at, because a record that answered is not a missing record.
pub const DETAIL_VALUE_COUNT: usize = 39;

pub const UPSERT_DETAIL_SQL: &str = concat!(
    "INSERT INTO item (id, name, name_lc, quality, quality_rank, item_level, required_level, item_type, item_subtype, slot, item_set, unique_flag, flavor, icon_url, sockets, socket_bonus, classes, stats, effects, faction_prices, dmg_min, dmg_max, dps, speed, vendor_sell, price_min, price_market, price_available, price_realms, price_region, price_scope, price_updated_at, api_url, html_url, tooltip_url, detail_etag, detail_synced_at, detail_missing_at, raw_json)",
    " VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    " ON CONFLICT (id) DO UPDATE SET name = ?, name_lc = ?, quality = ?, quality_rank = ?, item_level = ?, required_level = ?, item_type = ?, item_subtype = ?, slot = ?, item_set = ?, unique_flag = ?, flavor = ?, icon_url = ?, sockets = ?, socket_bonus = ?, classes = ?, stats = ?, effects = ?, faction_prices = ?, dmg_min = ?, dmg_max = ?, dps = ?, speed = ?, vendor_sell = ?, price_min = ?, price_market = ?, price_available = ?, price_realms = ?, price_region = ?, price_scope = ?, price_updated_at = ?, api_url = ?, html_url = ?, tooltip_url = ?, detail_etag = ?, detail_synced_at = ?, detail_missing_at = ?, raw_json = ?",
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

/// The one additive step from the v1 shape (v0.5.0 and earlier): the checked-missing
/// column. Applied in place so an existing catalogue keeps its rows.
pub const SQL_ADD_MISSING_COLUMN: &str = "ALTER TABLE item ADD COLUMN detail_missing_at INTEGER";

/// The v2 -> v3 steps (v0.6.0): the icon columns ride an existing row like the missing
/// stamp did, and the price-history table plus its index simply appear - a v2 file has
/// none, and ALTER/CREATE are the whole delta. Column ORDER differs from a file built
/// fresh by SCHEMA_SQL (ALTER appends), which is why every reader names columns
/// explicitly and none relies on positions. SQLite accepts the FK annotation nowhere
/// near here on its own: SQLite would not enforce a FOREIGN KEY clause anyway (probe:
/// an INSERT referencing a missing item succeeded), so the guard for price_history ->
/// item lives in Rust: samples are only ever written after the item upsert made the
/// row. See docs/DATABASE.md and [ADR 0007](../docs/DECISIONS/0007-price-history-sampling.md).
pub const SQL_MIGRATE_TO_V3: &str = concat!(
    "ALTER TABLE item ADD COLUMN icon_blob BLOB;",
    "ALTER TABLE item ADD COLUMN icon_at INTEGER;",
    "ALTER TABLE item ADD COLUMN icon_missing_at INTEGER;",
    "CREATE TABLE price_history (seq INTEGER PRIMARY KEY, item_id INTEGER, sampled_at INTEGER, price_min INTEGER, price_market INTEGER);",
    "CREATE INDEX price_history__item ON price_history (item_id);",
);

/// The v3 -> v4 steps (v0.8.0, ADR 0010): the tooltip door's own copy and stamps
/// (its 404 was measured answering upstream's own JSON error - `No item … in
/// forever.` - so it gets the same checked third state the detail and icon doors
/// grew), and the MANUAL LAYER: manual_at marks rows the user created (ids from a
/// reserved block - the measured max id on disk was 288 340, so 900 000 000+ is
/// clear of every upstream id), manual_overrides is the per-field JSON the edit
/// verbs write and every write path merges LAST (a user-said field beats the
/// next crawl; the fields the crawl owns keep refreshing), edited_at stamps the
/// last user edit, and deleted_at is the tombstone - a HARD delete would be
/// resurrected by the next list pass (the list stage upserts every listed id), so
/// the mark is the only shape that survives. All additive again: ALTER appends,
/// the stamp moves in one call, rows and cursors stay intact. No new index: the
/// pending rules read these stamps once per run, same rule as v0.5.3's pending
/// filter (a guessed index is a plan, not a measurement).
pub const SQL_MIGRATE_TO_V4: &str = concat!(
    "ALTER TABLE item ADD COLUMN tooltip_html TEXT;",
    "ALTER TABLE item ADD COLUMN tooltip_etag TEXT;",
    "ALTER TABLE item ADD COLUMN tooltip_at INTEGER;",
    "ALTER TABLE item ADD COLUMN tooltip_missing_at INTEGER;",
    "ALTER TABLE item ADD COLUMN manual_at INTEGER;",
    "ALTER TABLE item ADD COLUMN manual_overrides TEXT;",
    "ALTER TABLE item ADD COLUMN edited_at INTEGER;",
    "ALTER TABLE item ADD COLUMN deleted_at INTEGER;",
);

/// The v4 -> v5 steps (v0.9.0, ADR 0012): the READABLE copy behind every copied
/// tooltip. The card box must not paste upstream's own html (it hotlinks its icons
/// - `render.worldofwarcraft.com` and `thewowdb.com/assets` img tags were measured
/// inside two live tooltip answers - and the house rule says the DB gets the bytes,
/// nothing hotlinks), and `search` cannot grep markup for words. So the milestone
/// DERIVES plain text from the copy: tooltip_text is what a card renders, tooltip_lc
/// is its lowercase twin - the same pair-shape the name columns already ride
/// (name/name_lc, measured in SCHEMA_SQL since v0.1.x). Additive again: ALTER appends.
pub const SQL_MIGRATE_TO_V5: &str = concat!(
    "ALTER TABLE item ADD COLUMN tooltip_text TEXT;",
    "ALTER TABLE item ADD COLUMN tooltip_lc TEXT;",
);

/// The v5 -> v6 steps (v0.9.5, ADR 0016): the RESERVATION of the price columns the
/// user's testing asked for. Until today the app showed ONE price per row (the
/// stored copy's `price_min`, an auction-house number) beside the vendor sell, and
/// the user's own test said the truth needs FOUR separate homes: vendor sell (the
/// column already exists), vendor BUY (only when the item can be bought at all),
/// the CURRENT auction-house min buyout, and the historic "normal" price the live
/// trend answers. HOW each value gets fetched is explicitly NOT this release
/// (the user said so: "how we'll get to those values can be ignored for now, but
/// the columns should be allocated now") - this milestone creates the homes, and
/// NOTHING writes or reads them yet, so every door still answers the old shape
/// BYTE-FOR-BYTE and every new column is NULL everywhere. NULL here means "no
/// producer has ever written it" (docs/DATABASE.md names the third state); it is
/// NOT yet "looked up and found nothing". Additive again: ALTER appends, rows,
/// stamps and cursors stay intact, and readers keep naming columns explicitly,
/// so the fresh-CREATE order (these three sit before raw_json) vs the ALTER order
/// (appended at the end) cannot matter.
pub const SQL_MIGRATE_TO_V6: &str = concat!(
    "ALTER TABLE item ADD COLUMN vendor_buy INTEGER;",
    "ALTER TABLE item ADD COLUMN price_min_now INTEGER;",
    "ALTER TABLE item ADD COLUMN price_trend_normal INTEGER;",
);

/// The v6 -> v7 steps (v0.9.6, ADR 0017 "the live min buyout"): the STAMPS the
/// price refresh stage rides, beside the home ADR 0016 carved but never wrote.
/// price_min_now keeps NULL until the refresh stage's first write - the stamps
/// answer the stage's three states (refreshed? checked as "no price right now"?
/// never looked at?) exactly like the icon pair and the tooltip pair do, and the
/// pending rule excludes manual rows like every other stage's. Additive again:
/// ALTER appends, and every reader names columns, never positions.
pub const SQL_MIGRATE_TO_V7: &str = concat!(
    "ALTER TABLE item ADD COLUMN price_now_at INTEGER;",
    "ALTER TABLE item ADD COLUMN price_now_missing_at INTEGER;",
);

/// The v7 -> v8 step (v0.10.0, ADR 0021): the recipe table arrives. NOT another
/// ALTER on item - recipes are not items: the join measured this release is BY
/// VALUE (crafted_item_id names an item row; spell_id lives in the spell
/// namespace and joins nothing on disk yet). CREATE + index are the whole
/// delta, additive again: existing rows, stamps and cursors keep their
/// meaning, and the fresh build's SCHEMA_SQL block spells the SAME table text.
/// A file that never ran the recipes stage simply holds zero rows here.
pub const SQL_MIGRATE_TO_V8: &str = concat!(
    "CREATE TABLE recipe (spell_id INTEGER PRIMARY KEY, profession TEXT, recipe_name TEXT, crafted_item_id INTEGER, crafted_item_name TEXT, skill_required INTEGER, skill_tier TEXT, trivial_low INTEGER, trivial_high INTEGER, reagents_json TEXT, learned_json TEXT, acquisition TEXT, acquisition_note TEXT, source TEXT);",
    "CREATE INDEX recipe__crafted ON recipe (crafted_item_id);",
);

/// Marking an id as "the upstream has no forever record for it" (v0.5.3). The row
/// must already exist (the list walk created it): an INSERT here would answer the
/// item FK (id -> item.id) and invent a name-less row, so callers check existence
/// first (SQL_ROW_OF_ID) and this statement only ever moves the stamp.
pub const SQL_MARK_MISSING: &str = "INSERT INTO item (id, detail_missing_at) VALUES (?,?) ON CONFLICT (id) DO UPDATE SET detail_missing_at = ?";
pub const MARK_VALUE_COUNT: usize = 2;

/// Does this id have a row at all? Guards the mark above.
pub const SQL_ROW_OF_ID: &str = "SELECT COUNT(1) FROM item WHERE id = ?";

/// The manual layer (v0.8.0, ADR 0010): tombstone readers/writers plus the
/// reserved manual-id probe. The facts under the rules are measured, not
/// guessed: the list stage UPSERTs every listed id (so a hard delete would be
/// resurrected), and the measured max id on the user's file is 288 340, which
/// keeps the 900 000 000 block clear of every upstream id.
pub const SQL_MANUAL_EDITS_OF: &str = "SELECT manual_overrides FROM item WHERE id = ?";
pub const SQL_DELETED_AT_OF: &str = "SELECT deleted_at FROM item WHERE id = ?";
pub const SQL_MAX_MANUAL_ID: &str = "SELECT MAX(id) FROM item WHERE id >= ?";
pub const SQL_MARK_DELETED: &str = "UPDATE item SET deleted_at = ? WHERE id = ?";
pub const SQL_UNDELETE: &str = "UPDATE item SET deleted_at = NULL WHERE id = ?";

pub const MANUAL_ID_BASE: i64 = 900000000;

/// The editable whitelist (ADR 0010): TEXT is kind 0, INTEGER is kind 1, and
/// anything else is usage. Auction-house price columns are deliberately ABSENT
/// - they are live data the crawl refreshes, and an override "fixing" them
/// would be a permanent lie about CURRENT prices; `id` and the icon columns
/// are absent too (ADR 0010 rejects those doors for this build).
pub fn editable_kind(field: String) -> i64 {
    match field.as_str() {
        "name" => return 0,
        "quality" => return 0,
        "item_level" => return 1,
        "required_level" => return 1,
        "item_type" => return 0,
        "item_subtype" => return 0,
        "flavor" => return 0,
        "vendor_sell" => return 1,
        "tooltip_html" => return 0,
        _ => return -1,
    }
}

/// One TEXT column of one item row, by id: the live lookup's ETag reuse reads the
/// stored copy when the API answers 304 (docs/API.md, docs/DATABASE.md).
pub const SQL_DETAIL_ETAG_OF: &str = "SELECT detail_etag FROM item WHERE id = ?";

/// One price observation per fresh (200) detail write (v0.6.0, ADR 0007): a plain
/// INSERT, never an upsert - every visit IS the sample. `seq` is left out so SQLite
/// auto-assigns the primary key. The FK in the table heading is decorative (SQLite
/// does not enforce FK clauses - measured), so the guard is the ORDER in Rust:
/// upsert_detail only calls this after the item upsert has made/updated the row.
pub const SQL_PRICE_SAMPLE: &str = "INSERT INTO price_history (item_id, sampled_at, price_min, price_market) VALUES (?,?,?,?)";
pub const PRICE_SAMPLE_COUNT: usize = 4;

/// A sample answers 404-then-history: the card reads an item's own samples, newest
/// N first so a long history stays bounded, and the caller reverses for the chart.
pub const SQL_HISTORY_OF: &str = "SELECT sampled_at, price_min, price_market FROM price_history WHERE item_id = ? ORDER BY sampled_at DESC, seq DESC LIMIT ?";
pub const SQL_RAW_JSON_OF: &str = "SELECT raw_json FROM item WHERE id = ?";
pub const SQL_SYNCED_AT_OF: &str = "SELECT detail_synced_at FROM item WHERE id = ?";

/// The icon stage's copy write (v0.6.0): bytes ride the same row, like every other
/// stage-2 column. The INSERT..ON CONFLICT shape matches the marks', and the stage
/// hands ids that came FROM a SELECT on this table, so the row exists: a plain INSERT
/// here could invent a name-less row, an UPDATE could not find one. The trailing
/// icon_missing_at = ? binds NULL: a copied icon is not a checked-missing one, and
/// the two stamps stay mutually exclusive exactly like detail_synced_at/detail_missing_at
/// do (docs/DATABASE.md, ADR 0008).
pub const SQL_COPY_ICON: &str = "INSERT INTO item (id, icon_blob, icon_at, icon_missing_at) VALUES (?,?,?,?) ON CONFLICT (id) DO UPDATE SET icon_blob = ?, icon_at = ?, icon_missing_at = ?";
pub const ICON_VALUE_COUNT: usize = 4;

/// The icon's third state (v0.6.0): upstream answered 404 FOR THE ICON FILE of a
/// record that exists. Same guard as the detail mark (SQL_ROW_OF_ID via has_row_of_id)
/// so a check can never invent a row.
pub const SQL_MARK_ICON_MISSING: &str = "INSERT INTO item (id, icon_missing_at) VALUES (?,?) ON CONFLICT (id) DO UPDATE SET icon_missing_at = ?";
pub const ICON_MARK_VALUE_COUNT: usize = 2;

/// The icon URL the detail record stored: the icon stage fetches bytes from it
/// (absolute on one of the two grounded hosts, root-relative for the fixture -
/// docs/API.md measures both).
pub const SQL_ICON_URL_OF: &str = "SELECT icon_url FROM item WHERE id = ?";
/// The copied bytes for the card: the webview cannot open the DB, so the CLI
/// hands them out base64'd in the card JSON (the one helper without a dependency).
pub const SQL_ICON_BLOB_OF: &str = "SELECT icon_blob FROM item WHERE id = ?";

/// When the icon was copied (NULL = never) and when it was checked as no-icon-upstream
/// (NULL = never checked): the card prints both so the icon's three states stay visible
/// there, exactly like the detail pair's stamps drive the "fetched" line.
pub const SQL_ICON_AT_OF: &str = "SELECT icon_at FROM item WHERE id = ?";
pub const SQL_ICON_MISSING_AT_OF: &str = "SELECT icon_missing_at FROM item WHERE id = ?";

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

/// Open the DB file at an explicit path. The path this gets is already resolved:
/// `db_path` answers the four-way rule (`--db PATH` beats `WGT_DB` beats the Tauri
/// appDataDir beats the HOME default, docs/DATABASE.md) and hands this the path it
/// picked; the test harness and the audits hand a scratch path directly. Opening is
/// ONE code path for every entry point (v0.11.1, ADR 0024): the CLI resolves through
/// `db_path` then calls this (`cli.rs`'s `open_at` and its two direct calls), and the
/// GUI reaches the same door through `cli_ok`'s argv (ADR 0014's one-code-path), so
/// the wrapper that used to bundle `db_path` plus this died with no caller - rustc
/// called it never used, and the two doors into the schema helper are `db_path` (who
/// picks the file) and `open_db_at` (who opens and stamps it), exactly as DATABASE.md
/// now says.
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
            match meta_set(&conn, "schema_version".to_string(), format!("{}", SCHEMA_VERSION)) {
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
            if version == SCHEMA_VERSION {
                return Ok(());
            }
            // v0.9.0 chains EVERY stale shape up to v5: the user's own file sat at v1
            // for six milestones and migrated the moment a real run opened it
            // (2026-11-20, measured: schema_version meta now says 3, rows 19 258
            // intact), so refusing ANY older stamp would throw away a full catalogue
            // on the excuse of a schema bump. v1 -> v2 is v0.5.3's ALTER, v2 -> v3 is
            // the v0.6.0 batch, v3 -> v4 is v0.8.0's ALTERs, v4 -> v5 is v0.9.0's pair,
            // v5 -> v6 was v0.9.5's trio (ADR 0016), v6 -> v7 was v0.9.6's stamp
            // pair (ADR 0017's ALTERs), v7 -> v8 is this build's recipe table
            // (ADR 0021's CREATE + index) - all additive, so existing rows, stamps
            // and the cursor in sync_state keep their meaning. Anything newer or
            // foreign keeps the honest refusal below.
            match version {
                1 => {
                    match bind_sql(conn, SQL_ADD_MISSING_COLUMN, [].to_vec(), 0) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem.message))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V3) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V4) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V5) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V6) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V7) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V8) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match meta_set(conn, "schema_version".to_string(), format!("{}", SCHEMA_VERSION)) {
                        Err(problem) => return Err(problem),
                        Ok(_) => return Ok(()),
                    }
                },
                2 => {
                    match conn.execute_batch(SQL_MIGRATE_TO_V3) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V4) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V5) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V6) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V7) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V8) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match meta_set(conn, "schema_version".to_string(), format!("{}", SCHEMA_VERSION)) {
                        Err(problem) => return Err(problem),
                        Ok(_) => return Ok(()),
                    }
                },
                3 => {
                    match conn.execute_batch(SQL_MIGRATE_TO_V4) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V5) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V6) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V7) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V8) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match meta_set(conn, "schema_version".to_string(), format!("{}", SCHEMA_VERSION)) {
                        Err(problem) => return Err(problem),
                        Ok(_) => return Ok(()),
                    }
                },
                4 => {
                    match conn.execute_batch(SQL_MIGRATE_TO_V5) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V6) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V7) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V8) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match meta_set(conn, "schema_version".to_string(), format!("{}", SCHEMA_VERSION)) {
                        Err(problem) => return Err(problem),
                        Ok(_) => return Ok(()),
                    }
                },
                5 => {
                    match conn.execute_batch(SQL_MIGRATE_TO_V6) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V7) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V8) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match meta_set(conn, "schema_version".to_string(), format!("{}", SCHEMA_VERSION)) {
                        Err(problem) => return Err(problem),
                        Ok(_) => return Ok(()),
                    }
                },
                6 => {
                    match conn.execute_batch(SQL_MIGRATE_TO_V7) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match conn.execute_batch(SQL_MIGRATE_TO_V8) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match meta_set(conn, "schema_version".to_string(), format!("{}", SCHEMA_VERSION)) {
                        Err(problem) => return Err(problem),
                        Ok(_) => return Ok(()),
                    }
                },
                7 => {
                    match conn.execute_batch(SQL_MIGRATE_TO_V8) {
                        Err(problem) => return Err(AppError::db(format!("migration rejected: {}", problem))),
                        Ok(_) => {},
                    }
                    match meta_set(conn, "schema_version".to_string(), format!("{}", SCHEMA_VERSION)) {
                        Err(problem) => return Err(problem),
                        Ok(_) => return Ok(()),
                    }
                },
                _ => {},
            }
            return Err(AppError::db(format!("database on disk uses schema {}, this build wants {}; delete the file to rebuild it", version, SCHEMA_VERSION)));
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
            Ok(value) => Ok(value),
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

/// Does this id have a row? The list walk always makes the row before the detail
/// stage asks about it, so a "no" here means the caller is about to invent a
/// name-less record and must not mark anything.
pub fn has_row_of_id(conn: &Connection, id: i64) -> AppResult<bool> {
    let mut stmt = match conn.prepare(SQL_ROW_OF_ID) {
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
        Ok(None) => return Ok(false),
        Ok(Some(row)) => {
            let count: i64 = match row.get(0) {
                Err(problem) => return Err(AppError::db(format!("{}", problem))),
                Ok(value) => value,
            };
            return Ok(count > 0);
        },
    }
}

/// The checked-missing state (v0.5.3): remember that upstream answered 404 for this
/// id, so the next crawl does not re-ask for it. Only moves the stamp of an
/// existing row - see has_row_of_id, which callers must consult first.
pub fn mark_missing_at(conn: &Connection, id: i64, checked_at: i64) -> AppResult<bool> {
    let there: bool = match has_row_of_id(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match there {
        false => return Ok(false),
        true => {},
    }
    let stamp: Option<i64> = Some(checked_at);
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(id));
    head.push(int_value(stamp));
    match bind_sql(conn, SQL_MARK_MISSING, upsert_params(head), MARK_VALUE_COUNT * 2 - 1) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(true),
    }
}

/// The icon's checked-missing state (v0.6.0): upstream answered 404 for the ICON FILE
/// of a record that exists. Guarded like the detail mark so a check cannot invent a row.
pub fn mark_icon_missing_at(conn: &Connection, id: i64, checked_at: i64) -> AppResult<bool> {
    let there: bool = match has_row_of_id(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match there {
        false => return Ok(false),
        true => {},
    }
    let stamp: Option<i64> = Some(checked_at);
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(id));
    head.push(int_value(stamp));
    match bind_sql(conn, SQL_MARK_ICON_MISSING, upsert_params(head), ICON_MARK_VALUE_COUNT * 2 - 1) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(true),
    }
}

/// Copy an icon into the row: the bytes rusqlite binds as a BLOB (Value::Blob(Vec) is
/// the variant, and ToSql for Value exists - verified against the crate). icon_missing_at
/// is written NULL: an icon that copied is not a checked-missing one.
pub fn copy_icon(conn: &Connection, id: i64, blob: Vec<u8>, copied_at: i64) -> AppResult<usize> {
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(id));
    head.push(Value::Blob(blob));
    head.push(int_value(Some(copied_at)));
    head.push(int_value(None));
    return bind_sql(conn, SQL_COPY_ICON, upsert_params(head), ICON_VALUE_COUNT * 2 - 1);
}

/// The stored icon URL of an item (NULL when the detail record had none): what the
/// icon stage resolves into a fetch.
pub fn icon_url_of(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let mut stmt = match conn.prepare(SQL_ICON_URL_OF) {
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

/// The copied icon bytes for the card: NULL means no local copy yet (not "empty file").
/// Option<Vec> handles the NULL BLOB (FromSql for Option<T> is generic - verified).
pub fn icon_blob_of(conn: &Connection, id: i64) -> AppResult<Option<Vec<u8>>> {
    let mut stmt = match conn.prepare(SQL_ICON_BLOB_OF) {
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

/// The ONE base64 encoder (v0.6.0, moved beside the bytes it encodes when v0.9.3's
/// lists started shipping icons, so the card and the lists cannot encode the same
/// bytes two ways): the DB holds the icon BYTES and the one door to the webview is
/// a JSON verb answer, which has no binary - so the icon travels as a base64 string
/// and the card/lists turn it into a data URI. Our own twelve-line encoder rather
/// than a new dependency: the pinned dep graph carries no direct base64 crate
/// (docs/DEVELOPMENT.md pins by minor; the base64 entries in Cargo.lock belong to
/// tauri's own subtree), and the fixture audits decode what this emits, so the
/// contract is proven, not asserted. '=' is byte 61.
pub fn b64_text(data: Vec<u8>) -> String {
    let table: Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".as_bytes().to_vec();
    let mut out: String = "".to_string();
    let mut index: usize = 0;
    while index < data.len() {
        let left: usize = data.len() - index;
        let b0: u32 = data[index] as u32;
        let b1: u32 = if left > 1 { data[index + 1] as u32 } else { 0 };
        let b2: u32 = if left > 2 { data[index + 2] as u32 } else { 0 };
        let triple: u32 = (b0 << 16) + (b1 << 8) + b2;
        let mut parts: Vec<u8> = [].to_vec();
        parts.push(table[((triple >> 18) & 63) as usize]);
        parts.push(table[((triple >> 12) & 63) as usize]);
        if left > 1 { parts.push(table[((triple >> 6) & 63) as usize]); } else { parts.push(61); }
        if left > 2 { parts.push(table[(triple & 63) as usize]); } else { parts.push(61); }
        for one in parts { out.push(one as char); }
        index = index + 3;
    }
    return out;
}

/// The DB's own icon bytes as base64 - ONE helper for the card AND the lists
/// (v0.9.3, ADR 0014), so no door invents its own encoding. The three states stay
/// three states: bytes copied (`icon_b64` here), checked as gone upstream
/// (`icon_missing_at_of`), or never fetched (both null).
pub fn icon_b64_of(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let copy: Option<Vec<u8>> = match icon_blob_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    return match copy {
        None => Ok(None),
        Some(bytes) => Ok(Some(b64_text(bytes.clone()))),
    };
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

/// v0.9.4 (ADR 0015): the money words of ONE copper number, in the game's OWN
/// wording - and the unit is not an assumption, the DB's own tooltip copies prove
/// it: `45267` sits beside `Sell Price: 4g 52s 67c` on row 810, `10367` beside
/// `1g 3s 67c` on 2528, `100` beside `1s` on 733 and `7` beside `7c` on 36, so
/// 100 copper is 1 silver and 100 silver is 1 gold. The copies give the zero rule
/// too: the denomination at zero is simply DROPPED (`25000` ↔ `2g 50s` on 4499,
/// `20000` ↔ `2g` on 12720, `3000` ↔ `30s` on 18046, and the middle-zero `20042`
/// ↔ `2g 42c` on 14138 - the only such row in 4 231), so this formatter writes
/// what the copies write and never invents a `0s` upstream never stated. A copper
/// of 0 appears NOWHERE in the DB (measured: MIN over every copper column is `1`),
/// so the `0c` below is a RULE, not a measurement: it keeps a stated zero readable
/// instead of printing nothing. Money is DERIVED at answer time and NEVER stored
/// - the copper numbers stay the sort keys, the script fields and the numeric
/// contract of docs/DATABASE.md, so the schema stays `5` with nothing to migrate.
pub fn money_text(copper: i64) -> String {
    let gold: i64 = copper / 10000;
    let silver: i64 = (copper % 10000) / 100;
    let copper_only: i64 = copper % 100;
    let mut parts: Vec<String> = [].to_vec();
    if gold > 0 { parts.push(format!("{}g", gold)); }
    if silver > 0 { parts.push(format!("{}s", silver)); }
    if copper_only > 0 { parts.push(format!("{}c", copper_only)); }
    if parts.len() == 0 { return "0c".to_string(); }
    return parts.join(" ");
}

/// Money beside an OPTIONAL copper number: an absent price stays absent, it never
/// reads as `0c` - "the AH does not have this item" and "this item costs nothing"
/// are different facts, and ADR 0004's rule against made-up zeros applies here too.
pub fn money_of(copper: Option<i64>) -> Option<String> {
    match copper {
        None => return None,
        Some(value) => return Some(money_text(value)),
    }
}

/// Money beside an AVERAGE, which SQLite answers as a FLOAT (measured on the user's
/// own DB: `AVG(price_min)` is `54391.705896` copper). Money has no fraction of a
/// copper, so the average FLOORS to whole copper rather than rounding up: a rounded
/// money word would claim a price the ledger never observed (`5g 43s 92c` for a
/// mean that is not quite that high). The exact float still rides beside it.
pub fn money_of_avg(average: Option<f64>) -> Option<String> {
    match average {
        None => return None,
        Some(value) => return Some(money_text(value as i64)),
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

/// One user entry row of `manual_overrides`, as stored: an empty/NULL column
/// means "the crawl owns every field"; a malformed store is an honest db error,
/// not a silent reset (a lie about what the user said would be worse).
pub fn manual_edits_of(conn: &Connection, id: i64) -> AppResult<Vec<ManualEdit>> {
    let mut stmt = match conn.prepare(SQL_MANUAL_EDITS_OF) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Integer(id));
    let mut rows = match stmt.query(params_from_iter(single)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let text: Option<String> = match rows.next() {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(None) => None,
        Ok(Some(row)) => match row.get(0) {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(found) => found,
        },
    };
    match text {
        None => return Ok([].to_vec()),
        Some(body) => {
            let parsed: Vec<ManualEdit> = match from_str(&body) {
                Err(problem) => return Err(AppError::db(format!("stored manual overrides unreadable: {}", problem))),
                Ok(found) => found,
            };
            return Ok(parsed);
        },
    }
}

/// The tombstone state of one row (NULL = not deleted).
pub fn deleted_at_of(conn: &Connection, id: i64) -> AppResult<Option<i64>> {
    let mut stmt = match conn.prepare(SQL_DELETED_AT_OF) {
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

/// Next free id in the reserved manual block.
pub fn next_manual_id(conn: &Connection) -> AppResult<i64> {
    let mut stmt = match conn.prepare(SQL_MAX_MANUAL_ID) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Integer(MANUAL_ID_BASE));
    let mut rows = match stmt.query(params_from_iter(single)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    match rows.next() {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(None) => return Ok(MANUAL_ID_BASE),
        Ok(Some(row)) => {
            let top: Option<i64> = match row.get(0) {
                Err(problem) => return Err(AppError::db(format!("{}", problem))),
                Ok(found) => found,
            };
            return match top {
                None => Ok(MANUAL_ID_BASE),
                Some(best) => Ok(best + 1),
            };
        },
    }
}

/// The edit door: ONE whitelist field of one row. The column is set NOW and the
/// field rides the overrides array (the old entry for the field is replaced,
/// then this edit is appended - stored order stays chronological, the merge is
/// last-wins). Existence and tombstone guards belong to the caller (same house
/// rule as the missing marks, which check SQL_ROW_OF_ID first) so the CLI can
/// say which one it hit. The field NAME is whitelist-validated HERE because it
/// interpolates into the SQL string - only whitelist columns can be named.
pub fn set_item_field(conn: &Connection, id: i64, field: String, value: String, at: i64) -> AppResult<usize> {
    let kind: i64 = editable_kind(field.clone());
    match kind {
        -1 => return Err(AppError::usage("editable fields: name, quality, item_level, required_level, item_type, item_subtype, flavor, vendor_sell, tooltip_html".to_string())),
        _ => {},
    }
    let column_value: Value = match kind {
        0 => Value::Text(value.clone()),
        _ => {
            let typed: i64 = match value.parse() {
                Err(_) => return Err(AppError::usage("an editable INTEGER field wants a plain number".to_string())),
                Ok(number) => number,
            };
            Value::Integer(typed)
        },
    };
    let edits: Vec<ManualEdit> = match manual_edits_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let mut fresh: Vec<ManualEdit> = [].to_vec();
    let mut index: usize = 0;
    while index < edits.len() {
        if edits[index].field != field {
            fresh.push(edits[index].clone())
        }
        index = index + 1;
    }
    let json_text: String = match kind {
        0 => match to_string(&value) {
            Err(_) => return Err(AppError::usage("the override text could not be written".to_string())),
            Ok(text) => text,
        },
        _ => value.clone(),
    };
    let edit_value: serde_json::Value = match from_str(&json_text) {
        Err(_) => return Err(AppError::usage("an editable INTEGER field wants a plain number".to_string())),
        Ok(found) => found,
    };
    fresh.push(ManualEdit { field: field.clone(), value: edit_value, at: at });
    let stored: String = match to_string(&fresh) {
        Err(problem) => return Err(AppError::db(format!("manual overrides unwritable: {}", problem))),
        Ok(text) => text,
    };
    // the derived search columns ride the SAME write as the field they come from,
    // so renaming a row here is findable by `search` on the same breath (a stale
    // name_lc would hide the user's own correction until some crawl touched it).
    let mut clauses: Vec<String> = [].to_vec();
    let mut head: Vec<Value> = [].to_vec();
    clauses.push(format!("{} = ?", field));
    head.push(column_value);
    match field.as_str() {
        "name" => {
            clauses.push("name_lc = ?".to_string());
            head.push(Value::Text(value.to_lowercase()));
        },
        "quality" => {
            clauses.push("quality_rank = ?".to_string());
            head.push(int_value(rank_of(Some(value.clone()))));
        },
        "tooltip_html" => {
            let readable: String = readable_tooltip_text(value.clone());
            clauses.push("tooltip_text = ?".to_string());
            head.push(Value::Text(readable.clone()));
            clauses.push("tooltip_lc = ?".to_string());
            head.push(Value::Text(readable.to_lowercase()));
        },
        _ => {},
    }
    clauses.push("edited_at = ?".to_string());
    head.push(Value::Integer(at));
    clauses.push("manual_overrides = ?".to_string());
    head.push(Value::Text(stored));
    let sql: String = format!("UPDATE item SET {} WHERE id = ?", clauses.join(", "));
    head.push(Value::Integer(id));
    let want: usize = head.len();
    return bind_sql(conn, sql.as_str(), head, want);
}

/// The create door (ADR 0010): a row in the reserved block, stamped manual_at
/// AND wearing its fields as overrides from birth - if upstream ever answers
/// this id someday, the user's version wins by the merge rule, not by accident.
pub fn add_item_manual(conn: &Connection, id: i64, at: i64, fields: Vec<(String, String)>) -> AppResult<usize> {
    let there: bool = match has_row_of_id(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match there {
        true => return Ok(0),
        false => {},
    }
    let mut has_name: bool = false;
    let mut seen: usize = 0;
    while seen < fields.len() {
        if fields[seen].0 == "name" {
            has_name = true
        }
        if editable_kind(fields[seen].0.clone()) < 0 {
            return Err(AppError::usage("unknown editable field in add-item".to_string()))
        }
        seen = seen + 1;
    }
    match has_name {
        false => return Err(AppError::usage("add-item wants at least a name (name=...)".to_string())),
        true => {},
    }
    let mut cols: Vec<String> = [].to_vec();
    let mut head: Vec<Value> = [].to_vec();
    let mut edits: Vec<ManualEdit> = [].to_vec();
    cols.push("id".to_string());
    head.push(Value::Integer(id));
    cols.push("manual_at".to_string());
    head.push(Value::Integer(at));
    cols.push("edited_at".to_string());
    head.push(Value::Integer(at));
    let mut name_text: String = "".to_string();
    let mut quality_text: Option<String> = None;
    let mut stated_tooltip: Option<String> = None;
    let mut index: usize = 0;
    while index < fields.len() {
        let field: String = fields[index].0.clone();
        let value: String = fields[index].1.clone();
        let kind: i64 = editable_kind(field.clone());
        match kind {
            0 => {
                cols.push(field.clone());
                head.push(Value::Text(value.clone()));
                let json_text: String = match to_string(&value) {
                    Err(_) => return Err(AppError::usage("the field text could not be written".to_string())),
                    Ok(text) => text,
                };
                let edit_value: serde_json::Value = match from_str(&json_text) {
                    Err(_) => return Err(AppError::usage("the field text could not be written".to_string())),
                    Ok(found) => found,
                };
                edits.push(ManualEdit { field: field.clone(), value: edit_value, at: at });
                match field.as_str() {
                    "name" => name_text = value,
                    "quality" => quality_text = Some(value),
                    "tooltip_html" => stated_tooltip = Some(value),
                    _ => {},
                }
            },
            _ => {
                let typed: i64 = match value.parse() {
                    Err(_) => return Err(AppError::usage("an editable INTEGER field wants a plain number".to_string())),
                    Ok(number) => number,
                };
                cols.push(field.clone());
                head.push(Value::Integer(typed));
                let number_value: serde_json::Value = match from_str(&typed.to_string()) {
                    Err(_) => return Err(AppError::usage("an editable INTEGER field wants a plain number".to_string())),
                    Ok(found) => found,
                };
                edits.push(ManualEdit { field: field.clone(), value: number_value, at: at });
            },
        }
        index = index + 1;
    }
    match name_text == "" {
        true => return Err(AppError::usage("add-item wants a name (name=...)".to_string())),
        false => {},
    }
    cols.push("name_lc".to_string());
    head.push(Value::Text(name_text.to_lowercase()));
    match quality_text {
        None => {},
        Some(quality) => {
            cols.push("quality_rank".to_string());
            head.push(int_value(rank_of(Some(quality))));
        },
    }
    match stated_tooltip {
        None => {},
        Some(html) => {
            let readable: String = readable_tooltip_text(html.clone());
            cols.push("tooltip_text".to_string());
            head.push(Value::Text(readable.clone()));
            cols.push("tooltip_lc".to_string());
            head.push(Value::Text(readable.to_lowercase()));
        },
    }
    let stored: String = match to_string(&edits) {
        Err(problem) => return Err(AppError::db(format!("manual overrides unwritable: {}", problem))),
        Ok(text) => text,
    };
    cols.push("manual_overrides".to_string());
    head.push(Value::Text(stored));
    let sql: String = build_insert(cols);
    let want: usize = head.len();
    return bind_sql(conn, sql.as_str(), head, want);
}

/// DELETE is the tombstone (ADR 0010): the row survives, but search, stats,
/// pending queues and the card refuse it; a HARD delete would be resurrected
/// by the next list pass. False = no row or already marked (the caller says
/// which, and both read the same honest exit 4 family).
pub fn mark_deleted_at(conn: &Connection, id: i64, at: i64) -> AppResult<bool> {
    let there: bool = match has_row_of_id(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match there {
        false => return Ok(false),
        true => {},
    }
    let already: Option<i64> = match deleted_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    match already {
        Some(_) => return Ok(false),
        None => {},
    }
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(at));
    head.push(Value::Integer(id));
    let want: usize = head.len();
    match bind_sql(conn, SQL_MARK_DELETED, head, want) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(true),
    }
}

/// The mark's undo: a tombstone the user restores. False = not marked.
pub fn clear_deleted_at(conn: &Connection, id: i64) -> AppResult<bool> {
    let there: bool = match has_row_of_id(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match there {
        false => return Ok(false),
        true => {},
    }
    let already: Option<i64> = match deleted_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    match already {
        None => return Ok(false),
        Some(_) => {},
    }
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(id));
    let want: usize = head.len();
    match bind_sql(conn, SQL_UNDELETE, head, want) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(true),
    }
}

/// Merge for the list write: the list payload only carries these four.
pub fn apply_manual_edits_summary(summary: &mut ItemSummary, edits: Vec<ManualEdit>) {
    let mut index: usize = 0;
    while index < edits.len() {
        match edits[index].field.as_str() {
            "name" => {
                match edits[index].value.as_str() {
                    None => {},
                    Some(text) => summary.name = text.to_string(),
                }
            },
            "quality" => {
                match edits[index].value.as_str() {
                    None => {},
                    Some(text) => summary.quality = Some(text.to_string()),
                }
            },
            "item_level" => {
                match edits[index].value.as_i64() {
                    None => {},
                    Some(number) => {
                        if number >= -2147483648 && number <= 2147483647 {
                            summary.item_level = Some(number as i32)
                        }
                    },
                }
            },
            "required_level" => {
                match edits[index].value.as_i64() {
                    None => {},
                    Some(number) => {
                        if number >= -2147483648 && number <= 2147483647 {
                            summary.required_level = Some(number as i32)
                        }
                    },
                }
            },
            _ => {},
        }
        index = index + 1;
    }
}

/// Merge for the detail write. `tooltip_html` is NOT here: the detail payload
/// never carries it (measured key list, docs/API.md) - the tooltip stage and
/// the edit verb are its only writers, and the stage's write in sync.rs reads
/// the overrides itself before it binds.
pub fn apply_manual_edits_detail(detail: &mut ItemDetail, edits: Vec<ManualEdit>) {
    let mut index: usize = 0;
    while index < edits.len() {
        match edits[index].field.as_str() {
            "name" => {
                match edits[index].value.as_str() {
                    None => {},
                    Some(text) => detail.name = text.to_string(),
                }
            },
            "quality" => {
                match edits[index].value.as_str() {
                    None => {},
                    Some(text) => detail.quality = Some(text.to_string()),
                }
            },
            "item_level" => {
                match edits[index].value.as_i64() {
                    None => {},
                    Some(number) => {
                        if number >= -2147483648 && number <= 2147483647 {
                            detail.item_level = Some(number as i32)
                        }
                    },
                }
            },
            "required_level" => {
                match edits[index].value.as_i64() {
                    None => {},
                    Some(number) => {
                        if number >= -2147483648 && number <= 2147483647 {
                            detail.required_level = Some(number as i32)
                        }
                    },
                }
            },
            "item_type" => {
                match edits[index].value.as_str() {
                    None => {},
                    Some(text) => detail.item_type = Some(text.to_string()),
                }
            },
            "item_subtype" => {
                match edits[index].value.as_str() {
                    None => {},
                    Some(text) => detail.item_subtype = Some(text.to_string()),
                }
            },
            "flavor" => {
                match edits[index].value.as_str() {
                    None => {},
                    Some(text) => detail.flavor = Some(text.to_string()),
                }
            },
            "vendor_sell" => {
                match edits[index].value.as_i64() {
                    None => {},
                    Some(number) => detail.vendor_sell = Some(number),
                }
            },
            _ => {},
        }
        index = index + 1;
    }
}

/// INSERT skeleton for add_item_manual: the caller validated EVERY column name
/// against the whitelist before collecting, so assembling the statement here
/// can only ever name whitelist columns (same argument as set_item_field).
pub fn build_insert(cols: Vec<String>) -> String {
    let mut names: String = "".to_string();
    let mut marks: String = "".to_string();
    let mut index: usize = 0;
    while index < cols.len() {
        if index > 0 {
            names = names + ",";
            marks = marks + ",";
        }
        names = names + cols[index].as_str();
        marks = marks + "?";
        index = index + 1;
    }
    return format!("INSERT INTO item ({}) VALUES ({})", names, marks)
}

/// A card built from the DB's OWN columns (v0.8.0, ADR 0010): the row a user MADE
/// has no upstream payload and never will. Three measured facts say so: the CRUD
/// door stamps manual_at and puts the row in the reserved id block, the crawl's
/// pending rules all refuse a manual row (SQL_PENDING, SQL_PENDING_ICON and
/// SQL_PENDING_TOOLTIP every one carries manual_at IS NULL), and upstream cannot
/// know such an id (measured max upstream id on their file: 288 340, block starts
/// at 900 000 000). So the DB is the only source that can answer that row's card,
/// and the fields ONLY an upstream record can state answer null instead of being
/// invented: the wire's own labels (kind/version/version_label/build), the wire's
/// own links and attribution, the auction columns and the weapon columns (crawl-
/// owned: docs/DATABASE.md says the CRUD whitelist does not accept them) and the
/// JSON-shaped columns sockets/classes/stats/effects/faction_prices, which the
/// whitelist never writes either. name is read as a plain String because db.rs's
/// add_item_manual refuses to create a row without one. rusqlite reads rows by
/// DECLARED column type (docs/DATABASE.md), so the mapper's get() indices below ARE
/// the SELECT's order: rusqlite answers an out-of-range get with an error this door
/// turns into a database door (exit 4), which is what catches a SELECT edited
/// without its Rust twin.
pub const SQL_COLUMNS_CARD: &str = concat!(
    "SELECT id, name, quality, item_level, required_level, item_type, item_subtype, ",
    "item_set, slot, unique_flag, flavor, icon_url, socket_bonus, vendor_sell FROM item WHERE id = ?",
);
pub fn column_card(conn: &Connection, id: i64) -> AppResult<ItemDetail> {
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Integer(id));
    let found: ItemDetail = match conn.query_row(SQL_COLUMNS_CARD, params_from_iter(single), |row| {
        Ok(ItemDetail {
            id: row.get(0)?,
            name: row.get(1)?,
            kind: None,
            version: None,
            version_label: None,
            build: None,
            quality: row.get(2)?,
            item_level: row.get(3)?,
            required_level: row.get(4)?,
            item_type: row.get(5)?,
            item_subtype: row.get(6)?,
            item_set: row.get(7)?,
            slot: row.get(8)?,
            unique: row.get(9)?,
            flavor: row.get(10)?,
            icon_url: row.get(11)?,
            socket_bonus: row.get(12)?,
            sockets: None,
            classes: None,
            stats: None,
            effects: None,
            faction_prices: None,
            vendor_sell: row.get(13)?,
            price: None,
            weapon: None,
            links: None,
            attribution: None,
        })
    }) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    return Ok(found)
}

/// The tooltip door's checked third state (v0.8.0, ADR 0010): upstream answered 404
/// for the TOOLTIP of a record that exists. Guarded like the other marks, so a check
/// cannot invent a row.
pub fn mark_tooltip_missing_at(conn: &Connection, id: i64, checked_at: i64) -> AppResult<bool> {
    let there: bool = match has_row_of_id(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match there {
        false => return Ok(false),
        true => {},
    }
    let stamp: Option<i64> = Some(checked_at);
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(id));
    head.push(int_value(stamp));
    match bind_sql(conn, SQL_MARK_TOOLTIP_MISSING, upsert_params(head), MARK_VALUE_COUNT * 2 - 1) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(true),
    }
}

/// The tooltip write, override-aware (ADR 0010): when the user stated `tooltip_html`
/// themselves, the CRAWL's copy must not replace it - the stamp still moves (we did
/// ask, that is a fact) but the text stays user-said. Guarded by the row existing, so
/// an ask for an unknown id writes nothing.
pub fn write_tooltip(conn: &Connection, id: i64, html: String, etag: String, at: i64) -> AppResult<usize> {
    let there: bool = match has_row_of_id(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match there {
        false => return Ok(0),
        true => {},
    }
    let edits: Vec<ManualEdit> = match manual_edits_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let mut text: String = html;
    for one in edits.clone() {
        if one.field == "tooltip_html" {
            match one.value.as_str() {
                None => {},
                Some(said) => text = said.to_string(),
            }
        }
    }
    let readable: String = readable_tooltip_text(text.clone());
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(id));
    head.push(Value::Text(text));
    head.push(text_value(optional_text(etag.clone())));
    head.push(Value::Integer(at));
    head.push(text_value(optional_text(readable.clone())));
    head.push(text_value(optional_text(readable.to_lowercase())));
    match bind_sql(conn, SQL_SET_TOOLTIP, upsert_params(head), TOOLTIP_VALUE_COUNT * 2 - 1) {
        Err(problem) => return Err(problem),
        Ok(count) => return Ok(count),
    }
}

// The price refresh write (v0.9.6, ADR 0017): the live door's CURRENT min rides
// the trio home ADR 0016 carved, plus this stage's own stamp pair - same UPSERT
// shape as every stage-2 column write (INSERT..ON CONFLICT, guard first: an ask
// for an unknown id writes NOTHING, "the stage guards rows because inventing a
// row from a targeted ask" is v0.6.0's rule restated). No manual-layer merge
// here: the set-item whitelist (ADR 0010) never offered price_min_now to a user,
// so there is no user-said value this write could overwrite - the pending rule's
// manual_at exclusion is the whole interaction, measured by the audit below.
pub const SQL_SET_PRICE_NOW: &str = "INSERT INTO item (id, price_min_now, price_now_at) VALUES (?,?,?) ON CONFLICT (id) DO UPDATE SET price_min_now = ?, price_now_at = ?";
pub const PRICE_NOW_VALUE_COUNT: usize = 3;

pub fn write_price_now(conn: &Connection, id: i64, copper: i64, at: i64) -> AppResult<usize> {
    let there: bool = match has_row_of_id(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match there {
        false => return Ok(0),
        true => {},
    }
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(id));
    head.push(Value::Integer(copper));
    head.push(Value::Integer(at));
    match bind_sql(conn, SQL_SET_PRICE_NOW, upsert_params(head), PRICE_NOW_VALUE_COUNT * 2 - 1) {
        Err(problem) => return Err(problem),
        Ok(count) => return Ok(count),
    }
}

/// Reading the live observation back (v0.9.6, ADR 0017): the refresh wrote the
/// column, and the CARD door grows it the same breath (a column nobody reads is
/// fine on disk - the pending rule SQL already reads it - but a Rust accessor
/// nobody calls would be a dead-code warning the house does not sweep).
pub const SQL_PRICE_MIN_NOW_OF: &str = "SELECT price_min_now FROM item WHERE id = ?";

/// The live min buyout the refresh stage last saw - or None for "never
/// refreshed" AND for "checked as price-less right now" alike: the card door
/// does not tell those two apart yet (the stamp pair does, on disk, and grows
/// its own fragment when the UI asks for the when - recorded decision, this
/// release ships the value and its money words only).
pub fn price_min_now_of(conn: &Connection, id: i64) -> AppResult<Option<i64>> {
    let mut stmt = match conn.prepare(SQL_PRICE_MIN_NOW_OF) {
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

// The price stage's third state (v0.9.6, ADR 0017): "we asked and the door
// offered NOTHING right now" (measured live 2026-11-21: id 200's `fields.price`
// is literal null) - the mark twin of mark_icon_missing_at and mark_tooltip_missing_at,
// row-guarded like both.
pub const SQL_MARK_PRICE_NOW_MISSING: &str = "INSERT INTO item (id, price_now_missing_at) VALUES (?,?) ON CONFLICT (id) DO UPDATE SET price_now_missing_at = ?";

pub fn mark_price_missing_at(conn: &Connection, id: i64, checked_at: i64) -> AppResult<bool> {
    let there: bool = match has_row_of_id(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match there {
        false => return Ok(false),
        true => {},
    }
    let stamp: Option<i64> = Some(checked_at);
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(id));
    head.push(int_value(stamp));
    match bind_sql(conn, SQL_MARK_PRICE_NOW_MISSING, upsert_params(head), MARK_VALUE_COUNT * 2 - 1) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(true),
    }
}

/// The manual layer's own stamps (v0.8.0, ADR 0010): the card prints them, so a
/// row's history of human decisions is visible next to the crawl's - manual_at
/// says the row was MADE by a user (and lives in the reserved id block), edited_at
/// says the last field a user stated, deleted_at is the tombstone every query
/// hides. manual_overrides_of hands back the stored JSON ARRAY verbatim, so the
/// card and the UI read the same text the merge reads (docs/DATABASE.md's rule:
/// values are bound or copied whole, never re-derived).
pub const SQL_MANUAL_AT_OF: &str = "SELECT manual_at FROM item WHERE id = ?";
pub const SQL_EDITED_AT_OF: &str = "SELECT edited_at FROM item WHERE id = ?";
pub const SQL_MANUAL_OVERRIDES_OF: &str = "SELECT manual_overrides FROM item WHERE id = ?";

pub fn manual_at_of(conn: &Connection, id: i64) -> AppResult<Option<i64>> {
    let mut stmt = match conn.prepare(SQL_MANUAL_AT_OF) {
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

pub fn edited_at_of(conn: &Connection, id: i64) -> AppResult<Option<i64>> {
    let mut stmt = match conn.prepare(SQL_EDITED_AT_OF) {
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

pub fn manual_overrides_of(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let mut stmt = match conn.prepare(SQL_MANUAL_OVERRIDES_OF) {
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

/// The READABLE derivation (v0.9.0, ADR 0012, house rule: measured, not invented).
/// Measured: two live tooltip answers - one plain (forever/item/25) and one carrying
/// flavor prose (forever/item/728) - hold div/span/img tags with class/src/width/height
/// attributes and ZERO HTML entities. So the rule is exactly (v0.9.1 correction, measured):
/// replace every <...> span with ONE space (measured: a bare span-DROP glued words across
/// element boundaries - the user file's first twin read "Worn ShortswordItem Level 2Main
/// Hand..." after 4 602 copies), then collapse whitespace runs, trim. NO entity decoder
/// is written because none was measured - if
/// upstream ever ships entities they show verbatim, and the words still search. This
/// reads the MERGED copy (the manual layer's stated tooltip_html wins before the twin
/// is computed), so a card and a search can never disagree with the DB.
pub fn readable_tooltip_text(html: String) -> String {
    let mut stripped: String = "".to_string();
    let mut inside_tag: bool = false;
    for ch in html.chars() {
        match inside_tag {
            true => {
                if ch == '>' {
                    inside_tag = false;
                    stripped.push(' ')
                }
            },
            false => {
                match ch {
                    '<' => inside_tag = true,
                    _ => stripped.push(ch),
                }
            },
        }
    }
    let mut text: String = "".to_string();
    let mut seen_space: bool = true;
    for ch in stripped.chars() {
        let is_space: bool = ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r';
        match is_space {
            true => {
                match seen_space {
                    false => text.push(' '),
                    true => {},
                }
                seen_space = true
            },
            false => {
                text.push(ch);
                seen_space = false
            },
        }
    }
    let mut stop: usize = text.chars().count();
    let all: Vec<char> = text.chars().collect();
    while stop > 0 && all[stop - 1] == ' ' {
        stop = stop - 1
    }
    let mut trimmed: String = "".to_string();
    let mut seen_i: usize = 0;
    while seen_i < stop {
        trimmed.push(all[seen_i]);
        seen_i = seen_i + 1
    }
    return trimmed;
}

/// rusqlite reads rows by DECLARED column type (docs/DATABASE.md), so the get()
/// indices below ARE SQL_REFRESH_ROWS' order. EVERY row is COLLECTED before ANY
/// write happens: a sqlite row walk over the table this same door rewrites must
/// not be interleaved with its own UPDATEs, so the walk finishes first and the
/// writes follow. The twin is recomputed by the SAME readable_tooltip_text the
/// write paths call - there is no second derivation to disagree with it.
pub fn refresh_twins(conn: &Connection) -> AppResult<RefreshTally> {
    let mut stmt = match conn.prepare(SQL_REFRESH_ROWS) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let rows = match stmt.query_map([], |row|
        Ok(RefreshRow { id: row.get(0)?, html: row.get(1)?, text: row.get(2)? })
    ) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut copies: Vec<RefreshRow> = [].to_vec();
    for item in rows {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(value) => copies.push(value),
        }
    }
    let seen: i64 = copies.len() as i64;
    let mut filled: i64 = 0;
    let mut rewritten: i64 = 0;
    let mut unchanged: i64 = 0;
    for one in copies {
        let want: String = readable_tooltip_text(one.html.clone());
        let differ: bool = match one.text {
            None => {
                filled = filled + 1;
                true
            },
            Some(have) => {
                match have == want {
                    true => {
                        unchanged = unchanged + 1;
                        false
                    },
                    false => {
                        rewritten = rewritten + 1;
                        true
                    },
                }
            },
        };
        match differ {
            false => {},
            true => {
                let mut params: Vec<Value> = [].to_vec();
                params.push(Value::Text(want.clone()));
                params.push(Value::Text(want.to_lowercase()));
                params.push(Value::Integer(one.id));
                match bind_sql(conn, SQL_REFRESH_WRITE, params, 3) {
                    Err(problem) => return Err(problem),
                    Ok(_) => {},
                }
            },
        }
    }
    return Ok(RefreshTally {
        copies: seen,
        filled: filled,
        rewritten: rewritten,
        unchanged: unchanged,
        complete: true,
        message: format!("refresh-twins over {} copies: {} rewritten, {} filled with their twin from birth, {} already matching", seen, rewritten, filled, unchanged),
    });
}

/// The tooltip door's stamps and copy (v0.8.0, ADR 0010): the tooltip write is a
/// FOURTH crawl stage with the same three states as every other door (written /
/// checked gone upstream / never looked at) and its OWN ETag - the tooltip body
/// revalidates (measured 2026-11-20: ETag W/"2b111cbcd3bbb4d7078f6d05" and
/// `cache-control public, max-age=300`), so a targeted re-ask costs a 304 and
/// writes nothing, exactly like the detail door. The write is the INSERT-with-
/// ON-CONFLICT idiom the other writers use (docs/DATABASE.md: the DO UPDATE half
/// never repeats the key), and the stage guards rows because inventing a row from
/// a door's answer is not our call to make.
pub const SQL_TOOLTIP_ETAG_OF: &str = "SELECT tooltip_etag FROM item WHERE id = ?";
pub const SQL_TOOLTIP_HTML_OF: &str = "SELECT tooltip_html FROM item WHERE id = ?";
pub const SQL_TOOLTIP_TEXT_OF: &str = "SELECT tooltip_text FROM item WHERE id = ?";
pub const SQL_TOOLTIP_AT_OF: &str = "SELECT tooltip_at FROM item WHERE id = ?";
pub const SQL_TOOLTIP_MISSING_AT_OF: &str = "SELECT tooltip_missing_at FROM item WHERE id = ?";
pub const SQL_MARK_TOOLTIP_MISSING: &str = "INSERT INTO item (id, tooltip_missing_at) VALUES (?,?) ON CONFLICT (id) DO UPDATE SET tooltip_missing_at = ?";
pub const SQL_SET_TOOLTIP: &str = "INSERT INTO item (id, tooltip_html, tooltip_etag, tooltip_at, tooltip_text, tooltip_lc) VALUES (?,?,?,?,?,?) ON CONFLICT (id) DO UPDATE SET tooltip_html = ?, tooltip_etag = ?, tooltip_at = ?, tooltip_text = ?, tooltip_lc = ?";
pub const TOOLTIP_VALUE_COUNT: usize = 6;

/// The REFRESH door (v0.9.2, ADR 0013). ADR 0012 made the twin DERIVED, so a DB
/// whose copies were written under the OLD span rule can be re-derived from the
/// copy it already holds - the copy is the source of truth. This door reads the
/// DB and writes the DB: ZERO requests leave it, so ADR 0004's network door (3)
/// cannot be reached here and the door answers 0 or 4 only. Measured on the user's
/// own file the day this door was written: 4 602 copies, ZERO of them matching the
/// corrected rule, and the stored twin equal to the RECOMPUTED v0.9.0 glued shape
/// for ALL 4 602 - the refresh is fully determined by what is on disk, so a second
/// run answers 0 rewritten. Tombstoned and manual rows ride: ADR 0012's rule is
/// that the twin always matches what the DB SAYS, and a stated tooltip_html is
/// what the DB says for those rows too. A copy whose twin was never born (a DB
/// that copied at v0.8.x - the honest gap ADR 0012 names) counts as filled, which
/// closes that gap without re-fetching anything.
pub const SQL_REFRESH_ROWS: &str = "SELECT id, tooltip_html, tooltip_text FROM item WHERE tooltip_html IS NOT NULL";
pub const SQL_REFRESH_WRITE: &str = "UPDATE item SET tooltip_text = ?, tooltip_lc = ? WHERE id = ?";

#[derive(Serialize, Clone)]
pub struct RefreshRow {
    pub id: i64,
    pub html: String,
    pub text: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct RefreshTally {
    pub copies: i64,
    pub filled: i64,
    pub rewritten: i64,
    pub unchanged: i64,
    pub complete: bool,
    pub message: String,
}

/// The tooltip door's own ETag, for the conditional GET the tooltip stage sends.
pub fn tooltip_etag_of(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let mut stmt = match conn.prepare(SQL_TOOLTIP_ETAG_OF) {
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

pub fn tooltip_at_of(conn: &Connection, id: i64) -> AppResult<Option<i64>> {
    let mut stmt = match conn.prepare(SQL_TOOLTIP_AT_OF) {
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

pub fn tooltip_missing_at_of(conn: &Connection, id: i64) -> AppResult<Option<i64>> {
    let mut stmt = match conn.prepare(SQL_TOOLTIP_MISSING_AT_OF) {
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

pub fn tooltip_html_of(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let mut stmt = match conn.prepare(SQL_TOOLTIP_HTML_OF) {
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

/// The readable twin of the copied tooltip (v0.9.0, ADR 0012): derived text, so the
/// card box can render it - the raw html fragment is printed as text only.
pub fn tooltip_text_of(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let mut stmt = match conn.prepare(SQL_TOOLTIP_TEXT_OF) {
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

/// Stage 1 write: one row of a list page. Only summary columns are touched, so the
/// stage-2 write can fill price and detail columns later without fighting this write.
pub fn upsert_summary(conn: &Connection, summary: ItemSummary, etag: String, synced_at: i64) -> AppResult<usize> {
    let mut summary: ItemSummary = summary;
    let edits: Vec<ManualEdit> = match manual_edits_of(conn, summary.id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    // The manual layer merges LAST (ADR 0010): fields the user stated survive
    // this write - a corrected name beats the next list pass - while the
    // columns the crawl owns keep refreshing.
    apply_manual_edits_summary(&mut summary, edits);
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

/// One row of the recipe table, written from ONE dataset entry (v0.10.0, ADR
/// 0021): every column names what the measured wire says (docs/API.md's second-
/// source section records the entry keys); NULL means the dataset did not state
/// it (measured: 133 of 1240 classic entries carry NO crafted id - the enchanting
/// craft has no crafted item). `reagents_json` rides the entry's reagent ARRAY
/// as TEXT bytes - the same "the DB gets the bytes" rule raw_json rides - and the
/// door resolves each reagent item_id through the item rows it already owns.
/// `learned_json` is the dataset's own object bytes (its key is `type`, a
/// reserved word nothing must reach for); the door chip reads the entry's plain
/// `acquisition` string instead. UPSERT, so a re-fetch over one spell id moves
/// a row forward, never twins it.
pub const SQL_INSERT_RECIPE: &str = concat!(
    "INSERT INTO recipe (spell_id, profession, recipe_name, crafted_item_id, crafted_item_name, skill_required, skill_tier, trivial_low, trivial_high, reagents_json, learned_json, acquisition, acquisition_note, source)",
    " VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    " ON CONFLICT (spell_id) DO UPDATE SET profession = ?, recipe_name = ?, crafted_item_id = ?, crafted_item_name = ?, skill_required = ?, skill_tier = ?, trivial_low = ?, trivial_high = ?, reagents_json = ?, learned_json = ?, acquisition = ?, acquisition_note = ?, source = ?",
);
pub const RECIPE_VALUE_COUNT: usize = 14;

pub fn write_recipe(conn: &Connection, file_profession: String, entry: RecipeEntry) -> AppResult<usize> {
    let reagents_json: String = match to_string(&entry.reagents) {
        Err(problem) => return Err(AppError::db(format!("recipe {} cannot store its reagents: {}", entry.spell_id, problem))),
        Ok(found) => found,
    };
    let learned_json: Option<String> = match entry.learned_from.clone() {
        None => None,
        Some(whole) => match to_string(&whole) {
            Err(problem) => return Err(AppError::db(format!("recipe {} cannot store its learned_from: {}", entry.spell_id, problem))),
            Ok(found) => Some(found),
        },
    };
    let mut head: Vec<Value> = [].to_vec();
    head.push(Value::Integer(entry.spell_id));
    head.push(text_value(Some(file_profession)));
    head.push(text_value(entry.spell_name));
    head.push(int_value(entry.crafted_item_id));
    head.push(text_value(entry.crafted_item_name));
    head.push(int_value(entry.required_skill_value));
    head.push(text_value(entry.skill_tier));
    head.push(int_value(entry.trivial_low));
    head.push(int_value(entry.trivial_high));
    head.push(Value::Text(reagents_json));
    head.push(text_value(learned_json));
    head.push(text_value(entry.acquisition));
    head.push(text_value(entry.acquisition_note));
    head.push(text_value(entry.source));
    return bind_sql(conn, SQL_INSERT_RECIPE, upsert_params(head), RECIPE_VALUE_COUNT * 2 - 1);
}

/// The LOCAL recipe door (v0.10.0, ADR 0021): reads the fetched rows and joins
/// them BY VALUE through the item rows the DB already owns - the cooked item
/// answers its live name, its money words and its icon, and every reagent does
/// too. NULL in a row is the dataset NOT stating it (measured: the enchanting
/// entries carry no crafted id), while a crafted/reagent id WITHOUT an item row
/// is the other answer - the DB never saw that id - so the door ships both:
/// `crafted_db_name`/`in_db` name what the DB holds, the dataset's OWN bytes
/// (`crafted_dataset_name`/`dataset_name`) ride beside them, never instead.
/// `learned_json` is stored but has no reader yet: the door chip reads the
/// entry's plain `acquisition` word (ADR 0021 records this). ORDER is honest
/// NULLS-LAST (the pair rule every list door keeps): the skill of an
/// unstated entry stays at the end of its profession, never at the top.
pub const SQL_RECIPE_ROWS: &str = "SELECT spell_id, profession, recipe_name, crafted_item_id, crafted_item_name, skill_required, skill_tier, trivial_low, trivial_high, reagents_json, acquisition, acquisition_note, source, learned_json FROM recipe ";
/// v0.20.0 (ADR 0035): a PIN is a recipe the USER keeps, and it rides the DB's OWN
/// `meta` table (`key TEXT PRIMARY KEY, value TEXT`, docs/DATABASE.md's schema 8)
/// under the prefixed-key vocabulary the crawler already rides there: the list pages
/// keep `list-page-<n>-etag`/`-more`/`-count` keys (`meta_page_key` in sync.rs) and
/// the recipe files keep `recipe_etag_<file>` keys (`recipe_etag_key`), so a per-user
/// stamp on ONE key is ALREADY this DB's vocabulary — a pin is ONE key plus ONE
/// stamp, which needs NO new table (a new table would be a schema hop, and the
/// ladder's audits would then re-learn a schema the file on disk does not have) and
/// NO new column. The key NAMES the row it pins (`pin_recipe_<spell_id>`, built by
/// `pin_key` from the ONE `PIN_PREFIX` const below), and the value is the added-at
/// stamp beside the key: the same shape `watch`'s table keeps (`id`, `added_at`),
/// never a count of a list. `pin add` writes through `SQL_SET_META`'s upsert PUT —
/// the door the ETag and cursor writers already use — so re-pinning REFRESHES the
/// stamp, the same truth `watch add` tells about its `added_at`. The rejected
/// shapes and their measures live in ADR 0035; the honest facts here are: riding
/// `watch` would put a SPELL id into the table that feeds `sync --watch` a CRAWL
/// (spell ids and item ids are two namespaces, ADR 0021/0025's measured fact), and
/// the pinned tab's rows would then never be drawn from the table they name.
pub const PIN_PREFIX: &str = "pin_recipe_";
/// The pinned set, read the honest way: an exact PREFIX test, never a LIKE.
/// Measured on a scratch DB speaking this DB's OWN schema lines: a `key LIKE
/// 'pin_recipe_%'` read ALSO answered the planted `pinXrecipeY7` and `PIN_recipe_7`
/// keys — LIKE's `_` is a single-char WILDCARD and LIKE's matching is CASE-
/// INSENSITIVE — so a LIKE would hand back keys the DB never pinned and call them
/// pinned (ADR 0035's census row keeps the measure). `substr(key, 1, 11) = ?` states
/// the ONE thing that is true of these keys (this DB's `PIN_PREFIX` leads them), with
/// the prefix BOUND — the DB binds values, it never interpolates them (the rule beside
/// `recipes_rows`' paging). The door asks for the id its key NAMES as its OWN column
/// (`substr(key, 12)`: the tail behind the 11-character prefix; `pin_key` writes the
/// SAME prefix from the SAME const, so what the door writes is what it reads, and a
/// drift in the two position numbers would answer a WRONG id rather than crash — the
/// era's audits read a pin back BY ID to name that, ADR 0011's named-lies rule) and
/// orders NUMERICALLY like `SQL_WATCH_LIST` orders `id ASC`: pinned ids ordered as
/// TEXT would put `999` after `1 000`, a lie.
pub const SQL_PINS_OF: &str = "SELECT key, substr(key, 12) FROM meta WHERE substr(key, 1, 11) = ? ORDER BY CAST(substr(key, 12) AS INTEGER) ASC";
/// A pin's removal: the key goes, and an id that carries no pin removes ZERO rows
/// (rc 0 — `SQL_WATCH_DROP`'s idempotent truth kept: `un-pinned drop` answers the
/// ask it was given, it claims NOTHING about a row it never had).
pub const SQL_PIN_DROP: &str = "DELETE FROM meta WHERE key = ?";
/// Does the recipe table hold a row of this spell id? The PIN's write guard, riding
/// db.rs's OWN guard note beside `mark_missing_at` ("see has_row_of_id, which
/// callers must consult first"): a pin is kept so the pinned tab can DRAW its rows, so a key naming a
/// row the recipe table never got would be a stored lie the next draw never answers
/// — `pin add` refuses it (rc 4) BEFORE any write. Measured contrast the OTHER way
/// at v0.20.0: `watch add 999999` answers rc 0 and stores, because a watch feeds a
/// CRAWL whose own doors answer honestly about an id they never saw. No tombstone
/// guard here: the recipe table owns no `deleted_at` column (schema 8 PUT).
pub const SQL_RECIPE_ROW_OF: &str = "SELECT COUNT(1) FROM recipe WHERE spell_id = ?";
pub const SQL_RECIPE_COUNT: &str = "SELECT COUNT(1) FROM recipe ";
pub const SQL_RECIPE_ORDER: &str = "ORDER BY profession ASC, (skill_required IS NULL) ASC, skill_required ASC, spell_id ASC";
/// v0.20.0 (ADR 0035): the PIN's WHERE half - ONE CLAUSE the recipes door joins to
/// its own (see `recipes_where` beside it). A row is PINNED when the `meta` table
/// holds the key that NAMES that row: an `EXISTS` over the meta table's OWN PRIMARY
/// KEY (`key TEXT PRIMARY KEY`, docs/DATABASE.md's schema-8 CREATE line), so the
/// join costs ONE indexed lookup per row and the SQL stays a CONSTANT size no matter
/// how many pins a user carries (measured on a scratch DB speaking this DB's OWN
/// schema lines: 3 pins over 5 rows answered 3 rows; the SAME clause ANDed with the
/// profession filter and followed by `LIMIT ? OFFSET ?` answered the pinned slice,
/// params in the order the SQL states them; and with ZERO pins the clause answers an
/// EMPTY set and the count half answers `0`, NOT an error - narrowing a set never
/// errors, ADR 0011's rule that a filter never launders a set). Measured REFUSALS
/// kept honest: SQLite 3.54 refuses the bare aliased form outright (`EXISTS (meta
/// AS pin WHERE pin.key = ?)` answered `near "meta": syntax error`), so the subquery
/// form rides. NO LIKE is used in this join: LIKE's `_` is a single-char WILDCARD and
/// LIKE's matching is CASE-INSENSITIVE (ADR 0035's census row keeps the measure where
/// a LIKE over keys answered keys the DB never pinned), and `spell_id IN (SELECT
/// CAST(substr(key, 12)` would bake the prefix's LENGTH into the door's SQL twice.
/// Zero schema line, zero new table, zero new wire field: the flag NARROWS the rows
/// the door ALREADY answers.
pub const SQL_WHERE_PINNED: &str = "EXISTS (SELECT 1 FROM meta AS pin WHERE pin.key = ? || CAST(spell_id AS TEXT))";
pub const SQL_LIVE_NAME_PRICE: &str = "SELECT name, price_min, price_min_now, price_now_at, price_now_missing_at FROM item WHERE id = ? AND deleted_at IS NULL";

#[derive(Serialize, Clone)]
pub struct RecipeReagent {
    pub item_id: i64,
    pub dataset_name: Option<String>,
    pub count: Option<i64>,
    pub in_db: bool,
    pub db_name: Option<String>,
    pub price_copper: Option<i64>,
    pub price_money: Option<String>,
    /// v0.11.0 (ADR 0023): the LIVE twin beside the stored min - the copper AND
    /// its money words ride together (ADR 0015's contract needs BOTH sides, and
    /// the compiler caught the missing twin the same way it caught E0063 at v0.10.0),
    /// and the STAMPS carry the two different NULLs apart (ADR 0017's own rule).
    pub price_now_copper: Option<i64>,
    pub price_now_money: Option<String>,
    pub price_now_at: Option<i64>,
    pub price_now_missing_at: Option<i64>,
    pub icon_b64: Option<String>,
    pub icon_at: Option<i64>,
}

#[derive(Serialize, Clone)]
pub struct RecipeRow {
    pub spell_id: i64,
    pub profession: String,
    pub recipe_name: Option<String>,
    pub skill_required: Option<i64>,
    pub skill_tier: Option<String>,
    pub trivial_low: Option<i64>,
    pub trivial_high: Option<i64>,
    pub acquisition: Option<String>,
    pub acquisition_note: Option<String>,
    pub source: Option<String>,
    pub crafted_item_id: Option<i64>,
    pub crafted_dataset_name: Option<String>,
    pub crafted_db_name: Option<String>,
    pub crafted_price_copper: Option<i64>,
    pub crafted_price_money: Option<String>,
    /// v0.11.0 (ADR 0023): the crafted item's LIVE twin - the cell shows the
    /// STORED min money and this beside it when the two differ, and the stamps
    /// keep the two different NULLs apart (the AH offered nothing at the last ask
    /// vs this record's live price was never refreshed yet).
    pub crafted_price_now_copper: Option<i64>,
    pub crafted_price_now_money: Option<String>,
    pub crafted_now_at: Option<i64>,
    pub crafted_now_missing_at: Option<i64>,
    pub crafted_icon_b64: Option<String>,
    pub crafted_icon_at: Option<i64>,
    pub reagents: Vec<RecipeReagent>,
    /// v0.17.0 (ADR 0032): the total crafting cost tallies, DERIVED from the
    /// joined `reagents` above by the ONE `craft_cost` helper the list door and
    /// the card door SHARE (ADR 0014's one-code-path rule: the tab column and
    /// the card line cannot drift, they ride the same tally). They fill ONLY
    /// when the door was asked with `--costs`; every one rides serde's
    /// `skip_serializing_if` so an unasked answer keeps its v0.10.0..v0.16.0
    /// bytes EXACTLY (the pinned-pair discipline: cost fields ADD bytes only
    /// when asked). `cost_copper` answers ONLY a COMPLETE total (money beside
    /// every copper it labels, ADR 0015); `cost_missing` counts the materials
    /// with no live price - a count with WORDS beside it, never money beside
    /// it (ADR 0015's other half). Nothing is stored: schema `5` PUT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_copper: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_money: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_missing: Option<i64>,
    /// v0.18.0 (ADR 0033): WHERE a recipe is learned from - the fetched set's
    /// OWN learned-from object read BY KEY (its `type` word, the scroll id and
    /// the set's own scroll name) plus ONE fact from the item rows this DB owns:
    /// whether a LIVE row for the named scroll exists (`learned_scroll_seen`,
    /// read through the SAME `live_name_price_of` the crafted join rides, tombstone
    /// guard included - 747 of the user's 804 scroll ids have a row; the seen
    /// fact keeps "the set names a scroll" APART from "this DB never saw it",
    /// ADR 0004/0016's two-NULLs-apart rule restated). Unasked (`--learned` off)
    /// every field rides serde's skip so the answer keeps its bytes EXACTLY.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learned_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learned_scroll_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learned_scroll_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learned_scroll_seen: Option<bool>,
}

/// The door's raw pass (v0.10.0): the fetched rows as stored, plus the reagent
/// ARRAY bytes still to join - the joins happen AFTER this read finished, so the
/// door cannot meet a reagent whose item row a write moved mid-read (the same
/// collect-then-read order the `--icons` hits follow, ADR 0014's rule).
#[derive(Serialize, Clone, Debug)]
pub struct RawRecipe {
    pub spell_id: i64,
    pub profession: String,
    pub recipe_name: Option<String>,
    pub skill_required: Option<i64>,
    pub skill_tier: Option<String>,
    pub trivial_low: Option<i64>,
    pub trivial_high: Option<i64>,
    pub reagents_json: String,
    pub crafted_item_id: Option<i64>,
    pub crafted_dataset_name: Option<String>,
    pub acquisition: Option<String>,
    pub acquisition_note: Option<String>,
    pub source: Option<String>,
    /// v0.18.0 (ADR 0033): the learned-from bytes ride the raw pass so the list
    /// door can read them AFTER the row is in hand (the collect-then-join order
    /// every join on this door follows); parsed ONLY when asked `--learned`, and
    /// the bytes themselves never answer - the answer fields below are derived.
    pub learned_json: Option<String>,
}

/// One live item answer beside an id the recipe rows name: None = the DB never
/// saw that id alive (the dataset's OWN bytes stay visible next to it).
#[derive(Clone, Debug)]
pub struct LiveNamePrice {
    pub name: Option<String>,
    pub price_min: Option<i64>,
    /// v0.11.0 (ADR 0023): the LIVE twin the price stage wrote beside the stored
    /// min (ADR 0017's two columns of one kind), read in the SAME pass so the
    /// recipe joins cannot meet a row a write moved mid-read. The STAMPS ride
    /// because the two NULLs mean different things (ADR 0006/0016's rule):
    /// `price_now_missing_at` set = "looked, the AH offers NOTHING right now",
    /// both stamps NULL = "never asked yet" - the join fills words, a door or a
    /// cell NEVER merges them.
    pub price_min_now: Option<i64>,
    pub price_now_at: Option<i64>,
    pub price_now_missing_at: Option<i64>,
}

/// The live name + min price of an id, ONLY when its row is visible (the
/// tombstone rule rides): None answer = "the DB never saw this id alive".
pub fn live_name_price_of(conn: &Connection, id: i64) -> AppResult<Option<LiveNamePrice>> {
    let mut stmt = match conn.prepare(SQL_LIVE_NAME_PRICE) {
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
            Ok(name) => match row.get(1) {
                Err(problem) => Err(AppError::db(format!("{}", problem))),
                Ok(copper) => match row.get(2) {
                    Err(problem) => Err(AppError::db(format!("{}", problem))),
                    Ok(now_copper) => match row.get(3) {
                        Err(problem) => Err(AppError::db(format!("{}", problem))),
                        Ok(now_at) => match row.get(4) {
                            Err(problem) => Err(AppError::db(format!("{}", problem))),
                            Ok(now_missing_at) => Ok(Some(LiveNamePrice {
                                name: name,
                                price_min: copper,
                                price_min_now: now_copper,
                                price_now_at: now_at,
                                price_now_missing_at: now_missing_at,
                            })),
                        },
                    },
                },
            },
        },
    }
}

/// One reagent of one row, joined through the item rows the DB owns. The
/// dataset's OWN bytes ride beside the DB's answer, never merged: a reagent
/// id the DB never saw keeps its dataset name visible and says `in_db` false.
pub fn join_recipe_reagent(conn: &Connection, reagent: Reagent, with_icons: bool) -> AppResult<RecipeReagent> {
    let found: Option<LiveNamePrice> = match live_name_price_of(conn, reagent.item_id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let mut out: RecipeReagent = RecipeReagent {
        item_id: reagent.item_id,
        dataset_name: reagent.item_name,
        count: reagent.count,
        in_db: false,
        db_name: None,
        price_copper: None,
        price_money: None,
        price_now_copper: None,
        price_now_money: None,
        price_now_at: None,
        price_now_missing_at: None,
        icon_b64: None,
        icon_at: None,
    };
    match found {
        None => {},
        Some(live) => {
            out.in_db = true;
            out.db_name = live.name;
            out.price_copper = live.price_min;
            out.price_money = money_of(live.price_min);
            out.price_now_copper = live.price_min_now;
            out.price_now_money = money_of(live.price_min_now);
            out.price_now_at = live.price_now_at;
            out.price_now_missing_at = live.price_now_missing_at;
            match with_icons {
                false => {},
                true => {
                    out.icon_b64 = match icon_b64_of(conn, reagent.item_id) {
                        Err(problem) => return Err(problem),
                        Ok(value) => value,
                    };
                    out.icon_at = match icon_at_of(conn, reagent.item_id) {
                        Err(problem) => return Err(problem),
                        Ok(value) => value,
                    };
                },
            }
        },
    }
    return Ok(out);
}

/// The TOTAL crafting cost over ONE joined material list (v0.17.0, ADR 0032):
/// ONE helper the recipes LIST door and the RECIPE CARD door share (ADR 0014's
/// one-code-path rule: the tab's cost column and the card's cost line cannot
/// drift). The sum rides the join's OWN fields - count x `price_now_copper` -
/// the DB's live price layer (the `price_min_now` twin the price refresh
/// writes beside the stored min, docs/DATABASE.md), NEVER the stored min
/// (the ask says LIVE prices) and NEVER a second fetch (the local-door rule,
/// ZERO requests). Honest states, never invented numbers (ADR 0004's rule
/// again): a reagent whose join wrote no live copper - not in the DB at all
/// (measured over the fetched set: 136 of 3 778 materials) or in the DB with
/// no auction offered (ADR 0016's two different NULL homes ride apart in the
/// per-material cells ALREADY beside this count) - rides as a COUNT in
/// `cost_missing`. `cost_copper` answers ONLY when EVERY material is priced:
/// money rides beside the exact total (ADR 0015), and partial or empty costs
/// answer counts + words with ZERO money beside them - an "at least" money
/// word invites reading an incomplete sum as the recipe's cost, rejected at
/// the ADR with measures. The multiplier is the set's OWN stated count
/// (measured: every classic reagent entry states a whole count >= 1); a NULL
/// count reads as ONE, the minimum a craft's own material word cannot
/// undercut, and nothing here claims the set stated it.
pub struct CostTally {
    pub cost_copper: Option<i64>,
    pub cost_money: Option<String>,
    pub cost_missing: Option<i64>,
}

/// The tally itself (v0.17.0, ADR 0032): called ONLY under the door's OWN
/// `costs` flag, over the reagents that door has ALREADY joined.
pub fn craft_cost(reagents: Vec<RecipeReagent>) -> CostTally {
    let mut total: i64 = 0;
    let mut missing: i64 = 0;
    for one in reagents.clone() {
        match one.price_now_copper {
            None => missing = missing + 1,
            Some(copper) => {
                let times: i64 = match one.count {
                    None => 1,
                    Some(said) => if said > 1 { said } else { 1 },
                };
                total = total + times * copper;
            },
        }
    }
    let complete: bool = match reagents.len() {
        0 => false,
        _ => missing == 0,
    };
    let copper: Option<i64> = match complete {
        false => None,
        true => Some(total),
    };
    return CostTally {
        cost_copper: copper.clone(),
        cost_money: money_of(copper.clone()),
        cost_missing: Some(missing),
    };
}

/// v0.20.0 (ADR 0035): the recipes door's WHERE half, IN ONE PLACE for BOTH of its
/// doors (the rows read and the count echo). The door builds a WHERE CLAUSE from
/// clauses (a bare filter half is `""` = the table's own set), and every clause rides
/// WITH its bound values in the textual order, because SQLite numbers bare `?` by
/// TEXTUAL order over the whole line while `params` answers POSITIONALLY: a clause
/// that moved without its value would bind the wrong needle (the door's OWN rule,
/// stated beside its paging since v0.10.0). `LIMIT`/`OFFSET` values come from
/// `paging_clause` AFTER these and the caller appends them LAST.
pub fn recipes_where(profession: Option<String>, pinned: bool) -> (String, Vec<Value>) {
    let mut clauses: Vec<String> = [].to_vec();
    let mut bounds: Vec<Value> = [].to_vec();
    match profession {
        None => {},
        Some(word) => {
            clauses.push("profession = ?".to_string());
            bounds.push(Value::Text(word));
        },
    }
    match pinned {
        false => {},
        true => {
            clauses.push(SQL_WHERE_PINNED.to_string());
            bounds.push(Value::Text(PIN_PREFIX.to_string()));
        },
    }
    let mut joined: String = "".to_string();
    for one in clauses.clone() {
        if joined != "" {
            joined = format!("{} AND ", joined);
        }
        joined = format!("{}{}", joined, one);
    }
    let part: String = match clauses.len() {
        0 => "".to_string(),
        _ => format!("WHERE {} ", joined),
    };
    return (part, bounds);
}

/// Read the recipe rows of one door call: filter echo, ORDER, LIMIT/OFFSET are
/// bound values (never interpolated), and every reagent + the crafted item are
/// joined through the item rows AFTER the rows are collected - the door cannot
/// meet a reagent whose item row a write moved mid-read. v0.10.1 (ADR 0022): the
/// paging clause comes from the SAME `paging_clause` the search door rides - two
/// doors composing `OFFSET` alone was one defect in two places, and one rule now
/// answers it twice (`LIMIT -1` is SQLite's own "no bound", measured today).
/// v0.17.0 (ADR 0032): `costs` asks the door to fill each row's cost tallies
/// through the ONE `craft_cost` helper the card door shares; unasked the row
/// JSON keeps the bytes it answered since v0.10.0 (serde skip on three fields).
pub fn recipes_rows(conn: &Connection, profession: Option<String>, pinned: bool, with_icons: bool, costs: bool, learned: bool, limit: Option<i64>, offset: Option<i64>) -> AppResult<Vec<RecipeRow>> {
    // v0.20.0 (ADR 0035): the WHERE half comes from the SAME helper the count door
    // rides - as of v0.19.0 the two doors each wrote their own copy of
    // "WHERE profession = ? ", which is one rule in two places (ADR 0014's rule,
    // the same one the paging clause answers twice, ADR 0022); now ONE helper
    // answers BOTH doors. Clauses and their bound values move together in the order
    // the SQL states them, so `?` numbering and `params` cannot disagree about the
    // pin's WHERE half.
    let filter: (String, Vec<Value>) = recipes_where(profession.clone(), pinned);
    let filter_part: String = filter.0;
    let mut sql: String = format!("{}{}{} ", SQL_RECIPE_ROWS, filter_part.clone(), SQL_RECIPE_ORDER);
    let mut params: Vec<Value> = [].to_vec();
    for bound in filter.1 {
        params.push(bound.clone());
    }
    let page: (String, Vec<i64>) = crate::query::paging_clause(limit, offset);
    sql = format!("{}{}", sql, page.0);
    for whole in page.1 {
        params.push(Value::Integer(whole));
    }
    let mut stmt = match conn.prepare(&sql.clone()) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mapped = match stmt.query_map(params_from_iter(params), |row|
        Ok(RawRecipe {
            spell_id: row.get(0)?,
            profession: row.get(1)?,
            recipe_name: row.get(2)?,
            crafted_item_id: row.get(3)?,
            crafted_dataset_name: row.get(4)?,
            skill_required: row.get(5)?,
            skill_tier: row.get(6)?,
            trivial_low: row.get(7)?,
            trivial_high: row.get(8)?,
            reagents_json: row.get(9)?,
            acquisition: row.get(10)?,
            acquisition_note: row.get(11)?,
            source: row.get(12)?,
            learned_json: row.get(13)?,
        })
    ) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut out: Vec<RecipeRow> = [].to_vec();
    for item in mapped {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(raw) => {
                let reagents: Vec<Reagent> = match from_str(&raw.reagents_json.clone()) {
                    Err(problem) => return Err(AppError::decode(format!("stored reagents of recipe {} are not the array the writer described: {}", raw.spell_id, problem))),
                    Ok(found) => found,
                };
                let mut row: RecipeRow = RecipeRow {
                    spell_id: raw.spell_id,
                    profession: raw.profession.clone(),
                    recipe_name: raw.recipe_name.clone(),
                    skill_required: raw.skill_required.clone(),
                    skill_tier: raw.skill_tier.clone(),
                    trivial_low: raw.trivial_low.clone(),
                    trivial_high: raw.trivial_high.clone(),
                    acquisition: raw.acquisition.clone(),
                    acquisition_note: raw.acquisition_note.clone(),
                    source: raw.source.clone(),
                    crafted_item_id: raw.crafted_item_id.clone(),
                    crafted_dataset_name: raw.crafted_dataset_name.clone(),
                    crafted_db_name: None,
                    crafted_price_copper: None,
                    crafted_price_money: None,
                    crafted_price_now_copper: None,
                    crafted_price_now_money: None,
                    crafted_now_at: None,
                    crafted_now_missing_at: None,
                    crafted_icon_b64: None,
                    crafted_icon_at: None,
                    reagents: [].to_vec(),
                    cost_copper: None,
                    cost_money: None,
                    cost_missing: None,
                    learned_type: None,
                    learned_scroll_id: None,
                    learned_scroll_name: None,
                    learned_scroll_seen: None,
                };
                for one in reagents.clone() {
                    let joined: RecipeReagent = match join_recipe_reagent(conn, one.clone(), with_icons) {
                        Err(problem) => return Err(problem),
                        Ok(value) => value,
                    };
                    row.reagents.push(joined);
                }
                match costs {
                    false => {},
                    true => {
                        let tally: CostTally = craft_cost(row.reagents.clone());
                        row.cost_copper = tally.cost_copper.clone();
                        row.cost_money = tally.cost_money.clone();
                        row.cost_missing = tally.cost_missing.clone();
                    },
                }
                match learned {
                    false => {},
                    true => {
                        // v0.18.0 (ADR 0033): the SAME read the card door has done
                        // since v0.12.0 (ADR 0025) - same `from_str` decode shape, same
                        // `word_at`/`id_at` by-key helpers, same decode error WORDS - so
                        // the tab column and the card block cannot drift about one row
                        // (ADR 0014's one-code-path rule, restated at the learned facts).
                        let whole: Option<serde_json::Value> = match raw.learned_json.clone() {
                            None => None,
                            Some(blob) => match from_str(&blob.clone()) {
                                Err(problem) => return Err(AppError::decode(format!("stored learned_from of recipe {} is not the object the writer described: {}", raw.spell_id, problem))),
                                Ok(found) => Some(found),
                            },
                        };
                        match whole {
                            None => {},
                            Some(object) => {
                                row.learned_type = word_at(object.clone(), "type".to_string());
                                row.learned_scroll_id = id_at(object.clone(), "item_id".to_string());
                                row.learned_scroll_name = word_at(object.clone(), "item_name".to_string());
                            },
                        }
                        match row.learned_scroll_id.clone() {
                            None => {},
                            Some(id) => {
                                // ONE local read, no request, no second shape: the SAME
                                // live reader the crafted-item join on this very door rides
                                // (SQL_LIVE_NAME_PRICE, tombstone guard included) answers
                                // whether a live row for the named scroll exists here.
                                let seen: Option<LiveNamePrice> = match live_name_price_of(conn, id) {
                                    Err(problem) => return Err(problem),
                                    Ok(value) => value,
                                };
                                row.learned_scroll_seen = Some(match seen { None => false, Some(_) => true });
                            },
                        }
                    },
                }
                match raw.crafted_item_id.clone() {
                    None => {},
                    Some(id) => {
                        let found: Option<LiveNamePrice> = match live_name_price_of(conn, id) {
                            Err(problem) => return Err(problem),
                            Ok(value) => value,
                        };
                        match found {
                            None => {},
                            Some(live) => {
                                row.crafted_db_name = live.name.clone();
                                row.crafted_price_copper = live.price_min.clone();
                                row.crafted_price_money = money_of(live.price_min.clone());
                                row.crafted_price_now_copper = live.price_min_now.clone();
                                row.crafted_price_now_money = money_of(live.price_min_now.clone());
                                row.crafted_now_at = live.price_now_at.clone();
                                row.crafted_now_missing_at = live.price_now_missing_at.clone();
                                match with_icons {
                                    false => {},
                                    true => {
                                        row.crafted_icon_b64 = match icon_b64_of(conn, id) {
                                            Err(problem) => return Err(problem),
                                            Ok(value) => value,
                                        };
                                        row.crafted_icon_at = match icon_at_of(conn, id) {
                                            Err(problem) => return Err(problem),
                                            Ok(value) => value,
                                        };
                                    },
                                }
                            },
                        }
                    },
                }
                out.push(row);
            },
        }
    }
    return Ok(out);
}

/// v0.20.0 (ADR 0035): the PIN doors, mirroring the `watch` trio's shape (ADR
/// 0025's `add`/`list`/`drop`) over the DB's OWN `meta` keys. `pin_key` builds a
/// key from the ONE `PIN_PREFIX` const; `pin add` writes through `meta_set`, whose
/// `SQL_SET_META` upsert the ETag and count writers already ride, so re-pinning
/// REFRESHES the stamp — the same truth `watch add` tells about its `added_at`, and
/// a pin's key never doubles (the meta table's PRIMARY KEY is `key`). `pin drop`
/// deletes the key; an id that carries no pin removes ZERO rows and answers rc 0
/// (`SQL_WATCH_DROP`'s idempotent truth kept: the door claims NOTHING about a row
/// it never had). `pin list` answers the pinned set as ids in the DB's OWN numeric
/// order. `has_recipe_row` guards a pin's WRITE (the `SQL_ROW_OF_ID`/`has_row_of_id`
/// guard `mark_missing_at` consults first, db.rs's own precedent): a pin is kept so
/// the pinned tab can DRAW its rows, so a key naming a row the recipe table never
/// got is a stored lie the next draw never answers — and the guard runs over EVERY
/// asked id BEFORE any write, so a batch with one unknown id stores NOTHING (measured
/// at v0.20.0: `pin add 132 999999` answered rc 4 naming 999999 and `pin list`
/// stayed empty, while `watch add 999999` answered rc 0 and stored, because a watch
/// feeds a CRAWL whose own doors answer honestly about an id they never saw).
pub fn pin_key(spell_id: i64) -> String {
    return format!("{}{}", PIN_PREFIX, spell_id);
}

pub fn pin_add(conn: &Connection, spell_id: i64, added_at: i64) -> AppResult<usize> {
    return meta_set(conn, pin_key(spell_id), format!("{}", added_at));
}

pub fn pin_drop(conn: &Connection, spell_id: i64) -> AppResult<usize> {
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Text(pin_key(spell_id)));
    return bind_sql(conn, SQL_PIN_DROP, single, 1);
}

/// The pinned set as ids, in the DB's OWN numeric order (`SQL_WATCH_LIST`'s
/// `ORDER BY id ASC` precedent: pinned ids ordered as TEXT would put `999` after
/// `1 000`, a lie). The door reads the key AND the id its key NAMES (`SQL_PINS_OF`
/// hands the tail as its OWN column) and PARSES the tail — the same `parse` pass
/// `cli.rs`'s `as_int` and `db.rs`'s schema-version read ride; a tail that is not a
/// number is a stored byte this DB's writer never wrote, so it answers a decode
/// error that NAMES the key (ADR 0011's rule that stored bytes get checked, the
/// same shape `recipes_rows` answers for a `reagents_json` that is not the array the
/// writer described). The stamp the value carries beside the key rides PUT here:
/// `pin list` answers its OWN set of ids, and the pinned tab's rows ride the
/// recipes door's OWN order (`SQL_RECIPE_ORDER`), never a stamp order of their own.
#[derive(Serialize, Clone, Debug)]
pub struct RawPin {
    pub key: String,
    pub id_text: Option<String>,
}

pub fn pin_list(conn: &Connection) -> AppResult<Vec<i64>> {
    let mut stmt = match conn.prepare(SQL_PINS_OF) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut bounds: Vec<Value> = [].to_vec();
    bounds.push(Value::Text(PIN_PREFIX.to_string()));
    let mapped = match stmt.query_map(params_from_iter(bounds), |row|
        Ok(RawPin {
            key: row.get(0)?,
            id_text: row.get(1)?,
        })
    ) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut out: Vec<i64> = [].to_vec();
    for item in mapped {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(raw) => {
                match raw.id_text {
                    None => return Err(AppError::decode(format!("a pin key carries no id tail the writer described: {}", raw.key))),
                    Some(tail) => match tail.parse() {
                        Err(_) => return Err(AppError::decode(format!("a pin key's id tail is not a number the writer described: {}", raw.key))),
                        Ok(number) => out.push(number),
                    },
                }
            },
        }
    }
    return Ok(out);
}

pub fn has_recipe_row(conn: &Connection, spell_id: i64) -> AppResult<bool> {
    let mut stmt = match conn.prepare(SQL_RECIPE_ROW_OF) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Integer(spell_id));
    let mut rows = match stmt.query(params_from_iter(single)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    match rows.next() {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(None) => return Ok(false),
        Ok(Some(row)) => {
            let count: i64 = match row.get(0) {
                Err(problem) => return Err(AppError::db(format!("{}", problem))),
                Ok(value) => value,
            };
            return Ok(count > 0);
        },
    }
}

/// How many recipe rows a profession filter matches: the same WHERE chain as
/// the rows read, so the door and its count cannot drift (ADR 0009's "showing
/// 40 of 1250" rule, kept for the new door too).
pub fn recipes_count(conn: &Connection, profession: Option<String>, pinned: bool) -> AppResult<i64> {
    // v0.20.0 (ADR 0035): the WHERE half comes from the SAME `recipes_where` helper
    // the rows read ride, so the door and its count cannot drift about the pin's
    // filter (ADR 0009's "showing 40 of 1 250" rule, kept for the new door too; the
    // rule `recipes_count` itself stated since v0.10.0, which as of v0.19.0 was kept
    // by TWO doors each writing its own copy of the profession half).
    let filter: (String, Vec<Value>) = recipes_where(profession.clone(), pinned);
    let sql: String = format!("{}{}", SQL_RECIPE_COUNT, filter.0.clone());
    match conn.query_row(&sql.clone(), params_from_iter(filter.1), |row| row.get(0)) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(total) => return Ok(total),
    }
}

/// THE RECIPE CARD door (v0.12.0, ADR 0025): one entry of the fetched set, shown
/// as far as the TWO sources this tool owns say it - nothing is invented here.
/// The SET carries the entry's OWN words (job, name, skill band, materials) and
/// states the SCROLL that teaches it BY VALUE in `learned_json`; the item rows
/// this DB owns carry the IN-GAME cells (name, quality, level, the game's money,
/// the tooltip copy, the icon bytes). The join censuses are MEASURED on the user's
/// own file (ADR 0025's table): of 1 240 entries the set's `learned_json` answers
/// 804 `schematic_item` (747 of those ids have an item row, 477 of those carry the
/// scroll's OWN tooltip copy), 413 `trainer` and 23 `automatic` name no scroll; of
/// 1 240 entries 1 095 name a crafted id (1 095 have rows, 899 carry copies) and
/// 133 enchanting entries name NO crafted item at all. REJECTED as joins, measured
/// the same day: `spell_id` as an item id (651 land on a row but only 20 are
/// `Recipe` rows - spell ids and item ids are different namespaces) and the name
/// link `"Recipe: " + crafted name` (92 of 1 240 reach: the DB's own names differ
/// from the set's). Every gap keeps its OWN state word (ADR 0004's no-invented-
/// answers rule), and ZERO requests leave this door (ADR 0009/0012's local rule).
pub const SQL_RECIPE_CARD_ROW: &str = "SELECT spell_id, profession, recipe_name, crafted_item_id, crafted_item_name, skill_required, skill_tier, trivial_low, trivial_high, reagents_json, learned_json, acquisition, acquisition_note, source FROM recipe WHERE spell_id = ? ";

/// The four columns the LIST door never reads, for one visible id: the list join
/// answers name + money + icon through `live_name_price_of` and its readers, and
/// the card needs MORE of the in-game shape - quality, the two levels, the vendor
/// sell. The SAME tombstone guard `SQL_LIVE_NAME_PRICE` rides, so a row the user
/// buried stays buried in the card too (ADR 0010's rule on every join).
pub const SQL_CARD_CELL: &str = "SELECT quality, item_level, required_level, vendor_sell FROM item WHERE id = ? AND deleted_at IS NULL";

/// The card door's raw pass: one fetched row AS STORED, plus the `learned_json`
/// bytes still to read - the LIST door's SELECT never reaches that column (ADR
/// 0021 recorded it as stored-with-no-reader), so this door reads its own const.
/// Collect-then-join order like the list door: the cells are read AFTER this row
/// is in hand, so the door cannot meet an item row a write moved mid-read.
#[derive(Clone, Debug)]
pub struct RawRecipeCard {
    pub spell_id: i64,
    pub profession: String,
    pub recipe_name: Option<String>,
    pub crafted_item_id: Option<i64>,
    pub crafted_dataset_name: Option<String>,
    pub skill_required: Option<i64>,
    pub skill_tier: Option<String>,
    pub trivial_low: Option<i64>,
    pub trivial_high: Option<i64>,
    pub reagents_json: String,
    pub learned_json: Option<String>,
    pub acquisition: Option<String>,
    pub acquisition_note: Option<String>,
    pub source: Option<String>,
}

/// One cell of the card: what the item rows this DB owns say about ONE id the set
/// names. `in_db` false = "the DB never saw this id ALIVE" (the tombstone rule ADR
/// 0010 put on every join rides: a buried row answers through the set's bytes
/// alone, never through the columns the user buried) and the set's OWN words ride
/// BESIDE it, never instead of it (ADR 0021's side-by-side rule, kept here so the
/// card and a list row say the SAME thing about the same id - ADR 0014's one-code-
/// path rule). Money rides EVERY copper answer this cell carries (ADR 0015: the
/// vendor sell, the stored min, the live twin; NO auction count is carried here,
/// so the contract's count half never comes into play). The stamps keep the NULLS
/// apart (ADR 0016/0017's rule): `price_now_missing_at` set = "looked, the AH
/// offers NOTHING right now", both stamps NULL = "never refreshed yet" - the cell
/// ships BOTH facts and never merges them into one answer. The tooltip copy rides
/// as the readable twin ONLY (ADR 0012): upstream's OWN markup stays out of this
/// answer, because pasting an upstream body into a card would let upstream write
/// the page.
#[derive(Serialize, Clone)]
pub struct RecipeCell {
    pub item_id: i64,
    pub in_db: bool,
    pub dataset_name: Option<String>,
    pub db_name: Option<String>,
    pub quality: Option<String>,
    pub item_level: Option<i64>,
    pub required_level: Option<i64>,
    pub vendor_sell: Option<i64>,
    pub vendor_sell_money: Option<String>,
    pub price_copper: Option<i64>,
    pub price_money: Option<String>,
    pub price_now_copper: Option<i64>,
    pub price_now_money: Option<String>,
    pub price_now_at: Option<i64>,
    pub price_now_missing_at: Option<i64>,
    pub tooltip_text: Option<String>,
    pub tooltip_at: Option<i64>,
    pub tooltip_missing_at: Option<i64>,
    pub icon_b64: Option<String>,
    pub icon_at: Option<i64>,
    pub icon_missing_at: Option<i64>,
}

/// One string key of a stored Value: the KEY stays a string, never a Rust field -
/// `type` is a reserved word and models.rs' note says NOTHING may reach for it
/// (ADR 0021's reserved-word rule, kept here for the door's own read). A key the
/// stored object does not carry answers None, honestly.
pub fn word_at(whole: serde_json::Value, key: String) -> Option<String> {
    match whole.get(key.clone()) {
        None => return None,
        Some(found) => match found.as_str() {
            None => return None,
            Some(text) => return Some(text.to_string()),
        },
    }
}

/// One whole-number key of a stored Value, same rule as `word_at` above: the key
/// is a string, and a value that is NOT a number answers None, never a 0.
pub fn id_at(whole: serde_json::Value, key: String) -> Option<i64> {
    match whole.get(key.clone()) {
        None => return None,
        Some(found) => return match found.as_i64() {
            None => None,
            Some(whole_number) => Some(whole_number.clone()),
        },
    }
}

/// One card cell, read through the SAME readers the item card and the list joins
/// already use (ADR 0014's rule: a card cell CANNOT drift from what the item card
/// says about the same id). The name, the money and the live twin come from
/// `live_name_price_of` (visible rows only); the stamp readers above carry NO
/// tombstone guard of their own, so they are asked ONLY for a row this cell saw
/// alive - a buried row keeps its stamps buried here. The icon bytes ride the
/// `--icons` opt-in ADR 0014 measured (a copy is about 3.1 K chars once encoded);
/// this door is bounded by ONE spell id, but the flag keeps the rule honest.
pub fn recipe_cell(conn: &Connection, id: i64, dataset_name: Option<String>, with_icons: bool) -> AppResult<RecipeCell> {
    let mut cell: RecipeCell = RecipeCell {
        item_id: id,
        in_db: false,
        dataset_name: dataset_name.clone(),
        db_name: None,
        quality: None,
        item_level: None,
        required_level: None,
        vendor_sell: None,
        vendor_sell_money: None,
        price_copper: None,
        price_money: None,
        price_now_copper: None,
        price_now_money: None,
        price_now_at: None,
        price_now_missing_at: None,
        tooltip_text: None,
        tooltip_at: None,
        tooltip_missing_at: None,
        icon_b64: None,
        icon_at: None,
        icon_missing_at: None,
    };
    let known: Option<LiveNamePrice> = match live_name_price_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match known {
        None => {},
        Some(live) => {
            cell.in_db = true;
            cell.db_name = live.name.clone();
            cell.price_copper = live.price_min.clone();
            cell.price_money = money_of(live.price_min.clone());
            cell.price_now_copper = live.price_min_now.clone();
            cell.price_now_money = money_of(live.price_min_now.clone());
            cell.price_now_at = live.price_now_at.clone();
            cell.price_now_missing_at = live.price_now_missing_at.clone();
        },
    }
    match cell.in_db {
        false => {},
        true => {
            let mut stmt = match conn.prepare(SQL_CARD_CELL) {
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
                Ok(None) => {},
                Ok(Some(row)) => {
                    cell.quality = match row.get(0) {
                        Err(problem) => return Err(AppError::db(format!("{}", problem))),
                        Ok(value) => value,
                    };
                    cell.item_level = match row.get(1) {
                        Err(problem) => return Err(AppError::db(format!("{}", problem))),
                        Ok(value) => value,
                    };
                    cell.required_level = match row.get(2) {
                        Err(problem) => return Err(AppError::db(format!("{}", problem))),
                        Ok(value) => value,
                    };
                    cell.vendor_sell = match row.get(3) {
                        Err(problem) => return Err(AppError::db(format!("{}", problem))),
                        Ok(value) => value,
                    };
                    cell.vendor_sell_money = money_of(cell.vendor_sell.clone());
                },
            }
            cell.tooltip_text = match tooltip_text_of(conn, id) {
                Err(problem) => return Err(problem),
                Ok(value) => value,
            };
            cell.tooltip_at = match tooltip_at_of(conn, id) {
                Err(problem) => return Err(problem),
                Ok(value) => value,
            };
            cell.tooltip_missing_at = match tooltip_missing_at_of(conn, id) {
                Err(problem) => return Err(problem),
                Ok(value) => value,
            };
            match with_icons {
                false => {},
                true => {
                    cell.icon_b64 = match icon_b64_of(conn, id) {
                        Err(problem) => return Err(problem),
                        Ok(value) => value,
                    };
                    cell.icon_at = match icon_at_of(conn, id) {
                        Err(problem) => return Err(problem),
                        Ok(value) => value,
                    };
                    cell.icon_missing_at = match icon_missing_at_of(conn, id) {
                        Err(problem) => return Err(problem),
                        Ok(value) => value,
                    };
                },
            }
        },
    }
    return Ok(cell);
}

/// Read ONE recipe entry as the card answers it (v0.12.0, ADR 0025): the entry's
/// OWN set words, the learned-from object read BY KEY (its `type` word, the id and
/// name the SET stated), and the cells the two ids name - the scroll that teaches
/// this recipe and the item it crafts - through the joins the list door ALREADY
/// uses (`join_recipe_reagent` for the materials, `recipe_cell` for the two named
/// ids). A spell id the table does not hold answers None: the door turns that into
/// rc 4 (ADR 0004's no-invented-answers rule, the same shape the empty recipe
/// table answered at v0.10.0). `spell_id` is the table's PRIMARY KEY (docs/
/// DATABASE.md), so at most one row can answer this read. v0.17.0 (ADR 0032):
/// `with_costs` asks the door to fill the SAME cost tallies the list door fills
/// (ONE `craft_cost` over the already-joined materials - the card's cost line
/// and the tab's cost column cannot drift, ADR 0014's one-code-path rule);
/// unasked they ride absent (serde skip), so a `recipe N` line keeps its
/// v0.12.0..v0.16.0 bytes PUT (the pinned-pair discipline).
pub fn recipe_card(conn: &Connection, spell_id: i64, with_icons: bool, with_costs: bool) -> AppResult<Option<RecipeCard>> {
    let mut stmt = match conn.prepare(SQL_RECIPE_CARD_ROW) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut single: Vec<Value> = [].to_vec();
    single.push(Value::Integer(spell_id));
    let mut mapped = match stmt.query_map(params_from_iter(single), |row|
        Ok(RawRecipeCard {
            spell_id: row.get(0)?,
            profession: row.get(1)?,
            recipe_name: row.get(2)?,
            crafted_item_id: row.get(3)?,
            crafted_dataset_name: row.get(4)?,
            skill_required: row.get(5)?,
            skill_tier: row.get(6)?,
            trivial_low: row.get(7)?,
            trivial_high: row.get(8)?,
            reagents_json: row.get(9)?,
            learned_json: row.get(10)?,
            acquisition: row.get(11)?,
            acquisition_note: row.get(12)?,
            source: row.get(13)?,
        })
    ) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    // `spell_id` is the table's PRIMARY KEY, so AT MOST ONE row answers this read:
    // the read takes the row, it does not walk a table (the established single-row
    // doors ride the same shape; a `for` here could never take a second turn, which
    // is what `clippy::never_loop` called an ERROR at v0.12.0 - fixed BY NAME here,
    // never muted, ADR 0024's rule kept)
    if let Some(one_row) = mapped.next() {
        match one_row {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(raw) => {
                let reagents: Vec<Reagent> = match from_str(&raw.reagents_json.clone()) {
                    Err(problem) => return Err(AppError::decode(format!("stored reagents of recipe {} are not the array the writer described: {}", raw.spell_id, problem))),
                    Ok(found) => found,
                };
                let learned: Option<serde_json::Value> = match raw.learned_json.clone() {
                    None => None,
                    Some(blob) => match from_str(&blob.clone()) {
                        Err(problem) => return Err(AppError::decode(format!("stored learned_from of recipe {} is not the object the writer described: {}", raw.spell_id, problem))),
                        Ok(found) => Some(found),
                    },
                };
                let mut card: RecipeCard = RecipeCard {
                    spell_id: raw.spell_id.clone(),
                    profession: raw.profession.clone(),
                    recipe_name: raw.recipe_name.clone(),
                    skill_required: raw.skill_required.clone(),
                    skill_tier: raw.skill_tier.clone(),
                    trivial_low: raw.trivial_low.clone(),
                    trivial_high: raw.trivial_high.clone(),
                    acquisition: raw.acquisition.clone(),
                    acquisition_note: raw.acquisition_note.clone(),
                    source: raw.source.clone(),
                    learned_type: None,
                    learned_scroll_id: None,
                    learned_scroll_name: None,
                    scroll: None,
                    result: None,
                    materials: [].to_vec(),
                    cost_copper: None,
                    cost_money: None,
                    cost_missing: None,
                };
                match learned {
                    None => {},
                    Some(whole) => {
                        card.learned_type = word_at(whole.clone(), "type".to_string());
                        card.learned_scroll_id = id_at(whole.clone(), "item_id".to_string());
                        card.learned_scroll_name = word_at(whole.clone(), "item_name".to_string());
                    },
                }
                for one in reagents.clone() {
                    let joined: RecipeReagent = match join_recipe_reagent(conn, one.clone(), with_icons) {
                        Err(problem) => return Err(problem),
                        Ok(value) => value,
                    };
                    card.materials.push(joined.clone());
                }
                match raw.crafted_item_id.clone() {
                    None => {},
                    Some(id) => {
                        let cell: RecipeCell = match recipe_cell(conn, id, raw.crafted_dataset_name.clone(), with_icons) {
                            Err(problem) => return Err(problem),
                            Ok(value) => value,
                        };
                        card.result = Some(cell.clone());
                    },
                }
                match card.learned_scroll_id.clone() {
                    None => {},
                    Some(id) => {
                        let cell: RecipeCell = match recipe_cell(conn, id, card.learned_scroll_name.clone(), with_icons) {
                            Err(problem) => return Err(problem),
                            Ok(value) => value,
                        };
                        card.scroll = Some(cell.clone());
                    },
                }
                match with_costs {
                    false => {},
                    true => {
                        let tally: CostTally = craft_cost(card.materials.clone());
                        card.cost_copper = tally.cost_copper.clone();
                        card.cost_money = tally.cost_money.clone();
                        card.cost_missing = tally.cost_missing.clone();
                    },
                }
                return Ok(Some(card));
            },
        }
    }
    return Ok(None);
}

/// The card's answer shape: the entry's OWN words (job, name, the skill band the
/// game states as a number plus a tier word plus its trivial band) plus what the
/// learned-from object NAMED (its own `type` word, the scroll id and the set's own
/// name for that scroll) plus the three kinds of cell: the scroll that teaches it,
/// the item it crafts, the materials it asks for. Every NULL here is a source
/// NOT stating something, never a value this tool guessed (ADR 0004's rule again).
#[derive(Serialize, Clone)]
pub struct RecipeCard {
    pub spell_id: i64,
    pub profession: String,
    pub recipe_name: Option<String>,
    pub skill_required: Option<i64>,
    pub skill_tier: Option<String>,
    pub trivial_low: Option<i64>,
    pub trivial_high: Option<i64>,
    pub acquisition: Option<String>,
    pub acquisition_note: Option<String>,
    pub source: Option<String>,
    pub learned_type: Option<String>,
    pub learned_scroll_id: Option<i64>,
    pub learned_scroll_name: Option<String>,
    pub scroll: Option<RecipeCell>,
    pub result: Option<RecipeCell>,
    pub materials: Vec<RecipeReagent>,
    /// v0.17.0 (ADR 0032): the SAME three tallies the list rows carry, filled
    /// by the SAME `craft_cost` when the door was asked `--costs` - the card
    /// line and the tab column ride ONE answer (ADR 0014). Unasked they ride
    /// absent (serde skip), so a `recipe N` line keeps its bytes PUT (the
    /// pinned-pair discipline); money rides ONLY a complete total (ADR 0015).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_copper: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_money: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_missing: Option<i64>,
}

/// Stage 2 write: one detail record. `raw_json` keeps the verbatim payload so a future
/// schema bump can read fields this struct does not model yet.
pub fn upsert_detail(conn: &Connection, detail: ItemDetail, etag: String, raw_json: String, synced_at: i64) -> AppResult<usize> {
    let mut detail: ItemDetail = detail;
    let edits: Vec<ManualEdit> = match manual_edits_of(conn, detail.id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    // Same merge as the list write (ADR 0010): user-said TEXT/INTEGER fields
    // overwrite the payload's before they bind; ADR 0007's sample still rides
    // the DETAIL write unchanged - an override never adds its own ledger row.
    apply_manual_edits_detail(&mut detail, edits);
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
    // a record that answered is not a missing record: clear any older check.
    let cleared: Option<i64> = None;
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
    head.push(int_value(cleared));
    head.push(text_value(Some(raw_json)));
    let written: usize = match bind_sql(conn, UPSERT_DETAIL_SQL, upsert_params(head), DETAIL_VALUE_COUNT * 2 - 1) {
        Err(problem) => return Err(problem),
        Ok(count) => count,
    };
    // A fresh (200) write IS an observation, so it drops one sample point
    // (v0.6.0, ADR 0007): zero extra requests, and the trend is made of the
    // visits the user already runs. A 304 never reaches this helper (no new
    // numbers were answered), and a fetched-but-not-on-the-AH record has no
    // price to plot - absence of points, not blank points. The item row above
    // is what makes the sample's item_id real: SQLite does not enforce FK
    // clauses (measured: an INSERT referencing a missing item succeeded), so
    // the guard is this ORDER, in Rust - docs/DATABASE.md.
    match sample_price(conn, detail.id, synced_at, p_min.clone(), p_market.clone()) {
        Err(problem) => return Err(problem),
        Ok(_) => {},
    }
    return Ok(written);
}

/// Writes one price observation of one item: only ever called by upsert_detail
/// AFTER the item row exists, which is the FK guard SQLite will not give us.
pub fn sample_price(conn: &Connection, id: i64, sampled_at: i64, price_min: Option<i64>, price_market: Option<i64>) -> AppResult<()> {
    match price_min {
        None => return Ok(()),
        Some(_) => {},
    }
    let mut params: Vec<Value> = [].to_vec();
    params.push(Value::Integer(id));
    params.push(int_value(Some(sampled_at)));
    params.push(int_value(price_min));
    params.push(int_value(price_market));
    match bind_sql(conn, SQL_PRICE_SAMPLE, params, PRICE_SAMPLE_COUNT) {
        Err(problem) => return Err(problem),
        Ok(_) => return Ok(()),
    }
}

/// One observed price point of one item, as the card and the CLI print it.
#[derive(Serialize, Clone)]
pub struct PriceSample {
    pub sampled_at: i64,
    pub price_min: Option<i64>,
    pub price_market: Option<i64>,
    /// v0.9.4 (ADR 0015): the SAME copper numbers above, in the game's money words
    /// (measured: `454` copper reads `4s 54c`, `20628` reads `20g 62s 28c`), filled
    /// at ANSWER TIME by `history_of` from the copper the read already carries - so
    /// the ledger still holds integers, `--limit`-bounded sorts and scripts still
    /// work, and the money a sample shows can never drift from its own number.
    pub price_min_money: Option<String>,
    pub price_market_money: Option<String>,
}

/// An item's own samples, OLDEST first (the trend reads left to right): the read
/// takes the newest N (bounded, no matter how many crawls ran) and flips the
/// vector. seq is the read's second ORDER key: samples share seconds when a
/// chunk wrote them, so the chart order stays stable.
pub fn history_of(conn: &Connection, id: i64, limit: i64) -> AppResult<Vec<PriceSample>> {
    let mut params: Vec<Value> = [].to_vec();
    params.push(Value::Integer(id));
    params.push(Value::Integer(limit));
    let mut stmt = match conn.prepare(SQL_HISTORY_OF) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mapped = match stmt.query_map(params_from_iter(params), |row| {
        Ok(PriceSample {
            sampled_at: row.get(0)?,
            price_min: row.get(1)?,
            price_market: row.get(2)?,
            price_min_money: money_of(row.get(1)?),
            price_market_money: money_of(row.get(2)?),
        })
    }) {
        Err(problem) => return Err(AppError::db(format!("{}", problem))),
        Ok(value) => value,
    };
    let mut out: Vec<PriceSample> = [].to_vec();
    for item in mapped {
        match item {
            Err(problem) => return Err(AppError::db(format!("{}", problem))),
            Ok(value) => out.push(value),
        }
    }
    out.reverse();
    return Ok(out);
}

/// When the icon was copied (NULL = never): the card prints it like detail_synced_at
/// prints the detail stamp.
pub fn icon_at_of(conn: &Connection, id: i64) -> AppResult<Option<i64>> {
    let mut stmt = match conn.prepare(SQL_ICON_AT_OF) {
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

/// When upstream answered 404 for this record's icon file (NULL = never checked):
/// the card shows that state next to "no local copy".
pub fn icon_missing_at_of(conn: &Connection, id: i64) -> AppResult<Option<i64>> {
    let mut stmt = match conn.prepare(SQL_ICON_MISSING_AT_OF) {
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
