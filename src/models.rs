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
/// The grouped search answer, kept as the mirror of the wire (measured: top keys
/// `attribution, groups, q, version`; every group carries `has_more`, a `more`
/// page URL and `results`). `groups` stays a `Value` on purpose: the door prints
/// what upstream answered, and the card renders the groups it knows about, so a
/// new kind upstream cannot break a build (docs/API.md).
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

/// One tooltip answer (v0.8.0, ADR 0010): the shape docs/API.md measured on
/// `GET /api/v1/tooltip/forever/item/25` (200 application/json 3060 b, keys
/// attribution,build,fields,html,icon_url,id,kind,links,name,quality,version).
/// Only `html` mattered to the DB (it is the description text the user asked for,
/// and the forever/item body carries NONE - measured key list); the rest rode
/// `fields` as a `Value` so a key upstream adds cannot break a build and nothing
/// here invents a shape beyond what the probe read. From v0.9.6 (ADR 0017) `fields`
/// is the TYPED mirror TooltipFields: serde still ignores every key we have no
/// reader for, so the same promise holds, and the refresh stage now RE-ASKS this
/// door and reads `price.min` - the live price the door carries.
///
/// The mirror was NARROWED at v0.11.1 (ADR 0024) to exactly the two keys with a reader
/// - `html` (the copy stage writes it; the forever/item body carries none, the measured
/// key list) and `fields` (the price stage reads `price.min` through TooltipFields).
/// The other eight keys the probe read - attribution, build, icon_url, id, kind, links,
/// name, quality, version minus those two, the full list stays recorded whole in docs/API.md
/// - now ride ON THE WIRE: serde ignores a key we have no reader for, so an answer
/// carrying them, one without them, or one with a key upstream adds cannot break a
/// build. This is the rule TooltipFields and the recipe shapes (v0.9.6, v0.10.0) already
/// carry, applied at the top level too, and it is what killed rustc's "multiple fields
/// are never read" on this struct: nine names nothing ever read are dead code in the
/// door's shape - the standard ADR 0011 states and ADR 0021 kept when it called a stored
/// field with no reader dead code in the recipe shape (ADR 0024 records this fating). Recorded cost, not hidden: `id` was the only REQUIRED key here, so serde used
/// to refuse an answer that omitted it. Nothing compares a tooltip's own id against the
/// id we asked for - the request URL carries it, and the compiler proves no reader exists
/// - and a body that is not an object carrying an `html` string still fails to parse and
/// still aborts the stage with "not the JSON we expect", so no measured answer moved.
#[derive(Deserialize, Clone, Debug)]
pub struct Tooltip {
    pub html: String,
    pub fields: Option<TooltipFields>,
}

/// The `fields` object INSIDE a tooltip answer (v0.9.6, ADR 0017 - the price
/// refresh reads it). docs/API.md measured its 26 keys; ONLY `price` is typed
/// here because `price` is the only key with a reader: the refresh stage writes
/// `price.min` into the trio home ADR 0016 carved (measured live 2026-11-21:
/// keys available,market,min,realms,region,scope,updated_at, and id 200 answered
/// the WHOLE `fields.price` as literal null - "the AH offers nothing right now",
/// the door's own third state). The rest stays on the wire: the tool reads what
/// it has a column for, and a key upstream adds cannot break a build - ADR
/// 0010/0016's rule for fields objects, kept exactly.
#[derive(Deserialize, Clone, Debug)]
pub struct TooltipFields {
    pub price: Option<Price>,
}

/// The manual layer (v0.8.0, ADR 0010): `manual_overrides` rides one JSON ARRAY
/// per row (this codebase never reaches for a map): `field, value, at` entries
/// recording what a USER said and when. `value` stays a wire `Value` - the
/// editable whitelist is TEXT or INTEGER and rides back verbatim, so nothing
/// here invents a shape.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ManualEdit {
    pub field: String,
    pub value: Value,
    pub at: i64,
}

/// The recipe files' own shape (v0.10.0, ADR 0021): the SECOND source the engine
/// fetches (docs/API.md's second-source section measures it: the MIT dataset at
/// raw.githubusercontent.com, 8 classic files, entry keys measured 2026-11-22
/// fresh on the wire). A file answers `profession, recipe_count, recipes[]`;
/// an entry carries `spell_id, spell_name, crafted_item_id, crafted_item_name,
/// required_skill_value, skill_tier, trivial_low, trivial_high, reagents[{item_id,
/// item_name, count}], learned_from, acquisition, acquisition_note, source` -
/// keys NOT typed here (`auto_granted, confidence, vendor_sell_price, craft_spell_id,
/// skill_line_id, game_version, known_limitation`) are recorded in ADR 0021 and
/// stay on the wire: the tool reads only what has a reader, and serde ignores
/// unknown keys by default, so a key the dataset adds cannot break a build -
/// the same rule TooltipFields keeps. Measured on the wire: 133 of 1240 classic
/// entries carry NO crafted_item_id at all (the enchanting craft applies an
/// effect - there is no crafted ITEM), so every answer below a crafted id is
/// Option; and `learned_from` rides as a wire Value - its key is `type`, a
/// reserved word, and nothing reads INTO it yet: the door chip reads the entry's
/// plain `acquisition` string, while the object's BYTES go to the DB whole.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RecipeFile {
    pub profession: String,
    pub recipe_count: Option<i64>,
    pub recipes: Vec<RecipeEntry>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RecipeEntry {
    pub spell_id: i64,
    pub spell_name: Option<String>,
    pub crafted_item_id: Option<i64>,
    pub crafted_item_name: Option<String>,
    pub required_skill_value: Option<i64>,
    pub skill_tier: Option<String>,
    pub trivial_low: Option<i64>,
    pub trivial_high: Option<i64>,
    pub reagents: Vec<Reagent>,
    pub learned_from: Option<Value>,
    pub acquisition: Option<String>,
    pub acquisition_note: Option<String>,
    pub source: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Reagent {
    pub item_id: i64,
    pub item_name: Option<String>,
    pub count: Option<i64>,
}
