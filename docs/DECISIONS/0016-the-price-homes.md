# ADR 0016 — the price homes: four columns own four questions, carved now, filled later

**Status:** adopted (v0.9.5)
**Related:** `src/db.rs` (`SCHEMA_VERSION`, `SCHEMA_SQL`, `SQL_MIGRATE_TO_V6`, `check_schema`),
[ADR 0015](./0015-the-money-words.md) (money reaches whatever copper these columns will
carry), [ADR 0011](./0011-ci-asserts-only-what-is-true.md), [ADR 0007](./0007-price-sampling.md),
[ADR 0004](./0004-headless-cli-mode.md), [ADR 0010](./0010-manual-layer-and-tooltip-copy.md),
`docs/DATABASE.md`, `docs/DEVELOPMENT.md`

## The observation that forced it

The user TESTED the app: "the prices shown right now are not the vendor prices." True by
design — until today the answer carries ONE price per row (`price_min`, the stored copy's
cheapest AUCTION-HOUSE buy) plus the vendor SELL, while the game's honest shape has FOUR
separate numbers per item:

```
what a VENDOR SELLS            vendor_sell        the column EXISTS (4 231 rows on the
                                                   user's file, proven copper against the
                                                   copies' own "Sell Price:" words, ADR 0015)
what a vendor BUYS             vendor_buy         ONLY when the item can be bought at all —
                                                   "cannot be bought" is a STATE, not a 0
the CURRENT AH min buyout      price_min_now      today's price_min is the STORED copy's —
                                                   an aging payload's price is NOT a live price
the historic "normal" price    price_trend_normal the live trend answers it from past data;
                                                   "normal" ≠ "min": two questions, two numbers
```

The user said explicitly: "how we'll get to those values can be ignored for now, but the
columns should be allocated now." So the milestone is a RESERVATION: the schema hop carves
the three missing homes; producers, readers and doors are later releases.

## Options considered

* **Fetch the new values now.** Rejected twice over: the user deferred it, AND house rule
  — no door has EVER answered "vendor buy", "current min" or "trend normal" (docs/API.md
  measured keys, nothing near them), so writing a producer now would INVENT an upstream
  shape. The three column names are OURS by the same rule.
* **Compute live values at answer time instead of storing columns.** Rejected: unlike money
  (ADR 0015, a pure rendering of copper that already exists), these are OBSERVATIONS that
  age — "what did the AH offer at 14:05" cannot be re-derived from what is on disk, so the
  home IS the DB.
* **Overload `price_min` for the live value.** Rejected: one column answering two questions
  (stored-copy price vs live price) over time is two truths in one word; `price_min` stays
  what the payload said, `price_min_now` is what the AH says NOW.
* **Carve NOTHING and document a wish.** Rejected by the user's own words: "the columns
  should be allocated now."
* **Schema hop 5 → 6, three additive ALTERs, NULL = "no producer yet".** Chosen — the same
  honest migration chain v0.5.3/v0.6.0/v0.8.0/v0.9.0 grew, one step longer.

## Decision

* `SCHEMA_VERSION` moves 5 → 6; `SCHEMA_SQL` grows `vendor_buy INTEGER, price_min_now INTEGER,
  price_trend_normal INTEGER` BEFORE `raw_json`; `SQL_MIGRATE_TO_V6` is the same three ALTERs
  for stale files, and `check_schema` chains EVERY stale shape (v1..v5) up to 6 — the
  staircase v0.9.0's comment promised since its own hop.
* NULL in the trio means **NO PRODUCER HAS EVER WRITTEN IT** — deliberately NOT yet the
  "fetched / fetched-and-not-there / never-looked" triple the other price columns split;
  the third state arrives WITH the first producer, never before it (inventing a checked
  state for a door that does not exist would be a plan, not a measurement).
* The trio has NO reader and NO writer this release: every door answers BYTE-IDENTICAL
  across the hop (measured pairs below), the UI stays as it is, and money (ADR 0015) waits
  for copper — `money_of(NULL)` answers `null`, and zero rows carry values, so there is
  zero money to answer NOW.
