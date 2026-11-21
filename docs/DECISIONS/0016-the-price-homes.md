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
| CI | no new step, no ask | **31 `cargo run --` steps + one plant line** verified by step-line count after the comment edit (the naive `grep -c "cargo run --"` counts the comment's OWN mention — the honest count filters comment lines) |
