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

## CI row (measured green, tag pinned)

**45048079898** (this release's chain+tag push `0690f38..59b0185`, head = the TAGGED tree):
**success, 5/5 in 7 min 28 s** (2026-11-22T17:11:55Z → 17:19:23Z — quality 2 m 3 s · linux 6 m
37 s · test 6 m 29 s · macos 7 m 24 s · windows 6 m 35 s), exactly ONE run id for that head,
no CANCELLED sibling, no rerun. The era's mid-chain feature-push run (head `0690f38`) measured
where it stands: **45046599075 success 5/5 in 10 min 42 s** (16:56:24Z → 17:07:06Z). The tag
stays put: `v0.11.0` → `59b0185`. The verification fold's own push starts the NEXT run,
measured in the NEXT record — the ladder never chases a tag (ADR 0011's ordering rule;
every push in this era waited for the prior run to complete).

## Post-tag (measured FORWARD, folded in the DEVLOG's v0.11.0 entry)

The mirror replayed the GROWN block on a fresh fixture process + a DB born 8: **47/47 commands
rc 0, ZERO NONZERO** (`TOTAL_STEPS=47` = 44 step-lines + plant + fixture start + sleep), the
stage presses UNCHANGED; the new step read BOTH live states in one RUN (`16181 ↔ 1g 61s 81c`
stored; crafted 40 missing-stamped + its OWN copied bytes 2 596 B64; crafted 60 live `14181 ↔
1g 41s 81c` differs). The exit map re-ran GROWN (fresh 9622, DEAD 9997, 404-file server 9559):
**89 doors `0`×60 · `2`×17 · `3`×6 · `4`×6, ZERO mismatches** — the live doors ride rc 0, the
era moved NO rc semantics. The pinned pair (v0.10.1 binary @ `09c8d77` vs the new one, ONE
shared fixture, TWO fresh DBs): **19 non-recipes DB-only doors BYTE-IDENTICAL**, and EVERY
recipes door proved `new answer MINUS the live keys ≡ pinned answer` — the release ADDED
fields where and ONLY where it said, nothing MOVED (both DBs: schema 8 / 5 items / 2
`price_min_now` / 1 `price_less` / 12 recipe rows). Over the USER's own file (opens + LOCAL
doors, ZERO requests): `price_min_now` / `price_now_missing_at` answer **0 / 0** — the price
stage never re-pressed THIS host, so the REAL joined rows honestly answer `live price not
refreshed yet` (never the price-less word); `vendor_buy` / `price_trend_normal` 0/0 (ADR
0016's grounds hold); 6 read-only doors BYTE-IDENTICAL vs the pinned binary (md5-equal,
1 098 B … 37 648 B); the real `recipes` answers are struct-equal-after-strip on real 1 240-row
data.

**RIDDEN AT v0.17.0 ([ADR 0032](./0032-the-total-crafting-cost-on-the-recipe-doors.md)):** the live trio and its TWO different NULL homes rode UNCHANGED into the cost — the per-material cells still answer `not in your DB` / `no auction right now` / `not refreshed yet` beside the cost column's ONE honest `cost_missing` count (fresh user-set census: 136 of 3 778 material cells without live = 135 alive rows with NULL live + 1 id with NO row at all — the homes stayed APART, never merged), and the cost reads ONLY the live twin: the stored min NEVER feeds it (the ask says live; ADR 0023's measured difference on the SAME row is exactly why).
