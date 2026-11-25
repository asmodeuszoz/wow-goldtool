# ADR 0035: a PIN rides the DB's OWN `meta` keys `pin_recipe_<spell_id>` (ZERO schema line), the recipes door grows ONE bare `--pinned` flag, and the pinned tab is the panel's CONDITIONAL eighth nameplate

**Era:** v0.20.0 (the user's thirteenth ask, the half quoted verbatim: "if there are recipes pinned
this spawns a new tab 'pinned recipes', which holds all pinned recipes")

**Status:** ADOPTED — the PIN trio (`pin add` / `pin list` / `pin drop`) is born over the `meta` table,
the recipes LIST door grows ONE bare flag, the Recipes tab's `Watch the cooked items` affordance dies
ALL ITS HALVES at once, and the Pinned recipes tab rides as the SEVENTH nameplate — PAINTED off and
written visible ONLY while the pinned set answers ids. ZERO schema line, ZERO new table, ZERO new
column, ZERO new wire field, ZERO new money line, ZERO new request.

## The decision

- **A PIN is ONE `meta` key plus ONE stamp: `pin_recipe_<spell_id>` → the added-at stamp.** The `meta`
  table already speaks a PREFIXED-KEY vocabulary — `schema_version`, `list-page-<n>-etag`/`-more`/`-count`
  (written by `meta_page_key`) and `recipe_etag_<file>` (written by `recipe_etag_key`) — so a per-record
  stamp on ONE key is ALREADY this DB's shape, and a pin is the smallest member of it. ZERO schema line
  moved (schema `8` PUT, the CREATE block untouched, the file on disk never grows), ZERO new table, ZERO
  new column. The key NAMES the row it pins (built by `pin_key` from the ONE `PIN_PREFIX` const), and the
  write rides `SQL_SET_META`'s upsert PUT (via `meta_set`) — the SAME door the ETag and cursor writers
  already use — so re-pinning REFRESHES the stamp instead of doubling a key (`meta`'s PRIMARY KEY keeps a
  doubling from ever being possible; measured: `pin add 132` on an already-pinned id answered rc `0` at
  35 B and the table still carried exactly ONE row `pin_recipe_132` with the NEW stamp, `pin list` PUT).
- **The trio mirrors the `watch` TRIO's SHAPE, not its TABLE.** Usage entry sits after the watch trio
  (`pin add SPELL_ID [SPELL_ID ...] | pin list | pin drop SPELL_ID`), the verbs answer like `watch` does
  (`{"added": […], "at": N}` / the id array / the batch echo), and `pin drop` keeps the idempotent truth
  `SQL_WATCH_DROP` states: an un-pinned id removes ZERO rows and answers rc `0`, claiming NOTHING. The
  trio gets its OWN id pass (`collect_spell_ids`), because `collect_ids` names the ITEM namespace in its
  usage words ("watch wants numeric item ids") and would answer a spell-id ask with the WRONG namespace.
- **`pin add` GUARDS the whole batch BEFORE any write**, because a pin feeds a DRAW: `has_recipe_row`
  (`SQL_RECIPE_ROW_OF`) is the `SQL_ROW_OF_ID`/`has_row_of_id` guard `mark_missing_at` names in db.rs's
  OWN words — "see `has_row_of_id`, which callers must consult first" — and the refusal reuses the CARD
  door's OWN words (ADR 0025's no-invented-answers rule), so a pin over an id the recipe table never got
  answers rc `4` naming it. The batch is ALL-OR-NOTHING and measured rather than asserted: `pin add 132
  999999` answered rc `4` at 111 B naming 999999 while `pin list` answered the SAME `[]` BEFORE and AFTER
  — ZERO stored, ZERO lost.
