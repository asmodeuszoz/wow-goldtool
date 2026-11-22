//! Headless subcommands: the same functions the GUI calls, so CI and the app's own
//! integration test exercise one code path (docs/DECISIONS/0004).
//!
//! Output contract, because CI parses it: JSON on stdout only, human text on stderr,
//! and the exit code is the kind of failure (0 ok · 2 usage · 3 network or rate limit
//! · 4 database). Nothing here may print progress to stdout: it would break the JSON.

use crate::errors::AppError;
use crate::errors::AppResult;
use crate::errors::ErrorKind;

use crate::db::{add_item_manual, apply_manual_edits_detail, clear_deleted_at, column_card, count_items, db_path, deleted_at_of, detail_etag_of, edited_at_of, editable_kind, has_row_of_id, history_of, icon_at_of, icon_b64_of, icon_missing_at_of, manual_at_of, manual_edits_of, manual_overrides_of, mark_deleted_at, mark_missing_at, money_of, next_manual_id, open_db_at, price_min, price_market, price_min_now_of, raw_json_of, refresh_twins, set_item_field, synced_at_of, sync_state_get, upsert_detail, tooltip_at_of, tooltip_html_of, tooltip_missing_at_of, tooltip_text_of, watch_add, watch_drop, watch_list, DbTarget, PriceSample, RefreshTally, SyncState, unix_now};

use crate::query::{Filter, Hit, empty_filter, search, count_matching};
use crate::stats::{coverage, by_quality, cheapest, priciest, Coverage, QualityStat, Extreme};
use crate::sync::{sync_icons, sync_detail, sync_list, sync_prices, sync_tooltips, CancelFlag, SyncReport};
use crate::wowdb::{build_client, get_json, item_url, search_url, ApiResponse, HttpKind, RateBudget, classify, Client, ENV_FIXTURE_KEY};
use crate::models::{ItemDetail, ManualEdit, SearchResponse};

use rusqlite::Connection;

use serde_json::from_str;
use serde_json::to_string;

use std::env::set_var;

/// What the CLI can answer. Keeping the list here means an unknown verb becomes a
/// usage error that names what the user actually typed.
pub fn usage_text() -> String {
    return "usage: status | sync --stage list|detail|icons|tooltip|price [--budget N] [--ids A,B,C] [--watch] | search TEXT [--icons --where name|description|all --quality X --type Y --max-level N --priced --sort price|name|quality|level|id|price-now --limit N --offset N] | stats [--icons --limit N] | item ID [--cached] | live-search TEXT | watch add ID [note] | watch list | watch drop ID | set-item ID FIELD=VALUE | add-item FIELD=VALUE [...] | delete-item ID | undelete-item ID | refresh-twins | status [--db PATH] [--fixture URL]".to_string();
}

/// The verbs of one invocation plus its flags. Unknown flags are kept rather than
/// dropped, so the usage error can quote the typo.
pub struct Invocation {
    pub verb: String,
    pub words: Vec<String>,
    pub stage: String,
    pub sort: String,
    pub scope: String,
    pub quality: Option<String>,
    pub type_name: Option<String>,
    pub max_level: Option<i64>,
    pub priced: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub budget: Option<i64>,
    pub db: Option<String>,
    pub fixture: Option<String>,
    pub id_list: Vec<i64>,
    pub watch: bool,
    pub cached: bool,
    /// v0.9.3 (ADR 0014): ask the list doors for the DB's OWN icon bytes. Opt-in on
    /// the measured ground that an icon is about 3.1 K chars once encoded (avg 2 327
    /// bytes per copy) - an unbounded hit list would be huge, so `--icons` rides the
    /// doors the caller bounds with `--limit` and the default JSON stays exactly as
    /// small as it is today.
    pub icons: bool,
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
    if flag == "--where" { return true }
    if flag == "--max-level" { return true }
    if flag == "--ids" { return true }
    return false;
}

