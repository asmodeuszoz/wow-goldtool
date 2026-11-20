# ADR 0007 — price history samples what a fresh write saw, not every visit

**Status:** adopted (v0.6.0)
**Related:** `src/db.rs` (`price_history`, `SQL_PRICE_SAMPLE`, `SQL_HISTORY_OF`),
`src/sync.rs` (`upsert_detail`), `src/stats.rs` (`SQL_COUNT_SAMPLES`, `Coverage.history`),
`src/cli.rs` (`CARD_HISTORY_LIMIT`, card `history`), `ui/app.js` (trend bars),
docs/DATABASE.md, docs/API.md, [ADR 0006](./0006-checked-missing-third-state.md)

## The question that has no answer in the current schema

The catalogue stores **one** price per item: `price_min`, `price_market`, `price_available`
… are columns on the `item` row, overwritten by the next detail write. "What does this cost?"
is answerable; "is it getting cheaper?" is not — a card that draws a trend needs more than one
observation per item, and the app has never kept any.

Nothing upstream would give it either. `docs/API.md` records that a `forever/item` answer
carries the **current** auction-house state plus the upstream's own `price_updated_at` stamp,
and a record that did not change answers `304` with **no body at all** (measured since v0.3.0,
exercised by the fixture and the mirror's `written 0, unchanged 2` steps): a `304` says
"nothing moved", not "here is a cheaper price". So any growth of the ledger must come from
**our** fresh observations, and the question is what counts as one.

The sampling rule is also a cost question: the detail stage costs ≈ one request per record
(≈ 2.7 h at 2 req/s for the 19 258-row catalogue on the user's disk), and every rule below
moves what fraction of those visits may append a row.

## Options considered

* **Sample on every detail visit, `304`s included.** Rejected: a revalidation says nothing
  about the price — appending under it clones one observation across every later run, and the
  card's "trend" becomes a function of how often the user clicked Sync, not of the market. The
  mirror's reuse steps (`written 0, unchanged N`) would grow the ledger by N rows of identical
  numbers, which is the loudest lie this table could tell.
* **A dedicated sampling pass** (re-ask everything once a day just to sample). Rejected for
  now: it doubles the one request per record the write already paid for, and nothing measured
  says the upstream tolerates a sustained second crawl — the rate budget of a stage is a fact
  the API's own `Retry-After` answers already pace (docs/API.md), and a second full pass spends
  the budget on records the write visit could not even refresh.
* **Sample only when the payload's numbers MOVED** (compare against the row before the write).
  Rejected: invents a "move" rule the upstream never states, and it hides the one honest
  signal — a record re-fetched with identical prices IS a repeated observation (prices rarely
  move hour to hour, and a rule that never writes them draws no flat stretches, i.e. it lies
  about flat markets being flat).
* **A daemon or timer binary that samples hourly.** Rejected by the shape of the whole project
  (ADR 0004: the headless CLI IS the integration path; no second binary has ever shipped, and
  v0.1.0's roadmap has no such slot). A sample belongs to the write that saw it.
* **No second table (a JSON blob of samples inside the `item` row).** Rejected: an array
  crammed into a column cannot be read as "the newest N samples of one item" without
  re-parsing every row, the ledger grows with every crawl, and `docs/DATABASE.md`'s whole
  point is that the UI reads SQLite, not hand-picked text. SQLite is a database; a second
  table IS the answer.
* **Fold into v0.5.x patches.** Rejected: v0.5.x's whole job was making the crawl terminate
  honestly; the sampling rule is a storage decision that rides on a finished crawl.

## Decision

A second table, and one row per **fresh detail write**:

```sql
CREATE TABLE price_history (
  seq INTEGER PRIMARY KEY,          -- auto-assigned: the INSERT omits it
  item_id INTEGER, sampled_at INTEGER, price_min INTEGER, price_market INTEGER
);
CREATE INDEX price_history__item ON price_history (item_id);
```

* `upsert_detail` appends a sample **after** the item upsert ran — same stage, same visit —
  and only when the payload carried prices (`price_min` non-NULL): a record not on the AH
  samples nothing, and a blank chart point is a lie the card cannot draw honestly.
* A `304` revalidation, a list write, a `--cached` card read, a check-mark and a re-insert
  write **nothing** here. The ledger counts fresh observations, not requests — which is what
  makes the rule auditable: the mirror's reuse steps must leave the count unchanged, and a
  second crawl session must grow it exactly by the fresh writes it made.
* The `item_id` "FK" was NOT how the guard works: SQLite 3.54.1 does not enforce FK clauses
  on `INSERT` (probe below) — we ship NO clause at all, and the guard is the Rust order
  inside `upsert_detail` (the item row exists before the sample is written). Inventing an
  unenforced clause would decorate a contract the runtime does not provide.
* `seq` is a plain auto-`PRIMARY KEY` (not `item_id`-derived): two crawls in the same second
  cannot collide, and the card's window orders by `seq` (`SQL_HISTORY_OF`: `ORDER BY sampled_at
  DESC, seq DESC LIMIT ?`) — the index answers it — then reverses for the chart so time runs
  left to right.
* The card draws a **window** (`CARD_HISTORY_LIMIT = 40`, `src/cli.rs`), the DB keeps the
  whole ledger — a card is a card, and "how deep does the ledger go" stays a `stats` question
  (`SQL_COUNT_SAMPLES`, `Coverage.history` printed by `status`, `stats`, the sync panel grid
  and the item-card note).
* No dedup rule yet ("one sample per day per item"): that invents a clock granularity the
  upstream never promised, and the honest write rule (one per fresh write) makes duplicates
  a QUERY problem if they ever matter, not a write-time rule to guess at.

## Grounding, not invention

```
$ cargo run -- sync --stage detail --fixture http://127.0.0.1:8811     # fresh DB, budget 8
detail stage: requests 6, items 4, written 4, unchanged 0, missing 1, rate_limited 1, complete true
$ sqlite3 -json "$DB" "SELECT seq,item_id FROM price_history ORDER BY seq"
[{"seq":1,"item_id":25},{"seq":2,"item_id":40},{"seq":3,"item_id":60},{"seq":4,"item_id":12809}]
$ cargo run -- sync --stage detail --ids 25,40 --fixture http://127.0.0.1:8811   # same process
detail stage: requests 2, items 2, written 0, unchanged 2
$ sqlite3 -json "$DB" "SELECT COUNT(1) FROM price_history"
[{"COUNT(1)":4}]                       # the reuse step wrote NOTHING into the ledger
$ cargo run -- sync --stage detail --ids 25 --fixture http://127.0.0.1:8812      # NEW process
detail stage: requests 2, items 1, written 1, rate_limited 1                    # stored ETag is stale for a new window: 200
$ sqlite3 -json "$DB" "SELECT seq,item_id,price_min FROM price_history ORDER BY seq"
… {"seq":5,"item_id":25,"price_min":15311}   # grew by exactly ONE row; the older 20271 sample stays
$ sqlite3 -json "$DB" "SELECT COUNT(1) FROM item WHERE raw_json IS NOT NULL AND price_min IS NULL"
[{"COUNT(1)":1}]                       # the scratch catalogue really has a record with no
                                       # price column filled: the sample guard's case is live
```

The FK probe (SQLite 3.54.1, 2026-11-20, scratch file):

```
$ sqlite3 fkprobe.sqlite3
CREATE TABLE item (id INTEGER PRIMARY KEY, name TEXT);
CREATE TABLE price_history (seq INTEGER PRIMARY KEY, item_id INTEGER REFERENCES item,
                            sampled_at INTEGER, price_min INTEGER);
INSERT INTO item (id, name) VALUES (25, 'Worn Shortsword');
INSERT INTO price_history (item_id, sampled_at, price_min) VALUES (999, 1750000000, 100);
SELECT COUNT(1) FROM price_history;
SELECT * FROM price_history;
1
1|999|1750000000|100          -- the FK clause was ACCEPTED and NOT ENFORCED: the sample
                              # for a non-existent item id landed; and seq was auto-assigned
$ sqlite3 --version
3.54.1 2026-10-16 ...
```

## Audit (every claim above executed, not asserted)

| probe | expectation | observed |
| ----- | ----------- | -------- |
| migration of a scratch v2 catalogue (the 19 257-row copy) | rows survive, stamp moves, `price_history` empty, cursors intact | `19 257` rows, `schema_version 2 → 3`, `price_history` 0 rows, `sync_state` list row `complete … page 193`, detail row `partial … 23 of 75` kept; second `status` says 3, never re-migrates |
| migration of the user's own v1 file (copied, hashed read-only before/after) | the chain lands in one step, file untouched | v1 → `schema_version 3`, `19 258` rows, `7` detailed, `detail_missing` 0, `price_history` 0, `sync_state` list row `stopped … page 170` intact, index set `item__detail item__name_lc item__price item__rank price_history__item`; hash `7442814b…` and mtime identical before/after |
| CI mirror, fresh DB, fixture process A | the ledger counts fresh writes, never revalidations | detail pending `requests 6, written 4, missing 1` → samples seq 1..4; the three reuse steps (`--ids`, again, `--watch`) each `written 0, unchanged 2` → count stayed 4 |
| second fixture process B (new observation window) | exactly one new row for the fresh write | `detail ids 25`: `requests 2, written 1, rate_limited 1` → `price_history` 4 → **5**; the card answers `history` with the old 20271 and the new 15311 sample |
| FK probe (SQLite 3.54.1) | an FK clause would decorate an unenforced contract | `item_id INTEGER REFERENCES item` accepted; `INSERT INTO price_history` with `item_id = 999` and no such row **succeeded**; `seq` auto-assigned 1 → 2 |
| empty-price payloads | a record not on the AH never writes a blank sample | the scratch catalogue answers `1` detailed row with `price_min` NULL of 29 — the guard's case is live, not hypothetical |
| GUI pull-all audit (fixture, boot probe) | the card renders what the crawl wrote | three `sync` verbs with ticker `status` polls interleaved; DB ends `5` items, `4` detailed, `1` checked-gone, `price_history` 4 rows, both queues pending `0`, `sync_state` all three rows `complete` |
| exit-code audit | the ledger's door stays on the contract | `status 0 · search 0 · cached card 0 · usage 2 · 2 · 2 · detail-dead 3 · icons-dead 3 · list-dead 3 · not-found 4 · not-found-cached 4` |
| CI mirror (committed sequence, icon stage inside) | green through the card and stats steps | every step exit 0; `stats` prints `icons 3, icon_missing 1, history 4` |
