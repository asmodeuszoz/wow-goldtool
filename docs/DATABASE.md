# SQLite layout

Single file, disposable. Deleting it only throws away cache: the next sync rebuilds it.

## Where the file lives (`db.rs` resolves it)

1. the CLI's `--db PATH` flag, which beats the env var for one run,
2. `WGT_DB` env var, if set (tests and CI point this at a fixture),
3. the Tauri **appDataDir** once the GUI is wired (`wow-goldtool.sqlite` inside it),
4. otherwise `HOME/.local/share/wow-goldtool/wow-goldtool.sqlite`, created on demand.

Opening the file is one code path for every entry point: `Connection::open` (rusqlite,
bundled SQLite — no system library needed), then either `check_schema` refuses a foreign
or stale-schema file, or `execute_batch(SCHEMA_SQL)` writes it and stamps
`schema_version` into `meta`. The schema is idempotent, so a fresh process (CLI or GUI)
can always attach; `db_path` (who picks the file, the four-way rule above) and `open_db_at`
(who opens it and stamps `schema_version`) are the two doors into that helper — measured at
v0.11.1, when the wrapper that used to bundle the two died with no call site (ADR 0024).

**Seven stale shapes are migrated instead of refused** (v0.9.0 grew the chain to four;
v0.9.5 grew it to five; v0.9.6 grew it to six again; v0.10.0 grew it to seven): the file on disk
saying `schema_version = 1` (the user's own catalogue, 19 258 rows, still never touched by a
refusal), `2`, `3`, `4`, `5`, `6` or `7` differs from this build by additive steps only, so `check_schema`
chains them: v1 gets v0.5.3's one `ALTER` first, v1/v2 the v0.6.0 batch, v1/v2/v3 the v0.8.0
batch of eight, v1..v4 the v0.9.0 batch of two `ALTER`s (`tooltip_text`/`tooltip_lc`, the
readable twin, ADR 0012), v1..v5 the v0.9.5 trio (ADR 0016: carved, and `price_min_now` FILLED
from v0.9.6 by ADR 0017's producer), v1..v6 THIS build's stamp pair — `price_now_at INTEGER`,
`price_now_missing_at INTEGER` (ADR 0017: the price refresh stage's own three states) — and
v1..v7 the v0.10.0's `CREATE TABLE recipe` + `CREATE INDEX recipe__crafted` (ADR 0021: one
table, two doors — the dataset's rows arrive where a refusal would throw away a 19 258-row
catalogue on the excuse of a schema bump), with the `meta` stamp moving to `8` in the same
breath (measured pre-tag on a schema-7 scratch copy: the table + index land, rows intact,
a SECOND open re-migrates NOTHING). Refusing any file would throw
away a 19 258-row catalogue on the excuse of a schema bump. A migrated file's **column order
differs from a fresh file's** (ALTER appends, SCHEMA_SQL lists the new columns mid-table —
the trio sits BEFORE `raw_json` in a fresh CREATE and AFTER `deleted_at`… at the END in a
migrated file, and v0.9.6's pair sits BETWEEN them; the new `recipe` TABLE is separate, so
its position cannot drift at all) — every reader in the code names columns
explicitly and nothing reads positions. A file **newer** than this build is still refused —
that direction cannot be guessed at (measured consequences: a pinned v0.7.0 binary refuses a
schema-4 file, a pinned v0.8.1 binary refuses THIS release's schema-7 file with `database on
disk uses schema 7, this build wants 4`, file untouched — and it refused the 6-file the same
way through v0.9.5).
See [ADR 0006](./DECISIONS/0006-checked-missing-third-state.md),
[ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md),
[ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md),
[ADR 0016](./DECISIONS/0016-the-price-homes.md) and
[ADR 0021](./DECISIONS/0021-the-recipe-doors.md).

## Schema (`schema_version = 8`)

```sql
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);

