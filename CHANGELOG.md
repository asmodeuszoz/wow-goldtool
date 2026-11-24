# Changelog

All notable changes to **WoW Goldtool** are documented here, following
[SemVer](https://semver.org/) and [Conventional Commits](https://www.conventionalcommits.org/).
`Added` / `Changed` / `Fixed` / `Removed` follow the [Keep a Changelog](https://keepachangelog.com/)
sections. The full narrative of *why* each change was made lives in
[docs/DEVLOG.md](./docs/DEVLOG.md); each entry below points at the matching log entry and git tag.

## v0.17.0 — the SEARCH TAB owns its list surface (icons move BETWEEN id and name, grow 24 → 32 px under id-scoped rules) AND the two recipe doors answer the TOTAL CRAFTING COST: ONE `--costs` flag, ONE `craft_cost` behind the tab column + card line, money ONLY beside a COMPLETE total · 2026-11-24 · tag `v0.17.0`

The user's tenth ask (verbatim, its own typos kept): "Somne items to work on for the next version:in
tahe search tab show the icons between ID and name instead. Also make those icons a little larger. In
the recipes tab and detail cards add a column with the total crafting cost based on live prcices." TWO
fact families ride ONE banner move: the icon position/size ([ADR 0031](./DECISIONS/0031-the-search-list-owns-its-icon-position-and-size.md),
census + SEVEN rejects) and the cost ([ADR 0032](./DECISIONS/0032-the-total-crafting-cost-on-the-recipe-doors.md),
flag/helper/states census + SEVEN rejects). The ask names BOTH icon neighbours (absolute placement,
not a swap hint) and says `a little larger` (a notch — the card's 72 px is the NEXT jump, rejected with
measures); it says `live prcices` — and the live layer `price_min_now` is EXACTLY why the stored min
never feeds the cost (ADR 0023 measured the two differing ON THE SAME row).

Changed

- **The icon cell rides BETWEEN the id and the name** (`feat(ui) f064e2c`, TWO files, 9 ins / 2 del):
  the `#hits` thead's `col-icon` th moves BEFORE `col-name` (`ui/index.html:107`) and `hit_row` draws
  its `iconcell` td BEFORE the `namecell` td — the SAME reorder in BOTH halves, in ONE commit, because
  a header label sits ABOVE its cell POSITIONALLY (table layout; the move-in-the-row-only alternative
  would make the header LIE — rejected at the ADR). ZERO CSS moved for the MOVE itself (measured before
  one line moved: the header's ONE rule at `app.css:227` is class-keyed and joined — order-independent).
  **The click stayed PUT word-for-word**: the opener button lives in its OWN `namecell` wherever that
  cell sits (ADR 0030's opener, the delegated split BY CLASS, `runCard_from_row`, the `item ID --cached`
  door — the pair's search doors BYTE-IDENTICAL + `search sword --costs` 188 == 188 B through-ride say
  so); the OTHER tables' theads/rows rode PUT (mini keeps `group | id | name`).
- **The Search table's row icons grow 24 → 32 px, its icon cell 34 → 44 px** (`feat(ui) fe1b278`,
  ONE file `ui/app.css`, 15 ins): TWO id-scoped rules — `#hits .listicon {32 px}` + `#hits td.iconcell
  {width: 44px}` — the ADR 0027 id-scoping lesson kept (a bare `.listicon` edit would move the Recipe
  rows' material + cooked cells too: 3 778 of them over the user's file): plain `.listicon` STAYS 24 px,
  `table.mini .listicon` STAYS 20 px, the card's 72 px + its 21 id rules ride PUT (grep census: 2 base
  rules PUT, 21 card rules STILL 21); the cell rides the SAME notch (44 − the base rule's 12 px padding
  = 32 content exactly); ZERO new class, ZERO new colour token, the SAME `icon_cell` draws.

Added

- **The two recipe doors grow ONE flag `--costs`** (`feat(cli,db) 5c068d5`, TWO files, 134 ins / 6 del —
  the era's Rust step): the usage tail grew on the TWO door entries ONLY (`recipes ... [--costs]`,
  `recipe SPELL_ID [--icons] [--costs]`) — ZERO new verb, the SAME doors the tab clicks (a second door
  the rows could DISAGREE through was rejected at ADR 0032); `Invocation.costs` + `bare_flag` + parse
  ONE line each. The cost: ONE `CostTally` + ONE `craft_cost(reagents)` BOTH doors call AFTER their OWN
  joins — Σ (count>1 ? count : 1) × the join's `price_now_copper`: the DB's **LIVE** layer (the ask says
  live; NEVER the stored `price_min`), ZERO new SQL, ZERO request (LOCAL doors: the replay kept the
  crawl's tally at **51 GETs / 17 recipe-file GETs PUT** — that unchanged tally IS the locality proof,
  ADR 0012). Money = the frame's OWN `money_of` beside the copper, riding ONLY when EVERY material has
  a live price (ADR 0015: money beside EVERY copper it labels); an incomplete set answers the COUNT
  `cost_missing` with words, NEVER an invented zero (ADR 0004) and NEVER money beside a count — the
  fixture card 132 measured `cost_missing 2` with `cost_copper`/`cost_money` ABSENT (serde's skip even
  strips an incomplete total's money fields WHEN asked). The THREE fields on `RecipeRow` + `RecipeCard`
  ride serde `skip_serializing_if` — unasked the answer keeps its bytes EXACTLY (pinned pair **28/28
  BYTE-IDENTICAL + 13/13 md5-equal** vs the v0.14.0 worktree @ TAGGED `c065815`, fresh 9911; asked bytes
  ADD: `recipes --limit 3` 3 377 → 3 428 B, `recipe 467` 1 209 → 1 272 B; the PINNED binary REFUSES all
  THREE `--costs` shapes rc `2` `unknown flag --costs (run help to see the verbs)` — the BEFORE witness
  the flag is NEW not renamed, measured on the scratch pair, NEVER a green CI line, ADR 0011's split).
  Schema **`8` PUT** (read-only re-read says `8`): cost is DERIVED at answer time, NEVER a stored
  column (the stored-cost alternative would AGGE behind the live twin — the ADR's measure). The count
  multiplier rides the set's OWN stated `count`: the fresh user census read 3 778/3 778 material cells
  STATING a count (1 264 × 1 / 2 514 × >1 / **0 NULL**) — so "NULL reads as ONE" is a stated rule,
  never a measured need.
- **The Recipes tab grows a `col-cost` column and the recipe card a cost LINE** (`feat(ui) 96496cb`,
  TWO files, 44 ins / 7 del): ONE `cost_cell(one, mats)` answers BOTH surfaces — the tab td AND the
  card's `rcardcost` div — FOUR honest states: complete rides `money_cell` (words + the EXACT copper
  on hover), partial `cost incomplete - N of M materials have no live price`, all-unpriceable
  `no live prices yet - the price refresh must write them`, empty `nothing to cost - the set states
  no materials` — COUNTS with words, ZERO money beside a count (ADR 0015's OTHER half). The tabs ASK:
  tab argv `["recipes","--icons","--costs"]` + card `["recipe",N,"--icons","--costs"]` — EXACTLY 2
  `--costs` sites in the UI, so a `cargo run -- recipes --icons --costs` / `cargo run -- recipe N
  --icons --costs` line answers the SAME bytes (ADR 0014's one-code-path restated at the card); the
  tab's note grew ONE clause naming the cost rule beside every other column's truth (ADR 0019's note
  discipline kept).
- **The cost's paint rides the frame's OWN looks** (`feat(ui) 446f85d`, TWO files, 18 ins / 6 del):
  `th.col-cost` JOINS the ONE recipe-header rule and `.rcardcost` JOINS the rcard words rule (ONE look,
  joined selector, ZERO second copy — the v0.16.0 era's own frame lesson kept), and ONE new class
  `.costnote` (italic like the mat-none sibling it sits under, ZERO new colour token) carries the THREE
  WORD states — they are notes, not chips, because the counts they carry ride words beside them and
  NEVER money; the JS class string moved costchip → costnote in the SAME commit (drawn words ride PUT).
- **CI grows THREE `--costs` step-lines 49 → 52, plant line stays ONE** (`chore(ci) 4b6c024`, ONE
  file, 50 ins): the list step = the tab's OWN argv `cargo run -- recipes --icons --costs` after the
  v0.14.0 tab step, and TWO card steps `recipe 467 --costs` (the COMPLETE card) + `recipe 132 --icons
  --costs` (the honest-PARTIAL) after the v0.12.0 card quartet — every number quoted in the block's
  comments was MEASURED on a FRESH mirror BEFORE the block landed (driver parsed the committed 49 and
  pressed THESE THREE extra: 52/52 rc `0`, tally PUT, 12/12 rows carry `cost_missing`, money rides 1/12
  — the fixture's cost rides the RUN's moving price seeds: 14 604 copper / `1g 46s 4c` one window vs
  19 340 / `1g 93s 40c` the next, bytes 47 017 / 1 269 / 7 345 vs 47 027 / 1 272 / 7 347 — bytes ride
  the run, TALLIES do not); the rc-2 refusal pair stays measured-on-scratch, not a green line.

Measured (the era's OWN witnesses — fresh ports 9931/9941/9951/9961/9995/9911, FRESH processes/DBs,
ZERO real-host for the audit crawl):

- **Mirror, committed block replayed (fresh 9961): 52/52 rc `0`, ZERO NONZERO** (parse: 52 step-lines
  committed, plant one; presses UNCHANGED; fixture tally ends where the crawl left it **51 GETs / 17
  recipe GETs, ZERO added** = the cost doors ask NOTHING); the pre-landing candidate mirror (9931)
  measured the SAME 52/52 with the THREE new shapes BEFORE the block landed — the honest order the CI
  comment names.
- **Exit map fresh 9995: 36 doors = the BASE 31 at the SAME totals `0`×19 · `2`×5 · `4`×7** (every era
  since v0.15.0 kept these — kept AGAIN) + the era's FIVE `--costs` shapes: `0`×3 (`recipes --costs
  --limit 3`, `recipe 467 --costs`, and `search sword --costs` — the through-ride: flag PARSES and the
  door ignores, answers BYTE-EQUAL, 188 == 188 B) + `4`×2 (`recipe 999999 --costs` = the door's OWN
  no-row words grown with the flag, and `recipes --costs` over the EMPTY recipe table = the SAME no-rows
  refusal BEFORE any rows read — the flag cannot turn a refusal into an empty success, ADR 0004's
  rule). Money census over the doors' OWN answers: **0 pairing defects** grown with the new cost
  answers (the 12/13 `cost_missing` counts wear ZERO money).
- **Pinned v0.14.0 pair (fresh 9911, rebuilt worktree @ TAGGED `c065815`): 28/28 BYTE-IDENTICAL + 13/13
  md5-equal** (md5s pinned in this record: `stats` 83e3e784 · `stats --icons` 3cef3911 · `stats --limit
  3` b490fe3e · `item 25 --cached` 6f14fe9d · `item 40` 5ad258b3 · `item 12809 --cached` a22fd7b4 ·
  `status` 48bba880 · `watch list` dc0cb5ab · `refresh-twins` ea136265 · `search sword --where description`
  51482bc0 · Blacksmithing door ce77da4b · `recipe 132 --icons` ded6e551 · `recipe 638` d21b4486) — the
  cost fields adding ZERO bytes unasked, measured.
- **User file UNCHANGED** (`sqlite3 -readonly`, NEVER written, NEVER crawled): schema `8` (meta key
  `schema_version`) · 19 258 items · 4 602 tooltip copies / 14 656 without · 1 240 recipes / **1 240
  DISTINCT** `spell_id` · watch **134** · `vendor_sell` 4 231 · `price_min_now` **4 344** + **258**
  checked-empty · `vendor_buy` **0** · trend **0** · deleted **0**; the cost census over the SAME read-only
  set: **3 778 material cells → 3 642 live-priced, 136 WITHOUT live = 135 alive rows with NULL-live + 1
  id with NO row AT ALL** — said so honestly: an earlier dump read mis-split this 136/0, the fresh re-read
  splits 135/1, the ADR carries the fresh split) and rows **1 146 complete / 94 partial / 0 all-unpriceable**,
  counts 1 264 × 1 / 2 514 × >1 / 0 NULL.
- **UI launch witnesses (parse/grep ONLY — the webview NEVER renders in the build room, DOM cost NOT
  WITNESSED, NOT CLAIMED, the rule every record since v0.14.0 carries)**: `node --check` PARSE OK; thead
  `col-icon` between `col-id`/`col-name` grep-true; `hit_row` draws `iconcell` before `namecell`; TWO
  `#hits` icon rules; the TWO base rules PUT (24 px / 34 px); `table.mini` 20 px PUT; the card's 72 px +
  **21 id rules STILL 21**; `col-cost` EXACTLY ×1 in html/js/css; `--costs` argv sites EXACTLY 2; `data-tab`
  attrs STILL 15; the joined opener rule STILL ONE; nameplates STILL 7.
- **Builds named ZERO on every commit** (`cargo build` rc `0`; the hard-link incremental-cache notice is
  the FS artifact, never a named warning); `cargo clippy --all-targets` **0 errors / 822 advice lines —
  the number MOVED from the 811 of v0.16.0 because Rust GREW this era** (stated honestly, ADR 0011:
  expected-sets claimed only where a truth of ZERO holds; `-D warnings` gate stays OFF).
- **Measured DEFECTS: ZERO.** Said-so NOTES (not defects): the map's no-argv door `[]` HANGS because it
  LAUNCHES THE GUI — NEVER a CLI shape in an audit; bare `search` answers rc `0` (the empty-word search is
  legal); an unwatched `watch drop` answers rc `0` idempotent — the era's map rode TRUE usage/refusal doors
  (`live-search` bare rc `2`, `undelete-item 25` rc `4`) for those two slots.
- **The ladder row OWED by the v0.16.0 FOLD — measured (run #53 = 45268096674, push head = `2b18a7e`,
  the fold's OWN commit)**: attempt 1 — the **macos job CANCELLED by GitHub** (0 steps, EMPTY log,
  `ended_at` null: a SCHEDULING fact of GitHub cancelling the job, NOT a tree fact — the OTHER 4 jobs
  SUCCESS, the `test` job pressing **49 `cargo run --` steps green** on the pushed fold tree); attempt
  2 (via `gh run rerun --failed`) = **SUCCESS 5/5**: run started 2026-11-24T15:02:39Z, macos job (id
  138539803263, attempt 2) re-created 15:02:44Z and started 15:12:48Z, run closed at **15:19:55Z**; the
  macos re-run's own log: **12 end-actions, 11 success + ONE skipped** (the Ubuntu-libs installer's
  inner action — macos never builds it), apt line **0** on the macos log. Where jobs land stays a
  scheduling fact, said so rather than smoothed.

**Chain + honest close:** banner MOVED `0.16.0 → 0.17.0` (`chore ef9da01`; Cargo.lock's OWN line follows
in the SAME commit; ZERO Rust line in THIS commit); ADR 0031 + 0032 carry the censuses + SEVEN rejects
each; pointers: 0014/0015/0021/0025/0027 GROWN, 0016/0023/0030 RIDDEN. Tags never moved. Chain: `f064e2c`
feat(ui) · `fe1b278` feat(ui) · `5c068d5` feat(cli,db) · `96496cb` feat(ui) · `446f85d` feat(ui) ·
`4b6c024` chore(ci) · `ef9da01` chore · this record + tag `v0.17.0`.

## v0.16.0 — the SEARCH ROWS open their windows BY NAME: the name cell becomes the opener button, the separate `card` chip label dies, and the look rides the recipe name's ONE rule by JOINED selector · 2026-11-23 · tag `v0.16.0`

The user's ninth ask: "make it so in the search results I can click the item name just like it is
for the recipes." The Recipes tab already answers by its names (ADR 0025 drew the recipe name cell
AS the click target); v0.15.0 gave the search rows an opener BESIDE a plain-text name — a chip LABELED
`card` riding `.rowact`'s chip-box furniture (measured off the tree at `00f94ba` before one line moved:
`td.namecell` = name text + `view` anchor + watch chip + `pipbtn` chip; the handler split, `runCard_from_row`
and the `item ID --cached` door were already there PUT; SEVEN nameplates + 8 sections; `hit_row` ONE caller).
ADR 0030 keeps the census and the SEVEN rejected alternatives with measures.

Changed

- **The name cell becomes the opener** (`feat(ui) f78a6b0`, ONE file `ui/app.js`, 14 ins / 7 del): the
  search row draws `<button class="pipbtn" data-id>NAME</button>` FIRST in the cell — the SAME `data-id`
  the watch half carries, the label fallback riding the recipe cell's OWN shape (`name || hit.id` — a
  clickable name is never blank). The `card` word and the trailing chip die with it; `view` and the watch
  toggle stay PUT. **The click's other half stayed PUT word-for-word**: class, delegated handler split
  BY CLASS, `runCard_from_row`, the window's `card-id` field, `runCard(false)`, the `item ID --cached`
  door — ADR 0014's ONE code path kept, NOT grown. The search note moved with the surface (ADR 0019's
  rule): "each row's OWN NAME is its window opener (click it like the Recipes tab's names open theirs)".
- **The look is ONE joined rule, never a second copy** (`feat(ui) 239c2a4`, ONE file `ui/app.css`, 15
  ins / 14 del): `#recipe-hits tr td button.openrecipe` gains `#hits tr td button.pipbtn` in its selector
  list — the frame already joins selectors (its `th.col-job, th.col-recipe, ...` header rule is the
  precedent) — so the two names are ONE click shape: gold, `font: inherit`, dotted underline, no box,
  and NO hover on either (measured: `button.openrecipe` never HAD a hover rule; the chip's OWN gold hover
  died with the chip — "just like the recipes" rides the recipe shape EXACTLY). The chip-box TWO rules
  DIED; ZERO new rules, ZERO new colour tokens.
- **CI stays PUT at 49 step-lines + plant one** — ZERO door argv, ZERO fixture line, ZERO CI line; the
  mirror's OWN parse and GitHub's OWN test-job log both say 49. **The ladder row owed by the v0.15.0
  fold is now MEASURED:** the v0.15.0 fold push's own run **45260568565 SUCCESS**, 5/5 jobs green in
  **10 min 57 s** (20:33:18Z → 20:44:15Z, run **#51**, attempt 1, ONE run id, no CANCELLED sibling,
  head `00f94ba` = that FOLD commit, whose tree carries the tagged `eaeee89`); per-job ends `test` → 20:38:20Z
  · macos → 20:40:43Z · quality → 20:42:13Z · linux → 20:43:08Z · **windows → 20:44:14Z — the windows job
  CLOSED this run** (a scheduling fact, said so — the FIRST windows-closed run in the measured chain: the
  v0.14.0 era run closed on quality, its fold run closed on macos, the v0.15.0 run closed on linux). Each
  job answers **12 end-actions, 11 success + ONE skipped** (every skipped one the Rust-toolchain installer's
  OWN inner action `__dtolnay_rust-toolchain.__run_2/3`, never a workflow step); Ubuntu-libs apt line 7 on
  `test`/quality/linux logs, 0 on macos/windows. The `test` job pressed exactly **49 `cargo run --` steps**
  (the era moved ZERO CI line), and its step answers read 12 976 B (`recipes`) / **46 776 B (`recipes --icons`,
  ALL 12 of matching 12)** / 2 171 B (`--offset 10`) — the bytes ride the run's seed (my mirror: 46 793 B),
  the tallies do not.
- **The doors moved ZERO BYTES** (fresh 9911/9941/9971, FRESH processes/DBs, ZERO real-host for the crawl):
  mirror **49/49 rc `0`, ZERO NONZERO** (9811-style parse: 49 step-lines committed, ZERO moved, plant one;
  presses UNCHANGED 4/3/5/`rate_limited`1 · 5/5/7 complete · 8/8 pages/**8 SAME copy**/0 written; tallies
  end where the crawl left them **51 GETs / 17 recipe-file GETs, ZERO added** = locality proof; census 12/6/4/0/0).
  Exit map **SAME 31-door totals `0`×19 · `2`×5 · `4`×7** (fresh 9941; this era grew **ZERO new door shape** —
  the name click rides the door it already rode; the eight carried shapes re-measure `0`×7 + `4`×1: `recipes
  --icons` 46 793 B = ALL 12 of matching 12, `--limit 0` 0 rows at 52 B rc `0` EMPTY not 4, EMPTY-DB rc `4`)
  with money **0 pairing defects**, counts 19/19/19 wearing none. **Pinned v0.14.0 pair AGAIN 28/28 BYTE-IDENTICAL
  + 13/13 md5-equal** (fresh 9971 vs the worktree @ TAGGED `c065815`, banner `0.14.0` PUT, NINE nameplates = the
  BEFORE witness): nine card doors 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B, **the
  item doors `item 25/40 --cached` 4 341 / 4 076 B and the search doors 185/182 B measured byte-identical on
  BOTH binaries** (the name-button era touched not one answer byte), unbounded `recipes --icons` **46 793 B = ALL
  12 of matching 12 on BOTH binaries**; user file md5s UNCHANGED through BOTH binaries (LOCAL doors, ZERO requests):
  `stats` be44e700 · `item 25 --cached` 82e349c1 · `search` 5d8e2281 · `watch list` 02b3e228 · `recipes --limit 3`
  bbc17f2d · Blacksmithing c1fe9d13. The user's file census UNCHANGED (`sqlite3 -readonly`, NEVER written, NEVER
  crawled): schema **8** · 19 258 · 4 602 copies / **14 656 without** · 1 240/1 240 · 1 240 `learned_json` · watch
  **134** · `vendor_sell` 4 231 · `price_min_now` 4 344 + 258 price-less · `vendor_buy` **0** · trend **0** · deleted
  **0** · `sync_state` 6.
