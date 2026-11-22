# ADR 0023 — the live twin rides the joins and the recipe cell shows it beside the stored min

**Status:** decided (v0.11.0, 2026-11-22)
**Refs:** ADR 0020's deferred question ("live price IN the cell" — next release's
honest question), ADR 0017 (the price stage ALREADY writes `price_min_now` beside the
stored `price_min`, schema 7, its own stamp pair), ADR 0016 (the `vendor_buy` /
`price_trend_normal` trio: NO producer ever wrote them — still NULL here), ADR 0015
(the money contract), ADR 0006/0017 (the checked-missing THIRD state), ADR 0021's
by-value join (the recipe door), ADR 0014's one-code-path rule

## The question

The user's fifth ask delivered the deferred v0.9.9 question with a producer now on
it: the recipe tab's crafted cell must show what the AH offers NOW — not only the
stored min. Measured before the release (fixture DB, the price stage's OWN press):
id 25 stored `17520` / live `16520`, id 60 live `15520`, id 40 live `NULL` WITH
`price_now_missing_at` set, id 12809 both stamps `NULL` (the price stage never asked
it: `tooltip_at` NULL rides its pending rule). Four shapes, TWO of which a naive
"show the live column" would merge wrongly — that is the whole decision.

## The decision (four parts, one per truth)

* **One read carries the twin (ADR 0014's one-code-path).** `SQL_LIVE_NAME_PRICE`
  gains `price_min_now, price_now_at, price_now_missing_at` in the SAME SELECT, and
  `LiveNamePrice` mirrors them. No second query per row, no second DB pass: both
  by-value joins (`join_recipe_reagent` and the crafted join in `recipes_rows`)
  already call `live_name_price_of` ONCE, so the joins cannot meet a row a write
  moved mid-read — the twin rides that same pass. `RecipeReagent` and `RecipeRow`
  grow `price_now_copper/price_now_money/price_now_at/price_now_missing_at`
  (crafted: `crafted_price_now_*/crafted_now_*`). The money words are written by
  Rust's ONE formatter at answer time (ADR 0015's contract: copper + words
  together — the missing twin is the E0063 shape the compiler guards).
* **The two NULLs stay two answers (ADR 0017's own rule, restated at the cell):**
  only the STAMPS keep them apart, so the join fills the stamps and the cell NAMES
  them: `price_now_missing_at` set = "looked, the AH offers NOTHING right now"
  (ADR 0006/0017's third state); both stamps NULL = "the live price was never
  refreshed yet" (this record never met the price stage — ADR 0016's never-written
  pattern). `live_cell` draws the chips from those fields; a merge of the two NULLs
  would be the lie ADR 0006 refuses.
* **Cell rule (the ADR 0020/0015 pattern):** the money IN a cell stays the STORED
  min and its hover keeps the exact stored copper — UNTOUCHED. The LIVE money span
  (`class="live"`, words from Rust, exact live copper in ITS hover) rides beside it
  ONLY when the two DIFFER; live == stored shows NOTHING extra (a second identical
  number is noise); `vendor_sell`-style answers do not ride live words (the money
  contract's NEVER-beside-counts rule untouched — cells here are prices, and no
  auction COUNT moved). NO live noise at all when the DB has no row for the id
  (the join wrote no live fields — the caller guards; `not in your DB` /
  `the DB never saw this id` stay the honest words).
* **One conditional note sentence (ADR 0020's pattern, NO numbers):** shown ONLY
  when a live part fired on a shown row — naming cell = STORED min vs live = what
  the AH offered at the LAST ask. Numbers ride the cells, never the note.

## What this release does NOT do

* **No schema hop.** The columns are schema `7` (born with ADR 0016, written at
  ADR 0017): THIS release READS them; schema stays `8` (the v0.10.0 hop already
  moved for the recipe table).
* **`vendor_buy` and `price_trend_normal` stay NULL, untouched** — ADR 0016's
  honest grounds (no producer ever wrote them) still hold; filling them would
  invent, and the trio contract's NULL-vs-0 split stays put.
* **The SEARCH hit cell does not move** — ADR 0020's deferral stays where it stands
  (`HIT_COLUMNS` and the Hit money pair are a wire-shape change, measured unchanged
  here); this release answers the question IN THE RECIPE TAB, where the join already
  owns the row.
* No new request anywhere: the door is LOCAL (ADR 0012's rule), the joins are BY
  VALUE; the CI step added grows ZERO GETs.

## Chain and witnesses

`ff99bce` feat(db) (the SELECT widens, both joins fill copper + money + the two
stamps) → `0690f38` feat(ui) (`live_cell` + the conditional note + the CSS trio
`.live` / `.live-none` / `.live-missing`) → `ci` (43 → **44 step-lines**:
`cargo run -- recipes --profession Blacksmithing --icons --limit 3` reads BOTH
live states in ONE answer) → this record carries the annotated tag `v0.11.0`.

Measured (fresh DB, fresh fixture process, ZERO real-host requests): `recipes
--limit 40` answers 12 rows with stored `18490 ↔ 1g 84s 90c` (the RUN's seed) and
live `17490 ↔ 1g 74s 90c` / `16490 ↔ 1g 64s 90c` — live differs 5, "no auction
right now" 1, "not refreshed yet" 4, and the 2 enchanting crafts (crafted id NULL)
carry NO live noise; reagents: 11 of 19 join a row, 4 answer a live pair, 4 the
price-less state, 3 not refreshed yet. The Blacksmithing door answers BOTH states
side by side in 2 rows (crafted 40 = missing chip, crafted 60 = live differs).
UI witnesses are the launch-audit rule's: `node --check` PARSES the grown app.js,
grep names the EXACT 3 classes in app.js and the SAME 3 in app.css — the rendered
tab stays never-witnessed (no display in this build room).

Post-tag: the CI row and the verification fold are measured FORWARD (this era's
mirror is 47 commands = 44 step-lines + plant + fixture start + sleep); the ladder
never chases a tag, tags stay put.