- **The set read asks an EXACT prefix test, NEVER a LIKE.** `SQL_PINS_OF` = `SELECT key, substr(key, 12)
  FROM meta WHERE substr(key, 1, 11) = ? ORDER BY CAST(substr(key, 12) AS INTEGER) ASC` on a scratch DB
  speaking this DB's OWN schema lines (SQLite 3.54.1): a LIKE LAUNDERS — `key LIKE 'pin_recipe_%'` also
  answered the planted `pinXrecipeY7` and `PIN_recipe_7` (LIKE's `_` is a single-char WILDCARD and LIKE is
  CASE-INSENSITIVE), which would hand back keys the DB never pinned and call them pinned (ADR 0011's word).
  The door orders NUMERICALLY like `SQL_WATCH_LIST` orders `id ASC`, because TEXT order would answer
  `1 723` before `202` — a lie about the set's own size order — and the measure shows it (`pin add 1723 132`
  then `pin list` answered `[132,1723]`). The door PARSES the id tail in SQL (`substr(key, 12)`, the column
  it asks for, the id tail's start at `12` = `PIN_PREFIX`'s 11 chars + 1) and its decode error NAMES the key
  where the bytes are not what the writer described (ADR 0011's named-lies rule, the shape `recipes_rows`
  already rides for a `reagents_json`).
- **`--pinned` is ONE BARE FLAG on the recipes LIST door — ZERO new wire field.** The rows it returns ARE
  the pinned set, and `matching` answers the NARROWED count beside them (ADR 0009's "showing 40 of 1 250"
  rule kept for the new filter: `recipes --pinned --limit 1` answers 1 row of `matching 2`). The WHERE half
  is ONE clause — `SQL_WHERE_PINNED` = `EXISTS (SELECT 1 FROM meta AS pin WHERE pin.key = ? || CAST(spell_id
  AS TEXT))`, a key-EQUALITY join over `meta`'s OWN PRIMARY KEY (`key TEXT PRIMARY KEY`, docs/DATABASE.md's
  schema-8 CREATE line), so it costs ONE indexed lookup per recipe row (the door's EXPLAIN census answered
  `SeekGE`/`IdxGT`) and the SQL stays a CONSTANT size no matter how many pins a user carries. The flag rides
  PARSED-AND-IGNORED through the doors that never asked, so every through-ride stays BYTE-EQUAL.
