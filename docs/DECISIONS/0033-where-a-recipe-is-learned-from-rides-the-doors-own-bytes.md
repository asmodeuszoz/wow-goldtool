# ADR 0033: where a recipe is LEARNED FROM rides the doors' OWN bytes - the SET's words + ONE row-fact, NEVER an invented vendor

**Era:** v0.18.0 (user's eleventh ask, verbatim with its typos: "for recipes, also show
where it's learned from. Trainer or from items if it is an item where to get it from.")

**Status:** ADOPTED - the fetched set ALREADY states the whole learn half (`learned_json`
stored since v0.10.0, READ by the card door since v0.12.0); only the LIST door had never
REACHED the column (ADR 0021 recorded it stored-with-no-reader). ONE flag `--learned` on the
`recipes` door makes it answer; the tab asks; the card and its door rode PUT untouched.

## The decision

- **The words are the SET's own** (ADR 0004/0021's side-by-side rule): the learned-from object
  read BY KEY through the card door's OWN machinery - SAME `from_str` decode, SAME decode
  error WORDS, SAME `word_at`/`id_at` helpers - so the tab column and the card block CANNOT
  drift about one row (ADR 0014's one-code-path rule restated at the learned facts).
- **ONE fact from the rows this DB owns**: `learned_scroll_seen` - whether an ALIVE row for the
  named scroll exists - read through the SAME `live_name_price_of` reader the crafted join on
  this very door rides (`SQL_LIVE_NAME_PRICE`, tombstone guard included, ADR 0010's rule). The
  set-named home (747 of 804) and the DB-never-saw home (57) ride APART (ADR 0004/0016).
- **The set's OWN source word rides BESIDE, never instead**: `acquisition` (already answered
  unasked since v0.10.0) is the WHERE-half the ask names - vendor/drop/quest/multiple/unresearched
  are exactly the 804 schematic rows, trainer exactly the 413, automatic exactly the 23.
- **ZERO new verb, ZERO new SQL const, ZERO new request, ZERO stored bytes**: the flag parses
  globally (`bare_flag` + `Invocation.learned` + ONE parse line + ONE callsite), fills ONLY the
  list door; usage grew on the `recipes` door entry. `SQL_RECIPE_ROWS` grew ONE SELECTed column;
  the card's OWN SELECT rode PUT. The four answer fields ride serde `skip_serializing_if`
  (unasked = bytes PUT, pair-proven); cost is `money_of`'s contract untouched - ZERO copper in
  the column (words and ids only).
- **UI**: ONE `learn_cell` (the column's honest states incl "the set states NOTHING" / type-with-
  no-scroll / seen-vs-never-saw / no-source-word); the tab's OWN argv asks `--learned` (EXACTLY
  1 argv site); paint rides the frame's ONE header rule by JOINED selector (`th.col-learned`),
  ZERO new class, ZERO new colour token (`mat-none`/`matname` painted since v0.12.0).
- **"Where to get it" goes EXACTLY as far as the set says** - and the note says so: this DB owns
  NO vendor places (measured census below), so naming a vendor would INVENT.

## Measured (the user's own file, `sqlite3 -readonly`, NEVER written, NEVER crawled)

- learned kinds: `schematic_item` **804** / `trainer` **413** / `automatic` **23** = **1 240**,
  NONE silent; 804 name an id; **747** of those ids have an ALIVE item row (deleted 0); **477** of
  the 747 carry the scroll's OWN tooltip copy; **0 of 477** tooltip copies contain a vendor/zone/
  "sold/bought/reward/quest/drop/obtained" WORD (regex census) - the copy is stats+money text.
- acquisition words sum EXACTLY onto the kinds: vendor 254 + drop 361 + quest 64 + multiple 43 +
  unresearched 82 = **804**; trainer 413; automatic 23.
- fixture (fresh mirrors 9901/9911/9932): 132 = schematic naming scroll 60 BY VALUE + seen TRUE
  (even 60's icon-missing row is a ROW); 637 = schematic naming scroll 2589 + seen **FALSE** (the
  DB never saw the id - the two homes ride APART); 202 = the trainer word; 334 = the automatic
  word; the OTHER 8 rows keep ALL FOUR fields ABSENT even WHEN asked (serde skip, on the fixture);
  tab step `recipes --icons --costs --learned` 47 433 B asked vs 47 043 B unasked (bytes ride the
  run, tallies PUT at 51 GETs / 17 recipe GETs - the learned facts ask NOTHING of the wire);
  through-rides BYTE-EQUAL: `search sword --learned` == itself 188 == 188 B; card `recipe 637/467
  --learned` == unasked twins (1 507 == 1 507 / 1 211 == 1 211 - the card PARSES and IGNORES).
- pinned pair (fresh 9993 vs the v0.17.0 worktree @ TAGGED `473a63f`): **28/28 BYTE-IDENTICAL +
  13/13 md5-equal** unasked (the four fields add ZERO bytes unasked; unasked learned-byte census
  ZERO), and the PINNED binary REFUSES all THREE `--learned` shapes rc `2` `unknown flag --learned`
  - the BEFORE witness the flag is NEW not renamed; ADD-bytes when asked 3 382 -> 3 568.

## REJECTED alternatives (with their measures)

| rejected | why (measured the same day) |
| --- | --- |
| name the VENDOR/zone for scroll-sources | the DB owns no vendor data: 0 of 477 scroll tooltip copies carry a where-word (regex census); inventing a vendor = ADR 0004's invented-answer species |
| a SECOND door (`learned SPELL_ID`) the rows could disagree through | ADR 0021's door-drift species: the SAME rows would answer two doors; the flag rode the EXISTING door, ZERO new verb |
| store a `learned_where` column / vendor table | schema `8` PUT (re-read fresh); a stored WHERE would AGGE behind nothing - there IS no upstream where-data to refresh it from; schema hop rejected as fabrication |
| answer the learned fields UNASKED (always-on) | unasked bytes ride PUT since v0.10.0 (pair 28/28 + md5 13/13 vs the pinned v0.17.0); the fields ADD 3 382->3 568 B only WHEN asked |
| join the SCROLL's tooltip text as "where to get it" | 477 copies exist but 0 name a source; pasting upstream markup is rejected by ADR 0012 (readable twin only) anyway - and here the twin says NOTHING about sources |
| ride `vendor_sell` beside the scroll ("sells at Xc") | money beside a WHERE is the ask's word play, not a truth: vendor_sell is what a vendor TAKES, not where one IS; ADR 0015's contract has nothing to say - the column answers ZERO copper, measured |
| let the scroll name link to its ITEM card/window | the ask asked for a WORD, not a fourth window; the tab already opens ONE window per row (ADR 0025's click), and a second opener species on the same row is click-shape drift (ADR 0030's census measured the ONE-opener shape) |

## Consequences

- The LIST door now READS a column stored since v0.10.0 with no reader (ADR 0021's note is
  grown BY POINTER, not rewritten - history stays).
- Schema `8` PUT; the `recipe` table PUT; the `meta` census PUT.
- Pointers: 0014 GROWN (ONE machinery reads the bytes for BOTH surfaces), 0015 RIDDEN (zero
  copper rides the column), 0016 RIDDEN (seen fact rides the tombstone guard, TWO NULL homes
  ride apart), 0021 GROWN (the list door grew ONE flag + reaches the column; usage grown on the
  door entry), 0025 RIDDEN (the card block answered learned-from since v0.12.0 and rode PUT),
  0032 RIDDEN (costs rode PUT into this era - unasked and asked cost bytes both measured again).
