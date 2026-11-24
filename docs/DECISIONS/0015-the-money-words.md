# ADR 0015 — the money words: prices read like the GAME writes them, copper stays the number

**Status:** adopted (v0.9.4)
**Related:** `src/db.rs` (`money_text`, `money_of`, `money_of_avg`, `PriceSample`'s money pair,
`history_of`), `src/query.rs` (`Hit`'s money pair, `search`), `src/stats.rs` (`Coverage`'s
trio, `QualityStat`'s mean money, `Extreme`'s money), `src/cli.rs` (`card_money_parts` at the
four card sites), `ui/app.js` + `ui/index.html` + `ui/app.css` (`money_cell`, the money rows,
`.moneycell`), [ADR 0007](./0007-price-sampling.md), [ADR 0004](./0004-headless-cli-mode.md),
[ADR 0010](./0010-manual-layer-and-tooltip-copy.md), [ADR 0011](./0011-ci-asserts-only-what-is-true.md),
[ADR 0014](./0014-the-icon-doors.md), `docs/DATABASE.md`, `docs/API.md`

## The measurement that forced the decision

The unit of every copper column was an open question in the UI's eyes: the DB holds
integers, the game writes `2g 50s`. Nothing in this repo states the base — so the DB's
**own tooltip copies** settled it, because a copied tooltip already carries its sell price
in money words. One read of the user's own DB, before any code changed:

```
rows                                              19 258
rows with a vendor_sell INTEGER                    4 231
rows whose copy states a "Sell Price:" line         4 231  (the SAME rows)
money_text(vendor_sell) EQUALS the copy's own words 4 230  of 4 231
rows that DISAGREE with their own copy                  1  → item 25: column 500 ("5s")
                                                          versus its copy "Sell Price: 7c"
rows at copper 0 in ANY column (price_min / price_market / vendor_sell / price_history)
                                                    0  — MIN over each column is 1
```

The copies state the base and the zero rule for free:

```
810    vendor_sell   45 267  copy "Sell Price: 4g 52s 67c"   all three parts
2528   vendor_sell   10 367  copy "Sell Price: 1g 3s 67c"
45267 → 4g 52s 67c proves 100 copper = 1 silver AND 100 silver = 1 gold
4499   vendor_sell   25 000  copy "Sell Price: 2g 50s"       zero copper DROPPED
12720  vendor_sell   20 000  copy "Sell Price: 2g"           zero silver AND copper
733    vendor_sell      100  copy "Sell Price: 1s"           whole silver, no copper
36     vendor_sell        7  copy "Sell Price: 7c"           under a silver
14138  vendor_sell   20 042  copy "Sell Price: 2g 42c"       the MIDDLE zero: silver
                             dropped BETWEEN gold and copper - the only such row in 4 231
```

Money classes over the same column: 1 413 rows are copper-only, 810 are whole silver
with zero copper, 219 carry all three parts, 73 are whole gold, and exactly **1** is
middle-zero. Over `price_min` the middle-zero class has 19 rows and `price_market`
17, so the rule is not a coincidence of one column.

Spans, and the money each column reaches:

```
price_min      n 4 291  1 .. 14 006 500 copper  → "1c" .. "1400g 65s"   longest 12 chars "799g 99s 98c"
price_market   n 4 291  1 .. 5 000 001 000      → "1c" .. "500000g 1s"  longest 13 chars "1334g 99s 49c"
vendor_sell    n 4 231  1 .. 1 000 000          → "1c" .. "100g"        longest 10 chars "8g 42s 91c"
price_history  n 4 284  1 .. 14 006 500         → same as price_min
AVG(price_min) = 54 391.705896 copper           → floored "5g 43s 91c", rounded would be "5g 43s 92c"
```

## Options considered

* **Store money in the DB.** Rejected: money is a *rendering* of a number that already
  exists. A `*_money` column would split one truth in two places, break the numeric
  contract of `docs/DATABASE.md` (the price sorts, `ORDER BY (price_min IS NULL) ASC,
  price_min ASC` and the script fields read INTEGER copper), and would need a refresh
  door for DBs born before the change — v0.9.2's `refresh-twins` exists because a twin is
  derived from a copy; money would need no such door because it is written at answer time.