- **The WHERE half now lives in ONE helper `recipes_where(profession, pinned)` BOTH recipes doors ride**
  (the rows read and the count echo). As of v0.19.0 each door WROTE ITS OWN copy of `"WHERE profession = ? "`
  — one rule in two places (ADR 0014's rule, the same one the paging clause answers twice since ADR 0022) —
  and clauses ride WITH their bound values in the TEXTUAL order, because SQLite numbers bare `?` by textual
  order over the whole line while `params` answers POSITIONALLY: a clause that moved without its value would
  bind the wrong needle. `Invocation` grows ONE PARSE field `pub pinned: bool`, not a wire field.
- **The door's OWN guard half rides PUT: a filter never launders a set and never launders a refusal.**
  `recipes --pinned` over ZERO pins answers the door's OWN filter-empty refusal at rc `4` 103 B, and so do
  `recipes --pinned --profession Cooking` / `--profession First Aid` at 103 B — the words already say
  "(for this filter)". The split was measured and REJECTED (see the rejects): a second refusal shape for
  the pin filter would be one rule in two places.
- **The pinned tab is CONDITIONAL, and a half-dead tab is a LIE (ADR 0034's word).** The nameplate button
  `tab-pinned` AND its card section `pinned-card` ride ONE class, `pin-off`, written by `set_pin_visible()`
  and nowhere else, and app.css's `.pin-off { display: none; }` is what hides BOTH — while the pinned set
  answers NO spell id the panel shows SIX activities and never a seventh holding NOTHING. `show_tab` stays
  PUT (it knows only `tab-on`/`tab-off`, so the pin truth is a class its own loop never writes: ADR 0029's
  `pip`/`pip-closed` precedent, kept for a section that IS an activity), and when the last pin is dropped
  the view walks back to the Recipes tab (`set_pin_visible` calls `show_tab("recipes")` ONLY while
  `active_tab === "pinned"`) so the ONE truth never leaves an invisible tab active. `boot()` grows ONE
  awaited LOCAL verb (`await refresh_pin()` between `refreshWatched()` and `runStats()`), so the first paint's
  conditional nameplate is already the DB's truth — read by `pin list`, ZERO requests, ZERO crawl.
- **The watch-cooked affordance dies ALL ITS HALVES in ONE commit** — its `btn-recipes-watch` button, its
  listener, its `watchRecipes()` function WITH its OWN comment block, and the census sentence in `runRecipes`'
  note that named it — because a nameplate over an absent function is a lie (ADR 0034's word kept: a HALF-dead
  tab is a lie). The PIN affordance is born in the SAME commit: the ask is ONE swap. The `watch` VERB, its
  table, the Writes tab's `btn-watch-hits` and the Details tab's `btn-detail-watch` stay PUT — they name the
  OTHER namespace: `watch` feeds `sync --watch` a CRAWL of ITEM ids, and a spell id is NOT an item id
  (ADR 0021/0025's measured 651 of 1 240 lands, only 20 of them `Recipe` rows).
- **The pinned table is the recipes table's OWN narrowed set, never a copy of it.** `table.pinned-hits` is
  JOINED to the recipes table's rules (`table.recipehits, table.pinned-hits` share the width rule; `#recipe-hits
  tr td, #pinned-hits tr td` and the hover rule share it; `#recipe-hits tr td button.openrecipe, #pinned-hits
  tr td button.openrecipe, #hits tr td button.pipbtn` stays ONE click shape — ADR 0031's rule that a list row
  owns its look in ONE place), and its rows are painted by the SAME `recipe_row` the Recipes tab uses, so its
  `col-job`/`col-recipe`/`col-learned`/`col-skill`/`col-mats`/`col-cost`/`col-cooked` cells, its money, its
  learned words and its row-click window opener are ONE code path. Its delegated handler is the recipes table's
  OWN shape (`dataset.spell` → `runRecipeCard`). ZERO money line moved (ADR 0015/0023's contract: money beside
  EVERY copper, NEVER beside a count — the pin era draws no new money cell).

## Measured (this era's own witnesses — fresh ports 9945 / 9946 / 9947 / 9948 / 9951 / 9952 / 9953 / 9954 / 9987, fresh processes/DBs, ZERO real-host, the user's file read read-only and NEVER opened by a binary)

- **Census BEFORE one line moved** (off the tree at the tagged v0.19.0 record `d37866d`): the Recipes tab
  held ONE watch affordance — `btn-recipes-watch` 1/3/0 across `index.html`/`app.js`/`app.css`, `watchRecipes`
  0/2/0 — SIX nameplate buttons, `class="tab` 7 lines, `data-tab=` attrs 13, SEVEN `section[data-tab]`, `index.html`
  285 / `app.js` 1 488 / `app.css` 499 lines, and `node --check` PARSE-OK. Measured on the tree the era's FIRST
  commit rode: `recipes --pinned` answered rc `2` at 49 B `unknown flag --pinned (run help to see the verbs)`, and
  bare `pin` answered `unknown verb pin (run help to see the verbs)` at 44 B — the era's BEFORE witness, re-measured
  post-tag against a rebuilt worktree @ TAGGED `d37866d`.
- **Census AFTER:** SEVEN nameplate buttons, `class="tab` 8, `data-tab=` attrs 15 (7 buttons + 8 sections),
  EIGHT `section[data-tab]`, `index.html` 337 / `app.js` 1 638 / `app.css` 518 lines; `watchRecipes` ZERO,
  `btn-recipes-watch` 0/2/0 (the TWO survivors are the comment sentences that name the dead button BY NAME as
  history, never a listener); `btn-recipes-pin` 1/3/1 · `btn-pinned-read` 1/5/1 · `btn-pinned-unpin` 1/4/0 ·
  `tab-pinned` 1/1/0 · `pinned-card` 1/1/0 · `pinned-hits` 1/3/4 · `refresh_pin` 1/4/1 · `set_pin_visible` 0/2/1 ·
  `pin-off` 5/6/2; `node --check` PARSE-OK, the DOM NEVER WITNESSED (the launch-audit rule). Era diff EIGHT files /
  **645 ins / 63 del**: `src/cli.rs` 114 · `src/db.rs` 257 · `ui/app.js` 200 · `ui/index.html` 62 · `ui/app.css` 27 ·
  `ci.yml` 44 · the banner pair 2/2.
- **The doors, measured over a mirror-seeded DB** (the fixture's OWN 12 recipe rows, the app's OWN `sync --stage
  recipes` wrote them, tallies PUT at 51 GETs / 17 `/data/` recipe-file GETs, ZERO real host): `pin add 1723 132`
  rc `0` at 40 B → `pin list` 11 B `[132,1723]` md5 `9cf8ae5f` (numeric order; TEXT order would answer `[1723,132]`)
  · `recipes --pinned` rc `0` at 2 180/2 450 B answering `matching 2` (bytes ride the RUN's price seed) · `recipes
  --pinned --icons --costs --learned` rc `0` at 10 175/13 070 B with the SAME `matching 2` · `recipes --pinned --limit
  1` rc `0` at 981/1 249 B with `matching 2` while ONE row is shown (paging rides, `matching` says the SET) · `pin
  drop 132` rc `0` at 6 B → `pin list` 7 B `[1723]` md5 `eaa01f8e` · `pin drop 999999` rc `0` at 9 B `[999999]` with
  `pin list` BYTE-IDENTICAL before and after (ZERO removed, NOTHING claimed).
- **The guard, measured both ways:** over the SAME seeded DB `pin add 999999` answered rc `4` at 111 B naming the
  id, and `pin add 1723 999999` answered rc `4` at 111 B naming 999999 with `pin list` answering the SAME 11 B md5
  `9cf8ae5f` before and after — the batch stores NOTHING or ALL, measured. Over a FRESH app-born DB whose recipe
  table holds NO rows (`meta` a LONE `schema_version` key): `pin add 132 999999` rc `4` at 108 B naming the FIRST
  such id, `pin add 999999` alone rc `4` at 111 B, `meta` still a LONE `schema_version` key; while `watch add
  999999` on that SAME fresh DB answered rc `0` and STORED (the watch table then answering 1 row) — the measured
  contrast the OTHER way, because `watch`'s table feeds a CRAWL of ITEM ids whose doors answer honestly about an
  id they never saw. Refusals ride STDERR with stdout 0 B: bare `pin` rc `2` at 60 B naming the trio's OWN shape
  (`pin add SPELL_ID [SPELL_ID ...] | pin list | pin drop SPELL_ID` measures 60 B either way the usage words are
  written), `pin add` rc `2` at 35 B naming the action it wants, `pin foo` rc `2` at 35 B, `pin add abc` rc `2` at
  36 B naming the SPELL namespace ("pin wants numeric spell ids, got abc"), `pin list` rc `0` at 3 B `[]` (the pinned
  set over an empty recipe table answers the EMPTY set, it does not refuse).
- **The SQL shapes, measured on scratch DBs speaking this DB's OWN schema lines (SQLite 3.54.1):** the bare aliased
  EXISTS `EXISTS (meta AS pin WHERE pin.key = ?)` answers `near "meta": syntax error` OUTRIGHT, so the subquery form
  rides · a LIKE LAUNDERS (its `_` is a single-char WILDCARD and LIKE is CASE-INSENSITIVE: the planted `pinXrecipeY7`
  and `PIN_recipe_7` answered a `key LIKE 'pin_recipe_%'` read) · GLOB is case-sensitive but matches degenerate
  tails · `spell_id IN (SELECT CAST(substr(key, 12) AS INTEGER)` measures the SAME rows but bakes the prefix's LENGTH
  into the door's SQL TWICE · `substr(key, 1, 11) = ?` answers ONLY the true pins · EXISTS params bind the prefix
  FIRST and the caller appends the `paging_clause` values LAST, and with ZERO pins the clause answers an EMPTY set and
  the count half answers `0`, NOT an error · the join compiles to `SeekGE`/`IdxGT` (an indexed PRIMARY KEY lookup),
  and NO `EXISTS`/`GLOB` existed anywhere else in this DB's SQL before this era (the LIKE that escapes with `#` lives
  in `query.rs`, a DIFFERENT door).
- **Mirror of the LANDED block (fresh 9952 + the pair's OWN seed at 9987): 58 commands rc `0`, ZERO NONZERO** (the
  driver's OWN parse: 57 `cargo run --` step-lines + plant one, EXECs the ALREADY-BUILT binary so no compile is in
  flight during an audit, ADR 0006's driver-side lesson). The FIVE pin steps answered rc `0` at 39 / 10 / 13 070 /
  6 / 6 B, and the fixture's OWN tallies stayed PUT across them at **51 GETs / 17 `/data/` recipe-file GETs with
  ZERO GETs ADDED** — the SAME category census the v0.19.0 windows carried (detail 18 · icon-file 5 · tooltip 8 ·
  live-search 3 · recipe files 17). The unchanged tally IS the locality proof ADR 0012 keeps: the pin doors and the
  pinned draw are LOCAL.
- **Exit map GROWN (fresh 9953/9954, one fresh window seeding TWO DBs — A the full block, B with a genuinely EMPTY
  recipe table — plus a DEAD port):** **67 doors at `0`×41 · `2`×9 · `4`×17**, the PIN family alone 26 doors at
  `0`×16 · `2`×4 · `4`×6, and the v0.19.0 base riding PUT at **`0`×25 · `2`×5 · `4`×11 — the SAME totals** (the
  composition is stated in the driver: base 41 = the nine card doors + the extra word + the DEAD-host through-ride +
  v0.19.0's recipes/costs/learned shapes + the live-search pair, then the pin family in a deterministic order with
  the WRITES LAST). The new slots: bare `pin` rc `2`, `pin foo`/`pin add`/`pin add abc` rc `2`, the guard's rc `4`
  pair (both over a seeded DB and over the EMPTY one), `pin drop` un-pinned rc `0`, the narrowed rc `4` over ZERO pins,
  the pinned reads rc `0` (numeric order re-measured, the drop re-measured, the set emptied and re-refused). Through-rides
  BYTE-EQUAL with ZERO pins stored: `item 25 --cached` == itself 4 392 == 4 392 B · `search sword` 188 == 188 B ·
  `stats --limit 3` 1 533 == 1 533 B · `status` 1 103 == 1 103 B · `recipe 132` 2 119 == 2 119 B (the CARD door ignores
  the flag: ONE entry, ONE spell id, the flag names no filter there) · `search sword --where description` 763 == 763 B —
  bytes ride the process's OWN seed, the identity and the rc totals do not.
- **Pinned pair vs the v0.19.0 frame @ TAGGED `d37866d`** (a rebuilt worktree, banner `0.19.0`, its OWN build green
  naming ZERO): **28/28 doors BYTE-IDENTICAL + 13/13 md5-equal** — ONE fresh fixture process, ONE fresh DB seeded by
  the NEW binary's OWN crawl (the LANDED 57-line block, 58 commands ZERO NONZERO), and BOTH binaries pressing their OWN
  fresh `cp` copy of the user's 44 MB file for the LOCAL doors. This era GREW code (the flag, the helper, the trio),
  so this is the era's REGRESSION proof: the un-narrowed doors' answers did not move ONE byte. The era's BEFORE witness
  rides the SAME pair: the tagged frame answers rc `2` at 44 B `unknown verb pin (run help to see the verbs)` for
  `pin`/`pin list`/`pin add 132` and rc `2` at 49 B `unknown flag --pinned (run help to see the verbs)` for `recipes
  --pinned`, while the new tree answers `pin list` rc `0` 6 B, bare `pin` rc `2` 60 B naming its OWN shape, `pin add
  132` rc `0` 35 B, `recipes --pinned` rc `0` 1 249 B, `recipes --pinned --profession Cooking` rc `4` 103 B. Over the
  user's OWN copy: `pin list` rc `0` at 3 B `[]` (the file carries ZERO pins today), `recipes --pinned` rc `4` at 103
  B (the door's OWN filter-empty refusal, `--profession Cooking` the SAME 103 B), while `recipes --limit 3` rc `0` 4 293
  B md5 `bbc17f2d` and the Blacksmithing step rc `0` 21 090 B md5 `c1fe9d13` answered PUT on BOTH binaries.
- **The user's OWN file, read-only (python `sqlite3` over `file:…?immutable=1&mode=ro`, SQLite 3.54.1, NEVER written,
  NEVER crawled, NEVER opened by a binary):** schema `8` PUT · `meta` answers **588** rows and ZERO of them carry the
  `pin_recipe_` prefix — so the user's pinned set is EMPTY today and the conditional Pinned nameplate shows NOTHING on
  their screen today · 1 240 recipe rows / **1 240 DISTINCT** spell ids, so a pin ALWAYS names a real row and the guard
  can refuse nothing this tab shows · `watch` 149 today (the v0.19.0 windows measured 134: the user kept a watch list
  between windows, a DB fact said so, not a moved thing) · size 44 023 808 B + mtime-ns `1 795 566 381 543 858 680`
  verified IDENTICAL before and after the pair.
- **Gates:** `cargo build` rc `0` naming ZERO at EVERY commit (the incremental hard-link line is the FS-artifact class on
  this file system, never a NAMED warning); `cargo clippy --all-targets` **0 errors / 843 advice (843 `-->` arrows,
  844 on the warm second pass)** — the advice SET MOVED from v0.19.0's 824, and that IS the claim of an era whose Rust
  GREW ~370 lines (said so; the `-D warnings` gate stays OFF, the truth-of-ZERO remains the NAMED build-warning set).
- **Measured defects: TWO, recorded honestly, fixed FORWARD.** (1) `ui/app.js:33`'s census comment claimed "SEVEN
  nameplates and EIGHT `data-tab` sections" while the pushed v0.19.0 tree answers SIX nameplates + SEVEN sections +
  13 attrs — v0.19.0's tab death never grew that comment, so the comment LIED for ONE era; after this era's UI commit
  the DOM answers SEVEN buttons / EIGHT sections / 15 attrs, so the numbers are TRUE again AND this era grew the comment
  with the note naming the CONDITIONAL nameplate (the comment is honest on the tree that shipped, measured by grep).
  (2) `README.md`'s Status paragraph still opens "THE v0.16.0" while the ladder carries `v0.20.0` — four eras rode it
  PUT (the record measured `Status:` lines at `d37866d` / `70d2f20` / `473a63f` / `026d311`, all `v0.16.0`), so the
  paragraph LAGS its own table; the row pattern held and the paragraph's growth lands FORWARD.

## REJECTED alternatives (with their measures)

| rejected | why (measured the same day) |
| -------- | ------------------------- |
| ride the PIN on the `watch` table (a spell id there) | two namespaces, and the table lies about BOTH: `watch` feeds `sync --watch` a CRAWL of ITEM ids, and a spell id is NOT an item id — 651 of 1 240 spell ids land on an item row, only **20** of those carry `item_type Recipe` (docs/DATABASE.md + API.md:304 carry the measure). Measured on a fresh app-born DB: `watch add 999999` answered rc `0` and STORED, so the crawl would then ask for ids the catalogue never holds; and the pinned draw would read the WRONG table. |
| grow a `pin` table / a schema hop | a pin is ONE key plus ONE stamp, and the `meta` table ALREADY carries per-record keys with stamps (`list-page-<n>-etag`, `recipe_etag_<file>`), so a hop moves a schema line for NOTHING; measured: schema `8` PUT, the CREATE block untouched, the file on disk never grows, and the pinned pair stays 28/28 BYTE-IDENTICAL with the hop ABSENT. |
| ask the set with a LIKE (`key LIKE 'pin_recipe_%'`) | LIKE LAUNDERS: its `_` is a single-char WILDCARD and LIKE is CASE-INSENSITIVE, measured on a scratch DB speaking this DB's OWN schema lines — the planted `pinXrecipeY7` and `PIN_recipe_7` answered that read. A LIKE hands back keys the DB never pinned and CALLS them pinned (ADR 0011's word). |
| ask the set with a GLOB | case-sensitive (unlike LIKE) but it matches DEGENERATE tails: measured, `pin_recipe_` + anything answered, so a key whose tail is not a number would be called a pin; the EXACT prefix test answers ONLY the true pins. |
| the bare aliased EXISTS (`EXISTS (meta AS pin WHERE pin.key = ?)`) | SQLite refuses it OUTRIGHT: `near "meta": syntax error` (3.54.1). The subquery form rides. |
| `spell_id IN (SELECT CAST(substr(key, 12) AS INTEGER) …)` | measures the SAME rows, but bakes the prefix's LENGTH (`12`) into the door's SQL TWICE — a length the ONE `PIN_PREFIX` const owns; and the door's count half and rows half then disagree the next time the prefix moves. |
| a PERMANENT (always-visible) Pinned tab | a half-dead tab is a LIE (ADR 0034's word, re-measured HERE): with the pinned set EMPTY today — measured TRUE on the user's OWN file, ZERO `pin_recipe_` keys among 588 `meta` rows — a seventh nameplate would paint a tab whose table answers NOTHING forever, and `recipes --pinned` would answer the door's refusal (rc `4`, 103 B) INSIDE a tab that exists. The CONDITIONAL nameplate never exists while the set is empty. |
| a NEW wire field (`"pinned": true` in the answer, or a second `pinned` array) | a second shape for ONE set: the door's answer keeps its OWN shape `{"profession", "matching", "recipes"}` PUT, the rows it returns ARE the pinned set and `matching` answers the narrowed count. Measured: the pinned read at 13 070 B carries the SAME cells the un-narrowed read carries; a new field would move ADR 0009's count rule AND the UI's reader. |
| split the guard: a pin-specific refusal when the pin filter answers nothing | ONE rule in two places (ADR 0014's rule, ADR 0032's measured precedent that a flag cannot launder a refusal): `recipes --pinned` over ZERO pins already answers the door's OWN filter-empty refusal at 103 B, and its words ALREADY say "(for this filter)" — measured `recipes --pinned` rc `4` 103 B, `recipes --pinned --profession Cooking` rc `4` 103 B, `recipes --profession Nope` rc `4` 103 B. The split was authored and DROPPED, never landed. |
| pin the COOKED ITEMS of a recipe (item ids) instead of the recipe itself | the ask names recipes; and the cook is a different thing — the crafted item is already the row the `watch` trio owns (that's the affordance this era REPLACED). Pinning the item would put a pin on a row the recipe entry only mentions: 133 entries name NO crafted item (all Enchanting) and 10 crafted ids have no row, so a pin would be unrepresentable for 143 of 1 240 entries — measured on the fetched set. |
| keep the pin in the UI only (a JS array in `app.js`) | the pinned set must survive a restart AND a `cargo run -- recipes --pinned` line: ADR 0014's one-code-path rule says a click and a cargo line are ONE code path, and the pinned DRAW is the door's. A UI-only array would have the door answer NOTHING about what the UI remembers. |
| drop the `matching` echo for the pinned read (count only the shown page) | ADR 0009's rule: `showing 40 of 1 250` is a LIE. Measured: `recipes --pinned --limit 1` answers 1 row of `matching 2`. |

## Pointers

- **ADR 0018 GROWN (this era):** the strip answered SIX activities through this truth; SEVEN answer it NOW —
  SEVEN nameplate buttons, EIGHT `section[data-tab]`, `data-tab=` attrs 13 → 15 (the new button and its new section
  carried ONE each), `class="tab` lines 7 → 8. The ONE truth `show_tab` stayed PUT word-for-word: the conditional
  nameplate's hide is a class its loop never reads or writes (`pin-off`, not `tab-off`), which is ADR 0029's
  `pip`/`pip-closed` mechanism kept for a section that IS an activity.
- **ADR 0034 RIDDEN (this era):** "a HALF-dead tab is a LIE" is the word this era rides — the watch-cooked affordance
  died ALL ITS HALVES in ONE commit (button + listener + function WITH its OWN comment block + the census sentence
  naming it), and the Pinned tab is CONDITIONAL rather than permanently empty for the SAME reason. ADR 0034's OWN
  verdict (the live-search DOOR rides PUT) stayed PUT: the era moved ZERO door line, and the CI block's THREE
  live-search steps rode into the era's OWN run.
- **ADR 0032 RIDDEN (this era):** a flag cannot launder a refusal, and the pin filter keeps the door's OWN refusal
  (measured: `recipes --pinned` over ZERO pins rc `4` at 103 B, the SAME words as the un-flagged filter-empty door).
  ADR 0032's cost machinery (`cost_cell`, `live_cell`, the money contract) rode PUT UNTOUCHED — the pin era moved
  ZERO money line.
- **ADR 0009 RIDDEN (this era):** the `matching` echo answers the NARROWED count beside the pinned rows (measured
  `matching 2` over two pins and `matching 2` with `--limit 1` showing ONE row), the promise the count rule carries for
  a narrowed set.
- **ADR 0014 GROWN (this era):** the recipes door's WHERE half was ONE rule in TWO places (each door wrote its own
  `"WHERE profession = ? "` copy); ONE `recipes_where` helper now answers BOTH doors, and clauses ride WITH their bound
  values in textual order. The PIN's UI read rides the SAME argv the CLI answers (`recipes --pinned --icons --costs
  --learned` = a click and a `cargo run --` line), so the pinned rows' money, icons, learned words and clicks are the
  Recipes tab's, never a second source.
- **ADR 0029 RIDDEN (this era):** the `pip`/`pip-closed` precedent — a visibility class that `show_tab` never touches —
  is what the CONDITIONAL nameplate rides (`pin-off`), and the card windows' machinery stayed PUT.
- **ADR 0031 RIDDEN (this era):** a list row owns its look in ONE place — `table.pinned-hits` is JOINED to the recipes
  table's rules, never copied, and its rows are drawn by the SAME `recipe_row` with the SAME `col-*` classes.
- **ADR 0022 RIDDEN (this era):** the paging clause stays ONE rule per door and its values are appended LAST, after the
  WHERE half's values — measured with `--limit 1` over a pinned set of 2 (1 row of `matching 2`).
- **ADR 0021 / ADR 0025 RIDDEN (the namespace measure):** 651 of 1 240 spell ids land on an item row, only 20 of them
  `Recipe`-typed; the card door's OWN refusal words are what a pin refusal reuses; 1 240 recipe rows / 1 240 distinct
  spell ids on the user's file is what makes the guard unable to refuse a shown row.
