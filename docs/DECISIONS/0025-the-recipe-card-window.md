# ADR 0025 — a recipe row opens a CARD of its own, shown as the GAME shows it, through ONE local door

**Status:** decided (v0.12.0, 2026-11-22)
**Refs:** the user's ask ("when looking at a recipe, clicking it in the list should open a
detailed window showing it like it would appear in the game"), ADR 0021 (the recipe doors, the
by-value join, and `learned_json` recorded as stored-with-no-reader — THIS era gives it a reader),
ADR 0014 (one code path: a click and a `cargo run --` line), ADR 0004 (the exit-code contract,
identical CLI/GUI; no invented answers), ADR 0015 (money beside every copper, NEVER beside a count),
ADR 0012 (the readable tooltip twin is what a card renders — upstream's markup never reaches the
page), ADR 0010 (the tombstone rule every join carries), ADR 0017/0016 (the live twin's two NULLs
and the trio's honest NULL), ADR 0018 (one activity visible per tab), ADR 0011 (CI asserts only
what is true — the rc-4 half of this door is measured on scratch, never committed as a green step)

## The question

"Show it like it would appear in the game." The honest reading is not decoration: a recipe in the
game is a SCROLL you can read, an ITEM it crafts, the MATERIALS it asks for, and the SKILL band it
sits in. The question is therefore a JOIN question, and it was measured before one line of code
was written — because a window that guesses which item a recipe belongs to is a window that lies.

## The join, measured on the user's own file (read-only, ZERO requests, 2026-11-22)

The `recipe` table holds 1 240 fetched entries; the DB holds 19 258 item rows. What can honestly
be linked, and what cannot:

