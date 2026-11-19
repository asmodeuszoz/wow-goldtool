# SQLite layout

Single file, disposable. Deleting it only throws away cache: the next sync rebuilds it.

## Where the file lives (`db.rs::db_path()`)

1. `WGT_DB` env var, if set (tests + CI point this at a fixture),
2. otherwise the Tauri **appDataDir** (`wow-goldtool.sqlite` inside it),
3. otherwise, in CLI mode where no Tauri dirs exist, `./wow-goldtool.sqlite` beside the
   executable.

The DB opens with `SQLite_::set_database`, then `set_vlog(4, STDERR)` only in debug builds,
and every entry point calls `ensure_schema()` — the schema is idempotent, so a fresh
process (CLI or GUI) can always attach.

## Schema (`schema_version = 1`)

```sql
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);

CREATE TABLE item (
  id INTEGER PRIMARY KEY,            -- TheWoWDB item id (sparse: 25 .. 288340)
  name TEXT NOT NULL,
  name_lc TEXT NOT NULL,             -- lowercase copy, drives local search
  quality TEXT,
  quality_rank INTEGER,             -- 1 common .. 7 artifact/heirloom, sort helper
  item_level INTEGER,
  required_level INTEGER,
  item_type TEXT,                   -- Weapon, Armor, Consumable, ...
  item_subtype TEXT,                -- One-Handed Sword, Cloth, ...
  slot TEXT,                        -- Main Hand, Head, ...
  item_set TEXT,
  unique TEXT,
  flavor TEXT,
  icon_url TEXT,
  sockets TEXT,                     -- JSON array
  socket_bonus TEXT,
  classes TEXT,                     -- JSON array
  stats TEXT,                       -- JSON array
  effects TEXT,                     -- JSON array
  faction_prices TEXT,              -- JSON array, per region/faction price buckets
  dmg_min INTEGER, dmg_max INTEGER, dps REAL, speed REAL,   -- weapon block
  vendor_sell INTEGER,              -- copper
  price_min INTEGER,                -- copper, cheapest AH buy        <- "gold" column
  price_market INTEGER,             -- copper, AH market value
  price_available INTEGER,          -- offers on the AH
  price_realms INTEGER,             -- realms watched for the price
  price_region TEXT, price_scope TEXT, price_updated_at TEXT,
  api_url TEXT, html_url TEXT, tooltip_url TEXT,
  list_etag TEXT, detail_etag TEXT, -- conditional GET bookkeeping
  list_synced_at INTEGER, detail_synced_at INTEGER,   -- unix seconds
  raw_json TEXT                     -- verbatim detail payload, forward-compatible
);

CREATE TABLE watch (id INTEGER PRIMARY KEY, added_at INTEGER, note TEXT);

CREATE TABLE sync_state (
  kind TEXT PRIMARY KEY,            -- "list" | "detail"
  status TEXT,                      -- idle | running | paused | done | error | cancelled
  page INTEGER, done INTEGER, total INTEGER,
  message TEXT, updated_at INTEGER, error TEXT
);
```

Stage-1 (`?page=`) fills the summary columns and `list_etag`; stage-2 (`/item/<id>`) fills
the price/stat columns, `icon_url`, `raw_json`, `detail_etag`. NULL price columns mean
"not fetched yet", which is distinct from "fetched and not on the AH" — the difference is
visible in the UI and in `cargo run -- stats`.

Indexed for the queries the UI actually issues:

```sql
CREATE INDEX item__name_lc,  idx item__quality_rank, idx item__item_level,
CREATE INDEX item__price_min, idx item__detail_synced_at;
```

## Query recipes

Local search = substring on `name_lc`, ordered by a **named sort**, with typed filters.
`db_query.rs` builds these from a `Query` struct so the UI and the CLI share one path:

```sql
-- search + quality/type filter + cheapest first, virtualised page 3
SELECT id, name, quality_rank, item_level, item_type, slot, price_min, vendor_sell, icon_url
  FROM item
 WHERE name_lc   LIKE '%shortsword%' ESCAPE '\'
   AND quality_rank = 1
   AND item_type = 'Weapon'
   AND price_min IS NOT NULL
 ORDER BY price_min ASC, name ASC
 LIMIT 40 OFFSET 80;
```

```sql
-- facet counts for the filter chips (single pass, GROUP BY each facet)
SELECT quality, COUNT(1) FROM item WHERE name_lc LIKE '%herb%' ESCAPE '\' ;
SELECT item_type, COUNT(1) FROM item WHERE ... ;
```

```sql
-- coverage + price analytics (drives `cargo run -- stats` and the sync panel)
SELECT COUNT(1),
       SUM(price_min IS NOT NULL),
       AVG(price_min),
       MIN(price_min), MAX(price_min) FROM item;
```

```sql
-- items in the watchlist whose detail record is missing or older than 6 h
SELECT id FROM watch
  WHERE detail_synced_at IS NULL OR detail_synced_at < :six_hours_ago;
```

Rules: values are bound as parameters (never interpolated), user text is escaped for
`LIKE` (`%`, `_`, `\`), and every query carries an `ESCAPE '\'`.