- **UI launch witnesses** (parse/grep ONLY — the webview NEVER renders in the build room, DOM/IPC cost NOT
  WITNESSED, same rule the v0.15.0/v0.14.0 records carry): `node --check` PARSE OK; `>card</button>` label count
  **ZERO**; the name-button draw EXACTLY 1 (class `pipbtn` + `data-id`); the ONE joined selector `button.openrecipe,
  #hits tr td button.pipbtn` = 1; standalone `button.pipbtn` rule blocks **ZERO** and `button.pipbtn:hover` **ZERO**
  (died with the chip); `.pip`/`.pip-closed` still 2; nameplates still **SEVEN**; `data-tab` attrs still **15**
  (7 + 8, `writes` rides three); closers still 2; the `openrecipe` draw PUT; ADR 0027's 21 id-scoped rules **STILL
  21** (72/39 px); Search tab's `--limit 40` PUT; `recipes-pager` 0. Builds named **ZERO** on every commit (FS notice
  aside); `cargo clippy --all-targets` the SAME **0 errors / 811 advice** rc `0`. ZERO Rust, ZERO SQL, ZERO wire,
  ZERO schema (`8` stays), ZERO exit code, ZERO money formatter, ZERO request rule, ZERO fixture line, ZERO CI step.
- **Measured defects: ZERO.** Banner MOVED `0.15.0 → 0.16.0` (`chore fecb6c2`; the lock's OWN version line follows
  the banner in the SAME commit — settling `0.14.0 → 0.16.0` in one jump, the lock having lagged one banner since
  v0.15.0's build, a house note kept honest). ADR 0025 gained a GROWN pointer (its "the name cell IS the click target"
  now covers BOTH tables), 0029 a CHANGED pointer (the opener's LABEL moved onto the name, its LOOK onto the recipe
  look), 0021/0027 RIDDEN pointers; tags never moved. Chain: `f78a6b0` feat(ui) · `239c2a4` feat(ui) · `fecb6c2` chore
  · this record + tag `v0.16.0`.

**FOLDED — measured post-tag on the PUSHED tree at `4f3c217` (fresh ports 9921/9951/9981, FRESH
processes/DBs; the tag's tree verified through the remote's OWN `^{}` line `d5fab777…` → `4f3c217`, tag
object `d5fab777`, payload 9 559 B).** The era's OWN chain+tag run reads **45265336918 SUCCESS**, 5/5
jobs green in **9 min 50 s** (21:01:18Z → 21:11:08Z, run **#52**, attempt 1, ONE run id, no CANCELLED
sibling, head = this record = the tag's target) — the SHORTEST measured run in the chain so far, and the
**linux job CLOSED it at 21:11:08Z** (the second linux-closed run, after v0.15.0's own: where jobs land
is a scheduling fact, said so rather than smoothed; ends quality → 21:03:32Z · macos → 21:07:52Z · `test`
→ 21:08:28Z · windows → 21:10:38Z). Each job answers **12 end-actions, 11 success + ONE skipped** — every
skipped one the Rust-toolchain installer's OWN inner action (`__run_2` windows, `__run_3` the others), never
a workflow step; Ubuntu-libs 7 on `test`/quality/linux logs, 0 on macos/windows. The `test` job pressed exactly
**49 `cargo run --` steps** (ZERO CI line moved by the era — the block stayed PUT) and GitHub's OWN test-job log
step answers read `recipes` 12 992 B / **`recipes --icons` 46 792 B, ALL 12 of matching 12** / `--offset 10` 2
173 B — my mirror's 46 793 B: bytes ride the seed, tallies do not. Post-tag mirror **49/49 rc `0`, ZERO NONZERO**
(9921: parse 49 step-lines committed, ZERO moved, plant one; presses UNCHANGED; **51 GETs / 17 recipe GETs ZERO
added**; census 12/6/4/0/0). Exit map **SAME 31-door totals `0`×19 · `2`×5 · `4`×7** (9951; eight shapes AGAIN
`0`×7 + `4`×1, `--limit 0` 0 rows 52 B rc `0` EMPTY not 4, EMPTY-DB rc `4`) with money **0 defects** (counts
19/19/19 wearing none). **The pinned v0.14.0 pair AGAIN 28/28 BYTE-IDENTICAL + 13/13 md5-equal** (9981 vs the
worktree @ TAGGED `c065815`: item doors 4 341/4 076 B and search doors 185/182 B identical on BOTH binaries,
`recipes --icons` 46 793 B = ALL 12 of matching 12 on BOTH; user md5s UNCHANGED be44e700/82e349c1/5d8e2281/02b3e228/bbc17f2d/c1fe9d13).
**Tag census 32 tags / 64 remote lines, ZERO moved** (`v0.16.0` → `4f3c217`); CI on the pushed tree: **49 step-lines
+ plant one** PUT; user file UNCHANGED (schema 8 · 19 258 · 4 602/14 656 · 1 240/1 240 · 134 · `vendor_buy`/trend 0
— `sqlite3 -readonly`, never written, never crawled); UI witnesses re-measured PARSE OK + card-label count ZERO + ONE
joined selector + 21 id rules STILL 21 + `--limit 40` PUT (DOM/IPC NOT WITNESSED). The fold's OWN push starts the NEXT
run, measured in the NEXT record (ADR 0011's ladder).

## v0.15.0 — the CARD WINDOWS stop being tabs: a row click floats the card OVER the tab it came from, the two card nameplates die, and the doors answer BYTE-IDENTICAL · 2026-11-23 · tag `v0.15.0`

The user's eighth ask: "make the recipe card no longer a tab but a window-in-window overlay when
clicking a recipe in the Recipes tab — the same for the search: clicking an item should pull up a
picture-in-picture item card instead of the item card being a whole tab." A card's WHAT was decided
at v0.12.0 (ADR 0025); this ask is its WHERE, measured off the tree at `e449b60` before one line
moved: the strip answered **NINE nameplates**, **TWO of them ARE card windows**, and ADR 0018's
`show_tab` loop (the ONE truth, 10 `data-tab` sections) hides every other section the moment a card
tab turns on — so clicking a row and opening its card tab **hides the list the click came from**
(`display: none` gets no events: the frame's own comment above app.js says so, the clicked row
vanishes beside its own window). The cure the ask names: the window opens ON the tab that is already
in front. Measured too: the search rows carried **ZERO** open-a-card affordance today, while the
recipe rows already ride the `openrecipe` button + ONE delegated handler; and ADR 0027's **21
id-scoped `#recipe-card` rules** are id-scoped, NOT tab-scoped — they can ride an overlay untouched.

Changed

- **The two card nameplates die; the windows ride the clicks** (`feat(ui) b5bcb25`, TWO files
  `ui/index.html` + `ui/app.js`, 89 ins / 15 del): the `tab-card` / `tab-recipe-card` buttons die
  (strip NINE → **SEVEN** nameplates) and the two sections drop their `data-tab` — ADR 0018's loop
  NEVER sees a card window again; the windows' visibility is held by `open_pip`/`close_pip` ALONE
  (`pip-closed` hides with the frame's OWN `display: none` mechanism, `pip` floats). Each window
  grows its OWN `Close this window` button — the ONLY closer: closing hides the window back where
  it was born, the tab underneath stays where it was. The recipe click stays EXACTLY PUT (button,
  delegated handler, door `recipe SPELL_ID --icons`) with ONLY `show_tab("recipe-card")` →
  `open_pip("recipe-card")`; the search row grows the opener it never had (measured ZERO today):
  a `pipbtn` button carrying the SAME item id as the watch toggle, and the ONE delegated handler
  over `#hits` splits the halves BY CLASS — the opener fills the window's field and lands on
  `runCard(false)` = `item ID --cached`, ZERO requests, the window answers offline like its door.
  Notes move with the surfaces (ADR 0019's rule). **ZERO Rust line, ZERO `SQL_` const, ZERO wire
  struct, ZERO schema line (`8` stays), ZERO exit code, ZERO money formatter, ZERO request rule,
  ZERO fixture line, ZERO CI step.** ADR 0029 keeps the decision, the census table and the SEVEN
  rejected alternatives with their measures (restyle-with-nameplates-left, switch-underneath,
  second renderer — ADR 0014's drift, new door/flag — the pinned pair measures the doors
  BYTE-IDENTICAL so the window IS a paint fact, whole-cell clicks — the delegated pattern's
  `event.target` is the button, re-parent — `position: fixed` floats without moving a node,
  auto-close — a window with a hidden second lifecycle answers nothing about why).
- **The overlay's paint is the frame's OWN silhouette again** (`feat(ui) 6b4f50c`, ONE file
  `ui/app.css`, 32 ins, FOUR rules): `.pip` floats the SAME `.card` panel — fixed inset, lit-brass
  trim (`--edge-lit`), gold hairline, DEEPER shadow than a laid panel — so the window sits visibly
  IN FRONT of its tab: picture in picture; `button.pipbtn` rides `.rowact`'s furniture. ZERO new
  colour token, ZERO new mechanism, ZERO `index.html`/`app.js` line in this commit.
- **CI stays PUT at 49 step-lines + plant one** — this era grew ZERO door argv, so the smoke block
  did not move; the mirror's OWN parse says so, and measured **49/49 commands rc `0`, ZERO
  NONZERO** on fresh port **9711** with stage presses UNCHANGED (budget-3 recipes 4 / 3 / 5 /
  `rate_limited` 1 · resumed-from-3 5 / 5 / 7 complete · re-pull 8 requests / 8 pages / **8 the
  SAME copy** / 0 written), the fixture's tallies ending where the crawl left them — **51 GETs,
  17 recipe-file GETs, ZERO added** = the locality proof — census recipe 12 / item 6 / copies 4 /
  `vendor_buy` 0 / trend 0.
- **The exit map at the SAME 31-door totals `0`×19 · `2`×5 · `4`×7** on fresh port **9741** (fresh
  seed + fresh EMPTY DB + a DEAD port): ZERO door fact moved, and v0.14.0's OWN eight shapes answer
  AGAIN **rc `0`×7 + rc `4`×1** (`recipes` 12 968 B · `recipes --icons` 46 768 B = **ALL 12 of
  matching 12** · `--limit 0` 0 rows at 52 B rc `0` · `--limit -1` 12/12 · `--limit 12 --icons` the
  SAME 46 768 B · `--offset 10` 2 of 12 · `--profession Enchanting --icons` 2 of 2 at 4 632 B ·
  EMPTY-DB rc `4`); this era grew **ZERO new door shape** (the window rides existing doors). Money
  over the door's OWN answers: **0 pairing defects**, counts 19/19/19 with **none wearing money**.
- **The answers move ZERO BYTES, measured as the strong pair** (fresh port 9771 + ONE fresh DB):
  the PINNED v0.14.0 tree (worktree at the TAGGED `c065815`, its OWN build green ZERO named, banner
  `0.14.0` PUT, NINE nameplates + the card tabs PRESENT = the BEFORE witness) vs the new tree →
  **28/28 BYTE-IDENTICAL, ZERO DELTAs** (the nine card doors 2 124 / 7 330 / 6 673 / 4 107 / 643 /
  1 209 / 1 155 / 6 740 / 4 033 B, the recipes doors in BOTH shapes — unbounded `recipes --icons`
  **46 793 B on BOTH binaries** — and the pre-existing set `status` 1 242 / `stats` 1 773 / `item
  25/40 --cached` 4 341 / 4 076 / `search` 185 / 182 / `watch list` 8) — plus **13/13 md5-equal**
  through BOTH binaries over the user's OWN file (LOCAL read-only doors, ZERO requests). The pair's
  surface census rides too: pinned `data-tab="recipe-card"` PRESENT / `pip-closed` ABSENT / `open_pip`
  ABSENT vs new ABSENT / PRESENT / PRESENT, banners `0.14.0` PUT vs `0.15.0` MOVED.
- **UI launch witnesses** (parse/grep ONLY — the webview NEVER renders in the build room, DOM/IPC
  cost stays **NOT WITNESSED**): `node --check` PARSE OK, SEVEN nameplates, 15 `data-tab` attrs
  (7 buttons + 8 sections), ZERO `data-tab="card"`/`"recipe-card"`, ZERO `show_tab("card")`/`"recipe-card"`
  sites, `open_pip`/`close_pip` 3 each, 2 closers, the `pipbtn` census, FOUR CSS rules, ADR 0027's
  **21 id-scoped `#recipe-card` rules STILL 21** (`.listicon` 72 / `.noicon` 39 px), `recipes-pager`
  0, the Search tab's `--limit 40` PUT. The user's file census UNCHANGED: schema **8** · 19 258
  items · 4 602 copies / **14 656 without** · 1 240 recipes / 1 240 distinct spell ids · `learned_json`
  1 240 set · watch **134** · `vendor_sell` 4 231 · `price_min_now` 4 344 + 258 price-less · pending
  0 · `vendor_buy` 0 · trend 0 · tombstoned 0 (`sqlite3 -readonly`, NEVER written, NEVER crawled).
  Builds named **ZERO** on every commit (FS notice aside); clippy the SAME **0 errors / 811 advice**.
- **CI ladder, owed row MEASURED:** the v0.14.0 fold push's own run **45222898599 SUCCESS**, 5/5
  jobs green in **17 min 36 s** (17:00:47Z → 17:18:23Z, run #49, attempt 1, ONE run id, no
  CANCELLED sibling, head `e449b60` = that FOLD commit); per-job ends quality → 17:02:41Z · linux
  → 17:09:08Z · windows → 17:09:51Z · `test` → 17:15:53Z · **macos → 17:18:23Z** (the macos job
  CLOSED it — a scheduling fact, said so); each job **12 end-actions = 11 success + 1 skipped**
  (`__dtolnay_rust-toolchain.__run_2` on windows, `__run_3` on the others — the installer's OWN
  inner action, never a workflow step), Ubuntu-libs step ABSENT on macos/windows; the `test` job
  pressed exactly **49 `cargo run --` steps** (this era moved ZERO CI line) and the unbounded
  `recipes --icons` step answered ALL 12 of matching 12 at 46 792 B with **19 counts, 0 money
  beside them**. ADR 0018/0021/0025/0027/0028 gained CHANGED/GROWN/RIDDEN pointers; tags never
  moved. FEATURE era MOVES the banner (`chore ff84417`): `0.14.0 → 0.15.0`. Chain: `b5bcb25`
  feat(ui) · `6b4f50c` feat(ui) · `ff84417` chore · this record + tag `v0.15.0`.

**FOLDED — measured post-tag on the PUSHED tree at `eaeee89` (fresh ports 9811/9841/9871, FRESH
processes/DBs, the real host untouched, the user's file LOCAL read-only doors, ZERO requests, ZERO crawl);
the tag's tree verified through the remote's OWN `^{}` line `5f344fce…` → `eaeee89`.** The era's OWN
chain+tag run reads **45253921925 SUCCESS**, 5/5 jobs green in **10 min 16 s** (run **#50**, attempt 1,
ONE run id, no CANCELLED sibling, head `eaeee89` = this record = the tag's target); the linux job CLOSED
it at 20:05:35Z — a scheduling fact, said so — and its 10 min 16 s against v0.14.0's 18 min 15 s says
WHERE the jobs landed, nothing else. Each job answers **12 end-actions, 11 success + ONE skipped**, the
skipped one the Rust-toolchain installer's OWN inner action (`__run_2` windows, `__run_3` the other four),
never a workflow step; Ubuntu-libs ABSENT on the windows/macos logs. The `test` job pressed exactly
**49 `cargo run --` steps** (ZERO CI line moved by the era — the block stayed PUT) and, from GitHub's OWN
test-job log, the tab's OWN argv `recipes --icons` answered **`"matching": 12` at 46 792 B** (my mirror:
46 793 B on the same argv — **the bytes ride the seed, the tallies do not**). Post-tag mirror **49/49 rc
`0`, ZERO NONZERO** on fresh 9811 with the driver's OWN parse saying **49 step-lines committed, ZERO step
moved, plant one** and stage presses UNCHANGED (budget-3 recipes 4/3/5/`rate_limited` 1 · resumed-from-3
5/5/7 complete · re-pull 8/8 pages/**8 SAME copy**/0 written), tallies ending where the crawl left them —
**51 GETs, 17 recipe-file GETs, ZERO added** — census 12/6/4/0/0. Exit map fresh 9841: the **SAME 31-door
totals `0`×19 · `2`×5 · `4`×7**, this era's eight shapes AGAIN **`0`×7 + `4`×1** (its `recipes --icons` = ALL
12 of matching 12 at 46 793 B, `--limit 0` 0 rows at 52 B rc `0` with EMPTY not 4, EMPTY-DB rc `4`), money
over those answers **0 defects**, counts 19/19/19 wearing none. **The pinned v0.14.0 pair AGAIN 28/28
BYTE-IDENTICAL, ZERO DELTAs + 13/13 md5-equal** fresh 9871 (pinned worktree @ TAGGED `c065815`, its OWN
build green ZERO named, banner `0.14.0` PUT, NINE nameplates PRESENT = the BEFORE witness vs the new SEVEN
+ `pip-closed`/`open_pip` PRESENT, banner `0.15.0` MOVED). Tag census on the remote: **62 lines · 31 tags,
ZERO moved** (`v0.15.0` → `eaeee89`, `v0.14.0` → `c065815`). Pushed-tree re-measures (same rule: the webview
NEVER renders in the build room, DOM/IPC NOT WITNESSED): fresh build **49.07 s**, named **ZERO**; PARSE OK;
SEVEN nameplates; 15 `data-tab` attrs (`writes` rides three); ZERO card `data-tab`/`show_tab` sites; TWO
closers; the `pipbtn` census; FOUR `.pip` rules; ADR 0027's **21 id rules STILL 21** (72/39 px); `--limit 40`
PUT. CI on the pushed tree: **49 cargo step-lines + plant one** (parse). User's file UNCHANGED (`sqlite3 -readonly`):
schema 8 · 19 258 · 4 602/14 656 · 1 240/1 240 · 1 240 `learned_json` · watch 134 · 4 231/4 344+258 · `vendor_buy`
0 · trend 0 · deleted 0 · `sync_state` 6. The fold's OWN push starts the NEXT run, measured in the NEXT record — ADR 0011's ladder.

## v0.14.0 — the RECIPES TAB shows the WHOLE set: its pager dies, the door never had a cap, and ZERO door line moved · 2026-11-23 · tag `v0.14.0`

The user's seventh ask: "recipes tab, get rid of the limit for how many hits are to be shown and
just show all at all times." A limit is asked FOR by a surface, so the era started by finding WHERE
the Recipes tab's limit lived — measured off the tree at `v0.13.0` before one line moved: the tab
owned a `<select class="field" id="recipes-pager">` with options `25` / `50` / `100` "a page", asked
the door as `["recipes","--limit",limit,"--icons"]` (`--limit 100 --icons` answers **100 rows of
matching 1 240** at 1 363 515 B — the door HID 1 140 rows), while the DOOR already answers the rest
(ADR 0022's paging rule, riding SQLite's own negative-limit token): `recipes --icons` with NO bound
answers **ALL 1 240 rows of matching 1 240** at 16 696 238 B, and `--limit 1240 --icons` answers the
**SAME 16 696 238 B** — a bound at or above the count changes NOTHING, which is why the cap DIED
instead of growing. The Search tab's second bound (`argv.push("--limit","40")`, hardcoded) was left
PUT — the ask named the Recipes tab, and the record names which bound is which.

Changed

- **The tab stops bounding** (`feat(ui) d491877`, TWO files `ui/index.html` + `ui/app.js`, 19 ins /
  9 del): the `recipes-pager` select dies (a control with no question left to answer), the tab asks
  `["recipes","--icons"]` with `--profession` only when a job is picked — so the tab's argv IS the
  door's OWN argv and a `cargo run -- recipes --icons` line shows the SAME rows (ADR 0021 / ADR 0014's
  one-door rule kept) — and the tab's NOTE moves the same breath (ADR 0019's rule): no more "showing
  100 of 1240", instead **ALL the rows this filter answers are shown**, naming why a bound would be a
  lie and naming the Watch button's reach. **ZERO Rust line, ZERO `SQL_` const, ZERO wire struct,
  ZERO schema line (`8` stays, measured), ZERO exit code, ZERO money formatter, ZERO request rule,
  ZERO fixture line.** ADR 0028: a bound is a SURFACE fact — the door keeps `--limit` because a shell
  that prints 16.7 MB for the whole set must be able to bound ON REQUEST, and killing the flag would
  move ADR 0022's paging pair, four bounded CI steps, the map doors measured with bounds and the Search
  tab's bound. SEVEN rejected alternatives with measures, all measured before one line moved (grow the pager
  → a bound at/above matching answers the SAME bytes, so it cannot do what it claims; `--limit 0` → measured
  0 rows of matching 12, zero means NONE not ALL, ADR 0004's EMPTY half; `--limit -1` → works via SQLite's
  token but hides the fact behind it; a virtual scroll → a page size is a cap again, which is what the ask
  refused; re-capping the Watch button → ADR 0021's rule speaks of the "shown rows", so capping would give
  "shown" a second meaning; a second unbounded door `recipes-all` → refused by ADR 0014's one-code-path rule
  (a click and a list row could then disagree about the set); and removing the door's `--limit` flag itself
  → refused with the decision above, it would move ADR 0022's paging pair, four bounded CI steps, the map's
  bounded doors and the Search tab's bound).
- **CI grows ONE step, 48 → 49 step-lines** (`ci ebc690b`, plant one stays, 14 `- name:` steps
  unchanged): the block gains the tab's OWN argv `cargo run -- recipes --icons`, ZERO requests — over
  the fixture's 12 rows the door answers **12 of matching 12** at 46 793 B, 19 reagent cells (7 carrying
  the DB's OWN icon bytes, 8 money words, **19 counts carried with 0 money beside a count** — ADR 0015's
  count half), 6 crafted cells carrying icon bytes, 2 rows stating NO crafted item (the Enchanting shape).
  Measured on fresh mirrors BEFORE the block landed (ports 9521 / 9531, driver parsed the committed block
  + these very steps): **49/49 commands rc 0, ZERO NONZERO**; measured AGAIN on the landed block (9541):
  **49/49 rc 0**, stage presses UNCHANGED (budget-3 recipes 4 / 3 / 5 / `rate_limited` 1 · resumed-from-3
  5 / 5 / 7 complete · re-pull 8 requests / 8 pages / **8 the SAME copy** / 0 written), the fixture's OWN
  tallies ending where the crawl left them — **51 GETs, 17 recipe-file GETs, ZERO added**, which IS the
  locality proof — census recipe **12** / item 6 / copies 4 / `vendor_buy` **0** / trend **0**.
- **The answers move ZERO BYTES, measured as the strongest pair a UI era can draw:** the PINNED v0.13.0
  tree (worktree at the TAGGED `b927109`, its OWN build green with ZERO named in 1 m 22 s, banner
  `0.13.0` PUT, the pager select PRESENT) vs the new tree over one fresh fixture process (9561) + one
  fresh DB → **28/28 BYTE-IDENTICAL, ZERO DELTAs** — the nine card doors (2 119 / 7 325 / 6 670 / 4 105
  / 643 / 1 207 / 1 154 / 6 737 / 4 032 B), the recipes doors in BOTH shapes (unbounded `recipes --icons`
  **46 768 B on BOTH binaries**, `--limit 0` 52 B, `--limit -1` = the unbounded 12 968 B, `--limit 100
  --icons` 46 768 B) and the pre-existing set (`status` 1 239 · `stats` 1 761 · `item 25/40 --cached`
  4 336 / 4 072 · `search` 185 / 182 · `watch list` 8) — plus **13/13 md5-equal** through BOTH binaries
  over the user's own file (cards 25 213 / 7 004 / 9 430 / 3 596 / 3 453 B, rc `4` on `8800111` at 113
  B). The era's headline door over the user's WHOLE file is byte-identical across binaries too: **16 696
  238 B, md5 `4f1982d6`, 1 240 rows of matching 1 240** (3 778 reagent cells: 3 514 icon-byte cells /
  3 642 money-word cells / 3 778 counts carried with NO money beside them; 897 crafted cells with icon
  bytes, 873 money words, 884 live words; 133 rows with NO crafted item — the Enchanting shape — and 10
  crafted ids with no row; the same set WITHOUT `--icons` costs 3 399 938 B).
