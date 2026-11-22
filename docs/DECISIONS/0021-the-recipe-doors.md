# ADR 0021 — the recipe doors: the second source is FETCHED, the local tab joins it BY VALUE

**Status:** adopted (v0.10.0)
**Related:** `src/sync.rs` (`RECIPE_FILE_COUNT` / `recipe_file_at` / `recipe_etag_key` /
`sync_recipes`), `src/wowdb.rs` (`recipe_file_url` + `RECIPE_SOURCE_BASE`, the `WGT_FIXTURE`
override), `src/models.rs` (`RecipeFile` / `RecipeEntry` / `Reagent`), `src/db.rs` (`SCHEMA_VERSION
= 8`, `SQL_MIGRATE_TO_V8`, `SQL_INSERT_RECIPE` + `RECIPE_VALUE_COUNT`, `write_recipe`,
`recipes_rows`, `join_recipe_reagent`, `RecipeRow` / `RecipeReagent`), `src/cli.rs` (the
`recipes` verb, `--profession`, the stage whitelist, the whole-door refusal),
`ui/index.html` + `ui/app.js` + `ui/app.css` (the eighth tab, three buttons),
`tests/fixture_api.py` (the tiny `/data/<file>.json` grounds),
[ADR 0004](./0004-headless-cli-mode.md) (the exit contract the new doors keep),
[ADR 0006](./0006-checked-missing-third-state.md) (whose pattern the join's states reuse),
[ADR 0009](./0009-live-search-door.md) / [ADR 0012](./0012-the-readable-twin.md) (the local-door
rule: one read, ZERO requests), [ADR 0014](./0014-the-icon-doors.md) (a click and a cargo line
stay one path; icons ride the DB's OWN bytes), [ADR 0015](./0015-the-money-words.md) (money beside
copper, never beside counts), [ADR 0016](./0016-the-price-trio-homes.md) (whose NULL truth the
join's states keep apart), [ADR 0019](./0019-the-sync-tabs-stage-buttons.md) (the stage-button
shape kept for the second source), [ADR 0011](./0011-ci-asserts-only-what-is-true.md) (a red
path stays a scratch measurement, never a green step)

## The observation that forced it

The user's third ask: "pull every job with every recipe … new tab … Icons, descriptions,
items needed" — and the question under it: "search for APIs that give this for wow forever".
Measured the SAME day (2026-11-22), the answer is SPLIT, and the split is what shapes the
release. Nobody owns all of the tab's words:

* **The API has no professions door.** `/api/v1/forever/professions`, `/profession`, `/jobs`,
  `/skills`, `/tradeskills` answer **404** (five paths probed today; the developers page still
  names exactly FOUR doors).
* **The API's recipe CARDS carry no job, no skill, no materials.** The mirror's own copy holds
  **716** rows with `item_type "Recipe"`; **699** of 716 tooltip copies say "Teaches you how
  to…" (flavor prose); **0** copies contain "Requires Materials". `?q=Recipe` on the search door
  answers NAMES only — **259** ids, i.e. it UNDERCOUNTS the 716 type-`Recipe` rows (the
  names-only trap again), and a name cannot fill a tab.
* **Blizzard's own Game Data API is 404 today.** `api.blizzard.com/data/wow/recipe/2424` with
  the exact namespace its own 2025 bug report names (`static-11.1.5_60179-us`) answers **404**;
  the `11.2.0`-shaped guesses answer 404 too. Recorded as a dead end, never as a door: typing a
  shape nobody answers would invent upstream.
* **ONE measured carrier states job + skill + exact reagents — and it is not an API.**
  [ReagentRoute](https://github.com/Nyte-fyre/ReagentRoute) (**MIT**, text measured at the
  repo's `addon/LICENSE`) keeps plain JSON at
  `raw.githubusercontent.com/Nyte-fyre/ReagentRoute/main/data/<profession>_recipes.json`. The
  classic set measured today: **eight files**
  (`alchemy · blacksmithing · cooking · enchanting · engineering · first_aid · leatherworking ·
  tailoring`) — **NO Jewelcrafting in this set** (JC rides the set's `*_tbc_recipes.json`, the
  TBC-Anniversary era, which this DB does not keep) — **2 843 484 B** fetched whole
  (241 981 · 638 864 · 82 407 · 348 098 · 296 531 · 26 372 · 518 352 · 690 879 B), **1 240
  entries**, `recipe_count` stated per file matching the array every time (113 · 250 · 81 ·
  152 · 161 · 15 · 232 · 236), `profession` per file in the set's OWN words, entries carrying
  **16 keys on ALL 1 240** (`spell_id · craft_spell_id · spell_name · crafted_item_id ·
  crafted_item_name · required_skill_value · skill_tier · trivial_low · trivial_high ·
  reagents[{item_id,item_name,count}] · learned_from · acquisition · source · confidence ·
  auto_granted · vendor_sell_price`), `acquisition_note` on **745** of them, `source` values
  `cmangos-classic-db` / `cmangos-tbc-db+wago-tools-db2`.
* **The id spaces MEET the item rows.** The classic files speak CLASSIC ids (`spell_id 2339`
  "Minor Mana Potion" ↔ `crafted_item_id 2455`). Join measured the same day against the user's
  own file (19 258 rows): **1 105 distinct crafted ids → 1 095 have an item row, 10 do not**;
  **133 entries carry NO crafted id at all — all 133 are `Enchanting`** (a craft puts an ENCHANT
  on a slot, not an item in a bag); **416 distinct reagent ids → 415 have an item row, 1 does
  not**, and **392** of the joined reagents already answer a `price_min`. THIS is why the tab's
  chips exist: they are measured join facts, not decoration.
* **The other candidates were rejected on measured grounds, not taste.** The set's
  `*_forever_recipes.json` files carry `game_version "forever-beta"` in a DIFFERENT id space
  (`spell_id 1283400`, `crafted_item_id 279961`) that cannot join these rows. Napalmsteak's
  WoW-Classic-Item-Caches answers **404** at its LICENSE path (no license → not a carrier) and
  its `Forever Cache.json` keys are `sourceKeys/sourceLabels/sourceTypes` — acquisition words,
  never reagents.

## The decision

**Three parts, one per truth.**

1. **The dataset rides as a runtime FETCH stage, not as shipped bytes.** `sync --stage
   recipes` fetches the eight files — one request each, conditional GETs like every other door
   (the file ETags ride the `meta` table under `recipe_etag_<file>`; a re-pull pays real
   **304**s and writes NOTHING), the resume cursor is the FILE count already done, `--budget`
   bounds FILES. `WGT_FIXTURE` overrides the base EXACTLY as it does for the API's doors, so
   CI serves `/data/<file>.json` and cannot drift from the wire shape. The 2.6 MB set is
   **NOT vendored**: a committed copy of an external dataset would be a second, drifting truth
   the day the source moves; the docs carry the measured counts, the fetched file is the data.
2. **Schema 8 + a local door.** `CREATE TABLE recipe (…)` + `CREATE INDEX recipe__crafted`
   arrive fresh (SCHEMA_SQL) AND by hop (`SQL_MIGRATE_TO_V8`, the ladder's case `7 =>`), so a
   v0.9.9 file migrates in place. `write_recipe` UPSERTs the entry's OWN fields verbatim
   (14 columns, `RECIPE_VALUE_COUNT * 2 - 1` params, `bind_sql` counts them). The `recipes`
   verb (`recipes [--profession NAME] [--limit N] [--offset N] [--icons]`) reads the table
   and joins BY VALUE through the item rows this DB already owns — **ONE read, ZERO requests**
   (ADR 0012's local-door rule), `--profession` bound never interpolated, count and rows share
   ONE WHERE chain, and an empty table answers **rc 4** (`not_found`: "the recipe table holds
   no rows (for this filter): run sync --stage recipes…" — the door never prints a recipe from
   air, ADR 0004).
3. **The eighth nameplate tab, as the user described it.** job chip · recipe name · skill
   number + tier band · one material cell per reagent (icon = the DB's OWN bytes, `◇` when
   there are none; price in money words with the EXACT copper in the hover via the ONE
   `money_cell`) · the cooked item through its own card. The honest states ride as CHIPS:
   "not in your DB" beside a reagent the set named but the DB has no row for; "no crafted item
   — enchanting craft" for the 133; "no price" for a reagent row nobody priced (never 0c);
   "the DB never saw this id" beside a crafted join that stayed dataset-only. The Sync tab
   grows `Recipes: fetch the set` on the SAME `runSync(stage)` loop as every other stage
   button, and `Watch the cooked items` batches ONE `watch add` call over the cooked ids of
   the shown rows.

**Rejected / deferred, recorded where they stand:**

* **Shipping the dataset as repo bytes** — rejected (drift above).
* **Jewelcrafting / TBC / forever-beta rows** — out of this stage (measured era/id-space facts
  above); the tab's note and this ADR say so instead of guessing.
* **Recipe rows feeding item rows** — rejected: a recipe must never invent item facts; the join
  rides the OTHER direction (recipe rows read the item rows that exist).
* **A per-record third state inside the stage** — there is NONE. A 404 FILE is the second
  source MOVING, not one record being gone, so it ABORTS the run (rc 3, like the Failed kind).
  The door's third states (uncrafted recipe, unjoined id, unpriced reagent) are READ facts the
  reader reports, never stage-side stamps.
* **Chaining the recipes stage into "Pull all info"** — deliberately NOT chained (ADR 0019's
  discipline kept for the second source): the chain's contract is "the catalogue is complete"
  over the API's doors; the recipe set is a whole dataset on ANOTHER host, fetched whole,
  eight conditional files. Not a refresh either: it writes LOCAL rows from fetched bytes.
* **`learned_json` on the wire** — stored, NO reader yet (ADR 0011's dead-code rule kept: the
  door's chip reads the entry's plain `acquisition` word).
* **Money for the set's `vendor_sell_price` / `confidence`** — untouched: ADR 0015's money rides
  copper THIS DB has; a dataset number nobody's door reads would float money beside nothing.
* **Merging the API's recipe CARDS into the tab** — not merged: 699 copies' "Teaches you how
  to…" prose describes the same thing the set states precisely; two shapes of one fact is
  two truths in one word. The set's words are the tab's; the prose card stays on the item card.
* **A per-recipe row limit above 100** — capped like every list door (`--limit`/`--offset` ride
  the pager); 1 240 rows at ~1 KB is not a page to dump.

## What stays true under the doors — measured, not assumed

| rule | how it survived |
| ---- | ------------- |
| ADR 0004 exit codes | `recipes` on an empty table answers **4** (measured on a scratch DB before any stage ran, with and without a filter); the 404-file abort answers **3**; `--ids`/`--watch` on this whole-door stage answer **2**; the mirror's new steps answer 0. The map re-runs post-tag grown honestly |
| ADR 0014 one code path | the tab's `Read the DB` sends the SAME `cli_ok(["recipes","--limit",…,"--icons"])` argv; the fetch button rides `runSync("recipes")`, same chunk / cancel-at-chunk-end / `sync_state` resume |
| ADR 0015 money | money words ride beside EVERY copper answer (reagents AND the crafted item); the JS never does copper maths; NO money beside a count (the tab note's "showing N of M" is a count) |
| ADR 0016 NULL truth | three NULLs keep three answers apart: `crafted_item_id: null` = the SET states none; `crafted_db_name: null` = the DB has no row; `price_copper: null` = nobody priced it ("no price", never 0c) |
| ADR 0011 CI only asserts what is true | the committed fixture serves the eight files fine (the real host answers 200 + ETag today); the 404-file abort is measured on a scratch audit server, never committed as a green step; CI's numbers are the FIXTURE's 12 entries, the real 1 240 live in the docs |
| ADR 0008 no hotlinks | the tab's icons are the DB's OWN bytes (data URI) — the second source names ids, it never ships an image URL for them |
| the second source's license | MIT, measured at the repo; cited in the footer, the tab note and docs/API.md's new section — attribution stays REQUIRED wherever the data is shown |
| the API's doors | ZERO new asks: the recipe pull touches the second host only (8 GETs a run, 304s after); the `recipes` verb sends nothing |
| schema | 7 → 8 additive, born fresh AND by hop; measured on a schema-7 scratch copy born by the PINNED v0.9.9 binary: `7 → 8`, 5 rows / 4 priced intact, `recipe` + `recipe__crafted` land, 0 recipe rows, a second open re-migrates NOTHING (byte-identical `status`), and the pinned v0.9.9 refuses the 8-file honest (rc 4, file untouched) |

## The two fixes that landed inside the release (said so, not swept)

* **`fix(db)` `be9e934`:** the crafted join's money word had NO copper twin. `RecipeRow.crafted_price_copper`
  existed in the struct but the builder had no start and the join site filled only the words —
  the compiler caught the missing field (E0063) BEFORE a money word could ship without its
  number. ADR 0015's contract needs both sides (the hover reads the number); the start rides
  `None` like every other by-value join start.
* **`fix(cli)` `24e02bc`:** a targeted ask on a whole-door stage. Measured on the REAL host
  BEFORE the fix: `sync --stage recipes --ids 25` answered **rc 0 with the whole 8-file pull**
  (1 240 entries written) — the flag did nothing the report said, exactly the shape ADR 0004
  refuses. The refusal lands BEFORE `open_at`/`build_client`, so it spends ZERO requests (measured
  on a scratch DB; the fixture saw nothing).

## The CI block grew — honestly, seven steps CI CAN see

34 step-lines + plant one → **41 step-lines + plant one** (verified by STEP-LINE count, not the
naive grep, which counts the comment's own `cargo run --` mentions). Three stage presses
(budgeted → resumed → re-pulled) and four door steps reading the fixture's 12 rows with ZERO
requests. The block's comment quotes the fixture's own measured numbers so no grep can drift;
the real set's counts stay in docs/API.md and this ADR (ADR 0011: CI asserts only what it runs).

## Measured (pre-tag)

Full numbers in the DEVLOG entry of the same day and in the ci/test commit messages: the eight
files' sizes and counts fetched live, the join facts against the user's file, the stage's three
presses on a fresh mirror (3 files/5 rows/4 requests → 5 files/7 rows/resumed 3 → 8 files/8 same
copies/0 written), the door's 12-row answer with every join state named, the hop audit, the
rc 2/rc 3/rc 4 scratch measurements, the `node --check` PARSE witness and the grep witnesses for
the three buttons + `data-tab="recipes"` + the CSS classes app.js draws.

## Chain

`af864b4` feat(db) (schema 8: fresh CREATE + hop 7→8 + the ladder's case) →
`615c561` feat(db) (`write_recipe` + `recipes_rows` + typed `RecipeReagent`) →
`8061749` feat(sync) (the stage: 8 conditional files, meta ETags, resume cursor, `--budget`
bounds FILES) → `656c116` feat(cli) (the `recipes` verb + `--profession` + stage whitelist) →
`be9e934` fix(db) (the copper twin) → `40e97c7` feat(ui) (the eighth tab + three buttons) →
`24e02bc` fix(cli) (the whole-door refusal) → `fbb2721` test(fixture) (the `/data/` grounds) →
`c9efa43` ci (41 step-lines) → `c0de045` chore (banner 0.10.0) → this record carries the
annotated tag `v0.10.0`.

## CI row

v0.10.0's chain+tag push (head = this record's tree, the TAGGED one) and the verification
fold's own push are measured FORWARD in the verification record and docs/DEVELOPMENT.md's
ladder; the ladder's earlier rows stay put, tags never move.
