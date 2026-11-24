# ADR 0032 — the two recipe doors answer the TOTAL crafting cost: ONE `--costs` flag, ONE `craft_cost` helper over the ALREADY-joined materials, money ONLY for a COMPLETE total

**Status:** decided (v0.17.0, 2026-11-24)
**Refs:** the user's tenth ask ("add a column with the total crafting cost based on live prices" —
over BOTH the Recipes tab and the recipe detail cards), [ADR 0014](./0014-the-icon-doors.md)
(ONE code path: a tab column and a card line cannot drift),
[ADR 0015](./0015-the-money-words.md) (money beside EVERY copper it labels; a COUNT never wears
money), [ADR 0016](./0016-the-price-homes.md) (NULL-vs-0 homes; `vendor_buy`/`price_trend_normal`
stay 0/0), [ADR 0004](./0004-exit-codes.md) (no invented answers, no invented zeros),
[ADR 0012](./0012-the-local-search-door.md) (the LOCAL-door rule: ZERO requests — the fixture's
51/17 GET tally cannot move), [ADR 0021](./0021-the-recipes-second-source.md) (the set's OWN words
ride beside the DB's; its count fields), [ADR 0023](./0023-the-live-twin-in-the-cell.md)
(the live layer `price_min_now` + its two different NULL homes)

## The question

"The TOTAL CRAFTING COST based on LIVE prices." Four facts are born here: WHERE the number is summed
(the joined materials), WHICH price layer (the live `price_min_now` twin, NOT the aging stored min),
WHAT an incomplete cost answers (honest counts, NO money word), and HOW the tab column and the card
line stay THE SAME number (ONE helper, ONE flag on the doors that already answer).

## The census, measured BEFORE one line moved

| the fact | measured | what it meant |
| -------- | -------- | ------------- |
| the join | `join_recipe_reagent` ALREADY carries `count` (the set's stated multiplier) + `price_now_copper` + `price_now_missing_at` per material | the cost reads the door's OWN joined fields — ZERO new SQL, ZERO new query, ZERO request |
| the live layer | `price_min_now` written by the price stage (ADR 0017/0023); user file: 4 344 live values + 258 checked-empty | LIVE = this column; the stored `price_min` AGES — the ask says live, so it never feeds the cost |
| the user's set | 1 240 rows / 3 778 material cells: 3 642 live-priced, 136 WITHOUT live (135 rows alive with NULL live values + 1 id with NO row at all) — rows: **1 146 COMPLETE, 94 PARTIAL, 0 all-or-none**, counts 1 264 ×1 / 2 514 ×>1 / **0 NULL** | every row answers SOME state; the multiplier is always stated — a NULL count would read as ONE, a RULE stated not measured (the fetched set never leaves one) |
| the fixture | 12 rows: 1 complete (First Aid 467: its ONE reagent IS a priced row — money `1g 93s 40c` beside 19 340 exact copper, a RUN-moving seed), 3 partial (202/637/3024: 1 of 2 priced — the 2589/5113 ids have no rows), 8 all-unpriceable (25/40-row mats + never-listed ids) | costs over the FIXTURE are mostly PARTIAL — honest, stated where it can be MISread as complete |
| the money rule | `money_of`/`money_text` (ADR 0015) are the frame's OWN formatters | the cost's money words come from `money_of` beside the copper — NEVER a UI sum, NEVER hand-joined words |
| the doors | `recipes` and `recipe N` JOIN the materials ALREADY (1 240 / ≤1+N cells) | the cost rides THESE doors' answers; the tab and the card click the SAME door's fields |
| bytes | unasked recipe answers rode PUT through v0.10.0..v0.16.0 eras (pair audits) | an UNASKED answer must keep its bytes EXACTLY — the three cost fields ride serde `skip_serializing_if` so they are ABSENT when not asked (measured: pinned v0.14.0 answers vs new answers 28/28 BYTE-IDENTICAL, cost fields adding ZERO bytes unasked; asked: `recipes --limit 3` 3 377 → 3 428 B, `recipe 467` 1 209 → 1 272 B) |

## The decision

* **ONE flag, ZERO new verb**: `--costs` rides the EXISTING doors `recipes` and `recipe SPELL_ID`
  (bare_flag, Invocation.costs, usage grown on both door entries only). A new verb would be a SECOND
  door the same rows could disagree through (ADR 0014/0028's one-answer rules).
* **ONE `craft_cost(reagents) -> CostTally`** in `db.rs`, called by BOTH doors AFTER their OWN joins:
  `total += (count>1 ? count : 1) × price_now_copper` over materials WITH a live value; every live-less
  material increments `cost_missing`. `cost_copper` + `cost_money` (via `money_of`) answer ONLY when
  `cost_missing == 0` AND the set HAS materials; the fields ride `skip_serializing_if` — an unasked
  row/card carries ZERO cost bytes (pair PUT), and a PARTIAL row carries ONLY `cost_missing` (serde
  strips the money fields of an incomplete total EVEN WHEN ASKED — measured on the fixture card 132:
  `cost_missing 2`, money ABSENT).
* **The UI rides ONE renderer**: `cost_cell(one, mats)` answers the tab's NEW `col-cost` td and the
  card's `rcardcost` div — complete: `money_cell` (words + EXACT copper in the hover, ADR 0015);
  partial: `cost incomplete - N of M materials have no live price` (COUNTS with words, ZERO money
  beside them — ADR 0015's count half); all-unpriceable: `no live prices yet`; empty: `nothing to
  cost`; both tabs ASK `--costs` (their argv grew; the CLI without the flag stays byte-PUT — the
  through-ride measured `search sword --costs` == `search sword` 188 == 188 B: the flag PARSES and
  rides through the doors that never asked for it, exactly like `--icons` rode since v0.9.3).
* **The two different NULL homes ride apart ALREADY beside the count**: the per-material cells
  (ADR 0023) say `not in your DB` / `no auction right now` / `not refreshed yet`; the cost column
  says ONE honest count `cost_missing` — the gap census NEVER merges them (ADR 0006/0016's rule),
  and `vendor_buy` / `price_trend_normal` stay NULL homes 0/0 (ADR 0016 PUT).

## Rejected, measured before one line moved (SEVEN)

1. **SQL SUM over `reagents_json`** — it is a TEXT column of the set's OWN bytes (ADR 0021 wrote it
   that way): SQL cannot read its counts, cannot join each id through the live layer, and would
   bypass the door's joins (the tombstone guard ADR 0010 put on every join). Rejected.
2. **Cost from the STORED min (`price_min`)** — the ask says live; the stored copy AGES (ADR 0023
   measured live and stored differing ON THE SAME row). The cost would answer what the AH offered
   LAST CRAWL, not now. Rejected.
3. **A money word on PARTIAL totals ("costs at least …")** — invites reading an incomplete sum as
   the recipe's cost — a control-surface LIE shape; ADR 0028 refused a bound that HIDDEN rows, same
   species. Measured alternative: the fixture would answer 11 partial money words for 11 recipes
   that cannot be crafted from what the DB prices — the honest answer is the COUNT. Rejected.
4. **A `cost_copper` COLUMN in the recipe table** — cost is a DERIVED reading of the live layer
   (which MOVES with every price press): a stored number would AGGE behind the live twin — exactly
   what ADR 0015/0023 refused for money words ("never stored, derived at answer time"), and a schema
   hop `8 → 9` for an aging duplicate. Schema stays `8` (measured on the user's file, read-only).
   Rejected.
5. **Summing in the UI** — copper maths in JS was rejected frame-wide since v0.9.4 (money comes from
   Rust's ONE formatter); a UI sum would ALSO drift from the card's line (two sums). ONE `craft_cost`
   behind both doors + ONE `cost_cell` in front of both surfaces. Rejected.
6. **Cost WITHOUT the set's count multiplier** — recipe 132's TWO Mossborer cost twice; the census
   measured 2 514 of 3 778 materials stating a count `>1`: a no-multiplier cost would UNDER-price
   2 514/3 778 of the user's material cells. Rejected.
7. **Cost on the ITEM card** (`render_card`) — measured: the item card names NO recipe (its rows are
   the item's own fields; a card per item would need the reverse join spell→items, a NEW door shape
   the ask never asked for). Scope creep. Rejected — the cost lands on the recipe card, where the
   materials ALREADY ride.

## The cells and the contract, measured fresh

Exit map (fresh 9995, seed + EMPTY DB): the BASE 31 doors keep the SAME totals `0`×19 · `2`×5 · `4`×7
(every era since v0.15.0 kept them — kept again) and the era's FIVE `--costs` shapes answer
`0`×3 (`recipes --costs --limit 3`, `recipe 467 --costs`, `search sword --costs` — through-ride) +
`4`×2 (`recipe 999999 --costs` on the door's OWN no-row words, and `recipes --costs` over the EMPTY
table: the SAME no-rows refusal BEFORE any rows read — the flag cannot turn a refusal into an empty
success, ADR 0004). Money census over the door's OWN answers (fixture): cost fields pair 1/1
(467: money beside exact copper), the 12 `cost_missing` counts wear ZERO money — **0 pairing
defects** grown with the new answers. Pinned pair (fresh 9911, pinned v0.14.0 worktree @ TAGGED
`c065815`): **28/28 BYTE-IDENTICAL + 13/13 md5-equal** (md5s pinned in the record: `stats` 83e3e784
· `stats --icons` 3cef3911 · `stats --limit 3` b490fe3e · `item 25 --cached` 6f14fe9d · `item 40
--cached` 5ad258b3 · `item 12809 --cached` a22fd7b4 · `status` 48bba880 · `watch list` dc0cb5ab
· `refresh-twins` ea136265 · `search sword --where description` 51482bc0 · the Blacksmithing door
ce77da4b · `recipe 132 --icons` ded6e551 · `recipe 638` d21b4486) — and the PINNED binary REFUSES
all three `--costs` shapes rc `2` `unknown flag --costs (run help to see the verbs)`: the BEFORE
witness the flag is NEW, measured on the scratch pair, NOT committed as a green CI line (ADR 0011's
split). The fixture census the CI block quotes: 12/12 rows carry `cost_missing`, money rides 1/12
(467 COMPLETE at 19 340 / `1g 93s 40c` in that window — RUN-dependent bytes, tallies not).

## What grows and what does NOT

* Grows: `Invocation.costs` + `bare_flag` + `parse_invocation` ONE flag line each + the usage tail's
  TWO door entries, the `CostTally` struct + ONE `craft_cost`, THREE serde-skipped fields on `RecipeRow`
  AND `RecipeCard` + their fills in the two door builds (params `costs`/`with_costs`), the UI `cost_cell`
  + the tab's td/argv/note + the card's div/argv, the `th.col-cost`/`.rcardcost` JOINS + ONE new `.costnote`,
  and THREE CI step-lines (49 → 52, plant one stays).
* Does NOT grow: any `SQL_` const, any schema line (**8** stays — the user's file re-read fresh says
  `8`), any NEW verb, any request rule, the live layer's writers, `vendor_buy`/`price_trend_normal`
  (0/0 again), the Search doors' bytes (through-ride 188 == 188), the item card (`render_card` PUT —
  the cost lands on the recipe card only), `--limit`/`--offset` paging, the pager's absence (ADR 0028),
  the opener machinery (ADR 0030), ADR 0027's 21 id rules (STILL 21).
* Builds: `cargo build` rc `0`, named warnings ZERO (the hard-link incremental CACHE line is the FS
  artifact, never a fifth named); `cargo clippy` 0 errors / **822 advice lines — the number MOVED from
  the 811 of v0.16.0 because Rust GREW this era (stated honestly, ADR 0011: expected-sets only where
  a truth of ZERO can be claimed; the `-D warnings` gate stays OFF**.

## Consequences, kept honest

* Tags never move; ADR 0014 GROWN (ONE helper behind BOTH doors AND ONE renderer in front — the
  sentence "a click and a cargo line are one door" now covers a COST too), ADR 0015 GROWN (its money
  contract restated at the cost fields: money ONLY beside a complete copper, counts with words never
  money), ADR 0016 RIDDEN (the cost READS the live home; the reserved homes stay 0/0), ADR 0021 RIDDEN
  (the multiplier is the set's OWN stated count; 0 of 3 778 cells null in the user's set), ADR 0025
  GROWN (the card door grew fields; its click and door stay ONE path), ADR 0023 RIDDEN (the live trio
  and its two NULL homes ride unchanged into the cost).
* The webview NEVER renders in the build room: the cost COLUMN's paint/DOM is NOT WITNESSED and NOT
  CLAIMED; UI witnesses parse/grep ONLY (PARSE OK, `cost_cell` ONE caller-pair — tab + card, the
  `col-cost` td/thead/CSS-join ONE each, argv `--costs` EXACTLY 2 sites).

## Chain

Rides the era's chain above (`5c068d5` feat(cli,db) this decision's Rust · `96496cb`/`446f85d` its
UI · `4b6c024` the CI trio · `ef9da01` banner · this record's tag `v0.17.0`).