* **Format money in JavaScript.** Rejected: the CLI and the GUI would then hold two
  formatters of the same words, and they can drift (the house rule is one truth, one
  encoder — ADR 0014's `b64_text` move is the same precedent).
* **Keep three parts always visible (`2g 0s 0c`).** Rejected by the copies themselves:
  upstream's own wording drops a zero denomination (`25 000` ↔ `2g 50s`, `20 042` ↔
  `2g 42c`), and inventing `0s` upstream never wrote is putting words in its copy.
* **Money only in the UI, not in the CLI JSON.** Rejected: ADR 0004 made the CLI and the
  GUI one code path down to the same JSON; hiding money in the CLI would mean the headless
  door answers a different shape than the webview.
* **Money as a sibling fragment beside every copper number, written by ONE Rust
  formatter at answer time.** Chosen.

## Decision

* **`money_text(copper) -> String` in `src/db.rs`, beside the price accessors**: the unit
  and the zero rule are the copies' — a zero denomination is DROPPED, denominations join
  with one space (`25000 → "2g 50s"`, `454 → "4s 54c"`, `20042 → "2g 42c"`, `20000 → "2g"`).
  A copper of `0` answers `"0c"`, and that is a RULE, not a measurement: measured MIN over
  every copper column is `1`, so no row in the DB ever needed that answer. It exists so a
  stated zero (a script, a future payload) stays readable instead of printing nothing.
* **`money_of(Optional<i64>)`**: an absent price stays absent. `"the AH has no offer"` and
  `"this item costs nothing"` stay two answers — ADR 0004's rule against made-up zeros,
  applied to money.
* **`money_of_avg(Optional<f64>)` FLOORS.** SQLite answers `AVG` as a float (measured
  `54 391.705896` copper) and money has no fraction of a copper, so rounding up would print
  `5g 43s 92c`, a price no observation reached. The exact float still rides beside it.
* **Money is a top-level sibling fragment, never inside the upstream shape.** The card's
  money (`price_min_money`, `price_market_money`, `vendor_sell_money`) sits beside the
  `item` object, the same way `icon_b64` does (ADR 0014), because `ItemDetail` and `Price`
  in `models.rs` are the mirror of the wire (`docs/API.md`): a field added inside them
  would claim upstream answered a word it never answered.
* **Money reads the MERGED detail, not the DB's columns** — ADR 0010's merge-last rule
  applied to money, so a money word can never disagree with the number beside it. Measured
  on the fixture where the two could have differed: row 25's `vendor_sell` was stated by
  the CI at `606`, and the card prints `6s 6c` next to it.
* **Money rides EVERY copper number the product shows**, and it needs no flag — unlike
  ADR 0014's 3.1 K chars per icon, money costs about 56 bytes per priced row on the mirror
  (measured pairs below) and the longest money string in the user's whole DB is 13 chars:
  the hits table, both price lists, the card's price rows and its trend bars' hovers, the
  status grid's price range and average, the by-quality bar labels, the stats note's spread.
* **`price_available` and `price_realms` are auction COUNTS and never become money.**
  Measured: row 25 answers `available 12` for 12 offers of 15 000 copper, and `realms`
  spans 1..12 across the user's DB. Money riding a count would be a lie.
* **NO schema hop, no new column, no refresh door, no SQL touched.** `SQL_COVERAGE`,
  `SQL_QUALITY`, `SQL_CHEAP`, `SQL_RICH`, `SQL_PRICE_SAMPLE`, `SQL_HISTORY_OF` and
  `HIT_COLUMNS` are unchanged, so the read slots stay 0..14 and no aggregate or its CAST
  guard can drift; `schema_version` stays `5` on the mirror's and the user's DB after a
  replay that ran every money door.

## Consequences, measured and not smoothed

* the money disagreement upstream left is **recorded, not fixed**: item 25 is the ONE row
  of 4 231 whose `vendor_sell` (500) disagrees with its own copy's words (`Sell Price: 7c`).
  It is upstream's, the committed fixture carries the same shape (`"vendor_sell": 500`
  beside a copy saying `7c`) — and on that fixture, where money reads the COLUMN, ALL
  THREE rows that state Sell Price words disagree with their own column (606 → `6s 6c`,
  500 → `5s` beside `7c`). Money is money of the number, never a copy of a tooltip's words;
  nothing in this release reconciles a column against a copy, and no commit tries to.