* `price_available`/`price_realms` stay COUNTS (ADR 0015's rule); the trio joins the
  INTEGER-copper contract of docs/DATABASE.md the moment producers write them.
* No index for the trio: nothing queries them yet — a guessed index is ADR 0011's
  rejected plan.
* CI gains NO step (the trio sends no ask and moves no byte through the block); the smoke
  file is now BORN at 6, and the ALTER chain does NOT run in CI — the post-tag audits
  measure it on scratch copies (honest split, ADR 0011's rule).

## Consequences, measured 2026-11-21 and not smoothed

* the hop is the first release since v0.9.0 whose schema moves — v0.9.4's "no column, no
  hop" claim now reads as THAT release's claim, and the ci.yml comment says so explicitly;
* **the user's own file migrated the moment a door opened it**: 19 258 rows / 4 231 vendor_sell
  / 4 285 history samples intact, `schema_version` 5 → 6, trio NULL on ALL rows, and the
  SECOND open re-migrated NOTHING (second `status` byte-identical);
* the stale direction stays refused: the pinned v0.8.1 binary answers exit 4 —
  `database on disk uses schema 6, this build wants 4` — over the migrated file, untouched;
* a transient build fact kept honest in code, not swept: none — the new `SQL_MIGRATE_TO_V6`
  constant had its consumer (`check_schema`) in the SAME commit, so unlike v0.9.4's
  `money_of_avg` dead-code window, the builds stayed at the FOUR named warnings throughout;
* naming residue: `price_trend_normal`'s word "normal" is the user's own word for the live
  trend's historic price — if the future producer turns out to be a service with its own
  vocabulary, the COLUMN keeps our name and docs/API.md gains only the producer's measured
  response keys, never ours (ADR 0014/0015's shape rule).

## Audit

| probe | expectation | observed |
| ----- | ----------- | -------- |
| fresh create | trio present, BEFORE `raw_json` | `.schema item` on a fresh CI-born file: `vendor_buy INTEGER, price_min_now INTEGER, price_trend_normal INTEGER` before `raw_json`, `schema_version|6` |
| scratch 5 → 6 (schema-5 mirror copy, first door opens it) | additive, intact, idempotent | first `status` rc=0 → `schema_version|6`; rows 6 / 4 priced / 4 vendor_sell / samples intact, copper unchanged (25 keeps stated `vendor_sell 606`); trio NULL on all; SECOND `status` BYTE-IDENTICAL, meta stays 6 |
| the user's own file | same breath, honest | 5 → 6 on first open; **19 258 / 4 231 / 4 285 intact**, `COUNT(1) WHERE vendor_buy IS NOT NULL OR price_min_now IS NOT NULL OR price_trend_normal IS NOT NULL` = **0**; money doors re-answer (`20042 → 2g 42c`, no money inside `item`) |
| door pairs, v0.9.4 binary (tree `4875222`) vs this one, two copies of ONE fixture DB | the reservation moves ZERO bytes | **12 of 13 BYTE-IDENTICAL** at identical byte counts (search 1 941 / 763 / 755 / 3 363, stats 1 533 / 1 850 / 17 462, cards 4 336 / 1 103 / 3 868 / 4 022, refresh-twins 223); `status` differs ONLY in its `db.path` echo — pinned, its JSON is IDENTICAL (the paths cannot match: the old binary refuses the migrated file) |
| stale chain | v1..v5 all reach 6 in one open | arms 1–5 each end with `SQL_MIGRATE_TO_V6` + `meta_set 6` (arms re-verified after the edit; v5's own arm is new this release) |
| newer file | honest refusal, file intact | pinned v0.8.1 binary on the schema-6 file: exit **4**, `database on disk uses schema 6, this build wants 4`, file untouched after |
| post-tag mirror replay, driver verified EQUAL to the committed block (port-stripped compare) | homes move nothing through the block | **31/31 rc 0**, plant rc=0, GETs **31 = 200×15 / 304×9 / 404×3 / 429×4** — identical to the v0.9.3/v0.9.4 replays; tail `6|4|4|1|1|0`, icons `3|1`, twins `4|4|4|0`, made row ZERO asks, fixture untouched; schema prints **6** (the sed driver's OWN header carries v0.9.4's stale "must stay 5" claim — the ANSWER is 6, the committed ci.yml comment names the truth); trio NULL count **0** |
| pre-hop replay: 31 steps with the v0.9.4 binary (tree `4875222`, scratch worktree), then the v0.9.5 build reads the file (fresh port 9191) | NO refresh door: old files reach the reservation on the NEXT read | 31/31 green, same GET tallies, file stays 5; over two copies, 9 read doors: **8 BYTE-IDENTICAL**, `status` identical with its `db.path` echo pinned (the 1-byte delta IS the path strings); stamps equal row-by-row, trio NULL on all, `item 25 --cached` still `4 336` with `606 → 6s 6c` |
| exit-code audit re-run post-tag (fresh ports 9192..9199 + 9200, fresh DBs) | the map cannot move | **90/90 rc pairs IDENTICAL** to the v0.9.4 audit — money doors answer 2/2/0/0 still (NO flag for money OR the trio), totals `0` ×59 · `2` ×14 · `3` ×6 · `4` ×11; fixtures A 29 = 200×16/304×3/404×6/429×4, B 2, C 4 — the trio sent no ask |
| the audit driver's OWN first pass | — | door 65 opened a scratch DB ALREADY at 6 (left from an earlier measurement, `rm -f` missing for the pinned window): the pinned binary refused rc **4** — the PRODUCT was perfect, the DRIVER lied about its "fresh file" precondition; fixed with `rm -f` (DB + WAL sidecars) and re-measured: 65 rc=0 writes schema 4 (3 requests = 2 pages + one 429, matching C's 4-GET tally), 66 rc=0 migrates **4 → 5 → 6** in one open (5 items intact, meta 6), 67 rc=4 refusal, file untouched — the PINNED-era chain reaches 6 end to end |
| CI runs | green on the tagged tree | **run 44960088009** (chain+tag push `1bc4e50..23ceb4e`, `head_sha 23ceb4e…` — the TAGGED tree): attempt 1 **CANCELLED** 23:05:59Z → 23:12:07Z by GitHub's supersession when the verification push landed a new head — a run-management fact, not a build failure; rerun endpoint resubmitted the SAME head: **attempt 2 SUCCESS, 5/5 in 6 min 12 s** (jobs 23:14:47Z → 23:20:59Z — quality 1 m 57 s · test 4 m 40 s · linux 5 m 23 s · macos 6 m 10 s · windows 5 m 59 s). **run 44960596846** (verification push `23ceb4e..62a7501`, head `62a7501…`): attempt 1 **CANCELLED** (jobs 23:12:09Z → 23:14:3x–4x, only quality green) — superseded by the rerun above landing on the same branch mid-flight; observed rule recorded where it bit: a rerun or a fresh push cancels the branch's in-flight run, so NOTHING was pushed while runs were underway, the two rerun requests were the only submissions; **attempt 2 (rerun): SUCCESS, 5/5 in 9 min 35 s** (23:22:03Z → 23:31:38Z — quality 1 m 56 s · test 4 m 47 s · linux 5 m 14 s · macos 5 m 48 s · windows 9 m 35 s). The v0.9.4 ladder completes beside them: fold-push run **44957300299: success, 5/5 in 10 min 50 s** (head `1bc4e50…`), joining chain+tag **44954802500: 5/5 in 10 min 36 s** and verification push **44956129051: 5/5 in 10 min 44 s** on top of v0.9.3's **44946961792: 5/5 in 10 min 15 s** — every recorded push through v0.9.5 is green, straight or after an honest resubmission of the SAME head. This record's own commit starts a run that lands FORWARD, never chased; tags stay put (`v0.9.5` → `23ceb4e`). **Folded FORWARD by v0.9.6's record:** run **44962724652** (head `8e79e97…`, the CI-history fold's push): **SUCCESS, 5/5 in 8 min 37 s** (23:37:44Z → 23:46:21Z), attempt 1, no supersession — the pending verdict this ADR's own push started, now green off the fold; v0.9.6's own chain+tag push ran **44968797491** (head `4ced794…`, the v0.9.6 TAGGED tree): success, 5/5 in 9 min 15 s, one run id for that head, no supersession, and its verification-fold push ran **44969661861** (head `0c450ba…`): success, 5/5 in 9 min 17 s, one run id — every recorded push through v0.9.6 is green |
| CI | no new step, no ask | **31 `cargo run --` steps + one plant line** verified by step-line count after the comment edit (the naive `grep -c "cargo run --"` counts the comment's OWN mention — the honest count filters comment lines) |

## Amendment (v0.9.6, [ADR 0017](./0017-the-live-min-buyout.md)) — one home filled, two stay negative-grounded

`price_min_now` got its FIRST producer: `sync --stage price` re-asks the TOOLTIP door, measured
2026-11-21 answering a CURRENT min (`fields.price`, live id 25 `min 149` @ `updated_at 22:25:10`;
ids 202/237/763 answering 5000/3500/4425 while their STORED copies carried NO price at all — the
two-columns rule with evidence) and its OWN third state (id 200 answers `fields.price` literal
NULL, "the AH offers nothing right now"), so the fetched/checked-empty split this ADR deferred
arrives WITH the producer, exactly as written — hop **6 → 7** carries its stamp pair
(`price_now_at`/`price_now_missing_at`), and the card door grows the value + its money sibling
(+2 keys, this release's documented wire delta). The OTHER two homes stay as carved on measured
negatives: `vendor_buy` has NO door and NO key (4 602 detail copies answer the same 28-key set,
`vendor_sell` only; the developers page lists exactly FOUR doors — list, record, search, tooltip),
and the only upstream "can be bought" signal is its own flavor prose `"Available from enchanting
vendors."` on exactly ONE copy (id 247786, Mote of Magic) — prose is not a price;
`price_trend_normal` keeps ZERO trend words in 19 258 rows (all 20 "volume" hits are `Volume I/II/III`
book titles) and no door. NULL in THOSE TWO still means "no producer has ever written it".
