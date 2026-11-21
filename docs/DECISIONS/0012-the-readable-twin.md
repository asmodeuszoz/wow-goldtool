# ADR 0012 — the readable twin: copies are READABLE and FINDABLE, upstream markup never renders

**Status:** adopted (v0.9.0)
**Related:** `src/db.rs` (`readable_tooltip_text`, `SQL_MIGRATE_TO_V5`, `SQL_SET_TOOLTIP`),
`src/query.rs` (`Filter.scope`, `text_clauses`, `text_needles`), `src/cli.rs` (`--where`, `card_manual_parts`),
`ui/index.html` + `ui/app.js` (where select, card twin row), [ADR 0010](./0010-manual-layer-and-tooltip-copy.md),
[ADR 0009](./0009-live-search-door.md), [ADR 0004](./0004-headless-cli-mode.md), `docs/API.md`, `docs/DATABASE.md`

## The measurement that forced the decision

v0.8.0 stores each record's description copy as upstream's OWN tooltip `html`. Two live
tooltip answers, fetched for this decision and read back whole:

```
GET /api/v1/tooltip/forever/item/25    200 json 3060 b body, html 1404 b
  ETag W/"2b111cbcd3bbb4d7078f6d05"   cache-control public, max-age=300, s-maxage=900, swr=3600
GET /api/v1/tooltip/forever/item/728   1281 b html - the probe picked a record whose
  raw_json carries flavor (measured on the user's own DB: ids 728, 1132, 1357 answer
  it), so prose-shaped text went through the same reader
tags in both: <div class="twdb-*">, <span class="twdb-*">, <img class="twdb-icon" src=... >
  with class/src/width/height/alt/loading/decoding attributes
entities in both: ZERO (regex &[A-Za-z#0-9]+; answers nothing in either body)
img hotlinks INSIDE the copy: render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg
  and thewowdb.com/assets/pvp/faction/alliance.jpg - two hosts, pixels
```

And the shape that made the milestone necessary at all, measured at v0.7.0 (docs/API.md):
the UPSTREAM search door matches NAMES ONLY — `search?q=tooltip` answers empty groups even
though the tooltip body of 25 carries words. A DB that HOLDS descriptions but cannot FIND
them, and a card that shows markup-as-text, is half-finished work.

## Options considered

* **Render upstream's `html` in the card box.** Rejected by the house rule the icon stage
  already owns: `nothing is hotlinked, the DB gets the bytes` — the measured copy hotlinks
  pixels from TWO hosts, and pasting an upstream body as markup would also let upstream
  WRITE the card (v0.8.0's escape rule says so on the card fragment itself).
* **Keep storing html only.** Rejected at the door: `search` greps columns for words;
  markup text finds tags (`div`, `class`), not meaning — and would answer WRONG for real
  words hidden inside.
* **Write an entity decoder (`&amp;` → `&`).** Rejected on the project's own rule —
  inventing is not our call to make. ZERO entities were measured in both bodies, including
  a flavor-prose record. If upstream ever ships entities they show VERBATIM in the twin:
  words still search, and nothing silently rewrites meaning.
* **Derive the text at read time instead of a column.** Rejected: the card and `search`
  would run the derivation over 19 258 rows per query — and the pair-shape name/name_lc is
  already the measured precedent for storing the lowercase twin beside the display text.
* **Default `--where` to `all`.** Rejected: the UI's measured behaviour has ALWAYS been
  names-only; changing the default redefines every existing search. `name` stays the
  default; `all` is spelled out when wanted.
* **Migrate old copies' twins at schema-5 time (SQL derivation in ALTER).** Rejected:
  SQLite ALTER cannot run Rust's tag-stripper, and duplicating the reader across languages
  is exactly the drift the repo forbids. Honest gap instead: a schema-4 row with an old
  copy shows its markup row without a twin until the next tooltip write — measured true on
  any migrated DB; the user's OWN file has ZERO tooltip copies (measured 2026-11-21:
  19 258 rows, `tooltip_at` counts 0), so its tooltip pass runs AFTER this release and
  EVERY copy it ever makes carries a twin from birth.

## Decision

**One pair of columns, written in the same write as the copy, on EVERY write path.**

* schema `4 → 5`: `tooltip_text TEXT, tooltip_lc TEXT` appended (`SQL_MIGRATE_TO_V5`),
  and `check_schema` now chains EVERY stale shape up to v5 (`v1→v2→v3→v4→v5` plus the
  fresh `4→5` hop) — the user's file rides it the moment a 0.9.x build opens it;
* the derivation is exactly the measured moves — v0.9.1 correction INCLUDED: every
  `<...>` span leaves ONE space (measured after the user file's first live tooltip pass:
  live html is minified, and a bare span-DROP glued words across element boundaries —
  `Worn ShortswordItem Level 2Main Hand...` in 4 602 copies), then collapse whitespace,
  trim; the twin is derived from the **merged** copy (manual overrides merge
  LAST, ADR 0010, so the twin matches what the DB SAYS), and `SQL_SET_TOOLTIP` grows both
  columns in the SAME upsert — no window where a card shows text search cannot find;