| candidate link | measured today | verdict |
| -------------- | -------------- | ------- |
| `spell_id` used as an item id (the scroll's row) | 651 spell ids land on an item row, and only **20** of those rows carry `item_type Recipe` | **REJECTED** — spell ids and item ids are two namespaces; 631 of the 651 land on rows that are not scrolls at all |
| the name link `"Recipe: " + crafted_item_name` | **123** of 1 240 entries reach an item row (case-sensitive and case-folded answer the SAME); the DB names only **250** items `Recipe:*` while **716** items carry `item_type Recipe`, and only **122** of those 716 are named `Recipe:*` — and of the 123 that reach, **ZERO** land on the item id this very entry names (`crafted_item_id`): all **123** attach to a DIFFERENT item | **REJECTED** — not merely under-reaching: it silently MIS-attaches. The DB's OWN names differ from the set's (the detail stage overwrites, deprecated prefixes, non-localisation), so a name join writes lies onto the card |
| `learned_json` stated BY VALUE | the object is `{"item_id":…,"item_name":…,"type":"schematic_item"}` — **804** entries state `schematic_item` and name an id, **413** state the `trainer` word, **23** the `automatic` word (sums to 1 240: no entry leaves learning unstated in the fetched shape); of the 804 stated ids **747** have an item row, **477** of those rows carry the scroll's OWN tooltip copy | **ACCEPTED** — the set itself states the scroll; no invention anywhere on this path |
| `crafted_item_id` (ADR 0021's join, reused) | **1 095** entries name a crafted id that has a row (of 1 105 distinct), **897** of those rows carry a copy, **198** have a row but no copy, **133** entries name NO crafted id at all (all `Enchanting`: an enchant is a slot effect), **10** ids have no row | ACCEPTED, gaps kept |
| reagent ids (ADR 0021's join, reused) | **416** distinct ids → **415** have rows → **393** carry a copy (22 have a row but no copy, 1 has no row) | ACCEPTED, gaps kept |

The census is why the window has honest limits: a scroll cell exists for 804 of 1 240 entries, and
477 of those show the scroll's OWN words. Every other entry answers with a state chip instead —
`the set states the word alone — no scroll item exists for this kind of learning` (trainer /
automatic) or `the DB never saw this id alive` (a stated id with no visible row) or
`the set states NOTHING about how this recipe is learned` (the fetched shape's own NULL, kept by
the fixture's row 638). No chip is decoration: each names WHICH source is silent, ADR 0004's rule.

## The decision (five parts, one per truth)

* **One door, one verb, ONE code path (ADR 0014).** The click does not grow a second answer shape:
  `cargo run -- recipe SPELL_ID [--icons]` is the door, and the webview calls exactly that argv
  through `cli_ok`. A card is therefore auditable from a shell, and the click's answer IS the
  answer the CLI prints. Rejected: a flag on the list door (`recipes --id N`) — one argv answering
  two shapes is the drift ADR 0014 refuses — and a webview-only shape no audit can witness.
* **The door is LOCAL, ZERO requests (ADR 0009/0012's local-door rule).** It reads the rows the
  recipe stage wrote and joins BY VALUE through the item rows this DB owns. Nothing upstream is
  asked for a card: the scroll, the result and the materials are exactly the ids the set states,
  and the DB is the only place their words live. `--icons` rides because a copied icon is about
  3.1 K chars once encoded (ADR 0014's measured reason the bytes stay opt-in); the door is bounded
  by ONE spell id, but the rule stays the same rather than silently shipping a fat answer.
* **Cells REUSE the item card's and the list join's readers (no drift).** `live_name_price_of`
  answers name + stored min + the live twin + BOTH stamps, `money_of` writes the money words,
  `icon_b64_of`/`icon_at_of`/`icon_missing_at_of` and `tooltip_text_of`/`tooltip_at_of`/
  `tooltip_missing_at_of` answer the copy and icon states, and the MATERIALS ride `join_recipe_reagent`
  untouched — a card's material cell is the same code that draws a list row's material cell, so the
  two cannot say different things about one id. ONLY the four columns the list door never reads
  (quality, item level, required level, vendor sell) come from a SELECT of this door's own
  (`SQL_CARD_CELL`), and it carries the SAME `deleted_at IS NULL` guard as `SQL_LIVE_NAME_PRICE`.
  The tombstone rule is made true IN the door, not assumed: the stamp readers above select on `id`
  alone (measured: `SQL_TOOLTIP_TEXT_OF`, `SQL_TOOLTIP_AT_OF`, `SQL_TOOLTIP_MISSING_AT_OF`,
  `SQL_ICON_BLOB_OF`, `SQL_ICON_AT_OF`, `SQL_ICON_MISSING_AT_OF` carry NO `deleted_at` clause), so
  they are asked ONLY for a row this cell saw alive — a row the user buried keeps its stamps buried.
* **Money keeps ADR 0015's contract on the new surface, and ADR 0016's NULLs stay unwritten.**
  Measured on the fixture's own answers over these steps: **0 pairing defects** (vendor sell,
  stored min and the live twin each carry their money word beside their copper) and **13 counts**
  (a reagent's `count`) **carried with NO money beside them**. `vendor_buy` and `price_trend_normal`
  read **0 / 0** in the fixture census and stay NULL here: no producer ever wrote them, and this
  door READS the live twin, it never grows a producer for the other two homes (ADR 0016's rule,
  restated at the cell).
* **`learned_json` gets its first reader, and ADR 0021's debt closes where it was written.** The
  column is read BY KEY, never by field name: its `type` key is a reserved word, so the door reads
  `serde_json.Value` accessors (`word_at` / `id_at` take the KEY as a string) — the rule ADR 0021
  states next to that very struct, kept here rather than renamed into a serde `rename` (models.rs
  carries ZERO serde attributes, and one renamed field would have put a wire key's spelling on a
  type that answers other doors). ADR 0021's bullet now points here; its measured tables stay
  history (tags never move).

## The cell's NULLs, named (ADR 0006/0016's rule, restated where the card ships)

Three different NULLs ride a card and must stay three answers, never one word:

* `learned_type` NULL → the fetched entry stated NOTHING about learning (the dataset's own absence).
* a `learned_type` word with `learned_scroll_id` NULL → the set stated a WORD that names no item
  (measured 413 trainer / 23 automatic over the real set).
* `scroll` present with `in_db` false → the set stated an id this DB has no VISIBLE row for — the
  tombstone rule rides, so "the user buried it" and "the DB never saw it" answer the same chip
  HERE (the card never shows a buried row's columns either way, and the door's SELECT says so).

For the money pair: `price_now_missing_at` set = "looked, the AH offers NOTHING right now"; both
stamps NULL = "this record's live price was never refreshed yet". The card draws both chips from
the stamps via the frame's `live_cell` (the v0.11.0 trio `.live` / `.live-missing` / `.live-none`),
and NEVER merges them. On the user's own file the second chip is a TRUTH, not a defect: the price
pass covers only records that hold a tooltip copy (measured 2026-11-22: 4 602 copies, 14 656 rows
with none — the stage refuses those by ADR 0010's rule), so a card whose cell says
`live price not refreshed yet` is saying what the DB's own stamps say.

## The window (what "as the game shows it" is made of)

`#recipe-card` is a NINTH nameplate tab (ADR 0018's one-activity layout, the same mechanism as the
item card's panel): reached by typing an id, and by clicking a recipe row's name — the name cell is
a button carrying its own spell id, and ONE delegated handler over the hits table reads it (the
pattern the search hits and the tab strip already ride; a 100-row page plants no closures). The
list itself stays a list (the user's choice: a recipe card of its own, not a re-aimed item card,
not a drill-down).

The card composes, in this order and with these words: the entry's OWN set words (job chip · recipe
name · `spell id N` · `skill N` · the tier word · the set's grey-to-green band `trivial_low` to
`trivial_high` · the `acquisition` word + its note + the set's provenance) → the learned-from line
(the new type word, the scroll id the set states, and the set's OWN name for that scroll) → the
`teaches the spell` cell → the `crafts` cell (or `no crafted item — this craft puts an enchant on a
slot`) → the material cells via `recipe_mats` REUSED. A cell shows: the DB's name with the set's
name beside it when the two DIFFER, quality on the frame's `.chip c-<quality>` classes (the SAME
classes the search hits draw — no second palette here), the icon bytes or the gone-checked mark,
both levels, vendor sell as money with its copper, the stored min with its exact-copper hover, the
live twin beside it per `live_cell`, and the record's OWN tooltip copy printed as TEXT under a
label that names its source (ADR 0012's readable twin; upstream's markup stays out of the door's
answer entirely — the card carries `tooltip_text`, never `tooltip_html`).

## Measured (fixture: 48-step mirror + fresh scratch, ZERO real-host for the door)

The fixture now speaks the three real shapes (keys exactly `item_id · item_name · type`): 132
`schematic_item` naming 60 · 637 naming 2589 · 202 `trainer` · 334 `automatic` · 638 the explicit
NULL. The stored census after the stage's own write: `8 × NULL`, one of each of the four.

| door | rc | answer | what it shows |
| ---- | -- | ------ | ------------- |
| `recipe 132 --icons` | 0 | 7 330 B | scroll 60 `in_db true`, DB name "Fixture Detail 60" beside the set's own, `Common`, levels, `vendor 500 ↔ 5s`, stored `16876 ↔ 1g 68s 76c`, live `14876 ↔ 1g 48s 76c` at 1795394721, the copy's own words, icon answering CHECKED-GONE (`icon_missing_at` set — the third icon state, not empty bytes); result 25 carries 2 596 chars of icon bytes |
| `recipe 202 --icons` | 0 | 6 673 B | `learned_type "trainer"`, `learned_scroll_id` NULL, `scroll` NULL — the word alone; crafted 40 answers `price_now_missing_at` set ("looked, the AH offers NOTHING right now") |
| `recipe 637 --icons` | 0 | 4 107 B | `learned_type "schematic_item"` naming 2589 → scroll cell `in_db false` with the SET's name beside it, never an invented one; `result` NULL (the enchanting shape: the set names no crafted item) |
| `recipe 638` | 0 | 643 B | `learned_type` NULL (the explicit NULL the fixture kept) + no crafted item |
| `recipe 999999 --icons` | 4 | — | `no recipe row for spell id 999999: the recipe table answers only after sync --stage recipes wrote it (ADR 0021)` |

Real-host door answers (the user's OWN file, read-only, ZERO requests): `recipe 27835 --icons` rc
0 / 25 212 B — scroll 22390 answers IN GAME: `epic`, level 63, `vendor_sell 20000 ↔ 2g`, stored
`309600 ↔ 30g 96s`, live `420000 ↔ 42g` at 1795393867, copy 176 chars, icon 3 524 chars; result
"Persuader" `8870000 ↔ 887g` with live `8670000 ↔ 867g`; 6 materials, all in the DB, 5 with money
and live. `recipe 2171 --icons` rc 0 / 7 003 B — scroll 2404 has a row but nobody ever priced or
copied it (every price/copy/icon field NULL, the chip-less honest state). `recipe 2177 --icons` rc
0 / 9 429 B — the `trainer` word, NO scroll cell, result cell complete with a 208-char copy and
2 740 icon chars, live `230 ↔ 2s 30c` beside stored `250 ↔ 2s 50c`. Usage rc `2` measured: 48 B
"recipe wants a spell id, for example: recipe 132" and 40 B "recipe wants a numeric spell id, got
abc". rc `4` on the user's file measured (112 B).

## Consequences, kept honest

* Grows: one SQL const for the card row + ONE for the four extra columns; three wire structs
  (`RecipeCard` / `RecipeCell` / the raw pass); TWO tiny Value readers (`word_at`, `id_at`); the
  verb + its usage + its dispatch line; four fixture rows; the ninth tab + the panel + the click
  renderer; four CI step-lines.
* Does NOT grow: any request, any schema column (schema stays `8` — `learned_json` already exists),
  any new money formatter, any new quality palette, any new live-state chip, any second material
  renderer, any `vendor_buy` / `price_trend_normal` producer, any per-row listener in the webview.
* The named-warning set stays ZERO at this release: this era PLANTED nine names (the two consts,
  the three structs, the five readers) whose reader landed in the very next commit, and `feat(cli)`
  built with ZERO named. The FS notice (`hard linking … incremental cache`) is not a fifth named
  warning and answered ZERO times in this era's own cache-clear builds — it rode, together with an
  incremental-cache ICE, the DRIVER's `cargo run` rebuilds; the era's audit drivers now exec the
  already-built binary instead, so no compile is ever in flight during an audit (recorded as a
  driver-side defect, fixed in the driver, forward).
* Tags never move: ADR 0021 keeps its measured tables and gains a pointer at this door; ADR 0023's
  live-trio rule is reused, not restated.
* ONE build-time defect found AFTER the tag, named BY NAME and NOT muted (the fold's measurement,
  2026-11-23): the chain+tag run **45095439290 measured FAILURE** (4/5 jobs, 8 min 9 s, head
  `ace683a`) because the quality job's lint step rode **ONE clippy ERROR** — `this loop never
  actually loops` at `src/db.rs:3029:5` (`clippy::never_loop`, deny-by-default): `recipe_card`
  iterates the ONE row the `spell_id` PRIMARY KEY can produce and its body ALWAYS returns, so the
  loop can never take a second turn. `cargo build` stayed GREEN at every commit (an ERROR-level
  clippy diagnostic is a DIFFERENT gate from rustc's named dead-code set — the record keeps the
  two apart, ADR 0024's ZERO set is rustc's own words), and the DOOR WORKS (nine rc `0` witnesses
  on the exit map, the 48-step smoke block GREEN on GitHub). The repair lands FORWARD in the v0.12.1
  patch era: the loop takes the shape the established single-row doors already use (clippy's OWN
  suggestion `if let Some(one_row) = mapped.next()`), plus the step's name and the workflow's header
  comment move off their now-false "informational" claim — the step's rc DOES gate the run when an
  ERROR rides it. No mute, no gate, no `#pragma`-style suppression anywhere.
* **CLOSED, ONE PATCH COMMIT LATER (v0.12.1, `fix(db) 56e0356`, [ADR 0026](./0026-a-clippy-error-is-not-advice.md)):**
  the loop header became `if let Some(one_row) = mapped.next()` (clippy's OWN suggestion) with a comment
  naming the PRIMARY-KEY fact, the era's diff is **ONE file / 7 insertions / 2 deletions**, `cargo clippy
  --all-targets` measures **0 errors / the SAME 811 advice lines** (rc 101 → rc 0) while rustc's named
  tally stays **ZERO on both sides of the pair** (this was never a dead-code warning: the ERROR rides a
  DIFFERENT gate), and the pinned pair measures **22/22 BYTE-IDENTICAL** answers — the NINE card doors
  included — plus **13/13 md5-equal** through both binaries over the user's OWN file: behavior moved
  ZERO. The bullet above stays put, its measures stay history, and the workflow gained NO line: at 0
  errors the step's own NAME is true again, measured rather than assumed (ADR 0026 keeps the decision
  and its FOUR rejected alternatives, the mute among them). **MEASURED IN THE FOLD:** the era's own
  chain+tag run — head = the TAGGED tree `d2c26de` — answered **45101284057 SUCCESS**, 5/5 jobs green
  in **8 min 13 s** (02:37:33Z → 02:45:46Z, run #44), quality **7/7 with the Lint step GREEN** (the
  FIRST green lint step since v0.11.1) and `test` 8/8 with the CLI smoke step GREEN, while its sibling
  (the v0.12.0 fold push's run, head `89cab34`) measured **45098365666 FAILURE** with the SAME failing
  step: the verdict moved FAILURE → SUCCESS because the TREE's shape moved, with ZERO workflow lines
  moved and ZERO mutes (ADR 0026 keeps the fold and its witnesses — mirror 51/51 rc 0 on fresh 9341,
  the map at the SAME 31-door totals on fresh 9373, the pinned v0.12.0 pair 16/16 md5-equal over the
  user's own file, whose tallies carry the fact this fix rides: 1 240 recipe rows / 1 240 DISTINCT
  `spell_id`s).


## Chain

`0b717a4` feat(db) (the read + the cells, NINE names planted, build rc 0) → `29ac737` feat(cli)
(the verb, ZERO named, rc 4 + the two rc 2 usages measured) → `a03008b` test(fixture) (the three
shapes + the NULL) → `53d5977` feat(ui) (the ninth tab, the click, the renderer) → `2b1784f` ci
(44 → 48 step-lines, ZERO requests, the rc-4 half on scratch) → `aa70092` chore (banner
0.11.0 → 0.12.0 — a FEATURE era moves it) → this record carries `v0.12.0`.

**GROWN AT v0.13.0 ([ADR 0027](./0027-a-cards-sizes-are-scoped-to-the-window.md)):** the WINDOW's
paint moved, this door did not — its type ~1.45× and its icons 24 → 72 px through 21 id-scoped
rules in `ui/app.css` (ZERO `app.js`, ZERO Rust, ZERO SQL line), and the era measured the strongest
claim a paint era can make: **22/22 BYTE-IDENTICAL** answers vs the pinned v0.12.1 tree (the NINE
card doors included) plus **13/13 md5-equal** through BOTH binaries over the user's own file, the
mirror green 48/48 on three fresh ports at the SAME stage tallies and GET tallies (51 GETs, 17
recipe-file GETs, ZERO added), the exit map at the SAME **31-door totals `0`×19 · `2`×5 · `4`×7**,
and **0** money-pairing defects over the card doors' own answers. **THE FOLD MEASURED IT ON THE PUSHED
TREE TOO** (post-tag, FRESH ports/DBs, the real host untouched): mirror **48/48 rc 0, ZERO nonzero** on
fresh 9471 — the SAME stage presses and the SAME GET tallies (51 GETs, 17 recipe-file GETs, ZERO added =
the locality proof), four card steps at 7 333 / 6 673 / 4 107 / 643 B — the exit map at the **SAME 31-door
totals `0`×19 · `2`×5 · `4`×7** on fresh 9481, and the pinned pair **AGAIN 22/22 BYTE-IDENTICAL, ZERO DELTAs**
plus **13/13 md5-equal** on fresh 9501, so the door's bytes stayed put on BOTH sides of the tag, measured
TWICE rather than asserted; GitHub's own verdict for the tag's tree is **45199646115 SUCCESS**, 5/5 jobs
green in **16 min 4 s** (14:56:38Z → 15:12:42Z, run #46, head `b927109` = the tag's target) — quality 7/7
Lint step GREEN, `test` 8/8 with the CLI smoke step GREEN, linux 8/8, macos 7/8, windows 7/8.
**GROWN AT v0.15.0 ([ADR 0029](./0029-the-card-window-floats-over-the-tab-its-click-came-from.md)):**
WHERE its window opens moved — the card is no longer the NINTH nameplate tab but the WINDOW a row click
floats OVER the Recipes tab (nameplate + `data-tab` died; `Close this window` is the window's OWN closer
and the tab underneath stays where it was). Everything else above stays PUT word-for-word: the click path,
the delegated handler, the ONE local door, the composition and the honest gaps. The pinned pair measured
the card doors **BYTE-IDENTICAL** (fixture answers 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 /
6 740 / 4 033 B against the TAGGED v0.14.0 tree) — this era moved a SURFACE, not a truth.
**GROWN AT v0.16.0 ([ADR 0030](./0030-a-list-rows-name-is-its-window-opener.md)):** the sentence
"the name cell IS the click target" became true of BOTH item lists - the SEARCH row's name cell became
its window opener too (`<button class="pipbtn" data-id>NAME</button>` riding the SAME delegated pattern),
so ADR 0025's pattern now covers `#recipe-hits` AND `#hits`. Everything else above stays PUT: the click
path, the ONE local door, the composition, the honest gaps. The pinned pair measured the card doors and
the item/search doors BYTE-IDENTICAL across the move (fixture 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209
/ 1 155 / 6 740 / 4 033 B; `item 25/40 --cached` 4 341 / 4 076 B; `search sword` 185/182 B) - a name click,
a chip click and a `cargo run` line are ONE door (ADR 0014).

**GROWN AT v0.17.0 ([ADR 0032](./0032-the-total-crafting-cost-on-the-recipe-doors.md)):** the card door grew cost FIELDS (`cost_copper`/`cost_money`/`cost_missing` under serde skip — unasked the card keeps its bytes, pinned pair md5s `recipe 132 --icons` ded6e551 + `recipe 638` d21b4486 rode PUT on a fresh window) and the card grew a cost LINE via the ONE `cost_cell` the tab's cost column also calls; the click, the split BY CLASS and the `recipe N` door itself stayed RIDDEN PUT — the era's TWO new CI card steps (`recipe 467 --costs` complete, `recipe 132 --icons --costs` honest-partial) press the SAME door the name click opens.

**RIDDEN AT v0.18.0 ([ADR 0033](./0033-where-a-recipe-is-learned-from-rides-the-doors-own-bytes.md)):** the card's learned-from block — the ONE place the `learned_json` was read since
v0.12.0 — rode PUT WORD-FOR-WORD this era: its SELECT, its parse, its cells and its argv moved ZERO lines.
The flag `--learned` PARSES and the card IGNORES it — measured twins BYTE-EQUAL (recipe 637: 1 507 == 1 507 B /
recipe 334: 1 433 == 1 433 B on the SAME window / recipe 467: 1 211 == 1 211 B) — because the card ANSWERED
learned-from UNASKED since its era, while the LIST door grew the ask. The era's WHOLE point was meeting the
card's already-stated shape, not a second one: the tab column reads THROUGH this door's OWN machinery (ADR 0014),
so a row's learned answer CANNOT disagree between the two surfaces (reject list at ADR 0033: a second door
`learned SPELL_ID` would be ADR 0021's door-drift species on the SAME rows).