- **The exit map keeps the totals it had:** the 31 doors v0.13.0's map carried answer the **SAME totals
  `0`×19 · `2`×5 · `4`×7** (fresh port 9551 — this era moved ZERO door fact), and the era's OWN eight
  shapes answer **rc `0`×7 + rc `4`×1** (the EMPTY-DB door's own no-row words, never a new refusal; `--limit
  0` answers 0 rows at rc `0`, ADR 0004's EMPTY half). Money over the door's OWN answers: **0 pairing
  defects**, counts carried 19 / 19 / 19 with none wearing money.
- **The tab's cost, stated where it can only be stated:** the whole set costs 16 696 238 B of door answer
  and **4 411 icon data URIs** (3 514 reagent + 897 crafted) to paint, against the old cap's ≈1 363 515 B.
  The webview never renders in the build room (the launch-audit rule), so the DOM cost and the Tauri IPC
  cost are **NOT witnessed** — this era states measured bytes and refuses to claim a smooth 1 240-row paint.
  The Watch button's reach grew the same way (ADR 0021's rule UNCHANGED, NOT capped): its ONE batched
  `watch add` argv carries **1 095 ids of the 1 240 rows** (133 Enchanting crafts with no crafted item and
  10 ids with no row skipped honestly); measured on a **SCRATCH COPY** of the user's file the door answers
  rc `0` / 6 036 B — the user's file was NEVER written, NEVER crawled — and the Details door then fetches
  exactly those records under ADR 0017's once-EVER pending rule, so the cost is a user-chosen click.
- **CI banner:** a FEATURE era MOVES the version banner `0.13.0 → 0.14.0` (`chore 25846dc`, `Cargo.toml`
  + `Cargo.lock` together — the v0.11.0-through-v0.13.0 shape). `cargo build` answers ZERO named warnings
  and `cargo clippy --all-targets` the SAME **0 errors / 811 advice lines** (rc 0) — the era touched ZERO
  Rust, which is exactly what those witnesses claim.
- **CI ladder, owed rows MEASURED:** the previous era's two rows land here as measured. Its chain+tag run
  reads **45199646115 SUCCESS**, 5/5 jobs green in **16 min 4 s** (run #46, attempt 1, head `b927109` =
  that tag's target: quality **7/7** with the Lint step **completed/success** — the local measure of the
  SAME command reads 0 errors / the SAME 811 advice lines — `test` **8/8** with the CLI smoke step GREEN,
  linux **8/8**, macos **7/8**, windows **7/8**, one `skipped` Ubuntu-libs step on each non-linux target
  as designed; the run reads longer than v0.12.x's 8–9 min because the macos job opened 5 min 1 s after
  the run opened — a scheduling fact, said so). And the v0.13.0 **fold push's own run**, which this era
  owes and had never been waited for, reads **45204000066 SUCCESS**, 5/5 jobs green in **9 min 56 s**
  (15:19:50Z → 15:29:46Z, run #47, attempt 1, exactly ONE run id, no CANCELLED sibling, head `6fd2e3e` =
  the FOLD commit itself; per-job ends quality → 15:22:46Z · windows → 15:26:14Z · macos → 15:28:30Z ·
  linux → 15:29:44Z · `test` → 15:29:46Z; each job answers **12 end-actions, 11 success + ONE skipped**,
  and the skipped one is the Rust-toolchain installer's OWN inner action `__dtolnay_rust-toolchain.__run_N`,
  never a workflow step — the Ubuntu-libs step is simply ABSENT on the macos/windows jobs; the `test` job
  pressed exactly **48 `cargo run --` steps**, because the fold's tree still carries the 48-step block:
  this era's `ci` commit landed AFTER the fold). The run this era's chain+tag push starts is measured in
  the NEXT record (ADR 0011's ladder), never chased here.

**The fold — post-tag, the PUSHED tree, all FRESH ports/processes/DBs (the real host untouched, the user's
file read read-only through LOCAL doors, ZERO requests, ZERO crawl).** The tag's tree (`c065815`, verified
through the remote's OWN `^{}` resolution `868eff3…` → `c065815`) answers the way the record says, measured
AGAIN rather than cited. **The era's OWN chain+tag run reads 45217976658 SUCCESS**, 5/5 jobs green in **18
min 15 s** (16:34:12Z → 16:52:27Z, run #48, attempt 1, ONE run id, no CANCELLED sibling, head `c065815` =
the TAGGED tree): macos → 16:40:48Z · linux → 16:40:56Z · `test` → 16:42:20Z · windows → 16:42:27Z · **quality
→ 16:52:27Z** — the Lint job closed the run this time, so the run reads longer than v0.13.0's 16 min 4 s
because of WHERE the quality job landed: a scheduling fact, said so. Each job answers **12 end-actions, 11
success + ONE skipped**, the skipped one being the Rust-toolchain installer's OWN inner action, never a
workflow step; the `test` job pressed exactly **49 `cargo run --` steps** — this era's block, the new step
INCLUDED, all rc 0 — and that step (the tab's OWN argv) answers **12 rows of matching 12** there with 19
reagent cells (7 icon-byte cells, 8 money words, **19 counts with 0 money beside them**), 6 crafted icon-byte
cells, 2 rows with NO crafted item, at 46 792 B (my fresh mirrors measured 46 793 B: the bytes ride the seed,
the tallies do not). The Lint step reads **completed/success** on CI's own build at 0 errors / **858** advice —
the SAME 858 the v0.13.0 fold's run measured, which is what a ZERO-Rust era claims (the local invocation's 811
stays quoted separately, as ADR 0011 requires). Fresh post-tag witnesses on the pushed tree: mirror **49/49
commands rc `0`, ZERO NONZERO** (9611, block re-PARSED at 49 step-lines + plant one, ZERO step moved; stage
presses UNCHANGED — budget-3 recipes 4/3/5/`rate_limited` 1 · resumed-from-3 5/5/7 complete · re-pull 8/8 pages
/ **8 answered the SAME copy** / 0 written; fixture tallies **51 GETs / 17 recipe-file GETs, ZERO added** = the
locality proof; census recipe 12 / item 6 / copies 4 / `vendor_buy` 0 / trend 0); exit map at the **SAME 31-door
totals `0`×19 · `2`×5 · `4`×7** (9631, fresh seed + fresh EMPTY DB + a DEAD port) plus this era's OWN eight
shapes **rc `0`×7 + rc `4`×1** (`recipes` 12/12 at 12 993 B · `recipes --icons` 12/12 at 46 793 B · `--limit 0`
**0 rows** at 52 B with rc `0` · `--limit -1` 12/12 · `--limit 12 --icons` the SAME 46 793 B as unbounded ·
`--offset 10` 2 of 12 at 2 174 B · `--profession Enchanting --icons` 2 of 2 at 4 634 B · `--limit 100 --icons`
over the EMPTY DB rc `4` on the door's OWN no-row words), money over those answers **0 pairing defects** with
counts 19/19/19 wearing none; and the pinned v0.13.0 pair **AGAIN 28/28 BYTE-IDENTICAL, ZERO DELTAs** (9671 —
pinned at the TAGGED `b927109`, its OWN build green ZERO named in 1 m 22 s, banner `0.13.0` PUT, the pager select
PRESENT) + **13/13 md5-equal** over the user's own file (9 cards 25 213 / 7 004 / 9 430 / 3 596 / 3 453 B, rc 4
on `8800111` 113 B), the recipes doors in BOTH shapes (`recipes --icons` **46 793 B on BOTH binaries**, `--limit
0` 52 B, `--limit -1` = the unbounded 12 993 B, `--limit 100 --icons` 46 793 B) and the nine card doors at 2 124
/ 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B; this era's HEADLINE door over the user's whole
file is byte-identical across the binaries: **16 696 238 B, md5 `4f1982d6`, 1 240 rows of matching 1 240** (3
778 reagent cells — 3 514 icon-byte cells / 3 642 money-word cells / **3 778 counts carried with NO money beside
them**; 897 crafted icon-byte cells, 873 money words, 884 live words; 133 rows with NO crafted item and 10 crafted
ids with no row; 3 399 938 B asked without `--icons`). The tab's shape on the pushed tree, quoted: `node --check`
rc 0, `recipes-pager` answers ZERO in BOTH UI files, the argv reads `var argv = ["recipes", "--icons"];`, the
job select stays ONE, the Search tab's `--limit 40` stays PUT, CI answers 49 `cargo run` mentions, banner `0.14.0`
vs the pinned `0.13.0` PUT. The Watch button's reach, fresh: **1 095 ids of the 1 240 rows** in ONE batched
`watch add` → rc `0` / 6 036 B on a SCRATCH COPY, `watch list` then answering **1 129 ids** (134 watched + 995
new); the user's file stays schema **8** / 19 258 items / 4 602 copies / **14 656 without** / 1 240 recipes /
watch **134** / `vendor_buy` 0 / trend 0 read read-only. Tags stay put — the remote answers 30 tags, its OWN
`^{}` lines reading `v0.13.0` → `b927109` and **`v0.14.0` → `c065815`**, ZERO moved; CI stays 49 step-lines +
plant one, and the fold is docs-only, so it cannot move the build. The fold's OWN push starts the NEXT run,
measured in the NEXT ladder row (ADR 0011's ladder)

Refs: [ADR 0028](./docs/DECISIONS/0028-a-hits-tab-shows-the-whole-answer.md) (this era's decision: a bound
is a SURFACE fact, with the measures and SEVEN rejected alternatives), [ADR 0022](./docs/DECISIONS/0022-one-paging-rule-per-door.md)
(the paging rule the tab now rides: no `--limit` = the rest), [ADR 0021](./docs/DECISIONS/0021-the-recipe-doors.md)
(the tab and the door it asks), ADR 0014 / ADR 0012 (one door, local door = ZERO requests), ADR 0015 (money never
beside a COUNT — 3 778 counts stayed money-free), ADR 0011 (measured witnesses, the launch-audit rule); [docs/DEVLOG.md](./docs/DEVLOG.md)
carries the cap census, the fresh mirrors, the pinned pair and the map.

## v0.13.0 — the CARD WINDOW gets bigger: its type ~1.45×, its icons ×3 (24 → 72 px), and the frame's sizes stay PUT · 2026-11-23 · tag `v0.13.0`

The user's sixth ask: "in the recipe card, when I pull up a single recipe, everything needs to
be larger, the font and the icon need a lot larger too." A card is ONE entry on its OWN tab, read
once from a distance; the Recipes LIST, the search hits and the mini analytics rows are dense
tables read by scanning. So the sizes moved per SURFACE, and the frame's own numbers were measured
BEFORE anything moved: at v0.12.1 the card was drawn with LIST sizes — its OWN type at 10.5–13.5 px,
its icons through `.listicon` at 24 px, the SAME class the list and the hits ride (`table.mini
.listicon` already overrides that class to 20 px for the mini rows — scoping a shared class to one
surface is the frame's OWN mechanism, not a new one), its `no bytes` stand-in `.noicon` at 13 px,
its panel furniture at list grades (`h2` 15 · `.note` 12.5 · `.field`/`button.act` 13 px). The only
icon size in the frame that belongs to a WINDOW is the Item card's `.iconimg` at 56 px — and the
card's icons had to PASS it ("a lot larger").

Changed

- **The card window's type and icons grow; every other surface stays put** (`feat(ui) d2f532a`, ONE
  file `ui/app.css`): the card's OWN classes move in place (13 → 19 · 13.5 → 20 · 12.5 → 18 · 12 →
  17 · 11 → 16 · 10.5 → 15 px, a step of ≈ ×1.45), its icons take THREE times the list's box
  (`.listicon` 24 → **72 px**, larger than any icon the frame draws today, the Item card's 56 px
  included), and the `no bytes` stand-in rides the SAME ×3 at 13 → **39 px** (the frame's own 13/24
  ratio times 72 = 39.0 — the box keeps its SHAPE rather than lying about the space an icon would
  take). The shared classes grow ONLY inside the panel: 21 id-scoped rules (`#recipe-card .chip` 12
  → 17, `.jobchip` 11.5 → 17, `.skillchip` 11 → 16, `.tierchip` 10.5 → 15, `.matname`/`.mat-none`
  → 17, `.mat-not-in-db`/`.cooked-none`/`.cooked-not-in-db` 10.5 → 15, `.live` 11 → 16,
  `.live-missing`/`.live-none` 10.5 → 15, `.listicon` 72, `.noicon` 39) plus the panel furniture
  (`h2` 15 → 22, `.note` 12.5 → 18, `.field`/`button.act` 13 → 19, gaps and paddings with them).
  **ZERO `ui/app.js` line, ZERO `ui/index.html` line** — app.js draws the SAME TWELVE classes it
  always drew, so ADR 0014's one-code-path rule survives intact (a card's material cell IS the code
  that draws a list row's), and the sizes are the frame's own px mechanism, not `transform: scale()`
  (rejected on measure: it paints, it does not lay out) and not a root/`rem` step (rejected: it would
  move the 12+ `font-size` rules the OTHER tabs ride). PUT and still quoted in the same file: `.listicon`
  24 px, `table.mini .listicon` 20 px, `.iconimg` 56 px, `.note` 12.5 px outside the card, `h2` 15 px.