* the payload deltas, measured with **both** binaries (the pre-money v0.9.3 tree built in a
  worktree, and the money build) over the SAME mirror file: `search --limit 6` 1 587 → 1 941,
  `stats --limit 3` 1 210 → 1 533, `status` 1 035 → 1 125, `stats` 1 467 → 1 850,
  `stats --icons` 17 079 → 17 462, `item 25 --cached` 4 174 → 4 336, `item 12809 --cached`
  3 709 → 3 868, the manual row's card 1 023 → 1 103, `search sword --where description`
  650 → 763, `search sword --where all` 642 → 755, `search sword --where description --icons`
  3 250 → 3 363. Money sends no ask, so the fixture's GET tallies cannot move.
* `refresh-twins` over the mirror answers **4 copies, 0 rewritten, 4 already matching, ZERO
  requests** — money needed no door, which is the honest way to show a derived value added
  nothing to the DB.
* the money wording is upstream's, the numbers are ours: the copies show `4g 52s 67c` style
  with single-letter suffixes and no separators, so `money_text` writes exactly that. The
  UI's hover says `20,628 copper exact` — a UI wording of OURS, chosen so the number under
  the money is unambiguous.
* an unpriced row answers money `null` beside `null` and the UI cell keeps saying `no price`
  (never `0c`): measured on the mirror's `search --limit 6`, rows 900000000 and 127555 keep
  answering `None` beside `None`.
* the middle-zero row 14138 answers `2g 42c` beside its own copy's `Sell Price: 2g 42c` with
  no code path comparing them — the formatter and the copy agree by construction, which is
  the point of taking the rule from the copies.

## Audit

