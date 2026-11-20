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

**Two stale shapes are migrated instead of refused** (v0.6.0): the file on disk saying
`schema_version = 1` (the user's own catalogue, 19 258 rows, still never touched by a
refusal) or `2` differs from this build by additive steps only, so `check_schema` chains
them: v1 gets v0.5.3's `ALTER TABLE item ADD COLUMN detail_missing_at INTEGER` first,
then both v1 and v2 get the v0.6.0 batch — three `ALTER`s appending the icon columns and
a `CREATE` for `price_history` plus its index — and the `meta` stamp moves to `3` in the
same call. Refusing either file would throw away a 19 258-row catalogue on the excuse of
a schema bump. A migrated file's **column order differs from a fresh file's** (ALTER
appends, SCHEMA_SQL lists the new columns mid-table) — every reader in the code names
columns explicitly and nothing reads positions. A file **newer** than this build is still
refused — that direction cannot be guessed at. See
[ADR 0006](./DECISIONS/0006-checked-missing-third-state.md) and
[ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md).

## Schema (`schema_version = 3`)

```sql
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);

CREATE TABLE item (
  id INTEGER PRIMARY KEY,            -- TheWoWDB item id (sparse: 25 .. 288340)
  name TEXT, name_lc TEXT,           -- (drift corrected 2026-11-20: the file on disk
  quality TEXT, quality_rank INTEGER,--   and SCHEMA_SQL both say plain `TEXT` — the
  item_level INTEGER, required_level INTEGER,       -- `NOT NULL` this file used to
  item_type TEXT, item_subtype TEXT, slot TEXT,     --   claim was aspirational, never
  item_set TEXT, unique_flag TEXT, flavor TEXT,     --   enforced; same for the FK-ish
  icon_url TEXT,                                     --   `id` wording)
  sockets TEXT, socket_bonus TEXT, classes TEXT, stats TEXT, effects TEXT,
  faction_prices TEXT,
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
  detail_missing_at INTEGER,          -- unix seconds when upstream answered 404 (v0.5.3)
  icon_blob BLOB, icon_at INTEGER, icon_missing_at INTEGER, -- the icon's three states (v0.6.0)
  raw_json TEXT                     -- verbatim detail payload, forward-compatible
);

CREATE TABLE watch (id INTEGER PRIMARY KEY, added_at INTEGER, note TEXT);

CREATE TABLE sync_state (
  kind TEXT PRIMARY KEY,            -- "list" | "detail" | "icons"
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
CREATE INDEX item__name_lc ON item (name_lc);              -- v0.2.0
CREATE INDEX price_history__item ON price_history (item_id); -- v0.6.0 fresh CREATE and the v1/v2 -> v3 migration
CREATE INDEX item__detail ON item (detail_synced_at);      -- v0.5.3
CREATE INDEX item__rank ON item (quality_rank, item_level);
CREATE INDEX item__price ON item (price_min);
```

Stage-1 (the `?page=` list crawl) fills the summary columns and `list_etag`; stage-2 (`/item/<id>`)
fills the price/stat columns, `icon_url`, `raw_json`, `detail_etag`. A NULL price column means
"not fetched yet", which is distinct from "fetched and not on the AH" — the difference is visible
in the UI and in `cargo run -- stats`.

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
-- the SUM() columns are CAST (rusqlite reads rows by declared column type, and a bare
-- expression column has none, so an unCAST aggregate is a type error, not a NULL) and
-- IFNULL (an empty table makes SUM() NULL, which a INTEGER column cannot bind).
-- The icon pair is the v0.6.0 addition: copied + checked-as-gone, same read.
SELECT COUNT(1),
       CAST(IFNULL(SUM(name IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(price_min IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(detail_synced_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(detail_missing_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(icon_at IS NOT NULL), 0) AS INTEGER),
       CAST(IFNULL(SUM(icon_missing_at IS NOT NULL), 0) AS INTEGER),
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