CREATE TABLE item (
  id INTEGER PRIMARY KEY,            -- TheWoWDB item id (sparse: 25 .. 288340 upstream,
  name TEXT, name_lc TEXT,           --   and 900000000+ for rows the user MADE)
  quality TEXT, quality_rank INTEGER,-- (drift corrected 2026-11-20: the file on disk
  item_level INTEGER, required_level INTEGER,       --   and SCHEMA_SQL both say plain `TEXT` — the
  item_type TEXT, item_subtype TEXT, slot TEXT,     --   `NOT NULL` this file used to
  item_set TEXT, unique_flag TEXT, flavor TEXT,     --   claim was aspirational, never
  tooltip_html TEXT, tooltip_text TEXT, tooltip_lc TEXT, -- the copy AND its readable twin (v0.9.0, ADR 0012)
  icon_url TEXT,                                    --   `id` wording)
  sockets TEXT, socket_bonus TEXT, classes TEXT, stats TEXT, effects TEXT,
  faction_prices TEXT,
  dmg_min INTEGER, dmg_max INTEGER, dps REAL, speed REAL,   -- weapon block
  vendor_sell INTEGER,              -- copper, what a VENDOR SELLS (the copy's own "Sell Price:" words)
  price_min INTEGER,                -- copper, cheapest AH buy from the STORED COPY (not live!)  <- the "gold" column
  price_market INTEGER,             -- copper, AH market value from the stored copy
  price_available INTEGER,          -- offers on the AH  (COUNT, never money)
  price_realms INTEGER,             -- realms watched for the price  (COUNT, never money)
  price_region TEXT, price_scope TEXT, price_updated_at TEXT,
  api_url TEXT, html_url TEXT, tooltip_url TEXT, tooltip_etag TEXT,
  list_etag TEXT, detail_etag TEXT, -- conditional GET bookkeeping
  list_synced_at INTEGER, detail_synced_at INTEGER, tooltip_at INTEGER,   -- unix seconds
  detail_missing_at INTEGER, tooltip_missing_at INTEGER,   -- the doors' checked third states
  icon_blob BLOB, icon_at INTEGER, icon_missing_at INTEGER, -- the icon's three states (v0.6.0)
  manual_at INTEGER,                -- the row was MADE by a user (reserved block, v0.8.0)
  manual_overrides TEXT,            -- one JSON ARRAY {field,value,at} - merged LAST on every write
  edited_at INTEGER,                -- the last field a user stated
  deleted_at INTEGER,               -- the tombstone every query hides
  vendor_buy INTEGER,               -- RESERVED (ADR 0016): what a vendor BUYS - NULL = no producer
  price_min_now INTEGER,            --   EVER wrote it; ADR 0017's measured NEGATIVES keep two
  price_trend_normal INTEGER,       --   homes reserved (no vendor-buy door/key, no trend door)
  price_now_at INTEGER,             -- the price refresh asked the LIVE door at this time (v0.9.6)
  price_now_missing_at INTEGER,     -- or checked: the AH offered NOTHING now (fields.price NULL)
  raw_json TEXT                     -- verbatim detail payload, forward-compatible
);

CREATE TABLE watch (id INTEGER PRIMARY KEY, added_at INTEGER, note TEXT);

CREATE TABLE sync_state (
  kind TEXT PRIMARY KEY,            -- "list" | "detail" | "icons" | "tooltip" | "price" | "recipes"
  status TEXT,                      -- idle | complete | stopped | ceiling | partial | cancelled
  page INTEGER,                     -- absolute page cursor (list); 0 for the id-queue stages
  done INTEGER,                     -- records handled during the most recent run
  total INTEGER,                    -- queue size this run started from
  message TEXT, updated_at INTEGER, error TEXT
);

CREATE TABLE price_history (
  seq INTEGER PRIMARY KEY,          -- SQLite auto-assigns (the INSERT omits it)
  item_id INTEGER,                  -- NO FK clause here: SQLite would not enforce
  sampled_at INTEGER,               --   one anyway (probe measured 2026-11-20: an INSERT
  price_min INTEGER,                --   INTO price_history with item_id = 999 and no such
  price_market INTEGER              --   item row SUCCEEDED) - the guard is the Rust order
);                                  --   inside upsert_detail (ADR 0007)
CREATE TABLE recipe (                  -- v0.10.0 (ADR 0021): the SECOND source's rows
  spell_id INTEGER PRIMARY KEY,        --   the set's OWN id space (classic ids: the
  profession TEXT,                     --   files speak `spell_id 2339` .. `2455`, NOT the
  recipe_name TEXT,                    --   forever-beta space a probe measured today)
  crafted_item_id INTEGER,             -- NULL = the set states NO crafted item at all
  crafted_item_name TEXT,              --   (measured: 133 of the real set's 1 240 entries,
  skill_required INTEGER,              --   all 133 `Enchanting`); a crafted id WITHOUT an
  skill_tier TEXT, trivial_low INTEGER,--   item row is the OTHER answer (10 of 1 105
  trivial_high INTEGER,                --   distinct crafted ids today) - the door ships
  reagents_json TEXT,                  --   BOTH, `crafted_db_name` null + the set's own
  learned_json TEXT,                   --   words beside it, never an invented name
  acquisition TEXT, acquisition_note TEXT,
  source TEXT                          -- the set's OWN provenance words, verbatim
);
CREATE INDEX item__name_lc ON item (name_lc);              -- v0.2.0
CREATE INDEX price_history__item ON price_history (item_id); -- v0.6.0 fresh CREATE and the v1/v2 -> v3 migration
CREATE INDEX item__detail ON item (detail_synced_at);      -- v0.5.3
CREATE INDEX item__rank ON item (quality_rank, item_level);
CREATE INDEX item__price ON item (price_min);
CREATE INDEX recipe__crafted ON recipe (crafted_item_id);  -- v0.10.0 fresh CREATE and the 7 -> 8 hop
```

Stage-1 (the `?page=` list crawl) fills the summary columns and `list_etag`; stage-2 (`/item/<id>`)
fills the price/stat columns, `icon_url`, `raw_json`, `detail_etag`; stage-3 (the record's own
icon FILE) fills `icon_blob`/`icon_at`; stage-4 (`/api/v1/tooltip/forever/item/<id>`) fills
`tooltip_html`/`tooltip_at`/`tooltip_etag`; the REFRESH stage (v0.9.6, ADR 0017) fills `price_min_now`
or the `price_now_missing_at` stamp through the tooltip door's `fields.price`. The RECIPE stage
(v0.10.0, ADR 0021) is the only stage on the SECOND source: eight conditional GETs
(one per `<profession>_recipes.json`, `meta` key `recipe_etag_<file>` carries the file's
ETag — the per-record `<x>_etag` columns above never name it), each answer UPSERTing its
entries into `recipe` (`ON CONFLICT (spell_id) DO UPDATE`, so a re-pull that answers 304
writes NOTHING and a re-pull that answers 200 keeps the rows, never twins them). `reagents_json`
is the entry's OWN array (`[{item_id,item_name,count},…]`, verbatim); `learned_json` carried NO
reader from v0.10.0 until v0.12.0 — ADR 0021 wrote that debt where it lives, and the recipe CARD
door (v0.12.0, [ADR 0025](./DECISIONS/0025-the-recipe-card-window.md)) is its FIRST reader: it reads
the stored object BY KEY through `serde_json` `Value` accessors (`word_at` / `id_at` take the KEY as
a string), because its key is `type`, a reserved word nothing may reach for (the rule ADR 0021 states
next to that struct, kept rather than renamed into a serde `rename` — `models.rs` carries ZERO serde
attributes and one renamed field would put a wire key's spelling on a type that answers other doors).
Measured on the user's own file (read-only) the day that door was written: the object is
`{"item_id":…,"item_name":…,"type":"schematic_item"}` — **804** of the 1 240 fetched entries state
`schematic_item` and name a SCROLL id, **413** state the `trainer` word and **23** the `automatic`
word with NO id at all (the three sum to 1 240: the fetched shape leaves no entry silent about
learning); of the 804 stated ids **747** have a visible item row and **477** of those carry the
scroll's OWN tooltip copy. The list door still reads only the entry's plain `acquisition` word: the
CARD door is what grew the `SQL_RECIPE_CARD_ROW` SELECT that reaches `learned_json`. A NULL `crafted_item_id` means "the set states no crafted
item", which is NOT the same as "the DB never saw this id" — the first is the dataset's own
truth, the second is a join fact the door reports with `in_db`/`crafted_db_name`). A NULL
price column in `item` still means "not fetched yet", distinct from "fetched and not on the
AH" — the difference is visible in the UI and in `cargo run -- stats`, and the recipes door
keeps the SAME split: an unpriced reagent answers "no price", never 0c (ADR 0015/0016's
rules, restated in ADR 0021).
On top of every stage's write sits the manual layer: `manual_overrides`
merges LAST (ADR 0010), so a field the user stated survives the next crawl while the columns the
crawl owns keep refreshing. The manual layer and tombstones never reach `recipe`: the manual
verbs own ONE whitelist of `item` fields (ADR 0010's door), and a recipe row is FETCHED, never
stated — ADR 0021 records this, and `recipes --ids`/`--watch` on the stage answer 2 for the
same reason (a recipe row has no per-record door to re-ask).

**A detail record has three states** (v0.5.3, [ADR 0006](./DECISIONS/0006-checked-missing-third-state.md)):

| state | columns | meaning |
| ----- | ------- | ------- |
| fetched | `detail_synced_at` | upstream answered with a record | later passes revalidate with the stored `detail_etag` |
| checked and gone | `detail_missing_at` | upstream answered `404` for an id the walk lists | one request **per crawl**, never again inside one |
| never looked at | neither | the only thing that is *pending* | fetched once, then it becomes one of the two above |

The two stamps are mutually exclusive: the stage-2 upsert writes `detail_missing_at = NULL`
for a record that answered (a thing that answered is not missing), and marking an id checks
first that a row exists (`SQL_ROW_OF_ID`) so a mark can never invent a name-less row for an
id the walk never listed. `status`/`stats` count the third state, and the UI prints it.

**The icon of an answered record has three states too** (v0.6.0, [ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md)):

| state | columns | meaning |
| ----- | ------- | ------- |
| copied | `icon_blob` + `icon_at` | the asset host answered with bytes; the DB holds them |

### The icon doors (v0.9.3, ADR 0014): lists read these columns, they write nothing

The three columns above are schema `3`'s (ADR 0008's icon stage), and v0.9.3's doors
READ them — `search … --icons` and `stats --icons` call `icon_b64_of`, which calls the
ONE `b64_text` encoder that lives in `src/db.rs` beside the bytes it encodes (it moved out
of `cli.rs` so the card and the lists cannot encode the same copy two ways; the move was
measured byte-identical). `schema_version` stays `5`: writing nothing means no hop, so a
DB that already copied icons answers the icon doors with no migration at all.

Rules that live in this file and are enforced by the doors:

* the image is always the DB's own bytes. `icon_url` is upstream's address and rides the
  row shape, but is never an image src — measured on the user's DB: 4 602 detailed rows
  ALL carry an `icon_url` while only 4 030 carry bytes, so hotlinking would show an image
  the DB never claims to hold (ADR 0008's rule, hosts measured serving `image/jpeg`);
* the trio's keys are always present on a row, the base64 payload rides only when the door
  was asked with `--icons` — because one icon costs about 3.1 K chars once encoded (avg
  2 327 bytes, min 1 385, max 3 268; 9 380 048 bytes over 4 030 copies), so an unbounded
  hit list is a JSON answer of tens of MB. The UI bounds its own asks (40 hits, 6 extremes);
* the three states stay three answers: bytes + `icon_at` → the image; `icon_missing_at`
  set and no bytes → the placeholder glyph with "checked as not there upstream <when>" in
  the hover; neither stamp → the glyph with "never fetched". 4 030 of 19 258 rows on the
  user's file can show an image, and 15 228 honestly cannot — most rows show the glyph;
* rows are COLLECTED before the per-row icon reads run, so a list never straddles a write
  of the table it came from (the refresh door's rule);
* ZERO requests leave either door, so ADR 0004's network door (`3`) is unreachable there
  and the doors answer `0`, `2` (usage) and `4` (missing DB) only.
| checked and gone | `icon_missing_at` | the host answered `404` for that record's `icon_url` | one request, never asked again — nothing revalidates an icon |
| not pending, not copied | neither, and `icon_url` NULL or `''` | upstream answered no icon to copy (measured live: Peace Tea, Craftsman's Writ, Scroll of Imbue Iceknife, Lodestone answer `"icon_url": ""`) — nothing to fetch, no check to record |
| waiting | none of the stamps, `icon_url` a real url | the only thing the icon stage sees as pending |

`copy_icon` upserts `icon_missing_at = NULL` for the same reason `upsert_detail` clears
`detail_missing_at`; `mark_icon_missing_at` refuses to mark an id with no row (`SQL_ROW_OF_ID`
again — a stage's third state marks what upstream stated about an EXISTING record, never
invents a row). No icon ETag exists anywhere: the pending rule excludes copied icons, so the
copy path is asked exactly once per icon and there is no revalidation whose promise would need
one ([ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md) rejects the inline alternative
that would double a chunk's requests and leave pre-v0.6.0 records un-copied forever).

**The description copy of an answered record has the same three states** (v0.8.0,
[ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md)):

| state | columns | meaning |
| ----- | ------- | ------- |
| copied | `tooltip_html` + `tooltip_at` (+ `tooltip_etag`) | the tooltip door answered with the card's own markup — upstream's own `html`, kept as a VALUE and printed as escaped text, never rendered |
| checked and gone | `tooltip_missing_at` | the door answered `404` with its own JSON (`No item … in forever.`) for a record whose DETAIL exists | one request, never asked again inside one crawl |
| not pending, not copied | neither, and the record has no detail stamp | an id upstream does not have: asking this door costs a request AND would state a fact upstream never stated |
| waiting | a detail stamp, no tooltip stamp | the only thing the tooltip stage sees as pending |

### The readable twin (v0.9.0, ADR 0012)

`tooltip_text`/`tooltip_lc` are DERIVED from the copy, written in the SAME upsert on every
write path — the tooltip stage derives from the MERGED copy (manual overrides merge LAST,
ADR 0010, so the twin always matches what the DB SAYS), `set-item ID tooltip_html=…` derives
it in the same UPDATE as the stated value, and `add-item` births it beside the stated fields.
The derivation is the measured three moves (upstream tooltip bodies were read whole: div/span/img
tags, ZERO entities — no decoder exists because none was measured, and entities would show
verbatim): every `<...>` span leaves ONE space (v0.9.1 correction, measured: live tooltip html is
MINIFIED and a bare span-DROP glued words across element boundaries — the user file's first 4 602
twins read `Worn ShortswordItem Level 2Main Hand...`, while the FIXTURE's own card carries spaces
at the boundaries, which is why it never showed it), then collapse whitespace runs, trim. The local search door grew the
scope to match: `search TEXT --where name|description|all` (default `name`, the UI's old
behaviour), `description` reads `tooltip_lc` behind `IS NOT NULL` (never-fetched and checked-gone
records skip honestly), `all` is name OR description, and a bogus scope answers the usage door
(`2`). Tombstoned rows stay invisible in EVERY scope (`visible_clause` is pushed first).
That gap is closed by ONE door (v0.9.2, ADR 0013): `refresh-twins` calls the SAME Rust
reader over the whole DB and rewrites only the rows whose twin differs or is missing —
ZERO requests leave it, so nothing is re-fetched. Measured on the user's own file the day
that door was written: 4 602 copies, ZERO matching the corrected rule, and the stored twin
equal to the recomputed v0.9.0 GLUED shape for ALL 4 602 — the corrected rule reaches NEW
copies only, because the pending rules refuse a stamped copy (ADR 0007) and a `304` writes
nothing. Tombstones and manual rows ride (ADR 0010's stated `tooltip_html` is what the DB
SAYS for those rows), rows are COLLECTED before any write (a sqlite row walk over the table
this door rewrites must not interleave with its own UPDATEs), and the door is idempotent:
a second press answers 0 rewritten and still exits 0. No schema hop — it writes columns
schema `5` already has.

`write_tooltip` upserts and clears `tooltip_missing_at` (a copy is not a check);
`mark_tooltip_missing_at` checks the row exists first (`SQL_ROW_OF_ID`, ADR 0006's guard).
Unlike the icon door this one DOES keep an ETag (`tooltip_etag`): measured 2026-11-20 the tooltip
body answers with `ETag W/"2b111cbcd3bbb4d7078f6d05"` and `cache-control public, max-age=300`,
so a targeted re-ask costs a `304` and writes nothing — probe: `sync --stage tooltip --ids 25`
again in one fixture process answers `requests 1, items 1, written 0, unchanged 1`.
`tooltip_html` is the description copy the milestone asked for: the forever/item body carries
no description field at all (measured key list, docs/API.md).

**The manual layer — four columns, one rule** (v0.8.0, [ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md)):

| column | written by | meaning |
| ------ | ---------- | ------- |
| `manual_overrides` | `set-item`, `add-item` | ONE JSON ARRAY per row of `{field, value, at}` entries — what a user said and when. Every write path applies it AFTER the payload binds (list upsert, detail upsert, tooltip write), so the user's words merge LAST |
| `edited_at` | `set-item`, `add-item` | the last field a user stated on this row |
| `manual_at` | `add-item` only | the row was MADE by the user and lives in the reserved block (`MANUAL_ID_BASE = 900000000`; measured max upstream id on the user's own file: `288 340`, so the block is provably clear). A crawl never walks it, and its card answers from the DB's own columns (`SQL_COLUMNS_CARD`) with ZERO requests |
| `deleted_at` | `delete-item` (cleared by `undelete-item`) | the tombstone: DELETE is a mark, not an erase, because the list stage UPSERTs every id upstream still lists — a HARD delete would be resurrected by the next pass |

The whitelist the edit doors accept is `name, quality, item_level, required_level, item_type,
item_subtype, flavor, vendor_sell, tooltip_html` — TEXT or INTEGER columns this DB can bind
verbatim. The auction columns stay out on purpose (they are live data the crawl refreshes, and an
override "fixing" one would be a permanent lie about CURRENT prices); so do `id`, the icon
columns and the JSON-shaped columns (`sockets`, `classes`, `stats`, `effects`, `faction_prices`),
which a hand-typed blob would otherwise invent a shape inside. `set-item` rides the derived
search columns in the SAME write (`name_lc` for a `name`, `quality_rank` for a `quality`), so a
rename is findable by `search` on the same breath — probe: `set-item 40 name=ProbeRenamed` then
`search ProbeRenamed` answers `total 1` immediately.

**One price sample per fresh write** (v0.6.0, [ADR 0007](./DECISIONS/0007-price-sampling.md)):
`upsert_detail` appends a `price_history` row ONLY after a fresh (200) detail write, and only
when the payload carried prices (`price_min` non-NULL — a record not on the AH has nothing to
sample, and a blank chart point is a lie). A `304` revalidation, a list write, a check-mark and
a re-insert write NOTHING here, so the ledger counts fresh observations, not requests. There is
**no FK clause** on `item_id`: measured 2026-11-20 on SQLite 3.54.1, an `INSERT` with an
implicit FK clause referencing a missing `item` id SUCCEEDS (the clause is accepted and not
enforced), so shipping one would decorate a contract the runtime does not provide — the guard
is the Rust order inside `upsert_detail` (the item row is written before the sample), and
`seq` is auto-assigned so an append can never collide with an older observation.

Reuse bookkeeping (v0.3.0): `raw_json` is the verbatim last detail body, which is what
a live lookup answers when the API returns `304` for a stored `detail_etag` — no fresh
payload, no re-parse. List pages have no row to hang an ETag on, so `meta` holds the
revalidation state per page under a key convention:

| meta key | value | why |
| -------- | ----- | --- |
| `list-page-<n>-etag` | last ETag of that page | sent back as `If-None-Match` on every re-crawl |
| `list-page-<n>-more` | `1`/`0` — the page's `has_more` | a `304` has no body, so the crawl needs the hint to know whether to walk on |
| `list-page-<n>-count` | the page's `count` | the crawl keeps `total` honest across `304`s |
| `recipe_etag_<file>` | last ETag of that recipe FILE | v0.10.0 (ADR 0021): the SECOND source's conditional GETs, one key per file (`alchemy_recipes.json` .. `tailoring_recipes.json`); a re-pull sends it back as `If-None-Match`, pays a real `304` and writes NOTHING |

Targeted detail runs (v0.5.0) keep the same rules and add two readers: `item ID --cached`
answers the card from `raw_json` with **no request at all** (`not-found` when the row has no
stored copy), and `sync --stage detail --ids …` / `--watch` visit those ids instead of the
pending queue, sending the stored `detail_etag` so a second pass costs requests but no writes
(`written 0, unchanged 2`). `detail_synced_at` is what the card prints as "fetched …", which
is how a user tells today's prices from last month's without leaving the app.

The live-search door (v0.7.0) does not appear in this file's tables, and that is a decision,
not an omission: `live-search TEXT` never opens a DB — no row, no `meta` key, no `sync_state`
row, no sample, no mark (ADR 0009). An ask is one request whose answer is printed as it came,
upstream's own; the file's promise stays "everything the app answers offline lives here", and
the two doors — the local one reading these tables and the live one reading upstream — never
share a name or a payload.

Indexed for the queries the UI actually issues (SQLite's own index syntax, `ON` not `IN`
— the fresh `CREATE`s above; the v0.6.0 migration re-issues only `price_history__item`
because a migrated file already has the rest):

The pending filter gained no index in v0.5.3 and none in v0.6.0 either: the stage reads
that list once per run, and guessing at a two-column index SQLite may not even use is a
plan, not a measurement.

### Money (v0.9.4, [ADR 0015](./DECISIONS/0015-the-money-words.md)): the words ride the ANSWER, there is no money column

Every price column above is and stays **INTEGER copper** — 100 copper is 1 silver and 100
silver is 1 gold, and the base is not an assumption: the DB's own tooltip copies state it,
because a copied tooltip already writes its sell price in money words. Measured on the user's
file, `money_text(vendor_sell)` EQUALS that row's own copy words for **4 230 of 4 231** rows
(810 `45267` ↔ `Sell Price: 4g 52s 67c`, 4499 `25000` ↔ `2g 50s`, 12720 `20000` ↔ `2g`, 733
`100` ↔ `1s`, 36 `7` ↔ `7c`), and the ONE row that disagrees is upstream's own disagreement
(item 25: column `500` beside its copy's `Sell Price: 7c`), reproduced by the committed
fixture in the same shape. The copies give the zero rule too: a zero denomination is DROPPED
(`20042` ↔ `2g 42c` on row 14138 — the only middle-zero row in 4 231, while `price_min` has
19 and `price_market` 17 of that class), so `money_text` writes what the copies write.

Money is DERIVED at answer time by the ONE formatter in `src/db.rs` (`money_text`, `money_of`,
`money_of_avg`) from the copper the very same read already carries. Consequences that must stay
true, because the numeric contract of this file is the contract everything else reads:

* the schema does not grow and does not move: `schema_version` stays `5` (measured with
  `SELECT key, value FROM meta WHERE key GLOB 'schema*'` on the mirror's and the user's DB
  after a replay that ran every money door), and there is nothing to migrate;
* there is **no refresh door for money** — unlike the twin, which v0.9.2 re-derives because a
  twin is stored. A DB that never saw money still answers money on the next read, which is
  exactly why `refresh-twins` over the mirror still answers 4 copies / 0 rewritten / 4 matching
  / ZERO requests after this release;
* `SQL_COVERAGE`, `SQL_QUALITY`, `SQL_CHEAP`, `SQL_RICH`, `SQL_PRICE_SAMPLE`, `SQL_HISTORY_OF`
  and `HIT_COLUMNS` are UNCHANGED, so the read slots stay `0..14` and no aggregate or its CAST
  guard can drift;
* the sorts and the scripts keep the copper: `ORDER BY (price_min IS NULL) ASC, price_min ASC`
  and every `*_money` sibling are separate values, and money costs about 56 bytes per priced
  row on the mirror while the longest money string in the user's whole DB is 13 chars
  (`1334g 99s 49c` for `13 349 949` copper), so money always rides and needs no flag — the
  opposite of ADR 0014's 3.1 K chars per icon;
* `price_available` and `price_realms` are **counts**, not money, and never get a money
  sibling: measured, row 25 answers `available 12` for 12 offers of 15 000 copper, and `realms`
  spans `1..12` across the user's DB;
* an absent price answers `null` money beside `null`, never `0c` (ADR 0004's rule against made-up
  zeros). A copper of `0` would answer `"0c"`, and that is a RULE, not a measurement: measured
  MIN over every copper column is `1`, so 0 rows in the DB are at copper 0;
* the average FLOORS: SQLite answers `AVG` as a float (measured `AVG(price_min) = 54 391.705896`
  copper → `5g 43s 91c`), and rounding up would print `5g 43s 92c`, a price no observation
  reached. The exact float still rides beside it.

### The price homes (v0.9.5, [ADR 0016](./DECISIONS/0016-the-price-homes.md)): carved now, filled later — NULL means NO PRODUCER

The user's testing asked for FOUR price homes; the schema hop 5 → 6 carves three of them
(`vendor_buy`, `price_min_now`, `price_trend_normal` — the names are OURS: this is the user's
product wish, and NO upstream door has ever answered these words, so docs/API.md grows
nothing and no payload key changed). HOW the values get fetched is explicitly later work
(the user said so); this release creates the columns and NOTHING writes or reads them:

* every door answers BYTE-IDENTICAL across the hop (measured with BOTH the v0.9.4 binary —
  the tree at `4875222`, built in a scratch worktree — and this one over two copies of ONE
  fixture DB: `search --limit 6` 1 941 bytes both sides, `stats --limit 3` 1 533, `stats`
  1 850, `stats --icons` 17 462, `item 25 --cached` 4 336, the made card 1 103, 12809/40
  --cached 3 868/4 022, the --where steps 763/755/3 363, `refresh-twins` 223; `status`
  echoes its own `db.path`, so with that string pinned its JSON is IDENTICAL — the paths
  cannot match, because the OLD binary refuses the file the NEW one migrated);
* NULL in the trio means **"no producer has ever written it"** — deliberately NOT the same
  answer as the price columns' "not fetched yet vs fetched and not on the AH" pair; the
  third state arrives WITH the producers, in later releases, and only THEN gets its
  checked-missing shape;
* the hop itself is honest additive ALTERs: the user's own file went 5 → 6 the moment a
  door opened it, **19 258 rows / 4 231 vendor_sell / 4 285 samples intact**, trio NULL on
  all of them, and a second open migrated NOTHING (idempotent, second `status`
  BYTE-IDENTICAL);
* the money formatter (ADR 0015) will reach these columns the day producers write them —
  `money_of` takes any `Option<i64>` copper — but with ZERO measured rows there is ZERO
  money to answer now, and inventing money of NULL was rejected by ADR 0004's rule again;
* `price_min` (the stored copy's cheapest AH buy) and `price_min_now` (the CURRENT one) are
  TWO columns on purpose: a stored payload's price ages, a live refresh does not, and
  collapsing them would make one column answer two different questions over time.

No index for the trio (a guessed index for columns nothing queries is a plan, not a
measurement — ADR 0011's rule again), and the fresh-CREATE order (trio BEFORE `raw_json`)
vs the ALTER order (appended at the END) is exactly why every reader names columns — the
rule the v0.6.0 migration comment already states.

### The live min buyout (v0.9.6, [ADR 0017](./DECISIONS/0017-the-live-min-buyout.md)): the FIRST producer, the door that answers NOW

`sync --stage price` re-asks the TOOLTIP door for records that already hold a copy — measured
live 2026-11-21 that door answers `fields.price` with a CURRENT min (id 25 `min 149` @ its own
`updated_at 22:25:10`; ids 202/237/763 answering 5000/3500/4425 while their STORED copies carried
NO price at all — the live value MOVES APART from `price_min` ON THE SAME record), and its own
third state (id 200 answers `fields.price` as literal NULL: "looked, the AH offers NOTHING
right now"). The stage writes `price_min_now` + the pair above (`SQL_SET_PRICE_NOW` /
`SQL_MARK_PRICE_NOW_MISSING`, row-guarded like every stage-2 write), and `SQL_PENDING_PRICE`
(stats.rs, the twin of the tooltip/icon rules) walks COPY EXISTS + never written + never
checked-empty, manual rows and tombstones refused, `id ASC`: one request per copied record EVER
(measured: a second press asks zero). `price_now_at` is OUR ask time — the door's own
`updated_at` TEXT is deliberately NOT stored (two stamps of one fact; recorded in ADR 0017).
Fresh writes sample NOTHING: ADR 0007's sample stays on the DETAIL write. The card door grew
the value + money sibling (`SQL_PRICE_MIN_NOW_OF`/`price_min_now_of` + `card_price_now_parts`,
+2 keys — this release's documented wire delta, NOT byte-identity), while `vendor_buy` and
`price_trend_normal` stay NULL: ADR 0017 measured there is NO vendor-buy door or key (4 602
copies' 28-key set holds `vendor_sell` only, upstream lists exactly FOUR doors; the only
"can be bought" signal is flavor prose on exactly ONE copy, id 247786) and ZERO trend words in
19 258 rows (all 20 "volume" hits are book titles). NULL in THOSE two still means "no producer
has ever written it"; in `price_min_now` it now splits into refreshed / checked-price-less /
never-asked, the ADR 0006 pattern with its OWN stamp pair.

**v0.11.0 (ADR 0023) READS this twin where a join already pays** — no column, no hop
(schema stays `8`: the live trio is schema 7's OWN, born with ADR 0016 and written at
ADR 0017, THIS release reads it): `SQL_LIVE_NAME_PRICE` widens to `price_min_now,
price_now_at, price_now_missing_at` in the SAME SELECT the recipe joins already pay
(one read per row — the by-value joins cannot meet a row a write moved mid-read, and
ZERO new queries/requests, ADR 0014's ONE-path rule), and `RecipeReagent` /
`RecipeRow` carry the live copper + money words (the ONE formatter again, ADR 0015)
+ the two stamps. The recipe tab's cell draws the pair by the stamps, never merging
them: live money beside the STORED money ONLY when they differ, `no auction right
now` for a set `price_now_missing_at` (ADR 0017's third state), `live price not
refreshed yet` for both-NULL stamps, nothing for live == stored, NO live fields at
all for an id with no DB row. The search-hit shape did NOT move (`HIT_COLUMNS` +
the Hit money pair stay v0.9.4's — ADR 0020's deferral, measured unchanged);
`vendor_buy` / `price_trend_normal` stay NULL (ADR 0016).

### The recipe CARD door (v0.12.0, [ADR 0025](./DECISIONS/0025-the-recipe-card-window.md))

`cargo run -- recipe SPELL_ID [--icons]` answers ONE fetched entry as far as the TWO sources
say it — LOCAL, ZERO requests (ADR 0009/0012's local-door rule: it reads the `recipe` row and joins
BY VALUE through the `item` rows this DB owns; the UI's click on a recipe row's name runs exactly
this argv, so a click and a cargo line are ONE code path, ADR 0014). TWO SELECTs, and only ONE of
them is new to this door:

```sql
-- the door's OWN row read: the LIST door's SELECT never reaches learned_json, so this
-- door reads its own const (spell_id is the table's PRIMARY KEY: at most ONE row answers)
SELECT spell_id, profession, recipe_name, crafted_item_id, crafted_item_name, skill_required,
       skill_tier, trivial_low, trivial_high, reagents_json, learned_json, acquisition,
       acquisition_note, source FROM recipe WHERE spell_id = ?

-- the FOUR columns the LIST door never reads, for ONE id, tombstone-guarded like its twin
SELECT quality, item_level, required_level, vendor_sell FROM item WHERE id = ? AND deleted_at IS NULL
```

**v0.12.1 (ADR 0026) made the CODE say what the comment above already states:** the door TAKES
the ONE row the `spell_id` PRIMARY KEY states, it never walks it (`if let Some(one_row) = mapped.next()`,
`fix(db) 56e0356`) — clippy's DENY-by-default `never_loop` ERROR (`error: this loop never actually loops`,
`src/db.rs:3029:5`, the one build-time line that moved a CI verdict at v0.12.0) died BY NAME, NOTHING was
muted and NO gate was added, and the door's answers moved ZERO bytes (measured: 22/22 BYTE-IDENTICAL against
the pinned v0.12.0 binary over one fresh fixture DB, the NINE card doors included, plus 13/13 md5-equal
through BOTH binaries over the user's own file). The SELECT above did NOT move — only the SHAPE around it
did, and [ADR 0026](./DECISIONS/0026-a-clippy-error-is-not-advice.md) keeps why the mute was rejected.

EVERYTHING else in a cell rides readers that ALREADY exist (`live_name_price_of` for name + stored
min + the live twin + BOTH stamps, `money_of` for the words, `icon_b64_of`/`icon_at_of`/
`icon_missing_at_of`, `tooltip_text_of`/`tooltip_at_of`/`tooltip_missing_at_of`, and the materials
ride `join_recipe_reagent` UNTOUCHED) — a card's cell is the SAME code the item card and the list
joins use, so the two CANNOT say different things about one id (ADR 0014's one-path rule). The
tombstone rule is MADE true in the DOOR, not assumed: measured, the six stamp SELECTs
(`SQL_TOOLTIP_TEXT_OF` / `SQL_TOOLTIP_AT_OF` / `SQL_TOOLTIP_MISSING_AT_OF` / `SQL_ICON_BLOB_OF` /
`SQL_ICON_AT_OF` / `SQL_ICON_MISSING_AT_OF`) ask on `id` ALONE with NO `deleted_at` clause, so the
card door asks them ONLY for a row whose `SQL_CARD_CELL` read saw it alive — a row the user buried
keeps its stamps buried, and its cell answers through the set's own bytes with `in_db` false
(ADR 0010's rule, kept where the card ships). Money keeps ADR 0015's contract on the new surface
(vendor sell, stored min and the live twin each carry their words beside their copper; the audit
over the fixture's own answers measures 0 pairing defects and 13 reagent COUNTS carried with NO
money beside them), and `vendor_buy` / `price_trend_normal` stay NULL — this door READS the live
twin, it grows NO producer for the other two homes (ADR 0016's grounds, read back 0/0 in the
fixture census). `tooltip_html` is NOT in this answer at all: the card carries the readable twin
only, ADR 0012's rule that an upstream body printed into a page would let upstream write it.

**A recipe entry's scroll is a FOURTH state set, all named (v0.12.0)** — and the door's whole
honestness lives in these four NULLs, measured on the user's file (read-only, ZERO requests) the
day the door was written:

| state | columns / words | meaning |
| ----- | --------------- | ------- |
| the set states a SCROLL by value | `learned_json` carries `type: "schematic_item"` + an `item_id` | 804 of 1 240 entries: the SET itself states the scroll that teaches this recipe, so its cell reads the DB's OWN row for that id (747 reach a visible row, 477 of those carry the scroll's OWN tooltip copy) |
| the set states a WORD that names no item | `type: "trainer"` (413) / `type: "automatic"` (23) | the card states the type word and NO scroll cell at all — the gap is the DATASET's own truth, not a missing join |
| the set states NOTHING | `learned_json` NULL | the fetched shape's own absence (the fixture's row 638 keeps it; measured 8 NULLs of 12 rows there) — never the same answer as the two words above |
| a stated id this DB has no VISIBLE row for | `scroll.in_db` false + the set's own `dataset_name` | 57 of the 804 stated ids have no visible row (804 − 747); the set's name rides BESIDE the state word, never instead of it, and a BURIED row answers the same chip (the tombstone rule above) |

Measured and REJECTED as links, the same pass (this is why the card has honest limits, not
decorative ones): `spell_id` used as an item id answers **651** lands on an item row of which only
**20** carry `item_type Recipe` — spell ids and item ids are two namespaces, and 631 wrong cells
would ride a card that looks right; the name link `"Recipe: " + crafted_item_name` reaches only
**123** of 1 240 (case-sensitive and case-folded answer the SAME number) and of those 123 **ZERO**
land on the entry's own `crafted_item_id` — **all 123** attach to a DIFFERENT item, so that join does
not merely under-reach, it MIS-attaches (716 `Recipe`-typed rows all carry copies but only **122**
of them are named `Recipe:*`, and only 250 items anywhere are). No invented name, id or chip
anywhere on this path (ADR 0004's no-invented-answers rule again).


## Query recipes

`query.rs` builds these from a `Filter` struct so the UI and the CLI share one path. Values
are always bound parameters, never pasted into the SQL text:

```sql
-- search + quality/type filter + cheapest first, virtualised page 3
SELECT id, name, quality, quality_rank, price_min, price_market, price_available, html_url, icon_url
  FROM item
 WHERE name_lc LIKE ? ESCAPE '#'
   AND quality = ?
   AND item_type = ?
   AND price_min IS NOT NULL
 ORDER BY (price_min IS NULL) ASC, price_min ASC, name ASC
 LIMIT ? OFFSET ?;
```

The paging clause is ONE composition (v0.10.1, ADR 0022): a door that pages calls
`paging_clause(limit, offset)`, which emits `LIMIT ?` alone, `LIMIT ? OFFSET ?`, or
nothing — NEVER a bare `OFFSET ?` (SQLite refuses that outright: measured today, both
paging doors answered rc 4 carrying `near "OFFSET": syntax error`). "No `--limit`
given" means "the rest of the rows", and SQLite's OWN token for that is a NEGATIVE
limit value: `LIMIT -1 OFFSET 3` answered rows 4..10 of a 10-row scratch table,
IDENTICAL to the bare SELECT's tail (measured today on the DB this tool links, sqlite
3.54.1) — so the rule rides SQLite's documented semantics instead of inventing a
sentinel row-count or an arbitrary cap that would hide rows. The recipe door joins
the SAME rule (ADR 0014: two doors, one clause).

```sql
-- the count behind the same filter, so "showing 40 of 1250" cannot drift
SELECT COUNT(1) FROM item WHERE name_lc LIKE ? ESCAPE '#' AND ...;
```

```sql
-- coverage + price analytics (drives `cargo run -- stats` and the sync panel).
-- the SUM() columns are CAST (rusqlite reads rows by declared column type, and a bare
-- expression column has none, so an unCAST aggregate is a type error, not a NULL) and
-- IFNULL (an empty table makes SUM() NULL, which a INTEGER column cannot bind).
-- The icon pair is the v0.6.0 addition; v0.8.0 grows five more counts — the tooltip
-- door's two states and the manual layer's three (edited = rows the user stated something
-- about, manual = rows the user MADE, deleted = the tombstones the queries hide). The
-- price columns then moved from read slots 7-9 to 12-14: the expected-count guard in the
-- audits is what catches that drift, and the CLI, the UI and the audits read ONE answer.
SELECT COUNT(1),
       CAST(IFNULL(SUM(name IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(price_min IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(detail_synced_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(detail_missing_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(icon_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(icon_missing_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(tooltip_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(tooltip_missing_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(edited_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(manual_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(deleted_at IS NOT NULL), 0) AS INTEGER),
       MIN(price_min), MAX(price_min), AVG(price_min) FROM item;
```

```sql
-- the ids the detail stage still has to fetch: pending means **never fetched and
-- never checked-missing**. A fetched record with a NULL price is a different state
-- this file keeps apart on purpose — the item exists, it is simply not on the AH —
-- and under the old `price_min IS NULL OR` clause such items stayed pending forever,
-- so the detail stage never terminated and a one-click "pull everything" could never
-- honestly report nothing pending (v0.5.2). A record upstream has dropped is a third
-- state (v0.5.3): it costs one request per crawl instead of one per run, which is
-- what lets a catalogue holding `OLD…`/`Deprecated …` ids finish at all.
SELECT id FROM item
   WHERE detail_synced_at IS NULL AND detail_missing_at IS NULL
   ORDER BY id ASC LIMIT ?;
```

```sql
-- the arithmetic the v0.5.3 audits check on the scratch DB (live API, 3000 listed ids):
-- 3000 = 10 fetched + 3 checked-gone + 2987 pending, and the next crawl asked none of
-- the three marked ids again
SELECT COUNT(1) FROM item;
SELECT COUNT(1) FROM item WHERE detail_synced_at IS NOT NULL;
SELECT COUNT(1) FROM item WHERE detail_missing_at IS NOT NULL;
SELECT COUNT(1) FROM item WHERE detail_synced_at IS NULL AND detail_missing_at IS NULL;
```

```sql
-- items in the watchlist whose detail record is missing or older than 6 h
SELECT id FROM watch
   WHERE detail_synced_at IS NULL OR detail_synced_at < ?;
```

```sql
-- the ids the icon stage still has to copy: answered AND no icon copied AND not
-- checked as no-icon-upstream AND there IS a url to fetch. The url terms are not
-- decoration: 4 of 29 detailed scratch records answer "icon_url": "" (measured
-- live), and under a rule without them pending would never drain and "nothing
-- pending" would lie - v0.5.2's trap for prices, again. Plain id order here:
-- every icon candidate already carries a detail stamp, so the re-ask-first CASE
-- the detail queue needs would be a no-op on this one.
SELECT id FROM item
   WHERE detail_synced_at IS NOT NULL AND icon_at IS NULL
     AND icon_missing_at IS NULL AND icon_url IS NOT NULL AND icon_url <> ''
   ORDER BY id ASC LIMIT ?;
```

```sql
-- the card's trend window: the newest N samples of ONE item, read newest-first
-- (the index answers it) and reversed in Rust so the chart runs oldest -> newest
-- left to right. The DB keeps the whole ledger; the card takes CARD_HISTORY_LIMIT
-- (40) because a card is a card (ADR 0007).
SELECT sampled_at, price_min, price_market FROM price_history
   WHERE item_id = ? ORDER BY sampled_at DESC, seq DESC LIMIT ?;
```

```sql
-- the ledger count is a SECOND read off the second table (SQL_COUNT_SAMPLES,
-- straight from stats.rs): different rows, same Coverage answer, so the CLI,
-- the UI and the audits cannot drift apart.
SELECT COUNT(1) FROM price_history;
```

Rules: filter values are bound as parameters (never interpolated), user text is escaped for
`LIKE` (`#`, `%`, `_`) and every `LIKE` carries `ESCAPE '#'`, and NULL prices sort last rather
than being treated as cheap — a gold tool must not claim an unpriced item is free.