- **The door's answers move ZERO bytes** — measured, not asserted: the PINNED v0.12.1 tree (worktree
  at the TAGGED `d2c26de`, its OWN build green with ZERO named in 1 m 22 s, banner `0.12.0` PUT) vs
  the new tree over ONE fresh fixture process (port 9451) + ONE fresh DB answer **22/22 BYTE-IDENTICAL,
  ZERO DELTAs** (the NINE card doors included, 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 /
  6 740 / 4 033 B), and **13/13 md5-equal** through BOTH binaries over the user's OWN file (card doors
  25 213 / 7 004 / 9 430 / 3 596 / 3 453 B, rc `4` on `8800111` at 113 B). The mirror replay stays
  green on THREE fresh ports (48/48 rc 0, ZERO nonzero — the block re-PARSED at 48 step-lines, plant
  one, the era moved NO step; the four card steps 7 333 / 6 673 / 4 107 / 643 B; the fixture's OWN
  tallies 51 GETs with 17 recipe-file GETs, ZERO added = the locality proof; census recipe 12 / item
  6 / copies 4 / `vendor_buy` 0 / trend 0) and the exit map re-runs at the **SAME 31-door totals
  `0`×19 · `2`×5 · `4`×7** (fresh port 9407). Money over the card doors' OWN answers: **0 pairing
  defects**, reagent COUNTS carried with NO money beside them (ADR 0015's count half).
- **CI banner:** a FEATURE era MOVES the version banner `0.12.0 → 0.13.0` (`chore 1f4b08f`, `Cargo.toml`
  + `Cargo.lock` together — the v0.11.0 / v0.12.0 shape; contrast the v0.12.1 PATCH era that left it
  PUT). `cargo build` answers ZERO named warnings and `cargo clippy --all-targets` the SAME
  **0 errors / 811 advice** (rc 0) — this era touched ZERO Rust, which is exactly what those witnesses
  claim. Ladder row owed and measured: the v0.12.1 fold push's own run **45102837594 = SUCCESS**,
  5/5 jobs green in **9 min 31 s** (02:54:26Z → 03:03:57Z, run #45, ONE run id, no CANCELLED sibling,
  head `290da81`) — quality 7/7 Lint step GREEN, `test` 8/8 with the CLI smoke step GREEN, linux 8/8,
  macos 7/8, windows 7/8 (`skipped` Ubuntu-libs). This era's own chain+tag run is measured in the
  NEXT record (ADR 0011's ladder).

**The fold — post-tag, the PUSHED tree, all FRESH ports/DBs (the real host untouched, the user's
file read read-only through LOCAL doors, ZERO requests, ZERO crawl).** The tag's tree (`b927109`,
verified through the remote's OWN `^{}` resolution `b09d646` → `b927109`) answers the way the record
says, measured AGAIN rather than cited: the mirror replay is **48/48 commands rc `0`, ZERO NONZERO**
on fresh port 9471 (the driver re-PARSES the committed block: 48 `cargo run` step-lines + plant one,
the era moved NO step), the stage presses UNCHANGED (budget-3 recipes 4 requests / 3 pages / 5 rows /
`rate_limited` 1 · resumed-from-3 5 / 5 / 7 complete · re-pull 8 requests / 8 pages / **8 answered the
SAME copy** / 0 written), the FOUR card steps answering **7 333 / 6 673 / 4 107 / 643 B**, the fixture's
OWN GET tallies ending where the crawl left them — **51 GETs, 17 of them recipe-file GETs, ZERO added**,
which IS the locality proof — and the census at recipe **12** / item 6 / copies 4 / `vendor_buy` **0** /
trend **0**. The exit map holds the **SAME 31-door totals `0`×19 · `2`×5 · `4`×7** on fresh 9481 (nine
card doors rc 0, SEVEN rc-`4` doors in the SAME words, FIVE rc-`2` doors at the SAME words, rc `3`
UNREACHABLE from this door). And the PINNED pair answers BYTE-IDENTICAL **AGAIN**: the pinned v0.12.1
binary (`d2c26de`, own build green ZERO named 1 m 22 s, banner `0.12.0` PUT) vs the pushed tree = **22/22
BYTE-IDENTICAL, ZERO DELTAs** (the NINE card doors included: 2 116 / 7 322 / 6 668 / 4 104 / 643 / 1 206
/ 1 153 / 6 735 / 4 031 B in this window) and **13/13 md5-equal** through BOTH binaries over the user's
OWN file (cards 25 213 / 7 004 / 9 430 / 3 596 / 3 453 B, rc `4` on `8800111` 113 B) — so the same claim
this era drew pre-tag is measured a SECOND time on the pushed tree, and it did not soften. Money over
those very answers: **0 pairing defects**, reagent COUNTS 2 / 2 / 2 / 2 / 2 carried with NO money beside
them (ADR 0015's count half on the bigger words). GitHub's own verdict for the tag's tree: **45199646115
SUCCESS**, 5/5 jobs green in **16 min 4 s** (created/run_started 14:56:38Z → updated 15:12:42Z, run #46,
attempt 1, exactly ONE run id, no CANCELLED sibling, head = the TAGGED tree) — quality **7/7** with the
Lint step **completed/success** (the local measure of the SAME command says 0 errors / the SAME 811 advice
lines), `test` **8/8** with the CLI smoke step GREEN (the 48-step block, the four card steps included),
linux **8/8**, macos **7/8**, windows **7/8** (one `skipped` Ubuntu-libs step on each non-linux target, as
designed); the run reads longer than v0.12.x's 8–9 min because the macos job opened 5 min 1 s after the run
opened — a scheduling fact on GitHub's side, not a tree fact. The user's own file's tallies stay INTACT
(read `sqlite3 -readonly`): schema **`8`**, 19 258 items, 4 602 tooltip copies, **14 656 without**, 1 240
recipe rows / 1 240 distinct `spell_id`s, `price_min_now` **4 344** + `price_now_missing_at` **258** = the
4 602 each asked ONCE, pending **0**, `vendor_buy` **0**, `price_trend_normal` **0**, tombstoned **0** — the
card's live chips are a TRUTH, not a defect. Ladder owed and closed here: the era's OWN chain+tag run is
**45199646115 = SUCCESS** (above); the fold's OWN push starts the NEXT run, measured in the NEXT record
(ADR 0011's ladder), never chased here.

Refs: [ADR 0027](./docs/DECISIONS/0027-a-cards-sizes-are-scoped-to-the-window.md) (this era's decision:
a size is a SURFACE fact, with FOUR rejected alternatives and their measures),
[ADR 0025](./docs/DECISIONS/0025-the-recipe-card-window.md) (the door whose cells got bigger — its bytes
did NOT move), ADR 0014 (one code path — the reason a NEW icon class was rejected), ADR 0015 (the money
contract on bigger words), ADR 0011 (measured witnesses, the launch-audit rule: the webview never renders
in the build room, so NOTHING here claims a rendered card); [docs/DEVLOG.md](./docs/DEVLOG.md) carries
the before/after size censuses, the pinned pair and the CI verdict.

## v0.12.1 — the `never_loop` ERROR the era's own CI run rode dies BY NAME: clippy answers 0 errors again, NOTHING was muted and NOTHING moved · 2026-11-23 · tag `v0.12.1`

A PATCH era (contrast the v0.12.0 FEATURE era that MOVED the Cargo banner `0.11.0 → 0.12.0`: here the
banner stays PUT at `0.12.0` and `Cargo.toml` / `Cargo.lock` answer an EMPTY diff, the shape the v0.9.1 /
v0.10.1 / v0.11.1 patch eras rode). It answers the ONE build-time defect v0.12.0's fold measured on its
OWN chain+tag run: GitHub's run **45095439290 measured FAILURE** (4/5 jobs, 8 min 9 s) because the quality
job's lint step rode **ONE clippy ERROR** — `error: this loop never actually loops` at
`src/db.rs:3029:5`. The door was NEVER broken (its rc `0` witnesses are v0.12.0's, and the smoke block was
GREEN on GitHub): the shape it was written in is a lie about the read. `spell_id` is the `recipe` table's
PRIMARY KEY (measured: 1 240 rows, ids distinct, the card door's map answered rc `4` for absent ids, never
two rows), so AT MOST ONE row answers; a `for` around a body that always returns can never take a second
turn, which is exactly what `clippy::never_loop` — DENY-by-default — calls an ERROR. The honest repair is
clippy's OWN suggestion: take the row (`if let Some(one_row) = mapped.next()`), the shape the established
single-row doors already ride. NO mute, NO gate (ADR 0024's rule kept: a mute would keep the lie; ADR 0011's
`-D warnings` gate STAYS OFF for clippy's ADVICE — an ERROR is not advice, and it is fixed instead).

Fixed

- **`recipe_card` TAKES the one row the PRIMARY KEY states** (`fix(db) 56e0356`): the always-returning
  `for one_row in mapped {` becomes `if let Some(one_row) = mapped.next()` plus a comment naming WHY
  (the key states at most one row; the walk could never repeat; the ERROR is what v0.12.0's run rode,
  fixed BY NAME). `cargo clippy --all-targets` goes **1 ERROR / 811 advice → 0 errors / 811 advice**
  (rc 101 → rc 0, the same 811 lines, 0 `never` hits), so the lint step is informational AGAIN and its
  step NAME plus `ci.yml`'s header comment are true again — the workflow needed NO move. The named
  rustc tally stays ZERO before and after (this was never a dead-code warning: an ERROR-level clippy
  diagnostic is a DIFFERENT gate, and the record keeps the two apart). Behavior moves ZERO: measured on
  FRESH ports/DBs, the pinned v0.12.0 binary (`ace683a`, its OWN build green in 1 m 23 s, banner `0.12.0`)
  vs the fixed binary answer **22/22 BYTE-IDENTICAL** on the fixture DB — the NINE card doors included,
  where earlier patch pairs compared only the pre-existing doors — and **13/13 md5-equal** through BOTH
  binaries over the user's OWN 19 258-row file (read-only, ZERO requests), and the mirror replay stays
  **51/51 rc 0** with the same stage tallies (48 step-lines unchanged, plant one).
- **CI ladder, measured since the tag:** the v0.12.0 fold push's own run measured
  **45098365666 = FAILURE**, 4/5 jobs green in **11 min 20 s** (02:07:22Z → 02:18:42Z, run #43, attempt 1,
  ONE run id, no CANCELLED sibling, head `89cab34` = the fold record) — linux build 8/8 · macos 7/8 ·
  windows 7/8 (their Ubuntu-dev-libraries step `skipped`, as designed: only the linux target links gtk) ·
  `test` 8/8 with the **CLI smoke step GREEN** (the 48-step block, the four card steps included, passes on
  GitHub's own builds) — quality 6/7 with the **Lint step = failure**, the SAME step its v0.12.0 sibling
  rode, which is exactly what the tree implied BEFORE the read (NOTHING about the code moved between the
  tag and that push, so the ERROR still rode it); the expectation was stated first, the measure agreed, per
  ADR 0011's ladder. This era's own chain+tag run — the FIRST tree since v0.12.0 whose lint step measures
  0 errors — **MEASURED IN THE FOLD: `45101284057 = SUCCESS`**, 5/5 jobs green in **8 min 13 s** (02:37:33Z
  → 02:45:46Z, run #44, attempt 1, ONE run id, no CANCELLED sibling, head `d2c26de` = the TAGGED tree) —
  quality **7/7 with the Lint step completed/success**, GREEN for the FIRST time since v0.11.1 (so its
  "informational" NAME is true of this tree again), `test` **8/8 with the CLI smoke step GREEN** (the 48-step
  block, the four card steps included, on GitHub's own build of the tagged tree), linux 8/8 · macos 7/8 ·
  windows 7/8 (their Ubuntu-dev-libraries step `skipped`, as designed). The verdict moved **FAILURE → SUCCESS
  with ZERO workflow lines moved** — the step is the same text and the `-D warnings` gate is still off, only
  the tree's shape moved, and that is what this era claims, measured. Post-tag witnesses (fresh ports/DBs,
  the real host seen NOTHING, the user's file read read-only): mirror replay on fresh 9341 **51/51 rc 0** with
  the SAME stage tallies and the card steps at 7 333 / 6 673 / 4 107 / 643 B, the fixture's GET tallies ending
  where the crawl left them (51 GETs, 17 recipe-file GETs, **ZERO added** = the locality proof); the exit map
  on fresh 9373 at the **SAME 31-door totals `0`×19 · `2`×5 · `4`×7** (a PATCH era moves ZERO semantics); and
  the PINNED v0.12.0 binary (`ace683a`, banner `0.12.0` PUT) vs the pushed tree **16/16 md5-equal, ZERO DELTAs**
  over the user's OWN file — card doors 25 213 / 7 004 / 9 430 / 3 596 / 3 453 B plus `recipe 27835` BARE 3 705
  B, rc `4` on `638` 109 B · `8800111` 113 B · `999999` 112 B — with the PRIMARY-KEY fact the fix rides read
  ON that file (**1 240 recipe rows, 1 240 DISTINCT `spell_id`s**) and **0 money-pairing defects** over these
  very answers. The fold's own push starts the NEXT run, measured in the NEXT record (ADR 0011's ladder),
  never chased here.

Refs: [ADR 0026](./docs/DECISIONS/0026-a-clippy-error-is-not-advice.md) (the era's ADR: an ERROR rides
the run, so it dies BY NAME), [ADR 0025](./docs/DECISIONS/0025-the-recipe-card-window.md) (whose
consequence bullet this closes), [ADR 0024](./docs/DECISIONS/0024-the-four-named-warnings-die-by-name.md)
(fix-not-mute, and every rejected mute recorded), [ADR 0011](./docs/DECISIONS/0011-ci-asserts-only-what-is-true.md)
(the `-D warnings` gate stays off, CI states only measured truths); [docs/DEVLOG.md](./docs/DEVLOG.md)
carries the censuses, the pinned pair and the CI verdict.

## v0.12.0 — a recipe row OPENS ITS OWN CARD: one click reads ONE local door, and the card shows the entry as the GAME shows it · 2026-11-22 · tag `v0.12.0`

The user's ask: "when looking at a recipe, clicking it in the list should open a detailed window
showing it like it would appear in the game." "As the game shows it" is only honest if the window is
built from words the TWO sources already own — nothing was invented. The fetched set states the
entry itself (job, name, the skill number + tier word + its grey-to-green band) AND names the
**scroll that teaches it BY VALUE** in `learned_json` (measured on the user's own file today:
**804 of 1 240** entries state a `schematic_item` with an `item_id`, **413** state the `trainer`
word, **23** the `automatic` word — and 747 of the 804 stated ids have an item row, 477 of those
carry the scroll's OWN tooltip copy). The item rows this DB owns carry the in-game cells: name,
quality, the two levels, the game's money, the icon bytes and the record's copied description
(ADR 0012's readable twin). Two other candidate joins were REJECTED on measurement, not on looks:
`spell_id` as an item id (651 spell ids land on an item row but only 20 of those rows are `Recipe`
rows — spell ids and item ids are different namespaces) and the name link `"Recipe: " + crafted
name` (**123** of 1 240 entries reach an item row at all, while 716 items carry `item_type Recipe`
and just 250 are named `Recipe:*` — and of those 123 that reach, ZERO land on this entry's own
crafted id: all 123 attach to a DIFFERENT item, so the name join does not merely under-reach, it
MIS-attaches). So the card composes: the SCROLL cell through
the set's own stated id, the RESULT cell through `crafted_item_id`, and one cell per MATERIAL
through the reagent ids — every join BY VALUE, and every gap a state word the door ships. Bonus,
recorded where it lives: `learned_json` gets its FIRST reader here, which closes the stored-with-
no-reader debt ADR 0021 wrote about that very column.

Added

- **the card door's read + its cells** (`feat(db) 0b717a4`): `recipe_card(conn, spell_id, with_icons)`
  reads ONE `recipe` row (its own `SQL_RECIPE_CARD_ROW`, because the LIST door's SELECT never
  reaches `learned_json`) and answers `Option<RecipeCard>` — `None` = "this spell id has no row",
  which the verb turns into rc `4`. Each cell (`RecipeCell`) rides the readers the item card and
  the list joins ALREADY use — `live_name_price_of` (name + stored min + the live twin + BOTH stamps),
  `money_of`, `icon_b64_of` / `icon_at_of` / `icon_missing_at_of`, `tooltip_text_of` / `tooltip_at_of`
  / `tooltip_missing_at_of` — and the materials ride `join_recipe_reagent` UNTOUCHED, so a card
  cell cannot say something different from a list row about the same id (ADR 0014's one-code-path
  rule). Only FOUR columns the list door never reads come from a SELECT of its own
  (`SQL_CARD_CELL`: quality, the two levels, vendor sell) — and it carries the SAME `deleted_at IS
  NULL` guard, because the stamp readers above have no tombstone guard of their own, so the door
  asks them ONLY for a row it saw alive (ADR 0010's rule made true, measured: `SQL_TOOLTIP_*` /
  `SQL_ICON_*` select on `id` alone).
- **the verb `recipe SPELL_ID [--icons]`** (`feat(cli) 29ac737`): LOCAL, ZERO requests (ADR 0009/0012's
  local-door rule). The exit trio measured on the user's own file: an unknown spell id answers rc
  `4` naming the stage that would have written it (112 B), a missing id answers rc `2` "recipe wants
  a spell id, for example: recipe 132" (48 B), a non-numeric one answers rc `2` "recipe wants a
  numeric spell id, got abc" (40 B) — ADR 0004's contract, identical CLI/GUI. The usage line names
  the verb (`recipe SPELL_ID [--icons]`), and the parser's existing multi-word verb reading is
  untouched (`recipe abc` still answers "unknown verb recipe abc" — the door is ONE word).
- **the fixture speaks the THREE `learned_from` shapes the real host was measured carrying**
  (`test(fixture) a03008b`): keys exactly `item_id · item_name · type` — row 132 states a
  `schematic_item` naming item 60 (a row the detail stage wrote AND the tooltip stage copied),
  637 states one naming 2589 (an id the DB never saw), 202 states the `trainer` word alone,
  334 the `automatic` word, and 638 keeps the explicit NULL. The door's answer over the fixture's
  12 rows (fresh port + fresh DB, ONE observation window): 8 doors rc `0` (7 330 · 1 155 · 6 673 ·
  4 033 · 4 107 · 643 · 1 209 · 6 740 B) and rc `4` on `999999`; the scroll cell for 132 answers
  IN GAME (`in_db true`, db name "Fixture Detail 60" beside the set's own, `Common`, both levels,
  vendor `500 ↔ 5s`, stored `17624 ↔ 1g 76s 24c` with live `15624 ↔ 1g 56s 24c` at 1795394930 — the
  RUN's pair, fixture prices seed per process — the copy's own 44-char words ("Fixture Detail 60 Item
  Level 1 Sell Price: 7c") and row 60's icon answering CHECKED-GONE (`icon_at` NULL, `icon_missing_at`
  1795394929 SET, ZERO copied bytes — the third icon state); 637's scroll cell answers `in_db false` with the SET's name
  beside it and NO crafted cell at all (the enchanting shape); the money contract measured on
  these very answers: **0 pairing defects** (a money word beside EVERY copper — vendor sell, stored
  min, live twin) and **13 counts carried with NO money beside them** (ADR 0015's count half);
  `vendor_buy` / `price_trend_normal` read **0 / 0** in the fixture census, ADR 0016's NULLs still
  unwritten.
- **the NINTH nameplate tab + the row CLICK** (`feat(ui) 53d5977`): clicking a recipe row's name
  opens THAT entry's card — the name cell is a button carrying its own spell id, and ONE delegated
  handler over the hits table reads it (the same pattern the search hits and the tab strip ride, so
  a 100-row page plants no closures), fills the door's id and runs the SAME argv a `cargo run --
  recipe N` line takes (ADR 0014). The list stays a list. The panel draws the entry's own words
  (job chip · name · spell id · skill number · tier word · the set's grey-to-green band · the
  `acquisition` word + note + the set's provenance) and then the cells: `crafts` / `teaches the
  spell` / the material cells — via `recipe_mats` REUSED on the card's materials, so the material
  markup is the SAME code the rows draw. Quality rides the frame's OWN `.chip c-<quality>` classes
  exactly as the search hits draw them (no second palette); the tooltip block is labelled as the
  record's OWN copy, printed as TEXT (ADR 0012's twin, never upstream's markup). The new chips name
  the gaps: `the set states NOTHING about how this recipe is learned` / `the set states the word
  alone — no scroll item exists…` / `the DB never saw this id alive` / `no crafted item — this
  craft puts an enchant on a slot`. UI witnesses stay parse/grep (launch-audit rule, no display in
  the build room): `node --check` parses the grown app.js, and the CSS names the EXACT twelve new
  classes app.js draws (`openrecipe`, `rcardbox`, `rcardhead`, `rcardname`, `rcardlearn`, `rcardmats`,
  `rcell`, `rcellname`, `rcopy`, `rcopyhead`, `learn-word`, `learn-none`) with everything else riding
  existing classes.
- **CI's smoke block grows the card door's four green steps** (`ci 2b1784f`): **44 → 48 step-lines**,
  plant line stays one (measured: 14 `- name:` steps unchanged, raw `cargo run` mentions 46 → 50) —
  `recipe 132 --icons` / `recipe 202 --icons` / `recipe 637 --icons` / `recipe 638`, the four
  learned-from shapes, ZERO requests, so the fixture's GET tallies cannot move and the unchanged
  tally IS the proof the door asks nothing. The door's rc `4` is NOT committed as a green step
  (ADR 0011's split rule, kept from v0.10.0): it is measured on scratch, as is the pinned BEFORE
  witness that this verb is NEW — the v0.11.1 binary answering the same argv rc `2` "unknown verb
  recipe".
- **the Cargo banner rides 0.12.0** (`chore aa70092`): a FEATURE era, so the banner MOVES
  `0.11.0 → 0.12.0` (`Cargo.toml` + `Cargo.lock`) — the shape the v0.10.0/v0.11.0 feature eras
  moved and the v0.9.1/v0.10.1/v0.11.1 patch eras left put, honestly contrasted.


Changed

- **ADR 0021's `learned_json` bullet gets a pointer**: the column's stored-with-no-reader debt
  closes AT this door (its measured tables stay history, the pointer pattern ADR 0010/0011 established).
- **`docs/DATABASE.md`** names the card door (its SELECTs, the four columns it adds, the tombstone
  rule made true in the DOOR because the stamp readers carry none) and moves `learned_json`'s
  "no reader yet" line to what the card reads.
- **`docs/API.md`** grows the second source's `learned_from` census — an upstream fact, measured
  today (804 `schematic_item` / 413 `trainer` / 23 `automatic`, keys `item_id · item_name · type`,
  and the reach 804 → 747 rows → 477 copies).
- **README status + roadmap row**, **`docs/DEVELOPMENT.md` milestone row**: the named-warning set
  stays ZERO at this release (this era planted NINE names and killed them with the door that reads
  them — recorded, not muted).


## v0.11.1 — the FOUR named build warnings die BY NAME: the named set shrinks FOUR to ZERO, and not one of them was muted · 2026-11-22 · tag `v0.11.1`

A patch era answering "fix them and if not fixable mute them": the FOUR named warnings
that rode every build since v0.8.0 (`PendingSql`'s privacy, `Tooltip`'s unread fields,
`open_db`, `catalogue`) all died BY NAME. Every one of the four was fixable — a mute was
available for each and every rejected mute is recorded in ADR 0024, so the ask's second
branch never had to be taken. No behavior, no wire, no schema and no answer moved: the
era is the compiler's own verdicts made true.

Fixed

- **`pending_plan` stops claiming `pub`** (`fix(stats) 523dfcc`): rustc's `private_interfaces`
  warning said the signature cannot be honored — the fn reaches `pub(crate)` while its return
  type `PendingSql` is usable only `pub(self)`. Both `use crate::stats` lists leave the name
  out and its FOUR callers are this module's own queue doors, so the function (not the type)
  is what narrowed; widening `PendingSql` was rejected because nothing outside consumes a
  queue-plan type. The named tally goes FOUR → **THREE**.
- **`Tooltip` narrows to the keys a reader reads** (`fix(models) 7b3bdb0`): nine fields rustc
  called dead (`id, kind, name, quality, version, build, icon_url, links, attribution`) die,
  `html` (the copy stage's) and `fields` (the price stage's `price.min`) stay. The mirror
  already types ONLY what has a reader twice in this file (`TooltipFields` types only `price`,
  the recipe shapes the same), ADR 0021 already called a stored-without-reader field dead code,
  and `docs/API.md` keeps the whole measured key list, so the documentation duty rides where
  probe records live. The honest cost is recorded where the struct lives, not hidden: `id` was
  serde's required-key check, nothing compares a tooltip's own id against the request URL's,
  and a non-object body still fails to parse and still aborts its stage. The tally THREE → **TWO**.
- **`open_db` dies** (`fix(db) 6333783`): a wrapper bundling `db_path` with a body
  character-for-character identical to `open_db_at`'s, with zero call sites — the entry points
  resolve through `db_path` (`cli.rs`'s `open_at`, `cmd_status`) and the GUI reaches the same
  door through `cli_ok`'s argv (ADR 0014). Its comment's "every entry point goes through this"
  stopped being true when they moved, and `open_db_at`'s own comment named the dead function,
  so it was rewritten in the same breath. Muting it would have kept a second, unreachable copy
  of the schema rules — the drifting truth `docs/DATABASE.md` exists to prevent. Tally TWO → **ONE**.
- **`catalogue` dies WITH its import names** (`fix(sync) 316bb0e`): the sync module's one-line
  `return coverage(conn)` re-export lost its caller (`cli.rs` reads `stats.rs`'s `coverage`
  directly), and `sync.rs`'s `use crate::stats` line named `coverage` + `Coverage` ONLY for it
  — leaving either behind would have traded this dead-code warning for an unused-import one, the
  swap ADR 0011's "the set is expected" rule refuses. The removed wrapper's promise ("used by
  `status` and the UI footer") moved onto `coverage`'s own comment, so no claim was buried with
  a grave. Tally ONE → **ZERO**: this era's builds answer NOTHING named, and the `hard linking
  … incremental cache` notice — the FS artifact of the clear-the-cache mechanic, never a fifth
  named warning — answered ZERO times across this era's builds.

Changed

- **ADR 0024** records the four fates: fix-vs-mute per warning, the witnesses (call sites,
  readers, serde's derives), and the rejected alternative for each — including keeping `id` to
  preserve serde's required-key check, and every `#[allow(...)]` that was available and not
  chosen. It also states what did NOT move: ZERO SQL constant, wire shape, schema, exit code,
  money formatter, UI line or request-rule line, and the trio's NULL-vs-0 truth (ADR 0016)
  untouched.
- **`docs/DEVELOPMENT.md`** keeps its v0.8.0 census as history and names the current ZERO;
  **`docs/DATABASE.md`**'s "two doors into the schema helper" now names the doors that exist —
  `db_path` (who picks the file) and `open_db_at` (who opens and stamps it).
- **`ci.yml` moves ZERO steps** (44 step-lines + plant one, measured 14 `- name:` steps / 46 raw
  `cargo run` mentions on both sides): this era has no behavior delta to smoke. Its two LIVE
  claims are corrected because they would lie from here on — the header comment ("this build's
  four dead-code warnings … are intended") and the lint step's NAME — while keeping the history
  (FOUR through v0.11.0, ZERO from v0.11.1) and ADR 0011's reason the `-D warnings` gate stays
  off (clippy answers 0 errors and hundreds of its OWN advice lines).
- The version banner stays put at `0.11.0` — a PATCH era, the v0.9.1 / v0.10.1 shape: `Cargo.toml`
  and `Cargo.lock` answer an EMPTY diff. `tests/` too (no fixture delta to smoke), `ui/` ZERO lines.

Removed

- The two wrappers (`sync::catalogue`, `db::open_db`), `Tooltip`'s nine unread fields, and
  `pending_plan`'s unhonored `pub`. The era's whole Rust diff is FOUR files / 40 insertions /
  49 deletions (`git diff e1dc5c6..HEAD`), and the only `SCHEMA_SQL` / `meta_set` lines inside
  it are the REMOVED wrapper's own body copy — the schema constant itself never moved.

Chain: `523dfcc` fix(stats) · `7b3bdb0` fix(models) · `6333783` fix(db) · `316bb0e` fix(sync) ·
`092ab61` ci (the two claims, ZERO steps) · record + tag `v0.11.1`. Post-tag audits (the 44-step mirror
replay, the exit map grown at 89 doors, the pinned v0.11.0 binary vs the new one, the user's own
file read read-only) land FORWARD in this era's fold.

## v0.11.0 — the LIVE price IN the recipe cell: the stored min keeps its cell and hover, the live pair rides beside it, the two live NULLs stay two honest chips · 2026-11-22 · tag `v0.11.0`

A feature release answering the deferred question v0.9.9 and ADR 0020 recorded
("live price IN the cell = next release's honest question") — with a producer on
it now: the price stage (v0.9.6, ADR 0017) ALREADY writes `price_min_now` beside
the stored `price_min` (schema 7's OWN columns, its own stamp pair); the recipe
door was reading only the stored half.

Added

- **The live twin in the by-value joins** (`feat(db) ff99bce`): `SQL_LIVE_NAME_PRICE`
  widens to `price_min_now, price_now_at, price_now_missing_at` in the SAME SELECT
  (one read per row — no second query, ADR 0014's one-code-path), `LiveNamePrice`
  mirrors them, and BOTH joins (`join_recipe_reagent` + the crafted join in `recipes_rows`)
  fill live copper + live money words (Rust's ONE formatter, ADR 0015's pair) + the
  TWO stamps. The stamps are not decoration: the two NULLs mean different things and
  only they keep them apart — `price_now_missing_at` set = "looked, the AH offers
  NOTHING right now" (ADR 0017's third state), both stamps NULL = "the live price was
  never refreshed yet". Measured over the fixture's own set (fresh DB, ZERO real-host):
  stored `18490 ↔ 1g 84s 90c` (the RUN's seed) with live `17490 ↔ 1g 74s 90c` /
  `16490 ↔ 1g 64s 90c` — live differs 5, "no auction right now" 1, "not refreshed yet"
  4, and the 2 enchanting crafts (crafted id NULL) carry NO live noise; reagents 11 of
  19 join a row (4 live pairs, 4 price-less, 3 not refreshed). The Blacksmithing door
  answers BOTH states side by side in 2 rows (crafted 40 = missing-stamped, crafted 60
  = live differs) — the shape the new CI step reads.
- **The recipe tab's cell draws it** (`feat(ui) 0690f38`): the money IN a cell stays
  the STORED min with its exact-copper hover (ADR 0015 untouched); the LIVE money span
  (`class="live"`) rides beside it ONLY when the two DIFFER — its hover keeps the EXACT
  live copper, live == stored adds NOTHING (a second identical number is noise); the
  two honest chips `no auction right now` (`live-missing`) / `live price not refreshed
  yet` (`live-none`) ride the stamps; NO live noise at all for an id the DB has no row
  for (the joins wrote no live fields — `not in your DB` / `the DB never saw this id`
  stay the honest words); ONE conditional note sentence in the ADR 0020 pattern (cell
  = STORED vs live = LAST ASK, no numbers — numbers ride the cells), shown only when
  a live part fired. CSS names the EXACT trio `.live` / `.live-none` / `.live-missing`
  (parse + grep witnesses only, the launch-audit rule — the webview never rendered in
  the build room).
- **CI grew ONE step (43 → 44 step-lines + plant one)**: `cargo run -- recipes
  --profession Blacksmithing --icons --limit 3` reads BOTH live states in ONE answer
  with ZERO requests added (the door is LOCAL, ADR 0012's rule — the fields come from
  the price stage's OWN press above): fresh mirror measured BEFORE the block landed,
  driver generated from these very steps — **47/47 commands rc 0, ZERO NONZERO**
  (`TOTAL_STEPS=47`); the step's RUN: `matching 2`, spell 202 crafted 40 stored
  `20622 ↔ 2g 6s 22c` with `crafted_now_missing_at` 1795366692 set (missing state,
  its OWN copied bytes ride: b64 2 596 chars, stamp 1795366661), spell 683 crafted 60
  live `18622 ↔ 1g 86s 22c` DIFFERENT from stored `20622` with `crafted_now_at`
  stamped — and spell 683's crafted icon stays its honest "no copied bytes" state.

Changed

- **ADR 0023** records the decision (the four parts: one read carries the twin / the
  two NULLs stay two answers named by the stamps / the cell rule / the conditional
  note); ADR 0020's deferred bullet gets the pointer — the SEARCH-hit cell stays
  stored-only (`HIT_COLUMNS` and the Hit money pair measured UNCHANGED, its wire-shape
  deferral still stands where recorded); ADR 0021 gets the pointer that its join now
  carries the live trio (READ, no schema hop, ZERO new requests).
- **docs/DATABASE.md** names the READ beside the live-column paragraph: the columns are
  schema `7`'s OWN (born with ADR 0016, written at ADR 0017) — THIS release READS them,
  the schema stays `8` (no hop, no new column).
- `vendor_buy` and `price_trend_normal` stay NULL UNTOUCHED (ADR 0016's honest grounds:
  NO producer ever wrote them — this release reads `price_min_now`'s twin, never fills
  the other two); the trio contract's NULL-vs-0 split and ADR 0015's money rule
  (words NEVER beside auction COUNTS — no count moved) ride unchanged.
- The version banner moves `0.10.0 → 0.11.0` (Cargo.toml + lock, `chore`): a FEATURE
  release — a new cell shape, new door fields READ, a new CI step — not a patch over
  the old shape (contrast v0.10.1, which left the banner put through its patch chain).
  Schema stays `8`.

Refs: ADR 0023, ADR 0017 (the producer side already measured there), ADR 0016's trio
contract, ADR 0015's money rule, ADR 0020's deferred question, ADR 0021's by-value
join, ADR 0012's local-door rule, [docs/DEVLOG.md](./docs/DEVLOG.md), tag `v0.11.0`.

## v0.10.1 — the paging doors ride one rule: `--offset` without `--limit` answers, twice · 2026-11-22 · tag `v0.10.1`

A patch era: v0.10.0's post-tag map measured a defect class that lives on TWO doors and
named the fix FORWARD; this chain is that fix. One composition rule, called by the two
doors that page — no feature, no schema move, no new door.

Fixed

- **`--offset` WITHOUT `--limit` answered an error instead of rows — on BOTH paging doors.**
  `search sword --offset 3` and `recipes --offset 10` answered rc **4** carrying SQLite's raw
  refusal `near "OFFSET": syntax error … OFFSET ?`: SQLite refuses `OFFSET` unless a `LIMIT`
  precedes it. The recipe twin was recorded in ADR 0021's post-tag section; the search twin
  shows the class was BORN with v0.9.0's door and never measured until the v0.10.0 exit map
  ran it today. One rule answers it twice (ADR 0014's one-code-path): `query.rs::paging_clause`
  is the ONLY place a paging clause is written, `search_plan` and `recipes_rows` call it, and
  "the rest of the rows" rides SQLite's OWN negative-limit token — measured today on the DB
  this tool links (sqlite 3.54.1): `LIMIT -1 OFFSET 3` answers rows 4..10 of a 10-row scratch
  table, IDENTICAL to the bare SELECT's tail. Measured after the fix (a 12-row scratch store
  born by the DB's own writer, ZERO requests — both doors are local): `search paging --offset
  4` answers the 8-row tail, `--offset 11` answers 1 row, `--offset 40` answers an EMPTY set
  rc **0** (the door's no-row-past-the-end shape, never a 4); `recipes --offset 10` answers 2
  rows of `matching 12` (spell ids 3024/3023), `--offset 2` answers 10, `--offset 12` answers
  empty rc 0, `--profession Alchemy --offset 1` answers 1 of matching 2.
- **The working pair did not move** (ADR 0022): `--limit 4 --offset 10` answers the SAME rows
  before and after — measured md5-equal against the PINNED v0.10.0 binary over one shared fresh
  fixture (`recipes --limit 4 --offset 10` 1 629 B both sides, `search sword --limit 4 --offset
  4` 182 B both, `search sword --limit 4` 185 B both). The fix rides ONLY the broken composition.

Changed

- **The CI block grew TWO steps (41 → 43 step-lines + plant one)** so CI can SEE the fix:
  `recipes --offset 10` + `search sword --offset 4`, ZERO requests added (both doors are LOCAL,
  ADR 0012's rule, so the GET tallies cannot move). Fresh mirror measured BEFORE the block
  landed, driver generated from these very steps: **46/46 commands rc 0, ZERO NONZERO**
  (`TOTAL_STEPS=46`); the steps' rc delta measured BOTH binaries over the CI DB — rc **4**
  pinned (raw refusal) → rc **0** now (`search sword --offset 4` answers `total 0` + `hits []`:
  the store holds no sword row there, and an empty tail is the door's own shape, never a 4;
  `recipes --offset 10` answers 2 rows of `matching 12`).
- ADR 0022 records the rule (SQLite's measured negative-limit semantics, and the two shapes
  REJECTED instead: a second SELECT asking the DB how big the store is would add a query the
  door never needed, an arbitrary cap would hide rows); ADR 0021's post-tag section gains the
  CLOSED pointer; `docs/DATABASE.md` states the composition rule beside the search SQL it
  already documents. NO redaction parser was invented for a raw-SQL message class that can no
  longer occur (inventing a decoder for text the tool can no longer produce is invention) —
  ADR 0004's message rule is satisfied by composition, not by a filter.
- Money (ADR 0015) rides the SAME steps untouched: the fixture RUN's pair `18908 ↔ 1g 89s 8c`
  beside every reagent copper; yesterday's `17371 ↔ 1g 73s 71c` moved because the fixture seeds
  prices per RUN — recorded honestly, the rule did not. The README banner moves to v0.10.1;
  the Cargo banner stays put at `0.10.0` through a patch chain (the same honest shape the v0.9.1
  patch era left put), schema stays `8`.

Refs: ADR 0022, ADR 0021's post-tag defect record, ADR 0014's one-code-path rule,
ADR 0012's local-door rule, ADR 0004's exit-code contract, ADR 0015's money rule,
[docs/DEVLOG.md](./docs/DEVLOG.md), tag `v0.10.1`.

## v0.10.0 — the recipe doors: every job with every recipe, icons + money beside the materials · 2026-11-22 · tag `v0.10.0`

Added

- **`sync --stage recipes`, the SIXTH crawl stage and the only one on a SECOND host** (ADR
  0021): eight conditional GETs, one per classic `<profession>_recipes.json` of the MIT
  dataset docs/API.md now records (241 981 .. 26 372 B each, **2 843 484 B** and **1 240
  entries** measured the same day), the file ETags riding `meta` (`recipe_etag_<file>`)
  so a re-pull pays real `304`s and writes NOTHING, the UPSERT (`ON CONFLICT (spell_id)
  DO UPDATE`) keeping the rows a re-answered file writes, and the resume cursor being the
  FILE count already done — the same stop→resume discipline every other stage keeps, kept
  for a dataset instead of a queue. `--budget` bounds FILES per run. The stage is
  deliberately NOT chained into "Pull all info" (ADR 0021 records why: the chain's
  contract is "the catalogue is complete" over the API's doors).
- **Schema 8: the `recipe` table + the `recipe__crafted` index** — additive, born fresh
  and arriving by hop (`SQL_MIGRATE_TO_V8`, the ladder's case `7 =>`), so a v0.9.9 file
  (19 258 rows) migrates in place instead of being refused; measured pre-tag on a
  schema-7 scratch copy: the table + index land, rows intact, a SECOND open re-migrates
  NOTHING, and the pinned v0.9.9 binary refuses the new file honest (rc 4, file untouched).
- **The `recipes` verb — the LOCAL door** (zero requests, the local-door rule ADR 0012
  set for search and stats): `recipes [--profession NAME] [--limit N] [--offset N]
  [--icons]` reads the table and joins BY VALUE through the item rows this DB already
  owns, so the cooked item AND every reagent answer their live name, their money words
  and their icon. `--profession` rides the dataset's OWN word (measured per file:
  `Cooking`, `First Aid`, …), bound never interpolated; the count and the rows share ONE
  WHERE chain (the "showing 40 of 1250" rule cannot drift); an empty table is
  `not_found` rc 4 — the door says the stage never ran, it never prints a recipe from air
  (ADR 0004).
- **The EIGHTH nameplate tab** (ADR 0018's frame kept): one row per recipe — profession
  chip · recipe name · skill requirement + tier band · one material cell per reagent
  (icon = the DB's OWN bytes as a data URI, `◇` otherwise; price in money words with the
  EXACT copper in the hover, via the ONE `money_cell`) · the cooked item through its own
  card. The Sync tab grows its OWN button `Recipes: fetch the set` on the SAME
  `runSync(stage)` loop as every other stage button (ADR 0019's shape, ADR 0014's one
  code path), and `Watch the cooked items` batches ONE `watch add` call over the cooked
  ids of the shown rows, skipping honestly the two kinds it cannot watch (an
  enchanting craft has no crafted item; an id the DB never has a row for is refused by
  the watch verb's own guard).
- **CI grows SEVEN steps** (34 → 41 step-lines, plant one): the stage pressed THREE times
  (budgeted 3 files / 5 rows / 4 requests with the fixture's one-429 gate retried, resumed
  at file 3 and completed 5 more files / 7 rows, then the re-pull answering **8 files,
  8 answered the same copy, 0 rows written**), and four `recipes` door steps that read the
  fixture's 12 rows with ZERO requests. Measured mirror answers: `matching 12`, money
  beside EVERY copper answer (8 pairs at `19603` ↔ `1g 96s 3c`), 11 reagent slots riding
  DB rows (3 of them `price_copper: null` → money null → the UI's "no price"), 8 slots
  `in_db: false` with "not in your DB" beside the set's own words, 10 crafted slots riding
  DB rows (`Fixture Detail 25/40/60/12809` — the detail stage's overwrite, the set's
  `crafted_item_name` riding BESIDE it) and 2 answering `crafted_item_id: null` → "no
  crafted item — enchanting craft".

Honest about what did NOT move

- **The API's doors answer nothing about professions or materials** (measured 2026-11-22,
  docs/API.md's new section): the five probed professions paths answer `404`, 0 of 716
  recipe-card copies say "Requires Materials", 699 of 716 say "Teaches you how to…", the
  `?q=Recipe` door answers NAMES only (259 ids, undercounting the 716 type-`Recipe` rows),
  and Blizzard's own `data/wow/recipe/2424` answers `404` today (the exact namespace its
  2025 bug report names, plus the 11.2.0 guesses). The tab's job words/skill numbers come
  from the FETCHED set and its flavor prose — never invented from an API shape.
- **No Jewelcrafting, no TBC rows, no forever-beta rows** (ADR 0021 + the tab's own note
  say so): the classic set measured today has EIGHT professions, the set's TBC files are
  another era, its `*_forever_recipes.json` files another id space (`spell_id 1283400`,
  `crafted_item_id 279961`) that cannot join these rows.
- **Money keeps ADR 0015's rule for the new tab**: words beside every copper answer (reagents
  AND the crafted item), the exact copper in the hover, and NOTHING money-shaped beside an
  auction COUNT; an unpriced reagent answers "no price", never 0c (ADR 0016's NULL truth
  restated).
- **The dataset is FETCHED, never shipped**: 2.6 MB of an external set committed to the repo
  would be a second, drifting copy the day the source moves; the docs carry the measured
  counts, the fetched file is the truth, MIT attribution rides the footer, the tab note and
  docs/API.md.
- **`learned_json` is stored but has NO reader yet** (ADR 0021): the door's chip reads the
  entry's plain `acquisition` word; a stored field without a reader is dead code in the door's
  shape, the reader is a later honest question.
- **The four named warnings stay four** the whole chain (incremental-clear mechanic every
  build); `node --check` parses the grown `app.js` (PARSE witness only — the webview never
  launched in the build room, the launch-audit rule kept).

Fixed

- **`fix(db)`: the crafted join now carries the copper TWIN of its money word** — the field
  was in `RecipeRow` but the builder had no start and the join site filled only the words,
  so the compiler caught the missing field (E0063) before a money word could ever ship
  without its number (ADR 0015's contract needs both sides).
- **`fix(cli)`: a targeted ask on the recipe stage answers 2, never a whole pull** —
  measured BEFORE the fix on the REAL host: `sync --stage recipes --ids 25` answered rc 0
  with the whole 8-file pull (1 240 entries written), the flag doing nothing the report said.
  The refusal lands before `open_at`/`build_client`, so it spends ZERO requests (measured on
  a scratch DB, the fixture saw nothing).

## v0.10.0 — the recipe doors: every job with every recipe, fetched once, read locally ever after · 2026-11-22 · tag `v0.10.0`

Added

- **`sync --stage recipes`, the SIXTH crawl stage and the only one on a SECOND host** (ADR
  0021): the classic recipe set of the MIT second source docs/API.md now records — eight
  `<profession>_recipes.json` files (measured today: 241 981 .. 26 372 B each, **2 843 484 B**,
  **1 240 entries**, `recipe_count` matching every array), ONE request each, conditional GETs
  riding the `meta` table (`recipe_etag_<file>`) so a re-pull pays real 304s and writes NOTHING,
  the resume cursor being the FILE count already done and `--budget` bounding FILES per run — the
  same stop→resume discipline every other stage keeps, kept for a dataset instead of a queue.
  Against the real host the stage wrote **1 240 rows in 8 requests, rc 0** (the tool's own
  real-set witness, recorded in ADR 0021 and asserted nowhere in CI).
- **Schema 8: the `recipe` table + the `recipe__crafted` index** — additive, born fresh (SCHEMA_SQL)
  AND arriving by hop (`SQL_MIGRATE_TO_V8`, the ladder's case `7 =>`), so a v0.9.9 catalogue
  (19 258 rows) migrates in place instead of being refused. Measured pre-tag on a schema-7
  scratch copy born by the PINNED v0.9.9 binary: `7 → 8`, 5 rows / 4 priced intact, table +
  index land, 0 recipe rows, a SECOND open re-migrates NOTHING (byte-identical `status`), and
  the pinned v0.9.9 refuses the new file honest (rc 4, file untouched).
- **The `recipes` verb — the LOCAL door** (ZERO requests, the local-door rule ADR 0012 set for
  search and stats): `recipes [--profession NAME] [--limit N] [--offset N] [--icons]` reads the
  table and joins BY VALUE through the item rows this DB already owns, so the cooked item AND
  every reagent answer their live name, their money words and their icon. `--profession` rides the
  dataset's OWN word (measured per file: `Cooking`, `First Aid`, …), bound never interpolated;
  the count and the rows share ONE WHERE chain (the "showing 40 of 1250" rule cannot drift);
  an empty table answers **rc 4** — the door says the stage never ran, it never prints a recipe
  from air (ADR 0004).
- **The EIGHTH nameplate tab** (ADR 0018's frame kept): one row per recipe — profession chip ·
  recipe name · skill requirement + tier band · one material cell per reagent (icon = the DB's
  OWN bytes as a data URI, `◇` otherwise; price in money words, EXACT copper in the hover via
  the ONE `money_cell`) · the cooked item through its own card. The Sync tab grows its OWN
  button `Recipes: fetch the set` on the SAME `runSync(stage)` loop as every other stage button
  (ADR 0019's shape), NOT chained into "Pull all info" (recorded deferral, ADR 0021), and
  `Watch the cooked items` batches ONE `watch add` call over the cooked ids of the shown rows,
  skipping honestly the two kinds it cannot watch.
- **CI grows SEVEN steps** (34 → 41 step-lines + plant one): the stage pressed three times
  (budgeted 3 files / 5 rows / 4 requests / `rate_limited 1` → resumed at file 3, 5 more files /
  7 rows / `complete true` → the re-pull answering 8 files / **8 answered the same copy** /
  **0 rows written**), and four `recipes` door steps that read the fixture's 12 rows with ZERO
  requests — money beside every copper answer, the join states by name (`in_db: false` +
  "not in your DB", `crafted_item_id: null` + "no crafted item — enchanting craft",
  `price_copper: null` → "no price", the DB-side name `Fixture Detail 25` riding BESIDE the
  set's own `crafted_item_name`).

Honest about what did NOT move

- **The API answers none of this** (measured 2026-11-22, docs/API.md's new sections): the five
  probed professions paths answer `404`, 0 of 716 recipe-card copies say "Requires Materials",
  699 of 716 say "Teaches you how to…", `?q=Recipe` answers NAMES only (259 ids, undercounting
  the 716), and Blizzard's own `data/wow/recipe/2424` answers `404` today. The tab's job words /
  skill numbers / material LISTS are the FETCHED set's own — never invented from an API shape.
- **No Jewelcrafting, no TBC, no forever-beta rows** (ADR 0021 + the tab's note say so): the
  classic set has EIGHT professions; the set's TBC files are another era; its
  `*_forever_recipes.json` files carry `game_version "forever-beta"` in an id space
  (`spell_id 1283400`, `crafted_item_id 279961`) that cannot join these rows.
- **The join's states are measured, not guessed** (same day, the user's own 19 258-row file):
  1 105 distinct crafted ids → **1 095 have an item row, 10 do not**; **133 entries carry NO
  crafted id at all, all 133 Enchanting**; 416 distinct reagent ids → **415 have a row, 1 does
  not**, **392** already answer a `price_min`.
- **Money keeps ADR 0015's rule for the new tab**: words beside every copper answer (reagents AND
  the crafted item), exact copper in the hover, NOTHING money-shaped beside an auction COUNT or
  a row count; an unpriced reagent answers "no price", never 0c (ADR 0016's NULL truth restated).
- **The dataset is FETCHED, never shipped**: 2.6 MB of an external set committed to the repo would
  be a second, drifting copy the day the source moves; the docs carry the counts, the fetched
  file is the truth, and MIT attribution rides the footer, the tab note and docs/API.md.
- **`learned_json` is stored but has NO reader yet** (ADR 0021): the door's chip reads the entry's
  plain `acquisition` word; a stored field without a reader is dead code in the door's shape.
- **The four named warnings stay four** the whole chain (incremental-clear mechanic every build);
  `node --check` parses the grown `app.js` (PARSE witness only — the webview never launched in
  the build room, the launch-audit rule kept).

Fixed

- **`fix(db)` `be9e934`:** the crafted join's money word had NO copper twin — `RecipeRow.crafted_price_copper`
  existed in the struct but the builder had no start and the join site filled only the words, so
  the compiler caught the missing field (E0063) before a money word could ship without its number.
- **`fix(cli)` `24e02bc`:** a targeted ask on a whole-door stage now answers **2**, never a silently
  ignored flag — measured on the REAL host BEFORE the fix: `sync --stage recipes --ids 25` answered
  rc 0 with the WHOLE 8-file pull (1 240 entries written), the flag doing nothing the report said.
  The refusal sits before `open_at`/`build_client`, so it spends ZERO requests (measured on a scratch DB).

## v0.9.9 — the live-sort knob: order what the AH offers NOW, the shown cell stays the stored min · 2026-11-21 · tag `v0.9.9`

Added

- **`--sort price-now` on the search door** (ADR 0020): `ORDER BY (price_min_now IS NULL) ASC,
  price_min_now ASC, name ASC` — the SAME NULL-last pattern every other price sort already uses,
  and honest about WHICH NULLs land there: rows never refreshed AND rows checked price-less at
  ask time (their own stamps keep the kinds apart; the sort does not care and does not guess).
  Usage string lists the key; unknown keys still fall back to the default exactly as before.
- **`cheapest LIVE price first` in the Search tab's sort field** — the select's value rides the
  EXISTING `--sort` argv (zero new wiring, ADR 0014's path).
- **The note moves with the knob** (only when it is on): "this order reads the LIVE min buyout
  the Sync tab's Price button (or the CLI stage) wrote, one ask per copied record ever; the price
  column stays the DB's STORED min, and rows with no live price sort last…" — not decoration: the
  live order can make the price column LOOK out of order (measured mirror: 60 live 16 106 before
  25 live 17 106, both stored 18 106), so the note names which is which beside every hit.
- **The CI block grows ONE step** (33 → 34 step-lines, plant one): `search detail --where all
  --sort price-now --limit 8` — green BECAUSE the knob works (measured both binaries: v0.9.8
  answers the default fallback 12809,25,40,60; v0.9.9 answers the live order 60 before 25);
  ZERO requests added (search reads the DB), GET tallies cannot move.

Honest about what did NOT move

- **The price CELL stays the stored min** (deferred, said so in ADR 0020): showing the live price
  IN the cell needs per-row live fields on the wire + their own money pair — next release's honest
  question; the note carries the sentence instead this release. Money rule (ADR 0015) rides
  untouched: the grown sentence carries NO numbers at all, hover-copper unchanged.
- **`--priced` filter stays the stored-min rule** — the knob is an ORDER, not a FILTER.
- **Schema 7 unchanged — NO hop**: `price_min_now` is already a schema-7 column (born at 6 with
  ADR 0016); this release READS it for the first time.
- **The wire did not move for everything else**: default-order doors BYTE-IDENTICAL vs the v0.9.8
  binary (`search sword --where description --sort price --icons --limit 5` 3 359 B; `detail
  --where all --limit 8` md5 equal) — the knob rides ONLY the new key; bogus sort key still
  IDENTICAL hits vs default (fallback rule unchanged); exit-map totals re-run post-tag (101 doors,
  expected identical) in the verification fold FORWARD.
- The webview's rendered look stays never-witnessed (no display): the option is a grep+parse
  witness; ADR 0011's line: the CI block grew a witness of a DOOR's order, not the select.

Chain: `b8abbd8` feat(search) → `536ea1d` feat(ui) → `b700a57` ci → `8ed805d` chore → this record
carries the tag.

## v0.9.8 — the Sync tab grows its stage buttons: icons / descriptions / price refresh, same argv, one honest note each · 2026-11-21 · tag `v0.9.8`

Added

- **Three stage buttons in the Sync tab** (ADR 0019): `Icons: copy the files` · `Descriptions: copy` ·
  `Price: live refresh` — the stages the v0.9.6/v0.9.7 records had kept CLI-side ("said so, not
  slipped in") now have doors. All three land in the SAME `runSync(stage)` loop: same chunk of 25,
  same cancel-at-chunk-end, same `sync_state` resume, same "no progress this chunk → stop" guard, and
  same `cli_ok(["sync","--stage",X,"--budget",chunk])` argv the cargo line takes — ADR 0014's rule:
  a click and a `cargo run --` are ONE code path down to the report JSON. A second Price click asks
  ZERO (ADR 0017's pending rule: one live ask per copied record, EVER, measured on this same verb).
- **The runSync completion note names the third state when there is one**: `, N answered the stage's
  own third state (checked once, never asked again)` ONLY when the report carries `missing > 0` —
  deliberately generic words ("the stage's OWN third state", ADR 0006's pattern): detail = gone
  upstream, icons/tooltip = file/copy 404, price = the AH offers nothing right now. Reports without
  that field keep their note BYTE-IDENTICAL — no `missing 0` noise is invented.
- **Notes moved the SAME breath so none can lie beside the buttons**: the card's static note names
  the door ("the Sync tab's Price button or the CLI stage") and its API-touch claim gets its honest
  card-scope ("on the card itself…"); the card's dynamic sentence names the door that exists; the
  live-row comment replaces the v0.9.6 "the panel has no button for this stage" claim with what is
  true SINCE this release (gui.rs facts stay: the Rust shell still builds no card, reads no column —
  the button is webview-side).

Honest about what did NOT move

- **Zero Rust**: the release diff names `ui/index.html` + `ui/app.js` (buttons + notes) + Cargo.toml/lock
  (banner) — every stage verb, pending rule, stamp and exit code was ALREADY the CLI's; the wire is
  byte-unchanged BY CONSTRUCTION, and the post-tag replays (mirror replay on fresh ports, wire byte
  pairs vs the v0.9.7 binary, the 90-door map + 11 extension doors re-run) MEASURE that, landing
  FORWARD in the verification fold.
- **Pull-all chain stays list→detail→icons→tooltip, price NOT chained** (ADR 0019's recorded deferral):
  the chain's contract is "the catalogue is complete", the refresh re-asks a LIVE pair under the
  once-EVER rule — chaining it would make a second pull-all ask zero and say nothing new; the button
  is its own activity, said where the chain's note lives, not silently added.
- The webview itself stays unlaunchable here (no display): `node --check` is a PARSE witness, grep
  witnesses count 3+3+1 — the rendered panel is recorded as never-witnessed, exactly as ADR 0018 drew
  the line for the frame.
- CI block UNCHANGED (still 33 step-lines + plant one): the block already runs the very stage verbs
  the buttons reach; ADR 0011's rule holds (CI cannot see the webview; the release grew no witness it
  cannot keep).

Chain: `058d21b` feat(ui) → `d60bc84` chore (banner 0.9.8, lock rides) → this record carries the tag.

## v0.9.7 — the panel gets the AH frame: seven activity tabs, the same verbs under the skin · 2026-11-21 · tag `v0.9.7`

Added

- **Seven activity tabs** (ADR 0018): the crammed page (EIGHT cards stacked at once) becomes
  `Overview · Search · Sync · Item card · Writes · Live look · Analytics` on a nameplate strip
  inside the panel's top edge. The twin-refresh button JOINS the Writes tab — the four DB verbs and
  the twin refresh are ONE activity (both send ZERO requests). One truth in app.js (`show_tab`)
  keeps exactly one tab visible (`tab-off` display:none — the layout enforces the rule, hidden
  buttons get no events); the strip rides the delegated-handler pattern the hits table already
  used; the first paint is PAINTED by the static classes so it never waits on script.
- **The Auction House frame — CSS-ONLY**: deep blue-black room with light falling from the top,
  AH-panel fields with bronze trim and a gold inner hairline, the AH's corner ORNAMENTS as three
  borders (`main::before`/`main::after` L-brackets), nameplate tabs (uppercase, letter-spaced, lit
  tab gold with a gold underline), parchment-gold headers, squarer radii. NO Blizzard asset copied
  — only the SILHOUETTE (panel + gold trim + tabs) is borrowed, in plain CSS; still no webfonts,
  no remote assets, no generated images (the offline rule keeps its teeth).

Fixed

- **The live note's dangling fragment** (`fix(ui)`): the live card's success path ended with
  `+ hits_link` — an identifier with NO definition anywhere (grep counted ONE occurrence: the use
  site), so the note building ALWAYS threw a `ReferenceError` and the card answered with raw error
  text instead of the door's note. Measured with node both ways (the original expression throws;
  the FIXED real line runs and RETURNS the note), fixed FORWARD: the honest sentence the fragment
  promised but never delivered — every live hit row already carries its own upstream `view` link.

Honest about what did NOT move

- **Zero Rust**: the release's diff names 5 files (ui/×3 + Cargo.toml/lock banner); `SCHEMA_VERSION`
  stays `7`, no hop; the wire doors answer byte-identical across the release (mirror replay + exit
  maps re-run on FRESH ports post-tag, landing FORWARD in the verification fold — ADR 0011's rule:
  the CI block cannot see the webview, and this release did not touch it either: still 33 step-lines
  + plant one).
- **What cannot be witnessed here, recorded as such**: the window's LOOK (no display on this FS —
  never claimed), and the embed: the binary maps the ui FILENAMES in plaintext (`index.html` ×14,
  `app.css` ×1) but tauri v2 COMPRESSES the content (content greps answer 0) — this CORRECTS the
  v0.9.6-era phrasing "the green run re-embedded the edited files" where quoted in this record:
  the greppable witness is the filename map + green build, not the content bytes.
- The crawl buttons stay the five (list · pull-all · details:watch · details:hits · Cancel) —
  icons/tooltip/price stages stay CLI-side, notes unchanged; quality chips keep the GAME's hues
  (a fact stays a fact); money rules ride UNTOUCHED (ADR 0015); icons stay DB-bytes data URIs
  (ADR 0008/0014); the attribution footer stays OUTSIDE the tabs so it reaches every screen.
- bends recorded, not swept: the FIRST build after `fix(ui)` skipped the house's
  `rm -rf target/debug/incremental` mechanic and the FS's spurious hard-linking warning counted as
  a fifth line ("5 warnings" in that summary) — the FOUR named warnings were always exactly four,
  re-measured with the mechanic the same breath.

Chain: `35342b9` feat(ui) tabs → `24971fe` fix(ui) live note → `5c7eb09` feat(ui) frame skin →
`c6d2b18` chore banner → this record carries the tag.

## v0.9.6 — the live min buyout: ADR 0016's first producer, one live ask per copied record · 2026-11-21 · tag `v0.9.6`

Added

- **The FIFTH crawl stage, `sync --stage price`** (ADR 0017): one live ask per record that ALREADY
  holds a tooltip copy, and the door is the TOOLTIP door itself — measured live 2026-11-21 it
  answers `fields.price` with a CURRENT min (id 25 `min 149` @ its own `updated_at 22:25:10`; ids
  202/237/763 answer 5000/3500/4425 while their STORED copies carried NO price at all, so the
  live value moves APART from `price_min` ON THE SAME record — ADR 0016's two-columns rule with
  evidence). It writes ADR 0016's reserved `price_min_now` plus its OWN stamp pair (`price_now_at`
  = our ask time; the door's `updated_at` TEXT is deliberately not stored — two stamps of one
  fact, ours says when WE looked). Schema **6 → 7**, additive ALTERs, `check_schema` now chains
  v1..v6 up to 7.
- **The pending rule is the crawl twin** (copy EXISTS — only copied records have something to
  re-ask, ADR 0010's measured reason — AND never written AND never checked-empty, no manual rows,
  no tombstones, `id ASC`): **one request per copied record, EVER** (measured: a second press
  asks ZERO, `complete true`). NO `If-None-Match` even though the copy stage stored an ETag:
  `fields.price` was NEVER parsed before, so a 304 would answer "unchanged since a read that
  never saw this field" — the door's own ~5-min server cache keeps the pass polite. Fresh
  writes sample NOTHING: ADR 0007's price sample stays on the DETAIL write (the trend that
  would eat denser samples is later work). The stage counts and stamps the measured third
  state (live id 200 answers `fields.price` as literal NULL — "looked, the AH offers NOTHING
  right now" → `price_now_missing_at`, value NULL, never invented).
- **The card door grows the live pair** as top-level siblings: `price_min_now` + `price_min_now_money`
  (ADR 0015's money formatter rides the live copper; hover keeps the exact copper; `price_available`/`price_realms`
  stay auction COUNTS with no money), and the card UI grows ONE money row beside its AGED stored
  rows (measured pair: live `16190 ↔ 1g 61s 90c` against stored `17190 ↔ 1g 71s 90c` on the
  same row). Search/stats doors stay UNCHANGED (the sort column stays `price_min`, the live-sort
  knob is later work); `gui.rs` was read before the claim — the window gains nothing (it draws
  no card, reads no column).

Fixed

- **The third state COUNTS** (`fix(sync)`): the first mechanism run STAMPED the null-price row
  but the report answered `missing:0` — the stamp existed, the counter did not, so the report
  lied about the door's third state; counted the same breath, re-measured `missing:1` + its note.

Honest about what did NOT move

- **`vendor_buy` and `price_trend_normal` stay reserved on MEASURED negatives**: no vendor-buy door
  or key anywhere (4 602 detail copies answer the SAME 28-key set, `vendor_sell` only; upstream's
  OWN developers page lists exactly FOUR doors — list, record, search, tooltip), the only upstream
  "can be bought" signal is its own flavor prose `"Available from enchanting vendors."` on exactly
  ONE copy (id 247786) — prose is not a price; ZERO trend words in 19 258 rows (all 20 "volume"
  hits are `Volume I/II/III` book titles) and no trend door. NULL in THOSE two still means
  "no producer has ever written it".
- **The card door is NOT byte-identical this release: +2 keys** — that is THIS release's documented
  wire delta; v0.9.5's byte-identity claim stays scoped to that release's hop. `price_now_at` stays
  OFF the wire until a reader wants the when (recorded).
- bends recorded, not swept: the hop and the stage landed in ONE commit (`9f794e4 feat(db,cli)` —
  the planned separate `feat(db)` hop commit was never made, the "one logical change" rule bent
  and SAID so); the fixture's `RUN`-before-dict move needed TWO commits (the first only moved
  words — the rerun died with the same `NameError` — the second carried the actual move); the
  mirror shipped an unread `vendor_sell` field named by the build's FIFTH warning, trimmed the
  same breath (a mirror entry without a reader is dead code).
- CI **grew 31 → 33** cargo steps + plant line one (the refresh press + its idempotent second
  press), replayed **33/33 rc 0 + plant rc 0 BEFORE committing** on a driver generated from the
  committed block (fresh port 9207, fresh scratch DB): press 1 requests 3 / written 2 at live
  16 175/15 175 against stored 17 175 / missing 1 + note, press 2 ZERO asks. Post-tag audits
  (mirror replay, pre-hop pairs, the exit-code map grown honestly, the pinned window, the user's
  file 6 → 7) land FORWARD in the verification and CI folds.

## v0.9.5 — the price homes: FOUR columns own FOUR questions, carved now, filled later · 2026-11-21 · tag `v0.9.5`

Added

- **Three reserved columns** (ADR 0016): `vendor_buy` (what a vendor BUYS — only when the
  item can be bought at all), `price_min_now` (the CURRENT auction-house min buyout) and
  `price_trend_normal` (the historic "normal" the live trend answers) join the schema beside
  the `vendor_sell` / `price_min` / `price_market` homes that already exist. **Schema 5 → 6**,
  additive ALTERs, and `check_schema` now chains EVERY stale shape (v1..v5) up to 6 — the
  staircase v0.9.0's comment promised since its own hop.
- **The reservation moves ZERO bytes**: no producer, no reader, no door, no UI change this
  release — measured with BOTH binaries (the v0.9.4 tree at `4875222`, built in a scratch
  worktree, and this one) over two copies of ONE fixture DB: 12 of 13 doors BYTE-IDENTICAL
  at the same byte counts, and `status` identical once its own `db.path` echo is pinned
  (the paths cannot match: the OLD binary refuses the file the NEW one migrated).
- **The migration is real and idempotent**: the user's own file went 5 → 6 the moment a
  door opened it — **19 258 rows / 4 231 vendor_sell / 4 285 history samples intact**, trio
  NULL on all of them, and a SECOND open re-migrated NOTHING (second `status`
  byte-identical).

Honest about what did NOT move

- **NULL in the trio means "NO PRODUCER HAS EVER WRITTEN IT"** — deliberately NOT yet the
  "fetched / fetched-and-not-there" split the price columns carry; the third state arrives
  WITH the first producer, never before it (ADR 0011's rule: no plan for a door that does
  not exist). HOW the values get fetched is later work — the user said so, and no upstream
  door has EVER answered these words, so the column NAMES are ours and docs/API.md gains
  nothing (never invent upstream shapes).
- `price_min` (the stored copy's cheapest AH buy, an aging payload's number) and `price_min_now`
  (what the AH says NOW) are TWO columns on purpose: one column answering two questions over
  time is two truths in one word.
- **money (ADR 0015) waits**: `money_of` takes any `Option<i64>` copper, so the words will
  reach these columns the day producers write them — with ZERO measured rows there is ZERO
  money to answer now, and money of NULL was rejected by ADR 0004's rule again.
- **CI gains no step** (31 stays 31 + one plant line, verified by step-line count after the
  edit): the smoke file is now BORN at 6, and the ALTER chain does NOT run in CI — CI has
  no schema-5 file — so the migration gets measured on scratch copies in the post-tag audit
  (ADR 0011's honest split).
- **the stale direction keeps its honest refusal**: the pinned v0.8.1 binary answers exit
  4 — `database on disk uses schema 6, this build wants 4` — over a migrated file, untouched.

Refs: ADR 0016, ADR 0015's money formatter (it will reach these columns later), ADR 0007's
sampling rule, ADR 0011's CI rule; schema is `6` — the FIRST hop since v0.9.0's own.

## v0.9.4 — prices read like the GAME writes them (`2g 50s`), copper stays the number · 2026-11-21 · tag `v0.9.4`

Added

- **Money words beside every copper number** (ADR 0015): one formatter in `src/db.rs`
  (`money_text`, `money_of`, `money_of_avg`) writes the game's OWN wording — 100 copper is 1
  silver, 100 silver is 1 gold, and the unit is proven from the **DB's own tooltip copies**,
  because a copied tooltip already states its sell price in money words (`money_text(vendor_sell)`
  equals the row's own copy for **4 230 of 4 231** rows). A zero denomination is DROPPED exactly
  as the copies write it (`25000` ↔ `2g 50s`, `20000` ↔ `2g`, `20042` ↔ `2g 42c`), and the average
  **floors** to whole copper (`AVG(price_min) 54 391.705896` → `5g 43s 91c`).
- **The money siblings ride the answers, not the DB**: `Hit` grew `price_min_money`/`price_market_money`,
  `Coverage` grew `min/max/avg_price_money`, `QualityStat` and `Extreme` grew their mean/row money,
  `PriceSample` grew the sample pair (filled by `history_of` from the copper the read carries), and
  the card answers `price_min_money`/`price_market_money`/`vendor_sell_money` as top-level sibling
  fragments beside `item` — never inside it, the same ground `icon_b64` has: `ItemDetail` and `Price`
  mirror the wire (docs/API.md), and money is OURS, upstream never answered that word. Money reads the
  MERGED detail (ADR 0010's merge-last rule), so a money word cannot disagree with the number beside it.
- **The UI shows money in the cell and the exact copper in the hover**: one `money_cell` renders
  the words Rust wrote, so the webview does no copper maths and cannot drift. Money reaches the hits
  table, both price lists, the card's price rows and its trend bars' hovers, the status grid's price
  range and average, the by-quality bar labels and the stats note's spread; `ui/index.html` names money
  in the three price theads and in the notes a card shows before its first click, and `ui/app.css` gives
  `.moneycell` `tabular-nums` so a money column's digits align.

Honest about what did NOT change

- **the DB grew nothing**: money is derived at answer time, so there is no `*_money` column, no
  schema hop (`schema_version` stays `5` on the mirror's and the user's DB after a replay that ran
  every money door), nothing to migrate and **no refresh door** — `refresh-twins` over the mirror still
  answers 4 copies / 0 rewritten / 4 matching / ZERO requests. The sorts and the scripts keep reading
  INTEGER copper (docs/DATABASE.md's numeric contract survives unchanged), and `SQL_COVERAGE`/`SQL_QUALITY`/
  `SQL_CHEAP`/`SQL_RICH`/`SQL_PRICE_SAMPLE`/`SQL_HISTORY_OF`/`HIT_COLUMNS` are untouched, so the read slots
  stay `0..14`.
- **`price_available` and `price_realms` never become money**: they are auction COUNTS (measured, row 25
  answers `available 12` for 12 offers of 15 000 copper; `realms` spans `1..12`), and money riding a count
  would be a lie — the card's `available` row stays a plain count.
- **an unpriced row keeps answering `null` beside `null`** and the UI cell still says `no price`, never
  `0c` (ADR 0004's rule against made-up zeros): measured, 0 rows in the user's DB sit at copper 0 in any
  of the four copper columns, so `0c` is a RULE and not a measurement.
- **money adds no CI step and no request**: the smoke block stays 31 `cargo run --` steps + one plant line,
  every step keeps its own exit code, and the fixture's GET tallies cannot move (ADR 0011's rule again).
- **money needs no flag**, unlike ADR 0014's icons: money costs about 56 bytes per priced row on the mirror
  and the longest money string measured in the user's whole DB is 13 chars (`1334g 99s 49c` for `13 349 949`).

Refs: ADR 0015, ADR 0010's merge-last rule, ADR 0004's exit-code contract, ADR 0011's CI rule,
ADR 0014's sibling-fragment precedent; schema stays `5` — money writes nothing.

## v0.9.3 — the lists draw the DB's own icon: `--icons` on the list doors · 2026-11-21 · tag `v0.9.3`

Added

- **`--icons`** (ADR 0014), a no-value flag on the two list doors: `search TEXT … --icons`
  and `stats --icons` put each row's icon next to it, from the **DB's own bytes** — the
  doors read `icon_blob`/`icon_at`/`icon_missing_at` and call the ONE `b64_text` encoder,
  which moved into `src/db.rs` beside the bytes it encodes so the card and the lists
  cannot encode the same copy two ways. **ZERO requests leave either door**, so ADR 0004's
  network door (`3`) is unreachable here: these doors answer `0`, `2` (usage) and `4`
  (missing DB) only, and the mirror replay's GET tally not moving is the proof.
- **The icon trio on the row shapes**: `Hit` and `Extreme` grew `icon_b64` / `icon_at` /
  `icon_missing_at`. The keys are always present; the base64 payload rides only when asked
  (measured: about 3.1 K chars per icon — avg 2 327 bytes per copy — so an unbounded hit
  list would be huge and `--limit` bounds it). Rows are COLLECTED before the per-row
  icon reads run, so a list can never straddle a write of the table it came from.
- **The UI draws an icon column** on the search hits and on both price lists
  (cheapest/priciest), asks both with `--icons` at its own bounds (40 hits, 6 extremes),
  and shows a placeholder glyph (`◇`) where a row has no bytes — with the WHICH state of
  the row's own stamps in the hover (`icon_missing_at` set → checked as gone upstream;
  neither stamp → never fetched). ADR 0008's rule again: upstream's `icon_url` rides the
  row shape but is never an image src.

Honest about what did NOT need to change

- **the item card needed no code**: it has rendered an actual `<img>` (a data URI, never a
  hotlink) since v0.6.0. The lists were the gap, the card was not;
- **the watch list gained no panel**: `ui/index.html` has no watch table (measured — the only
  watch surface is the `Watch these hits` button and the sync scope), so the watched rows
  gain icons where they appear, in the hits table, and no panel was invented;
- **the live-search table keeps no icon column**: that door reads and writes NO DB (ADR 0009),
  so upstream's names have no local copy to show.

Grounded by measurement, on the user's own DB

- 4 030 rows hold icon bytes, 1 was checked as gone upstream, 15 228 were never asked; all
  4 602 detailed rows carry an `icon_url` but only 4 030 carry bytes — so hotlinking the
  address would show an image the DB never claims;
- the encoder did not change across the move: `item 25 --cached` answered `icon_b64` of 2 596
  chars before and after, same sha256 prefix, and the emitted base64 decodes back to the DB's
  own 1 945 bytes.

Refs: ADR 0014, ADR 0008's icon stage, ADR 0004's exit-code contract, ADR 0011's CI rule;
schema stays `5` — the doors write nothing.

## v0.9.2 — old copies get their twin re-derived, with ZERO requests · 2026-11-21 · tag `v0.9.2`

Added

- **`refresh-twins`** (ADR 0013): one ask re-derives every twin in the DB from the copy it
  already holds. The copy is the source of truth, so **ZERO requests leave the door** —
  ADR 0004's network door (`3`) is unreachable here and the doors are `0` and `4` only.
  It writes only rows whose twin differs or is missing, rides tombstoned and manual rows
  (ADR 0012's rule is that the twin matches what the DB SAYS), collects every row before
  writing any (a sqlite row walk over the table it rewrites must not interleave), and is
  idempotent: a second press answers `0 rewritten` and still exits `0`.
- **The UI button `Reword the copies`** calls the SAME verb through the same one-command
  path (ADR 0005), and prints the door's own numbers.
- **CI smoke grew the door** (24 → 29 cargo steps + one plant line): first press answers
  0 rewritten / 4 unchanged, the v0.9.0 glued shape is planted with python3's own sqlite3
  (the same interpreter the fixture server needs — no new dependency), the phrase search
  answers `0`, refresh answers `1 rewritten`, the same search answers `1`, a third press
  answers `0`.

Honest size of the gain, measured on the user's 4 602 copies

- readable card row on every copy, and `twin == derivation(copy)` for the whole DB —
  ADR 0012's gap and v0.9.1's residue both close here, without a re-fetch;
- the description door is a substring door, so only phrases spanning an ELEMENT boundary
  gain findability: `damage speed` +358 rows, `hand one` +90, `sword 1` +14, `1 main` +3;
  `main hand` stays 75 before AND after and `sell price: 7` stays 291.

Changed

- `docs/DATABASE.md`'s honest-gap paragraph now names the door that closes it; the version
  banner moves 0.9.0 → 0.9.2 (the v0.9.1 patch chain left it put). The schema does NOT
  move: the refresh writes columns schema `5` already has.

Refs: ADR 0013, ADR 0012, ADR 0010's merge rule, ADR 0004's exit-code contract,
[docs/DEVLOG.md](./docs/DEVLOG.md), tag `v0.9.2`.

## v0.9.1 — the scope door answers and the twin keeps its spaces: v0.9.0's measured defects fixed forward · 2026-11-21 · tag `v0.9.1`

Fixed

- **`search TEXT --where …` hung instead of answering** (found by measurement, not by theory):
  both post-tag replays of v0.9.0 froze **2 h 01 m in state R** at the same door, and a scratch
  `timeout 5` repro answers **rc 124** — the `--where` block's bare `continue` on VALID values
  skipped the parser's shared value-flag advance (`index = index + 2`). It now falls through
  like every other value flag (`--sort`/`--quality`/…); measured after: the `description`/`all`/
  `name` doors answer rc **0** in well under a second and a bogus scope still answers the usage
  door **2** with the same message.
- **The twin glued words at element boundaries** (`Worn ShortswordItem Level 2Main HandOne-Handed
  Sword1 - 3 DamageSpeed 1.9…` across the user file's first 4 602 copies): live tooltip html is
  MINIFIED — adjacent spans, no whitespace between them — and the v0.9.0 derivation DROPPED spans.
  The fixture could not show this: its stand-in card carries spaces AT the tag boundaries, so
  fixture-shaped data cannot disprove claims about the live shape (only a live pass can). The rule
  now: every `<…>` span leaves ONE space, then collapse whitespace runs, trim.
- Honest residue, stated not swept: the 4 602 glued twins already on the user's file stay in their
  v0.9.0 shape until a copy's CONTENT changes (the pending rule never re-asks stamped copies, and
  a `304` writes nothing per ADR 0007) — a refresh door is a named plan for a later milestone, not
  invented code.

Changed

- ADR 0012's derivation bullet and audit row carry the correction (with the hung-replay and
  cancelled-run facts); `docs/DATABASE.md` and `docs/DEVELOPMENT.md` state the corrected span →
  space rule; the DEVLOG gained the v0.9.0 post-tag verification section and this entry.

Refs: ADR 0012, ADR 0004's exit-code contract, [docs/DEVLOG.md](./docs/DEVLOG.md), tag `v0.9.1`.

## v0.9.0 — the copies become READABLE and FINDABLE: every copied tooltip gets a readable twin · 2026-11-21 · tag `v0.9.0`

Added

- **The readable twin** (ADR 0012): schema `5` — `tooltip_text` / `tooltip_lc` derived from the
  copy **in the SAME write** on EVERY write path: the tooltip stage derives from the MERGED copy
  (ADR 0010's overrides merge LAST, so the twin always matches what the DB SAYS), `set-item ID
  tooltip_html=…` derives beside the stated value, `add-item` births it with the stated fields —
  the `name` → `name_lc` pair shape the name columns have ridden since v0.1.x. The derivation is
  exactly the measured three moves (drop `<…>` spans, collapse whitespace runs, trim): TWO live
  tooltip bodies read whole — the plain id 25 (1404 b of `html`) and id 728, picked off the USER's
  own DB because its `raw_json` carries flavor (`"Teaches you how to cook Westfall Stew."` shows in
  its `twdb-quote`) — hold div/span/img tags with class/src/width/height/alt/loading/decoding
  attributes and **ZERO entities**, so NO decoder is invented (an entity would show verbatim; the
  words still search). Why it had to exist: the copy's `<img>`s hotlink pixels from
  `render.worldofwarcraft.com` and `thewowdb.com/assets` — pasting the copy into a card would
  break the icon stage's rule (nothing hotlinks, the DB gets the bytes) — and grepping markup for
  words finds tags, not meaning.
- **The local search door grew a scope**: `search TEXT --where name|description|all` (the LOCAL
  door — upstream's OWN was measured NAMES-only at v0.7.0, docs/API.md): default `name` keeps
  every existing search exactly what it was, `description` reads `tooltip_lc` behind `IS NOT NULL`
  (never-fetched and checked-gone records skip honestly), `all` is name OR description; a bogus
  scope answers the usage door (`2`); tombstoned rows stay invisible in EVERY scope
  (`visible_clause` stays first); the door stays DB-only — ZERO requests.
- **The UI grew both sides**: the search panel's where select (names only | descriptions |
  anywhere — ONE argv pair, the same door the CLI answers) and the card's `description (readable
  copy)` row that RENDERS the twin as words, while the raw upstream markup row stays printed-as-
  TEXT (v0.8.0's escape rule, unchanged: an upstream body cannot write our card).
- **CI's smoke block grew 21 → 24 steps**: the `add-item` step now ALSO states a description (its
  twin is born beside it), and the block ends with `search sword --where description`,
  `search sword --where all` and the manual row's DB-only card (`item 900000000 --cached` —
  `MANUAL_ID_BASE` on a fresh DB). The fixture grounds the counts with its OWN code: its detail
  server assigns `body["name"] = "Fixture Detail <id>"` to every detailed row, so the LIST door's
  row named "Worn Shortsword" gets its name OVERWRITTEN and NO final name carries "sword" while
  the copied tooltip of 25 does — both search steps answer 2 (row 25 + the manual mug), a name
  scope would answer 0: findable exactly where names cannot.
- **`check_schema` now chains EVERY stale shape up to 5** (v1→v2→v3→v4→v5, all additive): the
  user's own 19 258-row catalogue rides the chain the moment a 0.9.x build opens it.

Changed

- `SQL_SET_TOOLTIP` grew the two derived columns (value count `4 → 6`, `bind_sql`'s marker count
  moves with it); the card now carries BOTH tooltip shapes (`tooltip_html` printed as text,
  `tooltip_text` rendered); `docs/DATABASE.md` grew the readable-twin section with the honest
  gap — a row migrated from schema 4 that ALREADY had a copy has no twin until its next tooltip
  write (SQLite's `ALTER` cannot run Rust's reader; duplicating it across languages is exactly
  the drift that file exists to forbid), and `docs/API.md` records the tooltip copy's measured
  shape (ZERO entities in two live bodies, the two hotlink hosts) as ADR 0012's ground.

Added

- **The repo's own icon assets**: `icons/icon.png` (512 px) and `icons/icon.ico` (seven
  entries, 16–256 px) — our own art, drawn by hand in PIL (a dark navy card, a gold coin, an
  "W" made of four strokes, never a font), verified real by `file` (`PNG image data 512 x 512`,
  `MS Windows icon resource - 7 icons`). They are product work, not a CI side effect:
  `tauri.conf.json`'s `bundle.icon` ALREADY named `icons/icon.png` for a file that never
  existed, and the windows-2025 build answered `` `...icons/icon.ico` not found; required for
  generating a Windows Resource file during tauri-build`` (run #3, measured).

Changed

- **The workflow is rewritten at the measured truths** ([ADR 0011](./DECISIONS/0011-ci-asserts-only-what-is-true.md),
  commits `c29e45c` → `5c888b9` → `f29a2b2` → `59b8379`): every `uses:` names a resolvable
  action (`actions/checkout@v7` — the step the old file lacked — and `dtolnay/rust-toolchain@stable`),
  the Linux dev libraries ride the sonames our OWN binary answers (`ldd target/debug/wow-goldtool` →
  `libwebkit2gtk-4.1.so.0 libgtk-3.so.0 libgobject-2.0.so.0 libsoup-3.0.so.0 libpango-1.0.so.0`,
  i.e. apt `libgtk-3-dev libwebkit2gtk-4.1-dev`, NOT the template's gtk-2 + webkit-4.0 pair),
  `cargo clippy --all-targets` stays INFORMATIONAL (measured: rc 0, 0 errors, 600 warnings —
  this build's four dead-code warnings ARE the plan `docs/DEVELOPMENT.md` names, so `-D warnings`
  would lie about it) and there is deliberately **no `cargo fmt --check` gate** (measured: it
  answers a diff on our hand-written style; a rustfmt-clean tree would be its own milestone).
  The workflow's header comment states both limits, so the file documents itself.
- **The scratch DB rides a real path and the macOS row rides the real arch** — the second
  measurement round (run #3) caught three more lies: `${{ runner.tmpDir }}` is NOT a context
  property, it evaluated to the EMPTY string so the smoke sequence died at its FIRST step with
  `cannot open /fixture.sqlite` → exit 4 (the local mirror could not see this because it hardcodes
  its own DB path — only a runner evaluates the env map); and macos-latest IS arm64 — the same
  run's toolchain step answers `host: aarch64-apple-darwin` — so `x86_64-apple-darwin` std was
  never installed and the build died with `can't find crate for std` (exit 101). Fix:
  `WGT_DB: fixture.sqlite` (relative, the checkout root, `*.sqlite` gitignored) and the matrix
  row `aarch64-apple-darwin`. Three promises still ride — linux, windows, macos.

Removed

- The three action names that never resolved (`actions/rust-action`,
  `ubuntu-setup-action/install-ubuntu-system-dependencies`,
  `windows-setup-action/install-windows-system-dependencies` — `gh api orgs/…` answers HTTP 404
  for the two orgs), and the two gate claims that were never true: a `cargo fmt` step (rewrites,
  never gates) and a `cargo lint` step (`error: no such command: lint` in cargo 1.94.0).

Nothing about the RELEASE moved: v0.8.1 is the discipline of the repository's own
infrastructure (ADR 0011), no `src/` line changed, the 21-step smoke block survives
byte-identical, and `cargo build` answers rc 0 with exactly the four documented warnings.
The banner rides 0.8.1 (`chore` `fe691c9`), so the USER_AGENT line built from
`CARGO_PKG_VERSION` now says `wow-goldtool 0.8.1` on every upstream ask.
**GitHub run #4 (`59b8379`): 5/5 jobs green in 9 min 13 s** — quality 2 min 04 s ·
build (macos) 2 min 27 s · test 4 min 19 s · build (linux) 5 min 46 s · build (windows) 9 min 12 s.
The whole story — four runs, every error quoted — is in [docs/DEVLOG.md](./docs/DEVLOG.md).

## v0.8.0 — every forever item's description copy lands, and the DB takes edits by hand · 2026-11-20 · tag `v0.8.0`

Added

- The **description copy as a FOURTH sync stage** ([ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md)):
  `sync --stage tooltip [--budget N] [--ids A,B] [--watch]` reads `/api/v1/tooltip/forever/item/<id>`
  and stores upstream's own `html` in `tooltip_html` + `tooltip_at` + `tooltip_etag`. It exists at all
  because the `forever/item` body carries NO description of any kind (measured top-level keys read
  back off the user's own `raw_json`: `attribution, icon_url, id, item_level, item_subtype, item_type,
  links, name, price, quality, required_level, unique, vendor_sell, weapon` — no prose field, no
  `html`, no `fields`). The door has the same three states as every other door — copied, checked-gone
  (`tooltip_missing_at`, from the measured `404` whose body is upstream's own JSON
  `{"error":"not_found","message":"No item 999999999999 in forever.","status":404}`), never looked at
  — revalidates with its own ETag (measured `W/"2b111cbcd3bbb4d7078f6d05"`, `cache-control public,
  max-age=300`), honours `Retry-After` and retries a `429` IN PLACE, so a rate limit never aborts a
  stage (ADR 0002/ADR 0004's contract). Cost ≈ 19 258 requests ≈ 2.7 h, staged/resumable/cancellable
  like the other three. ADR 0007's price sample stays on the DETAIL write only: a tooltip re-ask is
  not a second observation of the auction house in this build.
- The **manual layer**, one rule: overrides merge LAST. `manual_overrides` is one JSON ARRAY per row
  of `{field, value, at}` entries, applied AFTER the payload binds on every write path (`upsert_summary`
  from the list stage, `upsert_detail`, `write_tooltip`) AND on both read paths (`cached_card`, `cmd_item`)
  — so a field the user stated survives the next crawl while the columns the crawl owns keep refreshing,
  and a card can never show a value the DB does not have. Four CLI verbs: `set-item ID FIELD=VALUE`
  (exactly one whitelist field of one row, stamps `edited_at`, and rides the derived search columns
  `name_lc`/`quality_rank` in the SAME write so a rename is findable on the same breath), `add-item FIELD=VALUE […]`
  (the DB picks the id in the reserved block at `MANUAL_ID_BASE = 900000000` — measured max upstream
  id on the user's own file: `288 340` — stamps `manual_at`, and the row wears its fields as overrides
  FROM BIRTH), `delete-item ID` = the tombstone `deleted_at`, `undelete-item ID` clears it. Whitelist:
  `name, quality, item_level, required_level, item_type, item_subtype, flavor, vendor_sell, tooltip_html`
  — the auction columns stay out on purpose (an override "fixing" a live price is a permanent lie for a
  gold tool), so do `id`, the icon columns and the JSON-shaped ones.
- A UI write panel — **Say it / Make one / Hide it / Bring it back** — whose buttons send exactly those
  four verbs, and a card that prints the tooltip copy as **escaped TEXT** (it is upstream's own markup:
  rendering it would let upstream write the card), marks each field the user stated from the card's own
  override array rather than from what the panel typed, and refuses a tombstoned row.
- The status grid tells both new door families (`descriptions copied`, `checked: no description
  upstream`, `fields the user said`, `items the user made`) and "Pull all info" chains the fourth leg.
- CI grew the smoke sequence to **twenty-one steps**: the fourth stage budgeted (budget 8) and drained,
  then `set-item 25 vendor_sell=606`, `add-item "name=CI Test Mug" quality=Rare`, `delete-item 60`
  and `undelete-item 60` on the same DB.
- The fixture grew the tooltip door GROUNDED BY THE PROBE before those steps landed (unlike v0.7.0,
  where the CI steps landed before the fixture): its own `html` in upstream's class names with its
  OWN words (the probe measured upstream's 1404 b; the fixture never copies upstream's prose), its own
  ETag revalidation, and upstream's own JSON `404` body for ids it has no tooltip for — so one ANSWERED
  record (`12809`) has a real twin for the tooltip stage's third state. Its `429` gate names the tooltip
  stage, since that stage retries in place.
- `stats`/`status` coverage grew five counts — `tooltips`, `tooltip_missing`, `edited`, `manual`,
  `deleted` — and the price columns moved from read slots 7-9 to 12-14; the expected-count guard in
  the audits is what catches that drift, and the CLI, the UI and the audits read ONE answer.
- A row the user **MADE** answers its card from the DB's OWN columns (`SQL_COLUMNS_CARD`, 14 columns)
  on BOTH card doors, and the live door short-circuits BEFORE building a client — ZERO requests, because
  upstream cannot know the id. Fields only an upstream record can state answer `null` rather than being
  invented (the wire's labels, its `links` and `attribution`, the auction and weapon columns, the
  JSON-shaped ones).

Changed

- Schema `3 → 4`: four tooltip columns and four manual columns, all additive; `SQL_MIGRATE_TO_V4` is
  eight `ALTER`s and the chain now runs v1→v2→v3→v4, so the user's own 19 258-row catalogue comes up
  in one call. A file NEWER than this build is still refused, and that consequence was measured: a
  pinned v0.7.0 binary refuses the user's migrated file with `database on disk uses schema 4, this
  build wants 3` — which is why the real pull-all had to be re-pinned at a v0.8.0 snapshot.
- All three pending queues refuse a tombstone and a manual row (`SQL_PENDING`, `SQL_PENDING_ICON`,
  `SQL_PENDING_TOOLTIP` carry `deleted_at IS NULL` and `manual_at IS NULL`): a crawl never asks for a
  record upstream cannot know, and never re-stamps a row the user buried. An explicit `--ids` ask still
  reaches both — targeting is a human decision, a queue is not.
- `query.rs` grew `visible_clause()`, pushed FIRST in `where_parts`, so `search`, its `count_plan` and
  the card agree BY CONSTRUCTION that the user's rows are invisible (measured: a bare search's total
  went 6 → 5 across `delete-item 60`, and back to 6 across `undelete-item 60`).
- The version banner rides `0.8.0` (`Cargo.toml` plus the pinned `Cargo.lock`, the v0.7.0 precedent),
  and the usage line names all four stages and all four manual doors.
- The build's dead-code set grew by exactly ONE: rustc's "multiple fields are never read" on `Tooltip`
  — the measured key list rides the model so a key upstream adds cannot break a build, and the door
  reads only `html` (ADR 0009's rule: keep the measured mirror, read what the door needs). `catalogue`,
  `open_db` and `pending_plan`'s private `PendingSql` stay the documented set.

Fixed

- The fresh DB's `CREATE TABLE item` lost a comma INSIDE the Rust string — `"… tooltip_missing_at INTEGER"`
  glued onto `"icon_blob BLOB"` — so SQLite read `INTEGERicon_blob` as a type name: the `CREATE` **succeeded**
  and BOTH columns came out absent, so a fresh file had no icon bytes and no tooltip third state and every
  reader naming them would have died. The chain migration masked it (its `ALTER`s name the columns), the
  fresh write did not. Caught by reading `.schema item` back off a fresh DB. Same family as the earlier
  notes: a schema string that BUILDS is not a schema that WORKS.

## v0.7.0 — the live-search door: one human ask, one request, a look not a sync · 2026-11-20 · tag `v0.7.0`

Added

- CLI verb `live-search TEXT` ([ADR 0009](./DECISIONS/0009-live-search-door.md)): the WHOLE
  verb tail joined with spaces is the query (`live-search Worn Shortsword` asks one name),
  ONE request to the grouped `/search/` door, payload `{status, retry_after, remaining, answer}`
  with `answer` a serde pas-through of `SearchResponse` (`groups` stays a `Value`, upstream's
  `+` inside its own `more` URLs rides verbatim). Names-only means an all-empty answer is a
  success (exit `0`), and the door writes NOTHING: no row, no `meta` key, no sample, no mark.
- A live-search panel in the UI: one field, one `Ask once` button, one request per click, a
  `group · id · name` table with a `view upstream` link per hit (attribution reaches the
  ANSWER, not only the footer), and a note printing `remaining` (honestly `null` when the
  host sends no burst header), the names-only fact, and whether any group said `has_more`.
- The fixture grew the grouped `/search/` answer from the measured transcript (six groups each
  `{has_more, more, results}`, names-only substring matching, the `+` form inside its own
  `more` URLs), and its `429` gate now never names the `search` stage — the door answers a
  `429` with exit `3` (like the `item` door) rather than retrying in place, so a gated search
  stage would turn the committed CI step red.
- CI grew the smoke sequence to fifteen steps: three live-search asks — one word, the two-word
  ask riding as `%20` on the wire (the fixture's GET log is the witness), and the names-only
  trap whose all-empty answer still passes at exit `0`.

Changed

- The build's dead-code set shrank as promised: `search_url`, `SearchResponse` and
  `ApiResponse.remaining` — compiled-but-unused through v0.6.0 — got their consumer, and `cargo
  build` now warns exactly `catalogue`, `open_db` and `pending_plan`'s private `PendingSql`
  (plus the incremental-cache hardlink notice on this filesystem).
- The version banner rides `0.7.0` (`Cargo.toml` and the `User-Agent` line naming the source),
  and the usage line says both search doors now: `search TEXT […]` (local, filtered) beside
  `live-search TEXT` (upstream's own answer).
- Nothing schema-wise: schema stays `3`. The door touches no DB — audited with an impossible
  `--db` path answering exit `0` with the answer while its directory is never created.

## v0.6.0 — price history + local icons: the offline catalogue starts paying · 2026-11-20 · tag `v0.6.0`

Added

- A second table, `price_history`, and one price sample per **fresh** detail write
  ([ADR 0007](./DECISIONS/0007-price-sampling.md)): `upsert_detail` appends a row
  after a fresh (200) write — and only when the payload carried prices, so a record
  not on the AH never becomes a blank chart point — while a `304` revalidation, a
  list write, a check-mark or a re-insert write **nothing**. The ledger is what the
  card draws (one bar per stored sample, newest-N window) and what `stats`, `status`
  and the sync panel print, so the sampling rule is auditable, not decorative.
  SQLite's FK clause turned out not to be the guard: measured 2026-11-20, an `INSERT`
  referencing a missing `item` id succeeds, so the order inside `upsert_detail` is
  the guard and `seq` is auto-assigned so an append can never collide with an older
  observation.
- The icon stage `sync --stage icons` ([ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md)):
  every answered record's OWN icon file is fetched once and copied into the DB as a
  `BLOB` — three states like the detail record's: copied + `icon_at`, checked-as-gone
  + `icon_missing_at` (one `404` on the icon FILE of an existing record, never asked
  again), and waiting. The pending rule requires a NON-EMPTY `icon_url`: live, 4 of
  30 detailed records answer `"icon_url": ""` (Peace Tea, Craftsman's Writ, Scroll
  of Imbue Iceknife, Lodestone), and under a NULL-only rule those would keep pending
  forever. The item card shows the local icon as a `data:` URI — nothing is hotlinked,
  which is the offline point — and CI's fixture now answers `/icon/<name>.jpg` with
  REAL `image/jpeg` bytes (1945 b and 1936 b, fetched once from the grounded asset
  host and embedded so the suite stays offline) plus one record whose icon FILE is
  gone upstream, the third state the stage must check once.
- Schema 3: three `ALTER`s append the icon columns, a `CREATE` adds `price_history`
  and its index, and the chain migrates **both** stale shapes in place — a file
  saying `schema_version = 1` (the user's own 19 258-row catalogue) gets v0.5.3's
  `detail_missing_at` first, then the v0.6.0 batch; a v2 file gets just the batch.
  A file **newer** than this build is still refused.

Changed

- `SCHEMA_VERSION` `2 → 3`; both fresh-DB stamps write the build's own version, so
  a fresh v3 file never refuses itself.
- `sync_state` gained the third stage (`kind = "icons"`); `status` prints three
  crawl rows, the icon pair (`copied`, `checked`) and the ledger count, and the
  sync panel's grid + "Pull all info" follow the chain list → detail → icons.
- The item card renders `stats` as name/value pairs (the one previously-unshaped
  field the live API has ever filled — measured) and gives the never-measured
  fields (`sockets`, `socket_bonus`, `classes`, `effects`) an honest raw row only
  when they carry something; the icon row and the trend bars join.

Fixed

- The icon queue's empty-url exclusion above (a NULL-only rule would never drain
  on the measured empty answers); and `ORDER BY` stays plain `id` here, because
  unlike the detail queue every icon candidate already carries a detail stamp, so
  a re-ask-first term would be a no-op — the docs now say what the code does.

[ADR 0007](./DECISIONS/0007-price-sampling.md) · [ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md) · [docs/DEVLOG.md](./DEVLOG.md)

## v0.5.3 — patch: a record the upstream dropped is a state · 2026-11-20 · tag `v0.5.3`

Fixed

- A detail record the upstream no longer has (`404`) aborted the whole crawl
  (`item 41 answered 404` → exit `3` on the live catalogue). The list walk lists
  those ids — `OLD…`, `Deprecated …`, `… (TEST)` names sit in the user's 19 k rows
  — so the one-click "Pull all info" from v0.5.2 could never finish. `classify` now
  answers `HttpKind::Missing`, the detail crawl marks the id
  (`detail_missing_at`, guarded by a row-exists check so a mark cannot invent a row)
  and counts it in the report (`missing`), then keeps walking; the stage-2 upsert
  clears that stamp, so "gone" and "fetched" cannot both be true.
- Pending is now "never fetched **and** never checked-missing" (`SQL_PENDING`), so a
  removed record costs one request per crawl instead of one per run. The arithmetic
  holds on the migrated scratch DB: `3000 = 10 fetched + 3 checked-gone + 2987
  pending`, and the next run did not re-ask the three marked ids.
- `SCHEMA_VERSION` moved `1 → 2` **with** a migration: a file saying `schema_version
  = 1` gets `ALTER TABLE item ADD COLUMN detail_missing_at INTEGER` in place and its
  `meta` stamp moves in the same call, because refusing it would throw away a 19 258-row
  catalogue. A file **newer** than this build is still refused. Both fresh-DB stamps now
  write the build's own version, so a fresh v2 file never refuses itself.

Added

- Coverage counts the third state (`missing` in `status`/`stats`), and the status card
  prints it (`checked as gone upstream`). The pull-all notes say what a completion
  means with removed records in it: fetched, gone, and what remains live.

Changed

- `item <id>` on a removed record answers the exit-code contract's `4` (`not-found`)
  and re-stamps the mark: a human-aimed ask may re-ask once per request, a crawl may
  not. A **list page** answering `404` stays a failure (exit `3`).
- The fixture now lists one id (`127555`) that has **no** detail record and answers
  upstream's own error body, so CI exercises the third state: `item 127555` → exit
  `4`, and the crawl after it asks nothing. The exit-code audit gained that probe:
  `2 · 2 · 3 · 4 · 4 · 4 · 0`.

See [ADR 0006](./docs/DECISIONS/0006-checked-missing-third-state.md) and
[docs/DEVLOG.md](./docs/DEVLOG.md).

## v0.5.2 — patch: one click finishes the catalogue · 2026-11-20 · tag `v0.5.2`

Fixed

- Pending is now "never fetched", not "no price" (`SQL_PENDING` in `src/stats.rs`).
  The old `OR price_min IS NULL` clause kept a fetched item that is simply **not
  on the auction house** pending forever: the detail stage walked it on every run,
  so the stage never terminated on the real catalogue and no "pull everything"
  flow could honestly promise nothing left to pull. `docs/DATABASE.md` keeps those
  two states apart on purpose; the pending rule now uses the same distinction.
- A sync verb launched from the GUI never reached the API at all: the marker
  audit stopped after the DB-open point with an empty fixture access log, while
  status polls kept completing around it. reqwest's blocking client builds a
  **private tokio runtime on an internal thread** and parks the calling thread
  waiting for that runtime's startup handshake; the verb body sat on a worker of
  tauri's tokio-backed async manager, so the handshake never landed and the verb
  stalled at `build_client` with zero requests issued. `cli_call` keeps the async
  wrapper from v0.5.1 (the page still answers immediately) and the verb body now
  hops to a fresh `std` thread — the runtime-free home the headless CLI has always
  used in the CI mirrors.

Added

- "Pull all info" replaces the "Sync pending details" button: one flow runs the
  list walk to its end, then the pending-detail queue to its end, so a single
  click retires all pending work. `runSync` reports an outcome
  (`complete`/`cancelled`/`stalled`/`failed`/`already-running`) so a cancelled
  chain stops at the stage in flight instead of jumping ahead — the next click
  resumes the list from `sync_state` and the pending queue recomputes what details
  remain. A stage that is complete with zero requests is reported as complete,
  not as "made no progress". The completion note is explicit about what "nothing
  left to pull" honestly means: every listed item has a detail record, a NULL
  price on a fetched item means not on the AH, and prices stay live, so later
  syncs only revalidate (every page answers `304` from the stored ETags).

## v0.5.1 — patch: the crawl no longer freezes the webview · 2026-11-19 · tag `v0.5.1`

Fixed

- A running sync chunk no longer parks the whole app. A bare-`fn` command
  gets tauri-macros' blocking wrapper — its last acts are
  `kind.block(result, resolver)` then `return true` (`wrapper.rs:425`/`:428`)
  — so one chunk (2/s pacing, plus a `Retry-After` sleep on a 429: minutes in
  the worst case) ran synchronously **inside** the webview's IPC handler,
  which is the page's message loop: hover stopped highlighting, clicks queued
  behind the crawl for minutes. `cli_call` is now an `async` command
  (`respond_async_serialized`, tauri's tokio-backed async manager — tokio is
  a required dependency of tauri 2.x, so no feature switch was needed): same
  argv, same dispatch, same envelope, only the wrapper kind flips.
- Mid-chunk the page now says so: a ticker re-reads the DB once a second
  (coverage grows per page in list, per item in detail) and prints the
  chunk's elapsed time in the sync note; the cursor line honestly stays at
  the last chunk end, because progress truth lives in `sync_state` (ADR 0002).

Added

- The launch audit gained a witness: during a crawl, `gui: ipc verb sync`
  alternates with one-per-second `gui: ipc verb status` polls — that
  interleaving is what proves the webview runs mid-chunk
  (see [docs/DEVLOG.md](./DEVLOG.md)).

## v0.5.0 — detail-sync targets, price analytics, item card · 2026-11-19 · tag `v0.5.0`

Added

- Targeted detail sync: `sync --stage detail --ids 25,40,60` and `sync --stage detail --watch`
  visit exactly those records instead of the pending queue, which is what turns the watch list
  from something that prints into something that acts. The UI drives it with two buttons
  ("Details: watch list", "Details: shown hits") that slice the ids 20 per verb call, so
  Cancel between slices stays meaningful and one slice stays far below the 120-request burst.
- `item ID --cached`: a DB-only card read — no request at all. A record that was never
  fetched is `not-found` (the first use of that error kind, which retires its dead-code
  warning) and the UI turns the word into an offer instead of an empty card.
- UI cards: an item card that answers offline first (`Read the DB` vs `Re-check online`,
  printing when the record was fetched), a price-analytics card that renders
  `stats --limit 6` as per-quality bars in the search-chip palette plus cheapest/priciest
  extremes, and watch toggles inside the hit rows (the id travels in `data-id`, one
  delegated handler serves the whole table) with a `Watch these hits` batch action.
- Detail fetches send the stored `detail_etag` when one exists, so re-asking for the same
  ids answers `304` and rewrites nothing — the audit reads `written 0, unchanged 2`.
- The CI smoke block grew by six steps (watch add → targeted fetch → the same ids again
  answering 304s → watch-list target → DB-only card read), all eleven green in the local
  mirror before the commit.

Fixed

- Three single-column lookups in `db.rs` inferred a bare `String` getter for a nullable
  column, so a NULL came back as rusqlite's `Invalid column type Null` **DB error** (exit 4)
  instead of `None`. The pending stage never read those columns before fetching; the
  targeted stage reads `detail_etag_of` for every id, so the latent wrongness surfaced on the
  first audit run. The getters now ask for the `Option` type they always claimed to return.
- Targeted runs now answer their own question: `remaining` counts the target list ("left in
  the list", not "left pending") and `complete` means the target set was fully visited.
- `boot()` reads status, the watch list and the analytics at startup, so no card paints
  empty; the second status read is the launch audit's witness — four `gui: ipc verb …`
  lines prove the analytics render ran to the end rather than throwing after its await.

## v0.4.0 — Tauri shell + webview UI · 2026-11-19 · tag `v0.4.0`

Added

- The GUI shell lands: `tauri.conf.json` (window declared in `app.windows`, `withGlobalTauri`,
  `frontendDist: "ui"`, `bundle.icon`, no `version` key — see below), `src/gui.rs` (the
  `Builder` + `generate_context!` shell whose `run_gui()` blocks until the window closes) and
  `src/commands.rs` (the app's **one** command, `cli_call(argv)`, returning the envelope
  `{"ok":true,"result":…}` / `{"ok":false,"error":{kind,exit,message,retry_after}}`).
- `ui/index.html`, `ui/app.css`, `ui/app.js`: a status card (DB path, coverage, crawl state),
  a search card (text, quality, sort order, `priced only`) rendering the `Hit` fields with
  quality chips and a `view` link per record, and a sync card whose Cancel button works at
  chunk granularity. The attribution footer is static HTML — no code path can hide the link
  TheWoWDB's developer terms require.
- `src/cli.rs` grew `dispatch(Invocation) -> AppResult<String>`: the CLI door (`run`) and the
  GUI door (`cli_call`) send verbs through the same function, so ADR 0004's single code path
  reaches into the UI (see [ADR 0005](./docs/DECISIONS/0005-gui-ipc-commands.md)).
- Chunked sync from the UI: `sync --stage … --budget 10` (list) / `--budget 25` (detail) in a
  row until `report.complete`; Cancel raises a JS flag read at the next chunk boundary, and
  every chunk boundary is a `sync_state` write, so stopping is always resume-safe (ADR 0002).
- `build.rs` calling `tauri_build::build()` plus `tauri-build = "2.7.6"` as a **build**-dependency:
  this is what writes the global-API script list into `OUT_DIR`, i.e. what makes
  `window.__TAURI__.core.invoke` exist. It is cargo's own build script, not the Tauri CLI —
  [ADR 0001](./docs/DECISIONS/0001-flat-cargo-layout-no-cli.md) carries an amendment.
- `icons/icon.png` is generated by `tools/gen_icon.py` (codegen reads `bundle.icon` at compile
  time, so the file must exist before the first GUI build); `.gitignore` gains `/gen/` for the
  JSON schemas `tauri-build` writes next to `Cargo.toml`.
- `gui:` log lines on stderr (`gui: ipc verb status (0 words)` …) are the app's audit channel —
  nothing in `tauri`/`wry` forwards the webview console on Linux.
- `Cargo.toml` version `0.4.0`. `tauri.conf.json` deliberately omits `version`:
  `tauri-codegen` falls back to `env!("CARGO_PKG_VERSION")`, so `Cargo.toml` is the single
  version source and the "bump both files" house rule was amended, not broken twice.

Fixed

- The `cli.rs` refactor re-audited green rather than assumed so: exit-code audit `2 · 2 · 3 · 4 · 4 · 0`,
  the CI smoke mirror all `exit=0`, and the v0.3.0 contract (`resumed_from: 2`, `unchanged: 2`,
  `item 25` answered `304 reused true` from stored `raw_json`) unchanged.

## v0.3.0 — sync hardening: reuse, resume, cancel · 2026-11-19 · tag `v0.3.0`

Added

- Conditional GETs everywhere the crawler revisits: each list page keeps its ETag plus
  `has_more`/`count` continuation hints in `meta` (a `304` has no body — the crawl walks
  on using the hints), and the `sync_state` page column is an absolute cursor.
- Stop→resume: a list run that stopped by budget or cancel resumes at the page after the
  one `sync_state` says it last fetched; a finished run re-crawls from page 1, where the
  stored ETags make that a walk of `304`s. Honest reports: `resumed_from`, `unchanged`.
- Live `item` lookups revalidate instead of re-download: stored `detail_etag` goes out as
  `If-None-Match`, a `304` is answered from `raw_json` on disk, a fresh `200` is written
  through the same upsert the sync stage uses. Output gained `"reused"`.
- Cancel plumbing: both crawl loops check a `CancelFlag` before every request; the CLI
  hands out a fresh one (Ctrl-C stays resume-safe), the GUI raises it via AppState at
  v0.4.0. A cancelled run records status `cancelled` and exits 0.
- The fixture server now speaks conditional GETs (stable per-endpoint ETags, real `304`
  no-body), and the CI smoke sequence exercises the whole v0.3.0 contract: budget stop
  → resume → full re-sync (all `304`) → detail crawl → `item` answered from the DB.

## v0.2.0 — data layer + working sync · 2026-11-19 · tag `v0.2.0`

Added

- `src/` headless core, one buildable unit: `errors.rs`, `models.rs`, `wowdb.rs`,
  `db.rs`, `query.rs`, `stats.rs`, `sync.rs`, `cli.rs` plus `main.rs` argv dispatch —
  the six CLI verbs of [ADR 0004](./docs/DECISIONS/0004-headless-cli-mode.md).
- SQLite storage ([docs/DATABASE.md](./docs/DATABASE.md)): wide `item` row plus
  `meta`/`watch`/`sync_state`, list + detail upserts, `--db PATH` overriding `WGT_DB`.
- HTTP client with pacing by construction: 90-request headroom under the 120-request
  burst, 2 req/s schedule, `429` honours `Retry-After` and never aborts.
- Both sync stages verified end-to-end against a fixture server, incl. retry-in-place
  for a `429`'d item and an honest `complete`/`partial` report derived from the pending
  count.
- CI smoke test is now live (was commented in the scaffold tag): fixture server
  `tests/fixture_api.py` + `sync --stage list` + `sync --stage detail` + `stats`.
- `Cargo.toml` version 0.2.0 with the pinned dependency line unchanged. `tauri.conf.json`
  joins the tree at v0.4.0 (GUI shell), so its version bump lands there — see the DEVLOG
  note on the amended house rule.

## v0.1.0 — scaffold · 2026-11-19 · tag `v0.1.0`

Added

- Repository layout: `src/` (Rust), `ui/` (frontend assets), `docs/`, `.github/`.
- Project documentation: [DEVELOPMENT](./docs/DEVELOPMENT.md) (architecture, roadmap, house
  rules), [API](./docs/API.md) (TheWoWDB notes: paging, ETag revalidation, rate limits),
  [DATABASE](./docs/DATABASE.md) (SQLite schema + query recipes), [DECISIONS](./docs/DECISIONS/)
  (ADR-style records).
- `README.md` with the roadmap and the required data attribution, `CHANGELOG.md`,
  `LICENSE` (MIT + data note), `.gitignore` for Rust/Tauri/IDE artefacts.
- GitHub-style repository metadata: issue + PR templates, CI workflow (`build`, `test`,
  `stats` on a seeded fixture DB).
- Default `hello world` binary kept untouched (base crate compiles as-is).