| probe | expectation | observed |
| ----- | ----------- | -------- |
| `money_text` against the DB's own copies | the formatter writes what the copies write | **4 230 of 4 231** rows agree exactly (name, column and copy compared); the ONE disagreement is item 25, upstream's own |
| middle-zero class | gold present, silver ZERO, copper nonzero | 1 row in `vendor_sell` (14138 → `2g 42c`, copy says `2g 42c`), 19 in `price_min`, 17 in `price_market` — the rule is per-value, not per-column |
| `0` copper | `"0c"` is a rule, not a measurement | measured **0 rows at copper 0** in any of the four copper columns; MIN over each is `1` |
| the average | floors, never rounds up | `AVG(price_min) 54 391.705896` → `5g 43s 91c` (rounded would print `5g 43s 92c`) |
| counts | never money | `price_available` and `price_realms` carry NO money sibling on any shape; row 25 answers `available 12` for 12 offers of 15 000 copper |
| card door, ZERO requests (`--cached`) | money beside the printed copper | user's DB: 3850 `10075 → 1g 75c`, `vendor_sell 7304 → 73s 4c`; 14138 `vendor_sell 20042 → 2g 42c`; mirror: 25 `20628 → 2g 6s 28c`, `17628 → 1g 76s 28c`, stated `vendor_sell 606 → 6s 6c`, its ONE sample `20628 → 2g 6s 28c`; the made row 900000000 answers `price null` and money `null` |
| the list doors, ZERO requests | money rides, unpriced rows keep `null` | `search --limit 6`: 12809/25/40/60 → `2g 6s 28c` / `1g 76s 28c`; 900000000 and 127555 → `null` beside `null`; `stats --limit 3` → coverage `min/max/avg 20628 / 20628 / 20628.0` beside three `2g 6s 28c`, and the Rare facet with ZERO priced items keeps `avg_price null` beside `avg_price_money null` |
| the UI cells, run over the REAL door JSON | money in the cell, copper in the hover | node runs the real `ui/app.js` over the mirror's answers: hits cells print `<span class="moneycell" title="20,628 copper exact">2g 6s 28c</span>`, the unpriced rows print `<span class="muted">no price</span>`, the card prints `min price 2g 6s 28c` / `market price 1g 76s 28c` / `vendor sell 6s 6c` and keeps `available 12` un-moneied, the made row answers `no price` on all three money rows, and a trend bar's hover reads `2g 6s 28c · 20,628 copper` off the sample that carries `price_min_money` |
| the CI smoke block | money adds no step, no request, no exit code | **31 `cargo run --` steps still, one plant line still**; every step keeps its own exit code; money costs measured old→new as above; the fixture's GET tallies unmoved (ADR 0011's rule, ADR 0014's proof restated) |
| the DB's shape | schema unmoved, no new column | `SELECT key, value FROM meta WHERE key GLOB 'schema*'` answers `schema_version|5` on BOTH the mirror's and the user's DB; `refresh-twins` answers 0 rewritten / 4 matching / ZERO requests |
| the exit-code audit, re-run post-tag on fresh ports 9178..9185 + 9186 | the map cannot move | **90 doors**: the v0.9.3 map's **86 pairs BYTE-FOR-BYTE identical**, zero changed; money adds four — invented flags answer usage (`search … --moneyx` → **2**, `stats --money` → **2**, money has NO flag), the money-bearing fixture door → **0**, `status` → **0**; totals `0` ×59 · `2` ×14 · `3` ×6 · `4` ×11; pinned trio 65/66/67 = `0/0/4`; fixture A GETs 29 = 200×16/304×3/404×6/429×4, B 2, C 4 (pinned window) — money doors sent nothing |
| the post-tag mirror replay, driver verified = the committed block | money adds no step, no ask, no answer drift | **31/31 steps rc 0**, plant rc=0, GETs **31 = 200×15 / 304×9 / 404×3 / 429×4** identical to the v0.9.3 replay; DB tail `6|4|4|1|1|0`, icons `3|1`, twins `4|4|4|0`, schema `5`, made row **ZERO** asks; this replay's own copper reads `20527 → 2g 5s 27c` / `17527 → 1g 75s 27c` |
| money over the replay's 31 step JSONs | money is money OF the copper in the SAME answer, never inside the mirror shape | **53 money keys** (`price_min_money` 28 · `price_market_money` 12 · `avg_price_money` 6 · `min/max_price_money` 2+2 · `vendor_sell_money` 3), **zero drift** (the float `avg 20527.0` → `2g 5s 27c`: the floor holds), **zero money keys inside any `item` object**; the driver's first pass had a probe bug (stripped `money`, 5 chars, not `_money`, 6) printing siblings as absent — fixed and re-measured, the product never changed |
| money on a DB the PRE-money binary wrote (31 steps with the v0.9.3 binary, fresh port 9187) | money answers with NO refresh door, schema unmoved | `schema_version` reads **5 before and after** the money build touches it; `search --limit 6` → `16702 → 1g 67s 2c` / `13702 → 1g 37s 2c`; coverage `16702 / 16702 / 16702.0` beside three `1g 67s 2c`; card 25 stated `606 → 6s 6c`; sample rides money; **zero** `*_money` columns — the opposite of the twin, which NEEDED v0.9.2's door because it is stored |
| money on the pinned-era migrated DB (schema 4→5, 5 rows / 0 priced) | migrated rows answer money on the next read, absent stays absent | every money answer `null` beside `null`, ZERO requests |
| money vs the fixture's OWN copies | money is money OF the COLUMN, the disagreement is upstream's and stays recorded | **3 of 3** rows stating Sell Price words disagree with their column (25 stated `606 → 6s 6c`, 40/60 `500 → 5s` beside `7c`; 12809 has 500 and no Sell Price line in its copy) — the user's DB shows the same shape on 1 of 4 231; nothing reconciles the two |
| CI runs | green on the tagged tree | **run 44954802500: success, 5/5 jobs in 10 min 36 s** (2026-11-21T22:04:26Z → 22:15:02Z — quality 2 m 15 s · test 4 m 23 s · linux 4 m 33 s · macos 7 m 16 s · windows 10 m 32 s) for the chain+tag push `1b56f45..4875222`, ran on `head_sha 4875222…` — the TAGGED tree; **run 44956129051: success, 5/5 jobs in 10 min 44 s** (22:19:17Z → 22:30:01Z — quality 2 m 16 s · test 4 m 40 s · linux 5 m 33 s · macos 6 m 19 s · windows 10 m 41 s) for the verification record's push `4875222..90cb963` on head `90cb963…`, joining the v0.9.3 verdict **run 44946961792: success, 5/5 in 10 min 15 s** for `e2db5a9..1b56f45` |

**GROWN AT v0.17.0 ([ADR 0032](./0032-the-total-crafting-cost-on-the-recipe-doors.md)):** the money contract restated itself at the cost fields — `cost_money` (the frame's OWN `money_of`) rides ONLY beside a COMPLETE `cost_copper`, and the 12 fixture `cost_missing` counts wear WORDS beside them and ZERO money (the audit over the very steps that carry costs keeps **0 pairing defects** grown with the new answers; serde's skip even strips an incomplete total's money fields when asked — the fixture card 132 answers `cost_missing 2` with money ABSENT). An unasked answer carries ZERO cost bytes — the pair's 28/28 BYTE-IDENTICAL is this half measured.
