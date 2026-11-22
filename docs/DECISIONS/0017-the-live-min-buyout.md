# ADR 0017 — the live min buyout: the FIRST producer for ADR 0016's reserved homes

**Status:** adopted (v0.9.6)
**Related:** [ADR 0016](./0016-the-price-homes.md) (the reservation this now fills —
`price_min_now` gets its producer), `src/db.rs` (`SCHEMA_VERSION` 7, `price_now_at`/`price_now_missing_at`,
`SQL_MIGRATE_TO_V7`, `SQL_PENDING_PRICE`-side twins `SQL_SET_PRICE_NOW`/`SQL_MARK_PRICE_NOW_MISSING`/
`SQL_PRICE_MIN_NOW_OF`), `src/sync.rs` (`sync_prices`, the FIFTH stage), `src/stats.rs` (`SQL_PENDING_PRICE`,
`price_pending_ids`), `src/models.rs` (`TooltipFields`), `src/cli.rs` (stage arm + the card's live pair),
`src/sync.rs`↔`ui/app.js` (the card's live money row), [ADR 0010](./0010-manual-layer-and-tooltip-copy.md)
(stage discipline + the pending-rule pattern), [ADR 0007](./0007-price-sampling.md) (whose sample stays
where it is), [ADR 0015](./0015-money-words.md) (money rides whatever copper), [ADR 0011](./0011-ci-asserts-only-what-is-true.md),
[ADR 0004](./0004-headless-cli-mode.md), [ADR 0006](./0006-checked-missing-third-state.md), `docs/API.md`

## The observation that forced it

ADR 0016 carved `price_min_now` and REJECTED writing it because no door had ever answered those
words. That was true of the DETAIL door; it stopped being true the moment the TOOLTIP door was
re-asked. Measured live 2026-11-21 (UA `wow-goldtool 0.9.5`, the tool's own UA):

| probe | the live door answered |
| ----- | ---------------------- |
| `tooltip/forever/item/25`, twice in one day | `fields.price` = `{available 5→7, market 1200, min 149, realms 3, region us, scope "WoW Forever Normal Alliance", updated_at 16:25:03Z → 22:25:10Z}` — the SAME door, its OWN `updated_at` moving, `available` moving 5→7: this door answers NOW-ness |
| `tooltip/…/202`, `237`, `763` — rows whose STORED copy had `price_min` NULL | `min 5000` / `3500` / `4425` (with market/available/realms/scope) — the live min moves APART from `price_min` ON THE SAME RECORD, evidence for ADR 0016's two-columns rule, not an assertion |
| `tooltip/…/200` — a row the DB stored with a copy and NO price | `fields.price` is **literal NULL** while its own copy says "Lowest buyouts": the door's OWN third state — "we looked, the AH offers NOTHING right now" |
| upstream's OWN docs (`thewowdb.com/developers/`) | exactly FOUR doors exist (list, record, search, tooltip), and the caching section states tooltips answer ~5 min "because they carry a live Auction House price" |
| `detail/200`, `detail/237` beside them | `price` NULL / present (`min 3500` @ `22:25:08`) — the detail door carries the SAME block but at ITS cache rhythm; the refresh rides the tooltip door, the one the crawl already owns |

So the value the user asked for ("current auction-house min buyout") HAS a measured door: the
tooltip door itself, re-asked. NULL in `price_min_now` could no longer mean only "no producer
exists" — with a producer landed, the column now carries the fetched/checked-empty split ADR
0006 started (its stamp pair, like the icon pair and the tooltip pair).

## The decision, and what was rejected

**Landed:** `sync --stage price`, a FIFTH crawl stage mirroring every crawl discipline:

* **Pending rule** (`SQL_PENDING_PRICE`, the twin of the tooltip/icon rules): tooltip COPY
  EXISTS (`tooltip_at NOT NULL` — ADR 0010's measured reason again: only a copied record has
  something to re-ask, and a 404 here would be "the door lost a copy", never an AH fact) AND
  `price_min_now` NULL AND `price_now_missing_at` NULL, no manual rows, no tombstones, `id ASC`:
  **one request per copied record, EVER** (measured: second press asks ZERO, `complete true`).
* **Third state marks** `price_now_missing_at` (measured: literal-null `fields.price`, live
  id 200 and the fixture's own twin on row 40); the door's 404 answers the SAME state for
  parity with its twin stage, marked and walked past — never measured on a copied row, said so.
* **Stamps:** `price_now_at` is OUR ask time. The door's own `updated_at` TEXT is deliberately
  NOT stored: two stamps of one fact, the door's word ages toward the NEXT ask while ours says
  when WE looked (recorded decision; the TEXT would need a parser for an ISO shape the ledger
  never carries).
* **NO If-None-Match**, even though the tooltip stage stored an ETag: the `fields.price` this
  ask needs was NEVER parsed before (the copy stage reads `html` only), so a `304` would answer
  "unchanged since a read that never saw this field" — the door's own ~5-min server cache is
  what keeps the pass polite.
* **Fresh writes sample NOTHING:** ADR 0007's price sample stays on the DETAIL write — the
  sampling contract is that release's, and the trend that would eat denser samples is later
  work. Re-asking must not quietly mint history points.
* **The mirror, not hand-parsing:** `Tooltip.fields` becomes `Option<TooltipFields>` with ONLY
  `price: Option<Price>` typed. The first build's FIFTH warning named `vendor_sell` unread in
  the new mirror — trimmed the same breath (a mirror entry WITHOUT a reader is dead code, the
  house rule not swept); serde ignores the other measured 25 keys, so v0.8.0's promise "a key
  upstream adds cannot break a build" keeps its exact teeth.
* **The reader:** the card door grows `price_min_now` + `price_min_now_money` as top-level
  SIBELINGS (ADR 0015's money rides the live copper; `price_available`/`price_realms` stay
  auction COUNTS with no money) — the card door is **NOT byte-identical this release: +2 keys**,
  this release's documented wire delta (the v0.9.5 byte-identity claim stays scoped to that
  release's hop). `price_now_at` stays OFF the wire: its only readers today are the pending-rule
  SQL and the write, so the stamp grows its own fragment when a reader wants the WHEN (recorded).
  The UI grows ONE card row (money cell, exact copper in the hover, reworded note). The search
  and stats doors stay UNCHANGED (the sort column stays `price_min`; the live-sort knob is later
  work), and `gui.rs` was READ before the claim: it builds the window and draws no card, so the
  GUI gains nothing.

**Rejected:** storing the door's `updated_at` (above); conditional asks (above); re-asking the
DETAIL door instead (the tooltip door is the one the crawl already owns, and ADR 0010's pending
rule keeps its reason); sampling on refresh writes (above); overloading `price_min` with the
live value (ADR 0016's rule restated); inventing producers for the OTHER two homes — measured
NEGATIVES, recorded where they stand:

* **`vendor_buy` has no door and no key:** the 4 602 detail copies in the user's file answer the
  SAME 28-key set with `vendor_sell` only; the developers page lists exactly four doors, none
  carries a vendor buy price. The ONLY upstream signal that an item "can be bought" is its OWN
  flavor prose — `"Available from enchanting vendors."` on exactly ONE copy (id 247786, Mote of
  Magic): prose is not a price, and a state from prose would be an interpretation, not a fetch.
* **`price_trend_normal` has no door either:** ZERO trend words in 19 258 rows (all 20 "volume"
  hits are `Volume I/II/III` book titles), no trend door upstream. ADR 0011's rule: the plan for
  these two stays with a door that must one day answer, and it has not been invented.

## What the mechanism measured (fixture, fresh ports, fresh DBs)

`test(fixture)` grew the tooltip door's `fields.price` to the MEASURED shapes (25/60 answer a
full block whose `min` MOVES with the fixture's RUN against their stored detail price; 40 answers
the whole `fields.price` literal null; 127555 keeps `fields: {}` — its row never gets a detail
stamp so the queue never points at it). Two fixture commits bent and straightened honestly: the
first RUN-order fix moved only the comment (the rerun died with the same `NameError`), the second
carried the actual move — recorded, not swept, tags stayed put throughout.

| measurement | answer |
| ----------- | ------ |
| hop 6 → 7, scratch schema-6 copy | `ALTER` pair lands, stamps NULL on all rows, rows/copper intact, SECOND open byte-identical; fresh DB BORN at 7 with the pair BEFORE `raw_json`; pinned v0.8.1 refuses the 7-file honest (exit 4 "wants 4", file untouched) |
| stage, fixture rows 25/40/60 pending (12809 tooltip-404 → not pending; 127555 no detail stamp) | rc 0, requests 3, written 2 at live 16 365/15 365 against stored 17 365 (live MOVES apart), row 40 `price_now_missing_at` stamped, value NULL — nothing invented |
| the report's own lie, found not swept | first run STAMPED row 40 but answered `missing:0`: the stamp existed, the counter did not move, the report lied about the third state — `fix(sync)` counts the null arm, re-measured `missing:1` + its note |
| second press | requests 0, `complete true`, exit 0 — once-per-record holds end to end |
| card door old-vs-new binary, SAME DB | added keys EXACTLY `price_min_now`/`price_min_now_money`, zero drift elsewhere: live 16 190 ↔ `1g 61s 90c` beside stored 17 190 ↔ `1g 71s 90c` |
| CI mirror replay from the committed block | 33/33 steps rc 0 + plant rc 0 on fresh port 9207: price press 1 requests 3 / written 2 (16 175/15 175 vs stored 17 175) / missing 1 + note; press 2 zero asks; the card step reads the pair (`16175` beside `1g 61s 75c`), the manual card answers null/null; `vendor_buy`/`price_trend_normal` NULL on EVERY row |

## Chain

`9f794e4` feat(db,cli) (hop 6→7 + the WHOLE stage: pending rule, write, mark, mirror, dispatch —
the bent "one logical change" rule here: the planned separate `feat(db)` hop commit was never made,
said so) → `55f58f8` test(fixture) (the door answers `fields.price`, shapes measured) →
`d3d004a` test(fixture) (the first fix moved only words — recorded) → `5c77769` test(fixture)
(the move, and the file compiles) → `1a281e3` fix(sync) (the third state COUNTS) →
`4cfd7a2` feat(db,cli) (card door reads the pair back, +2 keys; getter+caller same commit) →
`b1b2c8f` feat(ui) (the live money row) → `51121c9` ci (31 → 33 steps + plant one, replayed green
BEFORE committing) → `b920e7d` chore (banner 0.9.6) → this record carries the tag.

## CI row

Joining ADR 0016's ladder: the v0.9.5 verification fold's own push ran **44962724652** (head
`8e79e97…`, the CI-history fold): **success, 5/5 in 8 min 37 s** (23:37:44Z → 23:46:21Z), attempt
1, no supersession — folded FORWARD here, never chased. v0.9.6's chain+tag push `8e79e97..4ced794`
(head `4ced794…` — the TAGGED tree) ran **44968797491**: **success, 5/5 in 9 min 15 s** (00:53:13Z
→ 01:02:28Z — quality 1 m 50 s · test 4 m 40 s · linux 5 m 24 s · macos 6 m 25 s · windows 9 m 12
s), EXACTLY one run id for that head, no CANCELLED sibling, no rerun submitted (`.attempt` reads
null on this API, so the ladder matches by `id`/`created_at`/`head_sha` — the pattern it already
keeps); it was IN FLIGHT when the verification fold committed, and the fold waited, honestly,
rather than cancelling it. Its own push — the verification fold's, `4ced794..0c450ba`
(head `0c450ba…`) — ran **44969661861**: **success, 5/5 in 9 min 17 s** (01:04:00Z → 01:13:17Z —
quality 1 m 40 s · test 4 m 37 s · linux 5 m 14 s · macos 6 m 39 s · windows 9 m 14 s), one run id,
no supersession; the fold waited (staying LOCAL while the branch flew) and GREW with this verdict
before pushing, so only the CI fold's OWN push lands FORWARD in the next record. Tags stay put
(`v0.9.5` → `23ceb4e`, `v0.9.6` → `4ced794`).
