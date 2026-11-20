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
can always attach; `open_db` and `open_db_at` are the two doors into that helper.

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
  unique_flag TEXT,                 -- the API field is `unique`; SQLite reserves that word
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
  price_min INTEGER,                -- copper, cheapest AH buy        <- the "gold" column
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
  status TEXT,                      -- idle | complete | stopped | ceiling | partial | cancelled
  page INTEGER,                     -- absolute page cursor: last page fetched; a stopped
                                    -- or cancelled list run resumes at page + 1 (v0.3.0)
  done INTEGER,                     -- records handled during the most recent run
  total INTEGER,                    -- catalogue size the API itself reported
  message TEXT, updated_at INTEGER, error TEXT
);
```

Stage-1 (the `?page=` list crawl) fills the summary columns and `list_etag`; stage-2 (`/item/<id>`)
fills the price/stat columns, `icon_url`, `raw_json`, `detail_etag`. A NULL price column means
"not fetched yet", which is distinct from "fetched and not on the AH" — the difference is visible
in the UI and in `cargo run -- stats`.

Reuse bookkeeping (v0.3.0): `raw_json` is the verbatim last detail body, which is what
a live lookup answers when the API returns `304` for a stored `detail_etag` — no fresh
payload, no re-parse. List pages have no row to hang an ETag on, so `meta` holds the
revalidation state per page under a key convention:

| meta key | value | why |
| -------- | ----- | --- |
| `list-page-<n>-etag` | last ETag of that page | sent back as `If-None-Match` on every re-crawl |
| `list-page-<n>-more` | `1`/`0` — the page's `has_more` | a `304` has no body, so the crawl needs the hint to know whether to walk on |
| `list-page-<n>-count` | the page's `count` | the crawl keeps `total` honest across `304`s |

Targeted detail runs (v0.5.0) keep the same rules and add two readers: `item ID --cached`
answers the card from `raw_json` with **no request at all** (`not-found` when the row has no
stored copy), and `sync --stage detail --ids …` / `--watch` visit those ids instead of the
pending queue, sending the stored `detail_etag` so a second pass costs requests but no writes
(`written 0, unchanged 2`). `detail_synced_at` is what the card prints as "fetched …", which
is how a user tells today's prices from last month's without leaving the app.

Indexed for the queries the UI actually issues (SQLite's own index syntax, `ON` not `IN`):

```sql
CREATE INDEX item__name_lc ON item (name_lc);
CREATE INDEX item__rank ON item (quality_rank, item_level);
CREATE INDEX item__price ON item (price_min);
CREATE INDEX item__detail ON item (detail_synced_at);
```

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

```sql
-- the count behind the same filter, so "showing 40 of 1250" cannot drift
SELECT COUNT(1) FROM item WHERE name_lc LIKE ? ESCAPE '#' AND ...;
```

```sql
-- coverage + price analytics (drives `cargo run -- stats` and the sync panel).
-- the SUM() columns are CAST: rusqlite reads rows by declared column type, and a bare
-- expression column has none, so an unCAST aggregate is a type error, not a NULL.
SELECT COUNT(1),
       CAST(SUM(name IS NOT NULL) AS INTEGER),
       CAST(SUM(price_min IS NOT NULL) AS INTEGER),
       CAST(SUM(detail_synced_at IS NOT NULL) AS INTEGER),
       MIN(price_min), MAX(price_min), AVG(price_min) FROM item;
```

```sql
-- the ids the detail stage still has to fetch, oldest id first
SELECT id FROM item WHERE price_min IS NULL OR detail_synced_at IS NULL ORDER BY id ASC LIMIT ?;
```

```sql
-- items in the watchlist whose detail record is missing or older than 6 h
SELECT id FROM watch
   WHERE detail_synced_at IS NULL OR detail_synced_at < ?;
```

Rules: filter values are bound as parameters (never interpolated), user text is escaped for
`LIKE` (`#`, `%`, `_`) and every `LIKE` carries `ESCAPE '#'`, and NULL prices sort last rather
than being treated as cheap — a gold tool must not claim an unpriced item is free.