* the manual doors birth the twin too: `set-item ID tooltip_html=…` derives it in the
  same UPDATE (the name→name_lc pattern), `add-item` with a stated description derives it
  at birth;
* `search TEXT --where name|description|all` (default `name`): `description` reads
  `tooltip_lc` behind `(tooltip_lc IS NOT NULL AND ...)` so never-fetched/checked-gone
  records skip honestly; `all` is name OR description; a bogus scope answers the usage
  door (exit 2); the tombstone rule stays first in EVERY scope (`visible_clause` pushed
  before any scope clause, ADR 0010);
* the card carries BOTH shapes: the raw copy printed-as-text (v0.8.0) and a `tooltip_text`
  fragment the UI renders as a words-row;
* the UI search panel grew a where select; the card box a readable-copy row.

## Deliberately NOT done here

* no entity decoder (measured zero — see above), no HTML parser dependency for three
  string moves, no new INDEX (LIKE patterns carry the WHERE; guessing an index for a
  word-grep is a plan, not a measurement — v0.5.3's rule),
* the crawl's pinned snapshot keeps running: **no 0.9.x binary may open the user's DB
  before the pinned v0.8.0 pull-all finishes** — it would stamp schema 5 and the pinned
  snapshot would then REFUSE it (exit-4 door, ADR 0004's honest refusal). The tooltip
  pass on the user's file therefore rides a v0.9.0 snapshot AFTER the pinned crawl
  finishes, and every audit in this release works on scratch DBs only.

## Audit (executed, not asserted)

| probe | expectation | observed |
| ----- | ----------- | -------- |
| two live tooltip bodies (25 plain, 728 flavor-carrying) | what the twin may rely on | div/span/img tags with the listed attrs, ZERO entities, hotlink pixels from two hosts — quoted above |
| fixture server CODE, not payloads | grounding for the smoke numbers | the fixture's detail handler assigns `body["name"] = "Fixture Detail <id>"` to EVERY detailed row (`tests/fixture_api.py`, measured) — the list door's row named `Worn Shortsword` gets its name OVERWRITTEN at detail-write time, and the only row keeping its list name is the checked-404 127555; the copied tooltip of 25 carries `One-Handed Sword` while NO final name carries `sword` — so the name scope answers 0 and `description`/`all` BOTH answer 2 (row 25 + the manual row's stated sword mug): the copy makes the word findable exactly where names cannot |
| every Rust commit of this chain | builds | each `cargo build` rc=0 between commits; the final one answers 0 errors, the four documented code warnings + this FS's hardlink notice |
| compile fixes owned mid-chain | honest, not swept under | a Python-style `.format(` idiom leaked into the parser and cargo answered it; the regex function-clone for the twin reader failed twice and the exact-source insert landed instead — both recorded |
| user's own file at the decision date | twin-gap honesty | schema `4`, 19 258 rows, ZERO tooltip copies → after the post-release tooltip pass every copy has a twin from birth; the gap only shows on DBs that already copied at v0.8.x |
| post-tag mirror + exit audits | the 24-step block + scope doors green, ZERO-request rule intact | **measured NOT green**: both replays hung 2 h 01 m at the door `search sword --where description` (scratch repro: timeout rc 124) — the `--where` block's bare `continue` skipped the parser's shared `index = index + 2` advance; the tag-push GH run was cancelled before executing, its rerun hung 2 h 03 m to a hand-cancel. Fixed forward at `ee8c06d`/`5a36d19` (v0.9.1); green replays (9134 / 9135..9142: 24/24, GETs 31 = 429×4/304×9/404×3, twins 4|4|4|0) recorded at that tag |
| v0.9.1 pre-tag replays (fresh ports 9134 / 9135..9142) | the fixed tree answers | **green**: 24/24 steps, GETs 31 = 429×4/304×9/404×3, twins 4\|4\|4\|0, scope hits 2/2/0, 64 doors with the v0.8.1 map identical + scope 0,0,0,2 + twin 0,0,0, ZERO asks for the manual id — the schema-copy doors SKIPPED (their scratch copy had been wiped) |
| v0.9.1 post-tag replays (fresh ports 9144 / 9145..9152 + 9143) | same, and the migration trio answers instead of skipping | **green**: 24/24 rc 0, GETs 31 = 200×15/304×9/404×3/429×4, twins 4\|4\|4\|0, schema 5, ZERO asks; 67 doors, 01..64 map identical, scope 0,0,0 hits 1/1/0, bogus 2, twin 0,0,0; doors 65/66/67 REAL — the pinned v0.8.1 snapshot births `schema_version 4`, the current build rides the v4→v5 hop to 5, the pinned snapshot then refuses the bumped file (rc 4, file untouched); GH: fix-chain push 44934053776 success 8 min 11 s, tagged tree 44934961058 success 7 min 17 s, this record's own push 44935854879 success 10 min 9 s |
| the user's file under the v0.9.1 build | the residue the decision promised, on the real DB | rc 0, 19 258 rows, `tooltips 4 602 = twins 4 602 = twin_lc 4 602`, schema 5 — and the 4 602 twins are STILL the glued v0.9.0 shape: the pending rule will not re-ask a stamped copy and a 304 writes nothing, so the refresh door is a named plan for a later milestone |
