//! JSON models for the endpoints we use. Field names mirror the API's snake_case keys
//! exactly, so no serde renaming is needed and `docs/API.md` stays readable next to the
//! code. Anything the API adds later is ignored by serde and kept in `raw_json`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `links` on a single record: how to get back to the API, the HTML page and the tooltip.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Links {
    pub api: Option<String>,
    pub html: Option<String>,
    pub tooltip: Option<String>,
}

/// `links` on a list page: paging cursors.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PageLinks {
    pub docs: Option<String>,
    pub next: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Attribution {
    pub required: Option<bool>,
    pub terms: Option<String>,
    pub text: Option<String>,
    pub url: Option<String>,
}

/// Auction-house price block. All money values are copper.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Price {
    pub available: Option<i64>,
    pub market: Option<i64>,
    pub min: Option<i64>,
    pub realms: Option<i64>,
    pub region: Option<String>,
    pub scope: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Weapon {
    pub damage_max: Option<i64>,
    pub damage_min: Option<i64>,
    pub dps: Option<f64>,
    pub speed: Option<f64>,
}

/// One row of the list endpoint (stage 1 of the sync).
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ItemSummary {
    pub id: i64,
    pub name: String,
    pub kind: Option<String>,
    pub quality: Option<String>,
    pub item_level: Option<i32>,
    pub required_level: Option<i32>,
    pub version: Option<String>,
    pub links: Option<Links>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ListPage {
    pub build: Option<String>,
    pub count: Option<i32>,
    pub has_more: Option<bool>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub kind: Option<String>,
    pub version: Option<String>,
    pub attribution: Option<Attribution>,
    pub links: Option<PageLinks>,
    pub results: Vec<ItemSummary>,
}

/// Full record of `GET /forever/item/<id>` (stage 2 of the sync). Serialisable so a
/// command can hand the same shape back to the UI without a second parse.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ItemDetail {
    pub id: i64,
    pub name: String,
    pub kind: Option<String>,
    pub version: Option<String>,
    pub version_label: Option<String>,
    pub build: Option<String>,
    pub quality: Option<String>,
    pub item_level: Option<i32>,
    pub required_level: Option<i32>,
    pub item_type: Option<String>,
    pub item_subtype: Option<String>,
    pub item_set: Option<String>,
    pub slot: Option<String>,
    pub unique: Option<String>,
    pub flavor: Option<String>,
    pub icon_url: Option<String>,
    pub socket_bonus: Option<String>,
    pub sockets: Option<Value>,
    pub classes: Option<Value>,
    pub stats: Option<Value>,
    pub effects: Option<Value>,
    pub faction_prices: Option<Value>,
    pub vendor_sell: Option<i64>,
    pub price: Option<Price>,
    pub weapon: Option<Weapon>,
    pub links: Option<Links>,
    pub attribution: Option<Attribution>,
}

/// The search endpoint answers a grouped page; the door prints the answer, so the
/// model is only the mirror of the wire. `groups` stays a `Value`: the door prints
/// what upstream answered, and a new kind upstream cannot break a build.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SearchResponse {
    pub q: Option<String>,
    pub version: Option<String>,
    pub attribution: Option<Attribution>,
    /// Grouped hits (`item`, `spell`, `quest`, `npc`, `zone`, `sound`). Kept as a
    /// `Value` on purpose: the UI renders the groups it knows about and ignores the
    /// rest, so a new kind upstream does not break a build.
    pub groups: Value,
}