/// Flags that stand alone.
pub fn bare_flag(flag: String) -> bool {
    if flag == "--priced" { return true }
    if flag == "--watch" { return true }
    if flag == "--cached" { return true }
    if flag == "--icons" { return true }
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
        scope: "name".to_string(),
        quality: None,
        type_name: None,
        max_level: None,
        priced: false,
        limit: None,
        offset: None,
        budget: None,
        db: None,
        fixture: None,
        id_list: [].to_vec(),
        watch: false,
        cached: false,
        icons: false,
        bad: "".to_string(),
    };
    let mut index: usize = 0;
    while index < args.len() {
        let token: String = args[index].clone();
        if token.starts_with("--") {
            if bare_flag(token.clone()) {
                if token == "--priced" { out.priced = true; }
                if token == "--watch" { out.watch = true; }
                if token == "--cached" { out.cached = true; }
                if token == "--icons" { out.icons = true; }
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
            if token == "--where" {
                if value == "name" || value == "description" || value == "all" {
                    out.scope = value.clone();
                } else {
                    out.bad = format!("search --where wants name, description or all, got {}", value);
                    return out;
                }
            }
            if token == "--quality" {
                out.quality = Some(value.clone());
            }
            if token == "--ids" {
                match split_ids(value.clone()) {
                    Err(problem) => {
                        out.bad = problem.to_text();
                        return out;
                    },
                    Ok(numbers) => out.id_list = numbers.clone(),
                }
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

/// Samples the card's trend gets: a card-sized window, not the whole ledger - the
/// DB keeps every sample, the card shows the recent 40 (docs/DATABASE.md).
pub const CARD_HISTORY_LIMIT: i64 = 40;

/// One sample list as JSON for the card: serde again, never hand-joined money.
pub fn json_history(samples: Vec<PriceSample>) -> String {
    match to_string(&samples) {
        Err(_) => return "[]".to_string(),
        Ok(text) => return text,
    }
}

/// The v0.6.0 card extras, shared by BOTH card doors (live and --cached) so the
/// UI cannot see one shape from a button and another from the other: the local
/// icon (base64 or null), when it was copied, when it was checked as no-icon-
/// upstream, and the item's own price samples. A record with a fresh write has
/// its new sample in the history already: upsert_detail wrote it before this
/// read ran (the sampling rule lives there, ADR 0007).
pub fn card_icon_parts(conn: &Connection, id: i64) -> AppResult<Vec<String>> {
    let copy: Option<String> = match icon_b64_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let copied: Option<i64> = match icon_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let checked: Option<i64> = match icon_missing_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let samples: Vec<PriceSample> = match history_of(conn, id, CARD_HISTORY_LIMIT) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let icon_json: String = match copy {
        None => "null".to_string(),
        Some(text) => match to_string(&text.clone()) {
            Err(_) => "null".to_string(),
            Ok(made) => made,
        },
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("icon_b64".to_string(), icon_json));
    parts.push(fragment("icon_at".to_string(), json_opt_int(copied)));
    parts.push(fragment("icon_missing_at".to_string(), json_opt_int(checked)));
    parts.push(fragment("history".to_string(), json_history(samples.clone())));
    return Ok(parts);
}

/// v0.9.4 (ADR 0015): the money words of the copper this card is about to print, as
/// top-level SIBLING fragments beside the `item` object. Outside it for the same
/// reason `icon_b64` is outside: `ItemDetail` and `Price` are upstream's mirror
/// shapes (docs/API.md), so a field added inside them would claim upstream speaks
/// a word it never answered. Money reads off the MERGED detail - the very value the
/// fragment above prints - so a money word can never disagree with the number
/// beside it, which also covers a `vendor_sell` the user stated by hand (editable_kind
/// lists vendor_sell as an editable INTEGER field: the card shows the user's number,
/// so it shows the money of the user's number). An absent price answers `null`
/// beside `null`, never `0c`. Measured with ZERO requests on the user's own DB:
/// `item 3850 --cached` answers `price_min 10075` beside `1g 75c` (the middle-zero
/// class the copies settle, ADR 0015) and `vendor_sell 100` beside `1s`.
/// `price_available` and `price_realms` get NO money sibling anywhere: they are
/// auction COUNTS (measured on the user's row 25, `available 12` for 12 auctions
/// of 15 000 copper), and ADR 0015's rule says money never rides a count.
pub fn card_money_parts(detail: ItemDetail) -> Vec<String> {
    let sell_json: String = match money_of(detail.vendor_sell.clone()) {
        None => "null".to_string(),
        Some(words) => json_text(words.clone()),
    };
    let min_json: String = match money_of(price_min(detail.price.clone())) {
        None => "null".to_string(),
        Some(words) => json_text(words.clone()),
    };
    let market_json: String = match money_of(price_market(detail.price.clone())) {
        None => "null".to_string(),
        Some(words) => json_text(words.clone()),
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("price_min_money".to_string(), min_json));
    parts.push(fragment("price_market_money".to_string(), market_json));
    parts.push(fragment("vendor_sell_money".to_string(), sell_json));
    return parts;
}

/// The LIVE price observation beside the aged stored money words (v0.9.6, ADR
/// 0017): `sync --stage price` writes price_min_now through the live tooltip
/// door, and the card shows what it says NOW next to what the stored copy said
/// when the detail crawl copied it. money_of rides it like every other copper
/// (ADR 0015's rule: money words beside every copper answer, and NOT beside the
/// auction COUNTS - price_available/price_realms stay counts). Two keys, no
/// price_now_at fragment yet: the when-stamp's only readers are the pending
/// rule SQL and the write itself, so the wire grows the stamp when a reader
/// wants it, not before (recorded in ADR 0017). The pair comes from the DB,
/// like the icon parts do, so the LIVE card and the CACHED card show the same
/// observation: the door is the DB's, not the payload's.
pub fn card_price_now_parts(conn: &Connection, id: i64) -> AppResult<Vec<String>> {
    let live: Option<i64> = match price_min_now_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let live_json: String = match money_of(live.clone()) {
        None => "null".to_string(),
        Some(words) => json_text(words.clone()),
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("price_min_now".to_string(), json_opt_int(live)));
    parts.push(fragment("price_min_now_money".to_string(), live_json));
    return Ok(parts);
}

/// The manual layer and the tooltip copy on the card (v0.8.0, ADR 0010): the
/// card carries EVERY door's three states, so "the crawl has not reached it" and
/// "a user already said something about it" stay different answers. manual_overrides
/// rides the stored JSON ARRAY verbatim (the same text the merge reads), which is
/// what lets the UI mark the fields the user stated. tooltip_html is upstream's
/// OWN markup for the card, printed as text and never rendered as markup: pasting
/// an upstream body into the page would let upstream write the card. tooltip_text
/// (v0.9.0, ADR 0012) is the readable TWIN the card box CAN render: the derivation
/// dropped the measured hotlink img tags and collapsed whitespace, so it is plain
/// text of the copy - the card gets words, the bytes stay the DB's.
pub fn card_manual_parts(conn: &Connection, id: i64) -> AppResult<Vec<String>> {
    let said: Option<String> = match manual_overrides_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let manual: Option<i64> = match manual_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let edited: Option<i64> = match edited_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let buried: Option<i64> = match deleted_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let copy: Option<String> = match tooltip_html_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let readable: Option<String> = match tooltip_text_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let copied: Option<i64> = match tooltip_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let checked: Option<i64> = match tooltip_missing_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    let copy_json: String = match copy {
        None => "null".to_string(),
        Some(text) => json_text(text.clone()),
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("manual_overrides".to_string(), match said {
        None => "null".to_string(),
        Some(text) => text.clone(),
    }));
    parts.push(fragment("manual_at".to_string(), json_opt_int(manual)));
    parts.push(fragment("edited_at".to_string(), json_opt_int(edited)));
    parts.push(fragment("deleted_at".to_string(), json_opt_int(buried)));
    parts.push(fragment("tooltip_html".to_string(), copy_json));
    let readable_json: String = match readable {
        None => "null".to_string(),
        Some(text) => match to_string(&text) {
            Err(_) => "null".to_string(),
            Ok(json_text) => json_text,
        },
    };
    parts.push(fragment("tooltip_text".to_string(), readable_json));
    parts.push(fragment("tooltip_at".to_string(), json_opt_int(copied)));
    parts.push(fragment("tooltip_missing_at".to_string(), json_opt_int(checked)));
    return Ok(parts);
}


/// One flag value of ids written comma separated: `--ids 25,40,60`. Split by hand
/// rather than by pattern so a trailing comma becomes an honest usage error instead of
/// a silent empty id.
pub fn split_ids(text: String) -> AppResult<Vec<i64>> {
    let mut parts: Vec<String> = [].to_vec();
    let mut current: String = "".to_string();
    for ch in text.chars() {
        if ch == ',' {
            parts.push(current.clone());
            current = "".to_string();
        } else {
            current.push(ch);
        }
    }
    parts.push(current);
    let mut out: Vec<i64> = [].to_vec();
    for piece in parts {
        match as_int(piece.clone()) {
            None => return Err(AppError::usage(format!("--ids wants numeric item ids, got {} in {}", piece, text))),
            Some(number) => out.push(number),
        }
    }
    return Ok(out);
}

/// Where a detail run gets its ids. Explicit ids win, then the watch list; an empty
/// result means "everything still pending" and src/sync.rs owns that difference.
pub fn detail_targets(conn: &Connection, inv: &Invocation) -> AppResult<Vec<i64>> {
    if !inv.id_list.is_empty() {
        return Ok(inv.id_list.clone());
    }
    if inv.watch {
        return watch_list(conn);
    }
    return Ok([].to_vec());
}

/// One optional whole number as JSON: absent means `null`, never a made-up 0.
pub fn json_opt_int(opt: Option<i64>) -> String {
    match opt {
        None => return "null".to_string(),
        Some(number) => return format!("{}", number),
    }
}

/// `item ID --cached`: the card the UI shows before any request went out. A record that
/// was never fetched is `not-found` - the UI reads that word and offers to sync the item,
/// so "no data yet" and "broken data" stay two different answers.
pub fn cached_card(conn: &Connection, id: i64) -> AppResult<String> {
    // A tombstone hides the row from EVERY door (ADR 0010), so the card refuses it
    // before reading a single column: printing a buried row would un-hide it.
    let buried: Option<i64> = match deleted_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    match buried {
        Some(stamp) => return Err(AppError::db(format!("item {} is tombstoned at {} (undelete-item {} brings it back)", id, stamp, id))),
        None => {},
    }
    let stored: Option<String> = match raw_json_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    // A manual row has no upstream copy to read, so its card comes from the DB's
    // own columns (ADR 0010): a crawl will never fill them and upstream cannot name
    // them. A crawl row without a copy still gets the actionable answer below.
    let made: Option<i64> = match manual_at_of(conn, id) {
        Err(problem) => return Err(problem),
        Ok(found) => found,
    };
    match stored {
        None => match made {
            None => return Err(AppError::not_found(format!("item {} has no stored copy: fetch it with sync --stage detail --ids {}", id, id))),
            Some(_) => {
                let detail: ItemDetail = match column_card(conn, id) {
                    Err(problem) => return Err(problem),
                    Ok(value) => value,
                };
                let mut parts: Vec<String> = [].to_vec();
                parts.push(fragment("status".to_string(), "null".to_string()));
                parts.push(fragment("retry_after".to_string(), json_opt_seconds(None)));
                parts.push(fragment("cached".to_string(), "true".to_string()));
                parts.push(fragment("reused".to_string(), "false".to_string()));
                parts.push(fragment("detail_synced_at".to_string(), json_opt_int(None)));
                parts.push(fragment("item".to_string(), json_item(detail.clone())));
                let money: Vec<String> = card_money_parts(detail.clone());
                for one in money {
                    parts.push(one.clone());
                }
                let live_price: Vec<String> = match card_price_now_parts(conn, id) {
                    Err(problem) => return Err(problem),
                    Ok(found) => found,
                };
                for one in live_price {
                    parts.push(one.clone());
                }
                let extras: Vec<String> = match card_icon_parts(conn, id) {
                    Err(problem) => return Err(problem),
                    Ok(found) => found,
                };
                for one in extras {
                    parts.push(one.clone());
                }
                let manual: Vec<String> = match card_manual_parts(conn, id) {
                    Err(problem) => return Err(problem),
                    Ok(found) => found,
                };
                for one in manual {
                    parts.push(one.clone());
                }
                return Ok(object(parts));
            },
        },
        Some(text) => {
            // The card reads the DB, so it reads the MERGED row: the stored copy is
            // upstream's payload, and the user-said fields merge LAST (ADR 0010) - the
            // same rule the write path uses, applied to the read path so a card cannot
            // show a value the DB does not have.
            let mut detail: ItemDetail = match from_str(&text) {
                Err(problem) => return Err(AppError::decode(format!("the stored copy of item {} is not the JSON we expect: {}", id, problem))),
                Ok(value) => value,
            };
            let edits: Vec<ManualEdit> = match manual_edits_of(conn, id) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            apply_manual_edits_detail(&mut detail, edits.clone());
            let fetched: Option<i64> = match synced_at_of(conn, id) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            let mut parts: Vec<String> = [].to_vec();
            parts.push(fragment("status".to_string(), "null".to_string()));
            parts.push(fragment("retry_after".to_string(), json_opt_seconds(None)));
            parts.push(fragment("cached".to_string(), "true".to_string()));
            parts.push(fragment("reused".to_string(), "true".to_string()));
            parts.push(fragment("detail_synced_at".to_string(), json_opt_int(fetched)));
            parts.push(fragment("item".to_string(), json_item(detail.clone())));
            let money: Vec<String> = card_money_parts(detail.clone());
            for one in money {
                parts.push(one.clone());
            }
            let live_price: Vec<String> = match card_price_now_parts(conn, id) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            for one in live_price {
                parts.push(one.clone());
            }
            let extras: Vec<String> = match card_icon_parts(conn, id) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            for one in extras {
                parts.push(one.clone());
            }
            let manual: Vec<String> = match card_manual_parts(conn, id) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            for one in manual {
                parts.push(one.clone());
            }
            return Ok(object(parts));
        },
    }
}

/// A list of ids as a JSON array: `watch list` prints this, the UI reads it.
pub fn json_ids(ids: Vec<i64>) -> String {
    match to_string(&ids) {
        Err(_) => return "[]".to_string(),
        Ok(text) => return text,
    }
}

/// A string as a JSON string, through serde so quotes and escaping stay out of
/// hand-written JSON (same reason as json_ids and json_filter).
pub fn json_text(text: String) -> String {
    match to_string(&text) {
        Err(_) => return "\"\"".to_string(),
        Ok(found) => return found,
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
    let icons_state: SyncState = match sync_state_get(&conn, "icons".to_string()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("db".to_string(), json_db_target(target)));
    parts.push(fragment("items".to_string(), format!("{}", seen)));
    parts.push(fragment("coverage".to_string(), json_coverage(found)));
    parts.push(fragment("list".to_string(), json_state(list_state)));
    parts.push(fragment("detail".to_string(), json_state(detail_state)));
    parts.push(fragment("icons".to_string(), json_state(icons_state)));
    let tooltip_state: SyncState = match sync_state_get(&conn, "tooltip".to_string()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    parts.push(fragment("tooltip".to_string(), json_state(tooltip_state)));
    return Ok(object(parts));
}

/// sync: run one crawl stage. `--budget` bounds the run so CI and a test fixture can
/// crawl cheaply; the report says where the crawl stopped either way.
pub fn cmd_sync(inv: &Invocation) -> AppResult<String> {
    if inv.stage != "list" {
        if inv.stage != "detail" {
            if inv.stage != "icons" {
                if inv.stage != "tooltip" {
                    if inv.stage != "price" {
                        return Err(AppError::usage(format!("sync wants --stage list, detail, icons, tooltip or price, got {}", inv.stage)));
                    }
                }
            }
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
    let cancel: CancelFlag = CancelFlag::make();
    if inv.stage == "list" {
        eprintln!("crawling the item list: about 193 pages, one request per page");
        let report: SyncReport = match sync_list(client.clone(), &conn, budget, inv.budget, &cancel) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        eprintln!("list stage: {} pages, {} records seen, {} written", report.pages, report.items, report.written);
        return Ok(json_report(report));
    }
    if inv.stage == "tooltip" {
        eprintln!("copying the tooltip copy: one request per record whose detail answered, about two per second (about 19 258 requests over the catalogue, ≈ 2.7 h at the measured pace), through the door the description text lives behind - the forever/item body carries none (docs/API.md)");
        let wanted: Vec<i64> = match detail_targets(&conn, inv) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        if wanted.is_empty() {
            eprintln!("tooltip stage queue: every answered record whose tooltip was never fetched nor checked");
        } else {
            eprintln!("tooltip stage targets: {} records ({} asked by --ids, watch list asked by --watch)", wanted.len(), inv.id_list.len());
        }
        let report: SyncReport = match sync_tooltips(client.clone(), &conn, budget, inv.budget, &cancel, wanted) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        if report.missing == 0 {
            eprintln!("tooltip stage: {} copies fetched ({} requests)", report.items, report.requests);
        } else {
            eprintln!("tooltip stage: {} copies fetched, {} checked as no-tooltip-upstream", report.items, report.missing);
        }
        return Ok(json_report(report));
    }
    if inv.stage == "icons" {
        eprintln!("copying icon files: one request per icon of a record that answered, about two per second, on the asset hosts the detail payloads name (docs/API.md)");
        let wanted: Vec<i64> = match detail_targets(&conn, inv) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        if wanted.is_empty() {
            eprintln!("icon stage queue: every answered record whose icon was never fetched nor checked");
        } else {
            eprintln!("icon stage targets: {} records ({} asked by --ids, watch list asked by --watch)", wanted.len(), inv.id_list.len());
        }
        let report: SyncReport = match sync_icons(client.clone(), &conn, budget, inv.budget, &cancel, wanted) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        if report.missing == 0 {
            eprintln!("icons stage: {} icons copied ({} requests)", report.written, report.requests);
        } else {
            eprintln!("icons stage: {} icons copied, {} checked as no-icon-upstream", report.written, report.missing);
        }
        return Ok(json_report(report));
    }
    if inv.stage == "price" {
        eprintln!("refreshing the LIVE price: one request per record that already holds a tooltip copy, through the live door the tooltip lives behind (docs/API.md measures its `fields.price`; the developers page states tooltips cache ~5 min 'because they carry a live Auction House price') - the stored copy's price_min AGES, price_min_now is what the door says NOW, and the queue is once-per-record like every other stage's");
        let wanted: Vec<i64> = match detail_targets(&conn, inv) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        if wanted.is_empty() {
            eprintln!("price stage queue: every copied record whose live price was never refreshed nor checked as price-less");
        } else {
            eprintln!("price stage targets: {} records ({} asked by --ids, watch list asked by --watch)", wanted.len(), inv.id_list.len());
        }
        let report: SyncReport = match sync_prices(client.clone(), &conn, budget, inv.budget, &cancel, wanted) {
            Err(problem) => return Err(problem),
            Ok(value) => value,
        };
        if report.missing == 0 {
            eprintln!("price stage: {} live prices written ({} requests)", report.written, report.requests);
        } else {
            eprintln!("price stage: {} live prices written, {} checked as no auction price right now", report.written, report.missing);
        }
        return Ok(json_report(report));
    }
    let wanted: Vec<i64> = match detail_targets(&conn, inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    if wanted.is_empty() {
        eprintln!("fetching item details: this is the slow stage, about two records per second");
    } else {
        eprintln!("fetching item details for {} targeted records ({} asked by --ids, watch list asked by --watch)", wanted.len(), inv.id_list.len());
    }
    let report: SyncReport = match sync_detail(client.clone(), &conn, budget, inv.budget, &cancel, wanted.clone()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    if report.missing == 0 {
        eprintln!("detail stage: {} records fetched", report.items);
    } else {
        eprintln!("detail stage: {} records fetched, {} checked as removed upstream", report.items, report.missing);
    }
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
    out.scope = inv.scope.clone();
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
    let hits: Vec<Hit> = match search(&conn, filter, inv.icons) {
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
    let cheap: Vec<Extreme> = match cheapest(&conn, limit, inv.icons) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let rich: Vec<Extreme> = match priciest(&conn, limit, inv.icons) {
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
            // A tombstone hides the row from EVERY door (ADR 0010), so the live card
            // refuses it BEFORE any request goes out: re-asking upstream for a row the
            // user buried would also let the crawl's stamps land on a buried row.
            let buried: Option<i64> = match deleted_at_of(&conn, number) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            match buried {
                Some(stamp) => return Err(AppError::db(format!("item {} is tombstoned at {} (undelete-item {} brings it back)", number, stamp, number))),
                None => {},
            }
            // A manual row is the DB's own: upstream cannot know the id, so the live
            // door answers from the DB's columns and sends ZERO requests - asking
            // upstream would 404 and stamp the crawl's third states on a row the user
            // made. Same reason the pending rules refuse manual rows (ADR 0010).
            let made: Option<i64> = match manual_at_of(&conn, number) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            match made {
                Some(_) => {
                    let detail: ItemDetail = match column_card(&conn, number) {
                        Err(problem) => return Err(problem),
                        Ok(value) => value,
                    };
                    let mut parts: Vec<String> = [].to_vec();
                    parts.push(fragment("status".to_string(), "null".to_string()));
                    parts.push(fragment("retry_after".to_string(), json_opt_seconds(None)));
                    parts.push(fragment("cached".to_string(), "true".to_string()));
                    parts.push(fragment("reused".to_string(), "false".to_string()));
                    parts.push(fragment("detail_synced_at".to_string(), json_opt_int(None)));
                    parts.push(fragment("item".to_string(), json_item(detail.clone())));
                    let money: Vec<String> = card_money_parts(detail.clone());
                    for one in money {
                        parts.push(one.clone());
                    }
                    let live_price: Vec<String> = match card_price_now_parts(&conn, number) {
                        Err(problem) => return Err(problem),
                        Ok(found) => found,
                    };
                    for one in live_price {
                        parts.push(one.clone());
                    }
                    let extras: Vec<String> = match card_icon_parts(&conn, number) {
                        Err(problem) => return Err(problem),
                        Ok(found) => found,
                    };
                    for one in extras {
                        parts.push(one.clone());
                    }
                    let manual: Vec<String> = match card_manual_parts(&conn, number) {
                        Err(problem) => return Err(problem),
                        Ok(found) => found,
                    };
                    for one in manual {
                        parts.push(one.clone());
                    }
                    return Ok(object(parts));
                },
                None => {},
            }
            let stored_etag: Option<String> = match detail_etag_of(&conn, number) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            let send_etag: String = match stored_etag {
                None => "".to_string(),
                Some(text) => text,
            };
            if inv.cached {
                return cached_card(&conn, number);
            }
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
                HttpKind::Missing => {
                    // a direct ask answers once and remembers: the row gets the
                    // checked-missing stamp when we have a row to stamp.
                    match mark_missing_at(&conn, number, unix_now()) {
                        Err(problem) => return Err(problem),
                        Ok(_) => {},
                    }
                    return Err(AppError::not_found(format!("no item {} upstream: the forever catalogue has no record for it", number)));
                },
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
            let mut detail: ItemDetail = match from_str(&payload) {
                Err(_) => return Err(AppError::decode(format!("item {} is not the JSON we expect", number))),
                Ok(value) => value,
            };
            if response.status == 200 {
                match upsert_detail(&conn, detail.clone(), response.etag.clone(), response.body.clone(), unix_now()) {
                    Err(problem) => return Err(problem),
                    Ok(_) => {},
                }
            }
            // The live card answers with what the DB NOW says: the write path merged the
            // user-said fields into the columns, and the card merges them into the payload
            // it prints (ADR 0010's merge-last rule, one rule for both paths). The write
            // above stays the single merge point for the DB, so nothing here re-writes.
            let edits: Vec<ManualEdit> = match manual_edits_of(&conn, number) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            apply_manual_edits_detail(&mut detail, edits.clone());
            let reused: bool = response.status == 304;
            let mut parts: Vec<String> = [].to_vec();
            parts.push(fragment("status".to_string(), format!("{}", response.status)));
            parts.push(fragment("retry_after".to_string(), json_opt_seconds(response.retry_after)));
            parts.push(fragment("reused".to_string(), match reused { true => "true".to_string(), false => "false".to_string() }));
            parts.push(fragment("cached".to_string(), "false".to_string()));
            parts.push(fragment("item".to_string(), json_item(detail.clone())));
            let money: Vec<String> = card_money_parts(detail.clone());
            for one in money {
                parts.push(one.clone());
            }
            let live_price: Vec<String> = match card_price_now_parts(&conn, number) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            for one in live_price {
                parts.push(one.clone());
            }
            let extras: Vec<String> = match card_icon_parts(&conn, number) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            for one in extras {
                parts.push(one.clone());
            }
            let manual: Vec<String> = match card_manual_parts(&conn, number) {
                Err(problem) => return Err(problem),
                Ok(found) => found,
            };
            for one in manual {
                parts.push(one.clone());
            }
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

/// ids straight from the word list (collect_ids skips its first token, which is
/// how watch's action got through; here every word is an id).
pub fn collect_all_ids(words: Vec<String>) -> AppResult<Vec<i64>> {
    let mut out: Vec<i64> = [].to_vec();
    for one in words {
        match as_int(one.clone()) {
            None => return Err(AppError::usage(format!("these verbs want numeric item ids, got {}", one))),
            Some(number) => out.push(number),
        }
    }
    return Ok(out);
}

/// FIELD=VALUE pairs for the edit and create doors: one char pass per word (same
/// walk as encode_query's and join_tail's), then EVERY field name is checked
/// against db.rs's whitelist HERE, so the usage error can name the field the
/// user actually typoed instead of answering from db.rs with a generic list.
pub fn collect_pairs(words: Vec<String>) -> AppResult<Vec<(String, String)>> {
    let mut out: Vec<(String, String)> = [].to_vec();
    for one in words {
        let mut head: String = "".to_string();
        let mut tail: String = "".to_string();
        let mut seen_eq: bool = false;
        for ch in one.chars() {
            if seen_eq {
                tail = format!("{}{}", tail, ch);
                continue;
            }
            if ch == '=' {
                seen_eq = true;
                continue;
            }
            head = format!("{}{}", head, ch);
        }
        if !seen_eq {
            return Err(AppError::usage(format!("these verbs want FIELD=VALUE pairs, got {}", one)));
        }
        if head == "" {
            return Err(AppError::usage("FIELD=VALUE wants a field name before the =".to_string()));
        }
        if editable_kind(head.clone()) < 0 {
            return Err(AppError::usage(format!("{} is not editable (name, quality, item_level, required_level, item_type, item_subtype, flavor, vendor_sell, tooltip_html)", head)));
        }
        out.push((head.clone(), tail.clone()));
    }
    return Ok(out);
}

/// The edit door (ADR 0010): ONE whitelist field of ONE row. DB-only, so a bad
/// ask answers 2 and an unknown or tombstoned id answers 4 - never 3: no request
/// leaves this verb, which is also why the ask needs no budget. db.rs sets the
/// column AND rides the overrides array, so the crawl's NEXT write of this row
/// keeps the user's value (the merge in upsert_summary/upsert_detail).
pub fn cmd_set_item(inv: &Invocation) -> AppResult<String> {
    if inv.words.len() < 2 {
        return Err(AppError::usage("set-item wants ID FIELD=VALUE".to_string()));
    }
    let mut first: Vec<String> = [].to_vec();
    first.push(inv.words[0].clone());
    let ids: Vec<i64> = match collect_all_ids(first) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let mut rest: Vec<String> = [].to_vec();
    let mut index: usize = 1;
    while index < inv.words.len() {
        rest.push(inv.words[index].clone());
        index = index + 1;
    }
    let pairs: Vec<(String, String)> = match collect_pairs(rest) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    if pairs.len() != 1 {
        return Err(AppError::usage("set-item states exactly one FIELD=VALUE per ask".to_string()));
    }
    let id: i64 = ids[0];
    let conn: Connection = match open_at(inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match has_row_of_id(&conn, id) {
        Err(problem) => return Err(problem),
        Ok(false) => return Err(AppError::db(format!("no item {} in the DB (add-item creates one)", id))),
        Ok(true) => {},
    }
    match deleted_at_of(&conn, id) {
        Err(problem) => return Err(problem),
        Ok(Some(stamp)) => return Err(AppError::db(format!("item {} is tombstoned at {} (undelete-item first)", id, stamp))),
        Ok(None) => {},
    }
    let field: String = pairs[0].0.clone();
    let value: String = pairs[0].1.clone();
    let now: i64 = unix_now();
    match set_item_field(&conn, id, field.clone(), value.clone(), now) {
        Err(problem) => return Err(problem),
        Ok(_) => {},
    }
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("id".to_string(), format!("{}", id)));
    parts.push(fragment("field".to_string(), json_text(field.clone())));
    parts.push(fragment("value".to_string(), json_text(value.clone())));
    parts.push(fragment("edited_at".to_string(), format!("{}", now)));
    return Ok(object(parts));
}

/// The create door (ADR 0010): the id is the reserved block's next free slot,
/// NEVER one the user picked - a user-picked id could collide with a FUTURE
/// forever record (measured max upstream id on the user's file: 288 340), which
/// would overwrite their item on the next crawl. The new row wears its fields as
/// overrides from birth, so even that future collision cannot take it.
pub fn cmd_add_item(inv: &Invocation) -> AppResult<String> {
    let pairs: Vec<(String, String)> = match collect_pairs(inv.words.clone()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    if pairs.len() < 1 {
        return Err(AppError::usage("add-item wants FIELD=VALUE pairs (at least name=...)".to_string()));
    }
    let conn: Connection = match open_at(inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let id: i64 = match next_manual_id(&conn) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let now: i64 = unix_now();
    match add_item_manual(&conn, id, now, pairs.clone()) {
        Err(problem) => return Err(problem),
        Ok(_) => {},
    }
    let mut names: Vec<String> = [].to_vec();
    for pair in pairs {
        names.push(pair.0.clone());
    }
    let names_json: String = match to_string(&names) {
        Err(_) => "[]".to_string(),
        Ok(text) => text,
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("id".to_string(), format!("{}", id)));
    parts.push(fragment("manual_at".to_string(), format!("{}", now)));
    parts.push(fragment("fields".to_string(), names_json));
    return Ok(object(parts));
}

/// DELETE is the tombstone (ADR 0010): the row keeps its stamps and its bytes,
/// but search, stats, pending queues and the card refuse it, and the list stage
/// (which UPSERTs every id upstream still lists) never brings a hard-deleted row
/// back. An id with no row answers 4; a second delete says so instead of
/// pretending the mark moved.
pub fn cmd_delete_item(inv: &Invocation) -> AppResult<String> {
    let ids: Vec<i64> = match collect_all_ids(inv.words.clone()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    if ids.is_empty() {
        return Err(AppError::usage("delete-item wants an item id".to_string()));
    }
    let conn: Connection = match open_at(inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let now: i64 = unix_now();
    for one in ids.clone() {
        match has_row_of_id(&conn, one) {
            Err(problem) => return Err(problem),
            Ok(false) => return Err(AppError::db(format!("no item {} in the DB", one))),
            Ok(true) => {},
        }
        match mark_deleted_at(&conn, one, now) {
            Err(problem) => return Err(problem),
            Ok(true) => {},
            Ok(false) => return Err(AppError::db(format!("item {} is already tombstoned", one))),
        }
    }
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("deleted".to_string(), json_ids(ids)));
    parts.push(fragment("at".to_string(), format!("{}", now)));
    return Ok(object(parts));
}

/// The mark's undo: the row comes back with everything it had (icon bytes,
/// samples, overrides all kept - the tombstone only hid it).
pub fn cmd_undelete_item(inv: &Invocation) -> AppResult<String> {
    let ids: Vec<i64> = match collect_all_ids(inv.words.clone()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    if ids.is_empty() {
        return Err(AppError::usage("undelete-item wants an item id".to_string()));
    }
    let conn: Connection = match open_at(inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    for one in ids.clone() {
        match has_row_of_id(&conn, one) {
            Err(problem) => return Err(problem),
            Ok(false) => return Err(AppError::db(format!("no item {} in the DB", one))),
            Ok(true) => {},
        }
        match clear_deleted_at(&conn, one) {
            Err(problem) => return Err(problem),
            Ok(true) => {},
            Ok(false) => return Err(AppError::db(format!("item {} is not tombstoned", one))),
        }
    }
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("restored".to_string(), json_ids(ids)));
    return Ok(object(parts));
}

/// The refresh door (v0.9.2, ADR 0013): a DB whose twins predate the corrected
/// span rule gets them re-derived from the copies it already holds. The door reads
/// and writes the DB ONLY - ZERO requests leave it, so ADR 0004's network door
/// (exit 3) is unreachable here and a missing DB is the honest 4. It takes no
/// arguments: it refreshes EVERY copy in the DB, tombstones and manual rows
/// included (ADR 0012's rule is that the twin always matches what the DB SAYS).
/// A second run over a refreshed DB answers 0 rewritten and still exits 0 - the
/// report says so, 4 would mean "the DB does not have it", which is not this case.
/// The GUI reaches the same verb through the same dispatch, so the exit-code
/// contract stays one path.
pub fn cmd_refresh_twins(inv: &Invocation) -> AppResult<String> {
    if inv.words.len() != 0 {
        return Err(AppError::usage("refresh-twins takes no arguments (it refreshes every copy in the DB)".to_string()));
    }
    let conn: Connection = match open_at(inv) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let tally: RefreshTally = match refresh_twins(&conn) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("kind".to_string(), json_text("refresh".to_string())));
    parts.push(fragment("copies".to_string(), format!("{}", tally.copies)));
    parts.push(fragment("rewritten".to_string(), format!("{}", tally.rewritten)));
    parts.push(fragment("filled".to_string(), format!("{}", tally.filled)));
    parts.push(fragment("unchanged".to_string(), format!("{}", tally.unchanged)));
    parts.push(fragment("requests".to_string(), "0".to_string()));
    parts.push(fragment("complete".to_string(), format!("{}", tally.complete)));
    parts.push(fragment("message".to_string(), json_text(tally.message.clone())));
    return Ok(object(parts));
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

/// live-search: ONE request to the grouped search door, for a human ask. The door
/// prints what upstream answered (serde pas-through: `groups` stays a `Value`) plus
/// `remaining` from the burst headers, and touches NO DB - this is a look, not a
/// sync (ADR 0009 says why, and why a 429 here is the exit-3 door like `item`,
/// not the crawl's retry-in-place: one ask, one request). The answer matches NAMES
/// only (docs/API.md measures the trap: `?q=socket` answers empty groups) and each
/// group says `has_more` plus its own paged `more` URL - v0.7.0 stops at one request.
pub fn cmd_live_search(inv: &Invocation) -> AppResult<String> {
    if inv.words.len() < 1 {
        return Err(AppError::usage("live-search wants text, for example: live-search Worn Shortsword".to_string()));
    }
    let query: String = join_tail(inv.words.clone());
    let client: Client = match build_client() {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    let response: ApiResponse = match get_json(client, search_url(query.clone()), "".to_string()) {
        Err(problem) => return Err(problem),
        Ok(value) => value,
    };
    match classify(response.status) {
        HttpKind::Success => {},
        HttpKind::RateLimited => return Err(AppError::rate_limited("the API asked us to back off".to_string(), response.retry_after)),
        HttpKind::Missing => return Err(AppError::not_found("the search door answered 404: it has no answer for that query shape".to_string())),
        HttpKind::NotModified => return Err(AppError::http(response.status, response.body)),
        HttpKind::Failed => return Err(AppError::http(response.status, response.body)),
    }
    let found: SearchResponse = match from_str(&response.body) {
        Err(_) => return Err(AppError::decode(format!("the search answer for {} is not the JSON we expect", query))),
        Ok(value) => value,
    };
    let answer: String = match to_string(&found) {
        Err(_) => return Err(AppError::decode("cannot print the search answer".to_string())),
        Ok(value) => value,
    };
    eprintln!("live search {} answered NAMES only: a tooltip word is not searched, and a group can answer has_more with its own paged more URL (one ask, one request: this build does not chase it)", query);
    let mut parts: Vec<String> = [].to_vec();
    parts.push(fragment("status".to_string(), format!("{}", response.status)));
    parts.push(fragment("retry_after".to_string(), json_opt_seconds(response.retry_after)));
    parts.push(fragment("remaining".to_string(), match response.remaining {
        None => "null".to_string(),
        Some(left) => format!("{}", left),
    }));
    parts.push(fragment("answer".to_string(), answer));
    return Ok(object(parts));
}

/// The whole verb tail as ONE query: `live-search Worn Shortsword` asks the API for
/// "Worn Shortsword", not for two flags. Flag-shaped words stay words here.
pub fn join_tail(words: Vec<String>) -> String {
    let mut out: String = "".to_string();
    for one in words {
        if out != "" {
            out = format!("{} {}", out, one);
        }
        out = format!("{}{}", out, one);
    }
    return out;
}

/// Every verb through the same guard: JSON out, message and code to stderr.
/// One Invocation to its verb's JSON string: the single place both the headless CLI
/// and the GUI (commands.rs, ADR 0004) send verbs through. Prints nothing on stdout;
/// help still explains itself on stderr. The fixture switch is applied here, because
/// both doors may carry --fixture and both must see the same base URL.
pub fn dispatch(inv: Invocation) -> AppResult<String> {
    if inv.bad != "" {
        return Err(AppError::usage(inv.bad));
    }
    match inv.fixture.clone() {
        Some(url) => unsafe { set_var(ENV_FIXTURE_KEY, url); },
        None => {},
    }
    if inv.verb == "" {
        return Err(AppError::usage(usage_text()));
    }
    if inv.verb == "status" { return cmd_status(&inv); }
    if inv.verb == "sync" { return cmd_sync(&inv); }
    if inv.verb == "search" { return cmd_search(&inv); }
    if inv.verb == "stats" { return cmd_stats(&inv); }
    if inv.verb == "item" { return cmd_item(&inv); }
    if inv.verb == "live-search" { return cmd_live_search(&inv); }
    if inv.verb == "watch" { return cmd_watch(&inv); }
    if inv.verb == "set-item" { return cmd_set_item(&inv); }
    if inv.verb == "add-item" { return cmd_add_item(&inv); }
    if inv.verb == "delete-item" { return cmd_delete_item(&inv); }
    if inv.verb == "undelete-item" { return cmd_undelete_item(&inv); }
    if inv.verb == "refresh-twins" { return cmd_refresh_twins(&inv); }
    if inv.verb == "help" {
        eprintln!("{}", usage_text());
        return Ok("{}".to_string());
    }
    return Err(AppError::usage(format!("unknown verb {} (run help to see the verbs)", inv.verb)));
}

/// Every verb through one guard: JSON on stdout, the message and the reason on
/// stderr, the kind of failure as the exit code (docs/DECISIONS/0004). The GUI
/// reaches the same verbs through dispatch, wrapped by commands.rs.
pub fn run(args: Vec<String>) -> i32 {
    let inv: Invocation = parse_invocation(args);
    return guard(dispatch(inv));
}