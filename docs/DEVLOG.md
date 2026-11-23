# Development log

Dated, reverse-chronological notes on what was tried, measured and decided while building
WoW Goldtool. The changelog records *what* shipped; this file records *why*.

---

## 2026-11-23 · v0.16.0 — the SEARCH ROWS open their windows BY NAME: the name cell becomes the opener button, the separate `card` chip label dies, and the look rides the recipe name's ONE rule by JOINED selector

### What forced it

The user's ninth ask: "make it so in the search results I can click the item name just like it is
for the recipes." The Recipes tab already answers by its names — ADR 0025 drew each recipe row's
name cell AS the click target (`<button class="openrecipe" data-spell>`, gold, `font: inherit`, no
box, dotted underline, ONE rule, hover rule measured ABSENT). v0.15.0 (ADR 0029) had given the search
rows an opener too — but measured WHERE the recipe shape is NOT, off the tree at `00f94ba` before one
line moved:

| the click shape lived here | measured | what it meant |
| -------------------------- | -------- | ------------- |
| recipe rows (`#recipe-hits`) | `td.col-recipe` = `<button class="openrecipe" data-spell>` carrying the recipe's OWN name fallback `\|\| spell_id`; ONE CSS rule, hover **ABSENT** | the name IS the button — ADR 0025's "the name cell IS the click target" |
| search rows (`#hits`) | `td.namecell` = plain name text FIRST + `view` anchor + `rowact` watch chip + a separate `button.pipbtn` chip **LABELED `card`**; its paint = `.rowact`'s chip-box (TWO rules incl. its OWN gold hover) | the opener answers BESIDE the name — the row is not clicked BY its name; the ask refuses exactly this |
| the click halves | `#hits` delegated handler: `dataset.id` gate, class split `pipbtn → runCard_from_row` else `toggle_watch`; `runCard(false)` = door `item ID --cached` | the pattern and the ONE door are already there — the era can leave them PUT entirely |
| the rest of the frame | SEVEN nameplates / 8 `data-tab` sections; `hit_row` ONE caller; mini rows `namecell` plain text + chips, NO id button | the strip is PUT; the scope is the Search list alone |

### The decision (ADR 0030 — a list row's NAME is its window opener)

* **The NAME cell becomes the button** (`ui/app.js`): the search row draws
  `<button class="pipbtn" data-id>NAME</button>` FIRST in the cell, carrying the SAME `data-id` the
  watch half carries; the label fallback rides the recipe cell's OWN shape (`name || hit.id` — a
  clickable name is never blank). The word `card` and the trailing chip die with it; the `view` anchor
  and the watch toggle stay PUT where they sit.
* **The click's other half stays PUT word-for-word** (ADR 0014 kept, NOT grown): class, handler split,
  `runCard_from_row`, the window's OWN `card-id` field, the `item ID --cached` door — ZERO line moved.
  The AFFORDANCE moved; the VERB did not.
* **The look is ONE joined rule, never a second copy** (`ui/app.css`): `#recipe-hits tr td
  button.openrecipe` gains `#hits tr td button.pipbtn` in its selector list — the frame already joins
  selectors (its `th.col-job, th.col-recipe, ...` header rule is the precedent) — so the two names are
  ONE click shape (gold, `font: inherit`, dotted underline, NO hover: measured absence + the chip's own
  hover died with it). ZERO new rules, ZERO new colour tokens. The chip-box TWO rules die.

### Measured (the era's OWN witnesses — fresh 9911/9941/9971, ZERO real-host for the audit crawl)

* **Mirror fresh 9911:** **49/49 commands rc `0`, ZERO NONZERO** — the driver's OWN parse: 49 committed
  step-lines, ZERO moved, plant one (this era grew ZERO door argv, ZERO CI step). Stage presses UNCHANGED
  (budget-3 recipes 4/3/5/`rate_limited` 1 · resumed-from-3 5/5/7 complete · re-pull 8/8 pages/**8 SAME
  copy**/0 written); fixture tallies end where the crawl left them — **51 GETs, 17 recipe-file GETs, ZERO
  added** — locality proof; census recipe 12 / item 6 / copies 4 / `vendor_buy` 0 / trend 0.
* **Exit map fresh 9941:** the carried 31 doors answer the **SAME totals `0`×19 · `2`×5 · `4`×7** — ZERO
  door fact moved — and this era grew **ZERO new door shape** (the name click rides the door it already
  rode; the carried eight shapes re-measure `0`×7 + `4`×1: `recipes` 12 993 B 12/12 · `recipes --icons`
  46 793 B = ALL 12 of matching 12 · `--limit 0` 0 rows at 52 B rc `0` · `--limit -1` 12/12 · `--limit 12
  --icons` the SAME 46 793 B · `--offset 10` 2 of 12 at 2 174 B · Enchanting 2 of 2 at 4 634 B · EMPTY-DB
  rc `4`); money over the door's OWN answers **0 pairing defects**, counts 19/19/19 wearing none.
* **Pinned pair fresh 9971 + ONE fresh DB** (pinned worktree @ TAGGED `c065815`, banner `0.14.0` PUT,
  NINE nameplates = the BEFORE witness): **28/28 BYTE-IDENTICAL, ZERO DELTAs** — the nine card doors 2
  124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B, the item doors `item 25/40 --cached`
  **4 341 / 4 076 B**, the search doors `search sword --limit 4` 185 B / `--offset 1` 182 B, unbounded
  `recipes --icons` **46 793 B = ALL 12 of matching 12 on BOTH binaries** — plus **13/13 md5-equal** through
  BOTH binaries over the user's OWN file (LOCAL, ZERO requests): md5s `stats` be44e700 · `item 25 --cached`
  82e349c1 · `search` 5d8e2281 · `watch list` 02b3e228 · `recipes --limit 3` bbc17f2d · Blacksmithing c1fe9d13.
  The user's file census UNCHANGED (`sqlite3 -readonly`, NEVER written, NEVER crawled): schema **8** · 19 258
  · 4 602 copies / **14 656 without** · 1 240/1 240 · `learned_json` 1 240 · watch **134** · 4 231 · 4 344 + 258
  · `vendor_buy` **0** · trend **0** · deleted **0** · `sync_state` 6.
* **Ladder row owed and MEASURED:** the v0.15.0 **fold push's OWN run** — which that fold refused to chase —
  reads **45260568565 SUCCESS**, 5/5 jobs green in **10 min 57 s** (20:33:18Z → 20:44:15Z, run **#51**,
  attempt 1, exactly ONE run id, no CANCELLED sibling, head `00f94ba` = that FOLD commit, whose tree carries
  the tagged `eaeee89`); per-job ends `test` → 20:38:20Z · macos → 20:40:43Z · quality → 20:42:13Z · linux →
  20:43:08Z · **windows → 20:44:14Z — the windows job CLOSED this run** (a scheduling fact, said so — the
  FIRST windows-closed run in the measured chain: v0.14.0's era run closed on quality, its fold run closed on
  macos, v0.15.0's run closed on linux). Each job answers **12 end-actions, 11 success + ONE skipped** — every
  skipped one the Rust-toolchain installer's OWN inner action (`__run_2` windows, `__run_3` the other four),
  never a workflow step; the Ubuntu-libs apt line reads 7 on the `test`/quality/linux job logs, 0 on macos/windows.
  The `test` job pressed exactly **49 `cargo run --` steps** — the era moved ZERO CI line — and from GitHub's OWN
  test-job log the step answers read 12 976 B (`recipes`), **46 776 B (`recipes --icons`, ALL 12 of matching 12)**
  and 2 171 B (`--offset 10`) — the bytes ride the run's seed (my fresh mirror's 46 793 B), the tallies do not.
* **UI launch witnesses** (parse/grep ONLY — the webview NEVER renders in the build room, the name button's
  DOM/IPC cost stays **NOT WITNESSED**): `node --check` PARSE OK; the `>card</button>` label count **ZERO**;
  the name-button draw (class `pipbtn` + `data-id`) EXACTLY 1; the ONE joined selector
  `button.openrecipe, #hits tr td button.pipbtn` = 1; standalone `button.pipbtn` rule blocks **ZERO**;
  `button.pipbtn:hover` **ZERO** (died with the chip); `.pip`/`.pip-closed` still 2; nameplates still **SEVEN**;
  `data-tab` attrs still **15** (7 buttons + 8 sections, `writes` rides three); closers still 2; the `openrecipe`
  draw PUT; ADR 0027's 21 id-scoped `#recipe-card` rules **STILL 21**; Search tab's `--limit 40` PUT; `recipes-pager`
  0. **Builds named ZERO** on every commit (the FS notice rode); `cargo clippy --all-targets` rc `0`, **0 errors /
  the SAME 811 advice**.
* **Measured defects: ZERO** — nothing lands FORWARD from this record.

### Chain

`f78a6b0` feat(ui) (the name cell becomes the opener, the `card` label dies — ONE file ui/app.js, 14
ins / 7 del) → `239c2a4` feat(ui) (the look rides openrecipe's ONE rule by JOINED selector, the chip's
TWO rules die — ONE file ui/app.css, 15 ins / 14 del) → `fecb6c2` chore (banner `0.15.0 → 0.16.0` — a
FEATURE era moves it; house note kept honest: the lock's OWN version line follows the banner in the SAME
commit, settling `0.14.0 → 0.16.0` in one jump — the lock had lagged one banner since v0.15.0's build)
→ this record carries `v0.16.0` (annotated tag).

**FOLDED — measured post-tag on the PUSHED tree at `4f3c217` (all FRESH ports/processes/DBs:
9921/9951/9981; the real host untouched, the user's file read read-only through LOCAL doors, ZERO
requests, ZERO crawl).** The tag's tree was verified through the remote's OWN `^{}` line `d5fab777…`
→ `4f3c217` (tag object `d5fab777`, payload 9 559 B).

* **The era's OWN chain+tag run reads 45265336918 SUCCESS**, 5/5 jobs green in **9 min 50 s** (21:01:18Z
  → 21:11:08Z, run **#52**, attempt 1, exactly ONE run id, no CANCELLED sibling, head `4f3c217` = this record
  = the tag's target); per-job ends quality → 21:03:32Z · macos → 21:07:52Z · `test` → 21:08:28Z · windows →
  21:10:38Z · **linux → 21:11:08Z — the linux job CLOSED it** (the SECOND linux-closed run in the chain, after
  v0.15.0's own; where jobs land is a scheduling fact, said so — and 9 min 50 s is the SHORTEST measured run in
  the chain, the era grew ZERO CI step). Each job answers **12 end-actions, 11 success + ONE skipped** — every
  skipped one the Rust-toolchain installer's OWN inner action (`__run_2` windows, `__run_3` the other four), never
  a workflow step; Ubuntu-libs reads 7 on `test`/quality/linux logs, 0 on macos/windows. The `test` job pressed
  exactly **49 `cargo run --` steps** — the block stayed PUT — and from GitHub's OWN test-job log the step answers
  read `recipes` 12 992 B, **`recipes --icons` 46 792 B (ALL 12 of matching 12)** and `--offset 10` 2 173 B: my
  fresh mirror's 46 793 B on that argv again says the bytes ride the seed, the tallies do not.
* **Post-tag witnesses on the PUSHED tree, ALL SAME against the era's pre-tag measures:** mirror **49/49 rc `0`,
  ZERO NONZERO** (9921 — driver's OWN parse: 49 committed step-lines, ZERO step moved, plant one; presses UNCHANGED
  4/3/5/`rate_limited`1 · 5/5/7 complete · 8/8 pages/**8 SAME copy**/0 written; tallies ending where the crawl left
  them — **51 GETs, 17 recipe-file GETs, ZERO added** — locality proof; census 12/6/4/0/0). Exit map (9951, fresh seed
  + EMPTY DB + DEAD port): the **SAME 31-door totals `0`×19 · `2`×5 · `4`×7**, whole map 39 doors `0`×26 · `2`×5 ·
  `4`×8, the eight shapes AGAIN `0`×7 + `4`×1 (this seed: `recipes` 12 993 B 12/12 · `recipes --icons` 46 793 B =
  ALL 12 of matching 12, 7 reagent icon-byte cells + 6 crafted · `--limit 0` 0 rows at 52 B rc `0` EMPTY not 4 ·
  `--limit -1` 12/12 · `--limit 12 --icons` the SAME 46 793 B · `--offset 10` 2 of 12 at 2 174 B · Enchanting 2 of 2
  at 4 634 B · EMPTY-DB rc `4`), money **0 pairing defects**, counts 19/19/19 wearing none. **The pinned v0.14.0 pair
  AGAIN 28/28 BYTE-IDENTICAL, ZERO DELTAs + 13/13 md5-equal** (9981 vs the worktree @ TAGGED `c065815`: nine card
  doors 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B, item doors 4 341/4 076 B, search doors
  185/182 B — unbounded `recipes --icons` **46 793 B = ALL 12 of matching 12 on BOTH binaries**; user md5s UNCHANGED:
  `stats` be44e700 · `item 25 --cached` 82e349c1 · `search` 5d8e2281 · `watch list` 02b3e228 · `recipes --limit 3`
  bbc17f2d · Blacksmithing c1fe9d13). The pair's OWN surface census quotes the moved name shape both sides: new SEVEN
  nameplates + `pip-closed`/`open_pip` PRESENT + banner `0.16.0` MOVED vs pinned NINE + `pip-closed` ABSENT + banner
  `0.14.0` PUT.
* **Tag census on the remote: 64 lines · 32 tags, ZERO moved** — `v0.16.0` → `4f3c217` (object `d5fab777`), `v0.15.0`
  → `eaeee89`, `v0.14.0` → `c065815` and the older anchors stay where they were. CI on the pushed tree: parse still
  **49 cargo step-lines + plant one**. User's file census UNCHANGED (`sqlite3 -readonly`, NEVER written, NEVER crawled):
  schema **8** · 19 258 · 4 602/14 656 · 1 240/1 240 · 1 240 `learned_json` · 134 · 4 231 · 4 344 + 258 · `vendor_buy`
  **0** · trend **0** · deleted **0** · `sync_state` 6. The pushed tree re-measures the UI witnesses again (same rule:
  DOM/IPC NOT WITNESSED): PARSE OK, `>card</button>` count ZERO, the name draw EXACTLY 1, the ONE joined selector,
  standalone pipbtn blocks + hover ZERO, 7 nameplates + 15 attrs PUT, 21 id rules STILL 21, `--limit 40` PUT.
* The fold's OWN push starts the NEXT run — measured in the NEXT era's record (ADR 0011's ladder), never chased here.

---

## 2026-11-23 · v0.15.0 — the CARD WINDOWS stop being tabs: a row click floats the card OVER the tab it came from, the strip answers SEVEN activities, and the doors answer BYTE-IDENTICAL

### What forced it

The user's eighth ask: "make it so the recipe card is no longer a tab but a window in window
overlay when clicking on a recipe in the recipes tab — the same for the search: clicking an item
should pull up a picture-in-picture look version of the item card instead of the item card being
a whole tab." A card's WHAT was decided at v0.12.0; this ask is its WHERE, and the WHERE was
measured off the tree at `e449b60` before one line moved:

| where the frame lived | measured | what it meant |
| --------------------- | -------- | ------------- |
| the strip | **9 nameplates** (8 `button class="tab"` + the painted `tab-on`) | TWO of the nine ARE card windows — a card competed as an activity beside its own list |
| the sections | **10 carry `data-tab`** (21 attrs total: 9 buttons + 10 sections; `writes` rides two) | ADR 0018's ONE truth: `show_tab`'s loop keeps exactly ONE visible, the rest wear `display: none` |
| the recipe card's reach | `runRecipeCard` starts with `show_tab("recipe-card")` — the frame's ONLY JS visibility call for a card; the item card has ZERO (its nameplate alone made it visible) | turning the card tab ON turns the clicked list's tab OFF — and `display: none` gets no events (the frame's OWN comment above app.js says so): **the row you clicked vanishes beside the window that answers it** |
| the click paths | recipe rows: `openrecipe` name button + ONE delegated handler over `#recipe-hits`; search rows: `rowact` watch toggle + ONE delegated handler over `#hits` | the delegated pattern rides — measured: the search rows carry **ZERO** open-a-card affordance today |
| the window's sizes | **21 id-scoped `#recipe-card` rules** (ADR 0027: `.listicon` 72 / `.noicon` 39 px) | id-scoped, NOT tab-scoped: they keep riding whatever surface the id paints — so an overlay can wear them UNTOUCHED |

ADR 0018's layout made a card an ACTIVITY. The ask names the cure: the window opens **ON** the tab
that is already in front. So the era is a SURFACE fact and NOTHING else — which the pinned pair
below proves byte-wise.

### The decision (ADR 0029 — a card window is an OVERLAY on the tab its click opened on)

* **The two card nameplates die, and the two sections stop naming tabs** (`ui/index.html`): the
  `tab-card` / `tab-recipe-card` buttons die (strip: NINE → **SEVEN** nameplates), `#item-card`
  and `#recipe-card` drop their `data-tab` — ADR 0018's loop NEVER sees a card window again (it
  owns 7 nameplates + 8 sections), and a window's visibility is held by `open_pip`/`close_pip`
  ALONE: the sections are born wearing `pip-closed` (the frame's OWN `display: none` mechanism,
  under a name that is not a tab), `pip` floats them. Each window grows its OWN closer
  (`Close this window`): closing hides the window back where it was born and the tab underneath
  stays exactly where it was — a window never eats its tab. The ids stay PUT, so ADR 0027's 21
  id-scoped rules ride the floated panel byte-identical.
* **The clicks land windows ON the tab they came from, ONE door each, ZERO door line moved**
  (ADR 0014 kept): the recipe half stays EXACTLY PUT (button, handler, door argv) — only
  `show_tab("recipe-card")` becomes `open_pip("recipe-card")`. The search row grows the opener
  it never had (measured ZERO today): a `pipbtn` button carrying the SAME item id as the watch
  toggle, and the ONE delegated handler over `#hits` splits the halves BY CLASS — `pipbtn` fills
  the window's OWN field and lands on `runCard(false)`: `item ID --cached` — the window answers
  OFFLINE, ZERO requests, like its door. No per-row listener anywhere (ADR 0025's pattern).
* **The paint is the frame's OWN silhouette again, FOUR CSS rules, ZERO new colour token**
  (`ui/app.css`): `.pip` floats the SAME `.card` panel — fixed inset, lit-brass trim (`--edge-lit`),
  the gold hairline, a DEEPER shadow than a laid panel (0.55 vs 0.45, 16 px vs 10 px) so it sits
  visibly IN FRONT: picture in picture. `button.pipbtn` rides `.rowact`'s furniture — the SAME
  cell already carries one button. SEVEN rejected alternatives with measures live in ADR 0029
  (restyle-with-nameplates-left, switch-underneath, second renderer, new door/flag, whole-cell
  clicks, DOM re-parent, auto-close).

### Measured (the era's OWN witnesses — fresh ports, ZERO real-host for the audit crawl)

* **Mirror, fresh port 9711** (the committed block re-PARSED then replayed VERBATIM, the driver
  exec-ing the ALREADY-BUILT binary): **49/49 commands rc `0`, ZERO NONZERO** — the parse says
  **49 step-lines committed, ZERO step moved by this era** (a FEATURE era of paint and clicks:
  ZERO door argv grew, so the block stays exactly PUT). Stage presses UNCHANGED: budget-3 recipes
  **4 requests / 3 pages / 5 rows / `rate_limited` 1** · resumed-from-3 **5 / 5 pages / 7 rows /
  complete** · re-pull **8 / 8 pages / 8 answered the SAME copy / 0 written**. The fixture's OWN
  GET tallies end where the crawl left them — **51 GETs, 17 of them recipe-file GETs, ZERO added**
  — which IS the locality proof. Census: recipe **12** / item 6 / copies 4 / `vendor_buy` **0** /
  trend **0**.
* **Exit map, fresh port 9741** (fresh seed DB + fresh EMPTY DB + a DEAD port): the 31 doors the
  v0.13.0/v0.14.0 maps carried answer the **SAME totals `0`×19 · `2`×5 · `4`×7** — ZERO door fact
  moved — and v0.14.0's OWN eight shapes answer AGAIN **rc `0`×7 + rc `4`×1** (`recipes` 12 968 B
  12/12 · `recipes --icons` **46 768 B = ALL 12 of matching 12** · `--limit 0` 0 rows of matching
  12 at 52 B rc `0` · `--limit -1` 12/12 · `--limit 12 --icons` the SAME 46 768 B · `--offset 10`
  2 of 12 · `--profession Enchanting --icons` 2 of 2 at 4 632 B · `--limit 100 --icons` over the
  EMPTY DB rc `4` on the door's OWN no-row words). This era grew **ZERO new door shape** — unlike
  v0.14.0's eight, the window rides the doors it already rode. Money over the door's OWN answers:
  **0 pairing defects**, counts 19/19/19 with **none wearing money** (ADR 0015's count half,
  measured again).
* **The pinned pair, fresh port 9771 + ONE fresh DB:** the PINNED v0.14.0 tree (worktree at the
  TAGGED `c065815`, its OWN build green with ZERO named, banner `0.14.0` PUT, NINE nameplates +
  the card tabs PRESENT = the BEFORE witness) vs the new tree — **28/28 BYTE-IDENTICAL, ZERO
  DELTAs** (the nine card doors at 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 /
  4 033 B, the recipes doors in BOTH shapes with unbounded `recipes --icons` **46 793 B on BOTH
  binaries**) — plus **13/13 md5-equal** through BOTH binaries over the user's OWN file (LOCAL
  doors, ZERO requests). The window moved a SURFACE and the doors answered identical bytes on both
  sides of the pair: the strongest claim this era can draw, and it drew it.
* **The user's file census UNCHANGED** (`sqlite3 -readonly`, NEVER written, NEVER crawled): schema
  **8** · 19 258 items · 4 602 copies / **14 656 without** · 1 240 recipe rows / 1 240 distinct
  spell ids · `learned_json` 1 240 set · **134** watched · `vendor_sell` 4 231 · `price_min_now`
  4 344 + `price_now_missing_at` 258 · pending 0 · `vendor_buy` **0** · `price_trend_normal` **0**
  · tombstoned **0** — ADR 0010's copy rule and ADR 0016's NULL homes stay TRUE under the new skin.
* **The UI launch witnesses** (the webview NEVER renders in the build room — parse/grep ONLY, the
  DOM/IPC cost of the overlay stays **NOT WITNESSED**, same rule the v0.13.0/v0.14.0 records carry):
  `node --check` PARSE OK; **SEVEN nameplates** (6 + the painted `tab-on`); **15 `data-tab` attrs**
  = 7 buttons + 8 sections; **ZERO** `data-tab="card"` / `"recipe-card"`; **ZERO** `show_tab("card")`
  / `show_tab("recipe-card")` call sites left; `open_pip`/`close_pip` 3 definitions+calls each;
  TWO closer buttons; the `pipbtn` opener drawn in the row + its handler class split; FOUR CSS rules
  born; ADR 0027's **21 id-scoped `#recipe-card` rules STILL 21** with `.listicon` 72 / `.noicon`
  39 px in the census; `recipes-pager` stays **0** and the Search tab's `--limit 40` stays **PUT**.
* **Builds:** the era touched ZERO Rust — every commit builds with the named set **ZERO** (the FS
  notice rode, never named), and `cargo clippy --all-targets` measures **0 errors / the SAME 811
  advice** rc `0` on the era tree.
* **CI ladder, owed row MEASURED:** the v0.14.0 **fold push's own run** — which the fold record
  refused to chase and this era owes — reads **45222898599 SUCCESS**, 5/5 jobs green in **17 min
  36 s** (17:00:47Z → 17:18:23Z, run **#49**, attempt 1, exactly ONE run id, no CANCELLED sibling,
  head `e449b60` = the FOLD commit itself, whose tree carries the tagged `c065815`); per-job ends
  quality → 17:02:41Z · linux → 17:09:08Z · windows → 17:09:51Z · `test` → 17:15:53Z · **macos →
  17:18:23Z** — the macos job CLOSED this run (unlike v0.14.0's, which the quality job closed): a
  scheduling fact on GitHub's side, said so rather than smoothed away. Each job answers **12
  end-actions, 11 success + ONE skipped**, and the skipped one is the Rust-toolchain installer's
  OWN inner action (`__dtolnay_rust-toolchain.__run_N` — `__run_2` on windows, `__run_3` on the
  other four), never a workflow step; the Ubuntu-libs step is simply ABSENT on the macos/windows
  jobs (the apt line reads 1 on `test`/linux jobs, 0 on macos/windows). The `test` job pressed
  exactly **49 `cargo run --` steps** — this era moved ZERO CI line, the committed block stays
  PUT — and its unbounded `recipes --icons` step answered **ALL 12 of matching 12** at 46 792 B
  over the fixture's own seed with 19 reagent cells (7 icon-byte cells, 6 crafted icon-byte cells,
  **19 counts carried with 0 money beside them**). This era's OWN chain+tag run is measured in the
  NEXT record (ADR 0011's ladder), never chased here.
* **Measured defects this era found: ZERO** — nothing lands FORWARD from this record.

### Chain

`b5bcb25` feat(ui) (the two nameplates die, the windows ride the clicks, closers born — TWO files,
89 ins / 15 del) → `6b4f50c` feat(ui) (the overlay's paint: FOUR CSS rules, ZERO new colour token,
32 ins) → `ff84417` chore (banner `0.14.0 → 0.15.0` — a FEATURE era moves it) → this record carries
`v0.15.0` (annotated tag).

**FOLDED — measured post-tag on the PUSHED tree at `eaeee89`, all FRESH ports/processes/DBs (the
real host untouched, the user's file read read-only through LOCAL doors, ZERO requests, ZERO crawl).**
The tag's tree was verified through the remote's OWN `^{}` resolution line `5f344fce…` → `eaeee89`.

* **The era's OWN chain+tag run reads 45253921925 SUCCESS**, 5/5 jobs green in **10 min 16 s**
  (19:55:19Z → 20:05:35Z, run **#50**, attempt 1, exactly ONE run id, no CANCELLED sibling, head
  `eaeee89` = this record commit = the tag's target); per-job ends quality → 19:59:47Z · macos →
  20:01:09Z · `test` → 20:02:41Z · windows → 20:04:33Z · **linux → 20:05:35Z — the linux job CLOSED
  this run** (where the last job landed is a scheduling fact on GitHub's side, said so, not smoothed;
  it is WHY this run reads 10 min 16 s against v0.14.0's 18 min 15 s). Each job answers **12 end-actions,
  11 success + ONE skipped**, and the skipped one is every time the Rust-toolchain installer's OWN
  inner action (`__dtolnay_rust-toolchain.__run_2` on windows, `__run_3` on the other four jobs), never
  a workflow step; the Ubuntu-libs apt step is PRESENT on the `test`/linux/quality job logs (the package
  line reads 7 times there) and simply ABSENT on the windows/macos logs (0). The `test` job pressed
  exactly **49 `cargo run --` steps** — this era moved ZERO CI line, the committed block stayed PUT —
  and from GitHub's OWN test-job log, step-answer payload lengths: the tab's OWN argv `recipes --icons`
  answered **`"matching": 12` at 46 792 B**, its siblings `recipes` 12 992 B and `--offset 10` 2 173 B
  (my fresh mirror measured 46 793 B on the same argv: **the bytes ride the seed, the tallies do not**).
* **Fresh post-tag witnesses on the PUSHED tree:** mirror **49/49 commands rc `0`, ZERO NONZERO** on
  fresh **9811** — the driver's OWN parse says **49 committed step-lines, ZERO step moved**, plant line
  one — stage presses UNCHANGED (budget-3 recipes **4 requests / 3 pages / 5 rows / `rate_limited` 1** ·
  resumed-from-3 **5 / 5 pages / 7 rows / complete** · re-pull **8 / 8 pages / 8 answered the SAME copy /
  0 written**) · fixture's OWN GET tallies end where the crawl left them — **51 GETs, 17 recipe-file GETs,
  ZERO added** — the locality proof · census recipe 12 / item 6 / copies 4 / `vendor_buy` 0 / trend 0.
  Exit map on fresh **9841** (fresh seed + fresh EMPTY DB + a DEAD port): the 31 carried doors answer the
  **SAME totals `0`×19 · `2`×5 · `4`×7** and this era's OWN eight shapes answer AGAIN **rc `0`×7 + rc `4`×1**
  (`recipes` 12 993 B 12/12 · `recipes --icons` **46 793 B = ALL 12 of matching 12** with 7 reagent icon-byte
  cells + 6 crafted · `--limit 0` **0 rows** at 52 B rc `0` — EMPTY, not 4 · `--limit -1` 12/12 · `--limit 12
  --icons` the SAME 46 793 B as unbounded · `--offset 10` 2 of 12 at 2 174 B · `--profession Enchanting --icons`
  2 of 2 at 4 634 B · `--limit 100 --icons` over the EMPTY DB **rc `4`** on the door's OWN no-row words); this
  era grew ZERO new door shape; money over the door's OWN answers **0 pairing defects**, counts 19/19/19 with
  none wearing money. **The pinned v0.14.0 pair answered AGAIN, fresh 9871 + ONE fresh DB:** pinned worktree
  @ the TAGGED `c065815` (banner `0.14.0` PUT, NINE nameplates + the card tabs PRESENT = the BEFORE witness)
  vs the new tree → **28/28 BYTE-IDENTICAL, ZERO DELTAs + 13/13 md5-equal** over the user's OWN file; the
  pair's OWN surface census quotes BOTH trees: new SEVEN nameplates / `data-tab="recipe-card"` ABSENT /
  `pip-closed` + `open_pip(` PRESENT / `show_tab("recipe-card")` ABSENT / banner `0.15.0` MOVED vs pinned
  NINE / PRESENT / ABSENT / ABSENT / PRESENT, banner `0.14.0` PUT.
* **Pushed-tree re-measures** (same rule — the webview NEVER renders in the build room, the overlay's DOM/IPC
  cost stays **NOT WITNESSED**): fresh `cargo build` on the pushed tree **49.07 s**, named set **ZERO** (the FS
  notice rode); `node --check` PARSE OK; SEVEN nameplates; **15 `data-tab` attrs** (analytics/live/overview/recipes/
  search/sync ×2 + `writes` ×3); ZERO `data-tab="card"`/`"recipe-card"`; ZERO card `show_tab` sites; TWO `Close
  this window` closers; the `pipbtn` opener drawn in JS + the handler's class split riding; FOUR `.pip`-family
  rules (`display: none` hider, the float, the opener button + its hover); ADR 0027's `#recipe-card` id rules
  **STILL 21** with `.listicon` 72 / `.noicon` 39 px; `recipes-pager` 0 and the Search tab's `--limit 40` PUT.
* **Tag census on the remote: 62 lines · 31 tags, ZERO moved** — `v0.15.0` → `eaeee89` (tag object `5f344fce`,
  payload 8 430 B), `v0.14.0` → `c065815` and the older anchors stay where they were. CI on the pushed tree:
  parse still **49 cargo step-lines + plant one**. User's file census UNCHANGED (`sqlite3 -readonly`, NEVER
  written, NEVER crawled): schema **8** · 19 258 items · **4 602** copies (`raw_json`/`detail_etag`/`tooltip_html`
  all agree) / **14 656** without · 1 240 recipe rows / 1 240 distinct spell ids · `learned_json` 1 240 set ·
  watch **134** · `vendor_sell` 4 231 · `price_min_now` 4 344 + `price_now_missing_at` 258 · `vendor_buy` **0** ·
  trend **0** · deleted **0** · `sync_state` 6.
* The fold's OWN push starts the NEXT run — measured in the NEXT era's record (ADR 0011's ladder), never chased here.

---

---

---

## 2026-11-23 · v0.14.0 — the RECIPES TAB shows the WHOLE set: its pager dies, the door never had a cap, and ZERO door line moved

### What forced it

The user's seventh ask: "recipes tab, get rid of the limit for how many hits are to be shown and
just show all at all times." Read honestly, a limit is asked FOR by a surface, and the question was
where the Recipes tab's limit lived. Measured off the tree at `v0.13.0` before one line moved, the
frame answered in FOUR places and the cap was in the TAB, not the door:

| the bound lived here | its own words | measured |
| -------------------- | ------------- | -------- |
| the tab's control | `<select class="field" id="recipes-pager">` — options `25` / `50` / `100` "a page" | the tab could never show more than **100 of the set's 1 240** rows; it had no way to ask for more |
| the tab's argv | `var argv = ["recipes", "--limit", limit, "--icons"]` | `recipes --limit 100 --icons` answers **100 rows of matching 1 240** at 1 363 515 B — the door HID 1 140 rows |
| the door | `recipes_rows` through ADR 0022's `paging_clause` (no `--limit` = **the rest**, riding SQLite's own negative-limit token) | `recipes --icons` with NO bound answers **ALL 1 240 rows of matching 1 240** at 16 696 238 B (3 399 938 B without `--icons`); over the fixture's 12 rows it answers **12 of matching 12** at 46 793 B; and `--limit 1240 --icons` answers the **SAME 16 696 238 B** — a bound at or above the matching count changes NOTHING |
| the Search tab's control | `argv.push("--limit", "40")` (hardcoded, no select) | a SECOND bound in a different shape, on a tab the ask never named — kept PUT, and the record names which bound is which |

ADR 0022 had already put the rule on the door ("no `--limit` means the rest") for the paging doors;
the Recipes tab simply never rode it. So this ask needed ZERO door line — it needed the tab to stop
holding rows back, and to stop SAYING it holds rows back.

### The decision (ADR 0028 — a bound is a SURFACE fact: kill the tab's, keep the door's)

* **The tab's control dies** (`ui/index.html`): the `recipes-pager` select has no question left to
  answer; the era's comment in the SAME file names the measure that killed it.
* **The tab asks unbounded** (`ui/app.js`): `var argv = ["recipes", "--icons"]`, `--profession` only
  when a job is picked — the tab's argv is then the door's OWN argv, so a `cargo run -- recipes
  --icons` line shows the SAME rows (ADR 0021/0014's one-door rule, kept).
* **The note moves the same breath** (ADR 0019's rule): the frame stops saying "showing 100 of
  1240" and says **ALL the rows this filter answers are shown**, naming why a bound would be a lie
  (a bound at or above the count answers the SAME bytes, so it only HID rows) and naming the
  Watch button's reach honestly, since the shown set is now the whole set.
* **The door keeps `--limit`.** A shell that prints 16.7 MB for the whole set must be able to bound
  that ON REQUEST, and removing the flag would move ADR 0022's paging pair, the four bounded CI
  recipe steps, the map doors measured with bounds and the Search tab's own bound. The bound is the
  door's discipline; the tab simply stopped asking for it.
* **SEVEN rejected alternatives with measures** (ADR 0028 keeps the set, every one measured before
  one line moved): growing the pager ("2 500 a page", or an "all" option) — measured that a bound AT
  or above matching answers the SAME bytes, so a bigger cap cannot do what it claims while still
  offering a choice the ask refused; `--limit 0` as "all" — measured rc `0` / 52 B / **0 rows of
  matching 12**, zero means NONE, not ALL (ADR 0004's EMPTY half, the door's OWN meaning, not to be
  flipped); `--limit -1` — measured that it WORKS (SQLite's negative-limit token, 12 of 12 / 1 240 of
  1 240) but hides the fact behind an SQLite accident, so the fact "this tab does not bound" becomes
  invisible at every reader; removing the CLI's `--limit` flag — it would move ADR 0022's paging pair,
  FOUR bounded CI steps, the map doors measured with bounds and the Search tab's own bound; a client-side
  virtual scroll — a page size is a cap again, which is what the ask refused; re-aiming `btn-recipes-watch`
  at a bounded set — ADR 0021's rule speaks of "the shown rows", so capping would give "shown" a second
  meaning; and a second unbounded door (`recipes-all`) — refused by ADR 0014, a click and a list row would
  then be able to disagree about the set.

### Measured (the era's own, all fresh; the user's file read read-only, ZERO crawl)

* **The diff:** `ui/index.html` + `ui/app.js`, **19 insertions / 9 deletions**, and the CI step —
  ZERO Rust line, ZERO `SQL_` const, ZERO wire struct, ZERO schema line (the user's own file answers
  **`8`**), ZERO exit code, ZERO money formatter, ZERO request rule, ZERO fixture line.
* **CI grows ONE step, 48 → 49 step-lines** (plant one stays, 14 `- name:` steps unchanged, raw
  `cargo run` mentions 48 → 49): the block gains the tab's OWN argv `cargo run -- recipes --icons`,
  so CI can SEE the unbounded door — over the fixture's 12 rows it answers **12 of matching 12** at
  46 793 B, 19 reagent cells (7 carrying the DB's OWN icon bytes, 8 money words, **19 counts carried
  with 0 money beside a count** — ADR 0015's count half), 6 crafted cells carrying icon bytes, 2 rows
  stating NO crafted item (the Enchanting shape). Measured on a fresh mirror **BEFORE the block
  landed** (the driver parsed the committed block and added these very steps): **49/49 commands rc
  `0`, ZERO NONZERO** (ports 9521 / 9531); measured AGAIN on the LANDED 49-step block (port 9541):
  **49/49 rc 0, ZERO NONZERO**, stage presses UNCHANGED (budget-3 recipes 4 requests / 3 pages / 5
  rows / `rate_limited` 1 · resumed-from-3 5 / 5 pages / 7 rows / complete · re-pull 8 requests / 8
  pages / **8 answered the SAME copy** / 0 written), the fixture's OWN GET tallies ending where the
  crawl left them — **51 GETs, 17 of them recipe-file GETs, ZERO added**, which IS the locality
  proof (ADR 0012: the door is LOCAL) — and the census at recipe **12** / item 6 / copies 4 /
  `vendor_buy` **0** / `price_trend_normal` **0**.
* **The door's answers move ZERO bytes, measured as the strong pair:** the PINNED v0.13.0 tree
  (worktree at the TAGGED `b927109`, whose OWN build is green with ZERO named in 1 m 22 s, banner
  `0.13.0` PUT, the `recipes-pager` select PRESENT) vs the new tree over one fresh fixture process
  (port 9561) + one fresh DB → **28/28 BYTE-IDENTICAL, ZERO DELTAs**: the NINE card doors (2 119 /
  7 325 / 6 670 / 4 105 / 643 / 1 207 / 1 154 / 6 737 / 4 032 B), the recipes doors in BOTH shapes —
  unbounded `recipes --icons` **46 768 B on BOTH binaries**, `--limit 0` 52 B, `--limit -1` = the
  unbounded 12 968 B, `--limit 100 --icons` 46 768 B — and the pre-existing set (`status` 1 239 ·
  `stats` 1 761 · `item 25/40 --cached` 4 336 / 4 072 · `search` 185 / 182 · `watch list` 8 · the
  `recipes` doors 12 968 / 3 375 / 2 170 / 2 181 / 4 632 B); plus **13/13 md5-equal** through BOTH
  binaries over the user's own file (cards 25 213 / 7 004 / 9 430 / 3 596 / 3 453 B, rc `4` on
  `8800111` at 113 B, plus `status` 1 098 · `stats` 4 554 · `item 25 --cached` 2 949 · `search` 979
  · `watch list` 707 · the `recipes` doors 4 293 / 21 090 B). The era's HEADLINE door over the user's
  whole file is byte-identical across binaries too: **16 696 238 B, md5 `4f1982d6`, 1 240 rows of
  matching 1 240** — 3 778 reagent cells (3 514 with icon bytes, 3 642 with money words, 3 778 counts
  carried with NO money beside them), 897 crafted cells with icon bytes / 873 money words / 884 live
  words, 133 rows with NO crafted item (Enchanting) and 10 crafted ids with no row; asked WITHOUT
  `--icons` the same set costs 3 399 938 B.
* **The exit map, FRESH port 9551** (fresh seed DB + fresh EMPTY DB + a DEAD port): the 31 doors
  v0.13.0's map carried answer the **SAME totals `0`×19 · `2`×5 · `4`×7** — this era moved ZERO door
  fact — and the era's OWN eight shapes answer **rc `0`×7 + rc `4`×1**: `recipes` 12/12 at 12 993 B,
  `recipes --icons` 12/12 at 46 793 B, `--limit 0` 0 rows of matching 12 rc `0` (52 B — EMPTY, never
  a 4), `--limit -1` 12/12, `--limit 12 --icons` the SAME 46 793 B as unbounded, `--offset 10` 2 of
  12, `--profession Enchanting --icons` 2 of 2 at 4 634 B, and `recipes --limit 100 --icons` over the
  EMPTY DB rc `4` on the door's OWN no-row words. Money over the door's OWN answers: **0 pairing
  defects**, counts carried 19 / 19 / 19 with none of them wearing money.
* **The tab's OWN cost, stated where it can only be stated:** the whole set costs 16 696 238 B of
  door answer and 3 514 + 897 = **4 411 icon data URIs** to paint (vs the ≤100-row cap's 1 363 515
  B). The webview never renders in the build room (the launch-audit rule), so **the DOM cost and the
  Tauri IPC cost are NOT witnessed** — the era states the BYTES it measured and refuses to claim a
  smooth 1 240-row paint. That is this era's honest limit.
* **The Watch button's reach, measured where writing was possible:** ADR 0021's rule UNCHANGED
  ("the cooked items of the SHOWN rows"), its target set NOT capped — over the whole set its ONE
  batched `watch add` argv carries **1 095 ids of the 1 240 rows** (133 Enchanting crafts with NO
  crafted item and 10 crafted ids with no row skipped honestly, the row already says so). Measured on
  a **SCRATCH COPY** of the user's file (the user's file was NEVER written, NEVER crawled): rc `0`,
  6 036 B, and the Details door then fetches exactly those records under ADR 0017's pending rule
  (one ask per copied record EVER). The cost is a user-chosen click, not a click this era grew in
  silence.
* **CI ladder, owed rows MEASURED:** the previous era's rows land here as measured. Its chain+tag
  run read **45199646115 SUCCESS**, 5/5 jobs green in **16 min 4 s** (run #46, attempt 1, head
  `b927109` = that tag's target, quality **7/7** with the Lint step **completed/success** — the local
  measure of the SAME command reads 0 errors / the SAME 811 advice lines — `test` **8/8** with the CLI
  smoke step GREEN, linux **8/8**, macos **7/8**, windows **7/8**, one `skipped` Ubuntu-libs step on
  each non-linux target as designed; the run reads longer than v0.12.x's 8–9 min because the macos job
  opened 5 min 1 s after the run opened — a scheduling fact, said so). And the v0.13.0 **fold push's
  own run**, which this era owes and had never been waited for, reads **45204000066 SUCCESS**, 5/5
  jobs green in **9 min 56 s** (run #47, attempt 1, exactly ONE run id, no CANCELLED sibling, head
  `6fd2e3e` = the FOLD commit itself; per-job ends: quality → 15:22:46Z, windows → 15:26:14Z, macos
  → 15:28:30Z, linux → 15:29:44Z, `test` → 15:29:46Z; each job answers **12 end-actions, 11 success +
  ONE skipped**, and the skipped one is the Rust-toolchain installer's OWN inner action
  (`__dtolnay_rust-toolchain.__run_N`), never a workflow step — the Ubuntu-libs step is simply ABSENT
  on the macos/windows jobs; the `test` job pressed exactly **48 `cargo run --` steps**, because the
  fold's tree still carries the 48-step block: this era's `ci` commit landed AFTER the fold). The run
  that this era's chain+tag push starts is measured in the NEXT record (ADR 0011's ladder), never
  chased here.

### Chain

`d491877` feat(ui) (the pager dies, the tab asks unbounded, the note moves — TWO UI files, ZERO
Rust line) · `ebc690b` ci (48 → 49 step-lines: the tab's OWN argv, ZERO requests) · `25846dc` chore
(a FEATURE era MOVES the banner `0.13.0 → 0.14.0`, `Cargo.toml` + `Cargo.lock` together — the v0.11.0-
through-v0.13.0 shape) · this record + annotated tag `v0.14.0`. ZERO fixture commit (ZERO behavior
delta at any door — the byte-identical pair above IS the witness), ZERO Rust commit (ZERO Rust line),
ZERO chore-skip (a FEATURE era moves the banner).

### Post-tag (the FOLD — the PUSHED tree, all FRESH ports/processes/DBs, measured AGAIN rather than cited)

The tag's target (`c065815`, read through the remote's OWN `^{}` line `868eff3…` → `c065815`) answers
the way the record says, measured a SECOND time on the tree that actually landed:

* **The era's OWN chain+tag run reads 45217976658 SUCCESS**, 5/5 jobs green in **18 min 15 s**
  (16:34:12Z → 16:52:27Z, run #48, attempt 1, exactly ONE run id, no CANCELLED sibling, head
  `c065815` = the TAGGED tree, read off the remote's own `^{}` lines). Per-job ends: macos → 16:40:48Z
  · linux → 16:40:56Z · `test` → 16:42:20Z · windows → 16:42:27Z · **quality → 16:52:27Z** — the Lint
  job closed the run this time (in the v0.13.0 era's run it opened first), so the run reads LONGER than
  that era's 16 min 4 s because of WHERE the quality job landed: a scheduling fact, said so, not a tree
  fact. Each job answers **12 end-actions, 11 success + ONE skipped**, and the skipped one is the Rust-
  toolchain installer's OWN inner action (`__dtolnay_rust-toolchain.__run_N`), never a workflow step —
  the Ubuntu-libs step is simply ABSENT on the macos/windows jobs. The `test` job pressed exactly **49
  `cargo run --` steps**: this era's own block, the new step INCLUDED, all rc 0 (GitHub's own verdict
  says so, and my fresh mirror below says it again). The era's NEW step, the tab's OWN argv, answers on
  GitHub's build **12 rows of matching 12** with 19 reagent cells (7 carrying the DB's OWN icon bytes,
  8 money words, **19 counts carried with 0 money beside a count** — ADR 0015's count half), 6 crafted
  cells carrying icon bytes and 2 rows stating NO crafted item — 46 792 B there, 46 793 B on my fresh
  mirrors: the bytes ride the fixture process's own seed, the row tally and the rc totals do not. The
  Lint step is **completed/success** on CI's own build, answering 0 errors / **858** advice lines —
  the SAME 858 the v0.13.0 fold's run measured, which is exactly the claim of an era that touched ZERO
  Rust (the local `cargo clippy --all-targets` number stays 811: a different invocation, kept apart, as
  ADR 0011 requires).
* **The mirror replay, fresh 9611** (the driver re-PARSED the committed block — 49 `cargo run` step-lines
  + plant one, ZERO step moved since the era's own ci commit): **49/49 commands rc `0`, ZERO NONZERO**,
  the stage presses UNCHANGED (budget-3 recipes 4 requests / 3 pages / 5 rows / `rate_limited` 1 ·
  resumed-from-3 5 requests / 5 pages / 7 rows / complete · a re-pull 8 requests / 8 pages / **8 answered
  the SAME copy** / 0 written), the fixture's OWN GET tallies ending where the crawl left them — **51
  GETs, 17 of them recipe-file GETs, ZERO added** — which IS ADR 0012's locality proof, census recipe
  **12** / item 6 / copies 4 / `vendor_buy` **0** / `price_trend_normal` **0**.
* **The exit map, fresh 9631** (fresh seed DB + fresh EMPTY DB + a DEAD port): the 31 doors v0.13.0's map
  carried answer the **SAME totals `0`×19 · `2`×5 · `4`×7** — this era moved ZERO door fact — and this era's
  OWN eight shapes answer **rc `0`×7 + rc `4`×1**: `recipes` 12/12 at 12 993 B · `recipes --icons` 12/12 at
  46 793 B · `--limit 0` answering **0 rows of matching 12** at 52 B with rc `0` (EMPTY, never a 4 — ADR
  0004's half) · `--limit -1` 12/12 at the SAME 12 993 B · `--limit 12 --icons` the **SAME 46 793 B** as
  unbounded · `--offset 10` 2 of 12 at 2 174 B · `--profession Enchanting --icons` 2 of 2 at 4 634 B ·
  `--limit 100 --icons` over the EMPTY DB rc `4` carrying the door's OWN "the recipe table holds no row"
  words. Whole map 39 doors; money over the door's OWN answers **0 pairing defects**, counts carried 19 /
  19 / 19 (Enchanting 3) with NONE wearing money.
* **The PINNED pair, fresh 9671** — the v0.13.0 tree PINNED at the TAGGED `b927109` (a worktree whose OWN
  build is green with ZERO named in 1 m 22 s, banner `0.13.0` PUT, the `recipes-pager` select PRESENT: the
  BEFORE witness) vs the pushed tree over ONE fresh fixture process and ONE fresh DB: **28/28 doors
  BYTE-IDENTICAL, ZERO DELTAs** — the nine card doors at 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1
  155 / 6 740 / 4 033 B, the recipes doors in BOTH shapes (unbounded `recipes --icons` **46 793 B on BOTH
  binaries**, `--limit 0` 52 B, `--limit -1` = the unbounded 12 993 B, `--limit 100 --icons` 46 793 B) and
  the pre-existing set (`status` 1 242 · `stats` 1 773 · `item 25/40 --cached` 4 341 / 4 076 · `search`
  185 / 182 · `watch list` 8 · the `recipes` doors 12 993 / 3 382 / 2 174 / 2 185 / 4 634 B) — plus **13/13
  md5-equal** through BOTH binaries over the user's OWN file (cards 25 213 / 7 004 / 9 430 / 3 596 / 3 453
  B, rc `4` on `8800111` at 113 B, `status` 1 098 · `stats` 4 554 · `item 25 --cached` 2 949 · `search` 979
  · `watch list` 707 · the `recipes` doors 4 293 and 21 090 B). And this era's HEADLINE door over the user's
  WHOLE file is byte-identical across the two binaries: **16 696 238 B, md5 `4f1982d6`, 1 240 rows of
  matching 1 240** — 3 778 reagent cells (3 514 carrying icon bytes, 3 642 carrying money words, **3 778
  counts carried with NO money beside them**), 897 crafted cells carrying icon bytes / 873 money words /
  884 live words, 133 rows stating NO crafted item (the Enchanting shape) and 10 crafted ids with no row;
  the same set asked WITHOUT `--icons` costs 3 399 938 B.
* **The tab's shape on the PUSHED tree, quoted not remembered** (the launch-audit rule: the webview never
  renders in the build room, so these are parse/grep witnesses, never a rendered claim): `node --check`
  parses `ui/app.js` rc `0`, `recipes-pager` answers **ZERO** in BOTH `ui/index.html` and `ui/app.js`, the
  tab's argv reads `var argv = ["recipes", "--icons"];` (line 271), the job filter's select stays ONE
  (`recipes-profession`), the Search tab's hardcoded `argv.push("--limit", "40")` stays PUT (line 192),
  the workflow answers **49** `cargo run --` mentions, and `Cargo.toml`'s banner reads `version = "0.14.0"`
  while the pinned tree's still reads `0.13.0`.
* **The Watch button's reach, fresh and HONEST** (ADR 0021's rule unchanged, its target set NOT capped): over
  the whole set its ONE batched `watch add` argv carries **1 095 ids of the 1 240 rows** (133 Enchanting
  crafts with NO crafted item + 10 crafted ids with no row skipped honestly — the row already says so), and
  on a **SCRATCH COPY** of the user's file the door answers rc `0` / 6 036 B while `watch list` then answers
  **1 129 ids** (the file's 134 already watched + 995 new). The user's own file was NEVER written and NEVER
  crawled and still reads, through `sqlite3 -readonly`: schema **8** · 19 258 items · 4 602 tooltip copies ·
  **14 656 without** · 1 240 recipe rows / 1 240 distinct `spell_id`s · `learned_json` 1 240 set · watch **134**
  · `vendor_sell` 4 231 · `price_min_now` 4 344 + `price_now_missing_at` 258 · pending 0 · `vendor_buy` **0** ·
  `price_trend_normal` **0** · tombstoned **0** — so ADR 0010's copy rule and ADR 0016's NULL homes stay TRUE,
  and the DOM / Tauri IPC cost stays **NOT WITNESSED** (the era refuses to claim a smooth 1 240-row paint).
* **Tags stay put, read through the remote's OWN `^{}` lines** (30 tags on the remote): `v0.1.0` → `8685dea`
  · `v0.10.0` → `e4d3447` · `v0.10.1` → `09c8d77` · `v0.11.0` → `59b0185` · `v0.11.1` → `3f20b51` · `v0.12.0`
  → `ace683a` · `v0.12.1` → `d2c26de` · `v0.13.0` → `b927109` · **`v0.14.0` → `c065815`** — ZERO tag moved.
  CI stays **49 step-lines + plant one**, and the leg after the tag is docs-only, so it cannot move the build.
  The fold's OWN push starts the NEXT run — measured in the NEXT ladder row, never chased here (ADR 0011's
  ladder).

---

## 2026-11-23 · v0.13.0 — the CARD WINDOW gets bigger: its type ~1.45×, its icons ×3 (24 → 72 px), and the frame's sizes stay PUT

### What forced it

The user's sixth ask: "in the recipe card, when I pull up a single recipe, everything needs to
be larger, the font and the icon need a lot larger too." Read honestly, this is a SURFACE fact,
not a door fact: a card is ONE entry opened on its OWN tab and read once from a distance, while
the Recipes LIST, the search hits and the mini analytics rows are dense tables read by scanning.
Before any line moved, the frame's OWN numbers were measured off the tree at `v0.12.1` (`ui/app.css`
quoted, not remembered): the card was drawn with LIST sizes — its OWN type at 10.5 / 11 / 12 /
12.5 / 13 / 13.5 px, its icons through `.listicon` at 24 px, the SAME class the Recipes LIST and
the hits table ride (`table.mini .listicon` overrides that very class to 20 px for the mini rows
— the frame already scopes a shared class to one surface), its `no bytes` stand-in `.noicon` at
13 px, and its panel furniture at the list's grades (`h2` 15 · `.note` 12.5 · `.field`/`button.act`
13 px, `.rcardbox` gap 8 / padding 12+14, `.rcell` gap 3 / padding 7+9, `.rcopy` gap 2 / padding
5+8, border-left 2 px). The one size in the frame that belongs to a WINDOW rather than to a list
is the Item card's own `.iconimg` at 56 px — and a card icon had to pass it, because the ask said
"a lot larger".

### The decision (ADR 0027 — grow the WINDOW, scope the SHARED sizes to it)

* **The card's OWN classes move in place**: `rcardhead` 13 → 19 · `rcardname` 13.5 → 20 ·
  `rcardlearn`/`rcardmats` 12 → 17 · `rcellname` 12.5 → 18 · `rcopy` 12 → 17 · `rcopyhead` 10.5 →
  15 · `learn-word` 10.5 → 15 · `learn-none` 11 → 16 px — a step of ≈ ×1.45 with no half-pixel
  residue (the frame's half-px values disappear rather than multiply).
* **The icons take THREE times the box**: `#recipe-card .listicon { width: 72px; height: 72px }`
  (24 → 72 px, larger than ANY icon the frame draws today, the Item card's 56 px included), and
  the `no bytes` stand-in rides the SAME ×3 at 13 → **39 px** — not an invented number: the
  frame's own 13/24 ratio times 72 = 39.0, so the box keeps its SHAPE instead of lying about the
  space an icon would take.
* **The panel furniture follows the type**: `#recipe-card` padding 16+18 → 22+24, `h2` 15 → 22,
  `.note` 12.5 → 18 (and the card's error note 13 → 18), `.fields` gap 10 → 14, `.field`/`button.act`
  13 → 19 px, and the container gaps with them (`.rcardbox` gap 8 → 14 / padding 12+14 → 18+20,
  `.rcell` gap 3 → 6 / padding 7+9 → 12+14, `.rcopy` gap 2 → 4 / padding 5+8 → 9+12, border-left
  2 → 3 px).
* **The shared classes move ONLY inside the panel** — 21 id-scoped rules (`#recipe-card .chip` 12
  → 17 · `.jobchip` 11.5 → 17 · `.skillchip` 11 → 16 · `.tierchip` 10.5 → 15 · `.mat` gap/padding ·
  `.matname`/`.mat-none` → 17 · `.mat-not-in-db`/`.cooked-none`/`.cooked-not-in-db` 10.5 → 15 ·
  `.cooked` 12 → 17 · `.live` 11 → 16 · `.live-missing`/`.live-none` 10.5 → 15 · `.listicon` 72 ·
  `.noicon` 39 px). Scoping is the FRAME's OWN mechanism (`table.mini .listicon` already does it),
  NOT a new one — so app.js draws the SAME classes it always drew and ADR 0014's one-code-path rule
  survives: a card's material cell IS the code that draws a list row's.
* **FOUR rejected alternatives, each with its measure** (ADR 0027 keeps them all): growing
  `.listicon` itself would triple every Recipes LIST and hits row — the very doors whose paging
  ADR 0022 measured at their row heights — and break the frame's `table.mini` fact; a NEW class
  (`.cardicon` drawn by app.js) is a second icon renderer that could say something different about
  ONE id's icon, which the v0.12.0 block in this file already forbids ("listicon/noicon: no second
  copy of a money or icon cell"); `transform: scale()` scales painted output, not layout — the gaps
  and paddings stay at their px values while the pixelated glyph overflows them; a root/`rem` step
  would move the 12+ `font-size` rules the OTHER tabs ride. Inline `style="…"` in the JS was
  rejected too: a paint fact that needs ZERO JS lines.
* **NOTHING else moves**: the list/hits/mini/item-card sizes stay PUT and stay quoted (`.listicon`
  24 px, `table.mini .listicon` 20 px, `.iconimg` 56 px, `.note` 12.5 px outside the card, `h2`
  15 px), the quality palette stays the frame's (`.chip.c-<quality>` colours untouched), and the
  money contract keeps ADR 0015's rule on the bigger words.

### Measured (the era's own, all fresh)

* **The diff:** `ui/app.css` ONLY — **49 insertions / 10 deletions**, 21 id-scoped rules + the
  card's OWN classes moved in place; ZERO `ui/app.js` line, ZERO `ui/index.html` line (the section
  already carries `id="recipe-card"`, measured — the tab mechanism owns it), ZERO Rust line, ZERO
  `SQL_` const, ZERO schema line (schema stays **8**), ZERO exit-code line, ZERO money formatter,
  ZERO request rule, ZERO fixture line, ZERO CI step-line (**48 step-lines stay**, plant one). The
  FEATURE era's chore moves the Cargo banner **`0.12.0 → 0.13.0`** (`Cargo.toml` + `Cargo.lock`
  together, the v0.11.0/v0.12.0 shape — contrast the v0.12.1 PATCH era that left it PUT).
* **The witnesses (parse/grep, the launch-audit rule — the webview never renders in the build room):**
  `cargo build` rc 0 with **ZERO named** warnings (11.49 s after the banner moved — the era touched
  ZERO Rust, so a green build here is the claim that nothing ELSE moved); `cargo clippy --all-targets`
  answers **0 errors / the SAME 811 advice lines**, rc 0 (unchanged, measured); `node --check` parses
  `ui/app.js` rc 0 (it is the SAME file, unchanged); the CSS braces balance **142/142**; the card's
  OWN class census answers the SAME **TWELVE** names app.js draws (`openrecipe rcardbox rcardhead
  rcardname rcardlearn rcardmats rcell rcellname rcopy rcopyhead learn-word learn-none`) — ZERO new
  class drawn; 21 `#recipe-card` rules present; the PUT sizes still quoted in the same file.
* **The door's bytes do not move AT ALL** — the era is a paint era, so the honest claim is the
  strong one: the PINNED v0.12.1 tree (worktree at the TAGGED `d2c26de`, whose OWN build is green
  with ZERO named in 1 m 22 s, banner `0.12.0` PUT) vs the new tree, ONE fresh fixture process
  (port 9451) + ONE fresh DB → **22/22 BYTE-IDENTICAL, ZERO DELTAs**: the NINE card doors (2 124
  / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B) AND the thirteen pre-existing
  (`status` 1 242 · `stats` 1 773 · `item 25/40 --cached` 4 341/4 076 · `search` 185/182 ·
  `watch list` 8 · the four `recipes` doors 12 993 / 2 185 / 4 634 / 2 174 B); and **13/13 md5-equal**
  through BOTH binaries over the user's OWN file (25 213 / 7 004 / 9 430 / 3 596 / 3 453 B cards,
  rc `4` on `8800111` at 113 B) — LOCAL doors, ZERO requests, read-only.
* **The mirror replay and the exit map, UNCHANGED:** the driver re-PARSED the committed block
  (**48 `cargo run` step-lines**, plant one, ZERO step moved by this era) and replayed it on THREE
  fresh ports (9391 / 9421 / 9461) → **48/48 rc 0, ZERO nonzero**, with the same stage presses
  (budget-3 recipes 4 requests / 3 pages / 5 rows / `rate_limited` 1 · resumed-from-3 5 requests /
  5 pages / 7 rows / complete · re-pull 8 requests / 8 pages / **8 answered the SAME copy** / 0
  written), the four card steps answering **7 333 / 6 673 / 4 107 / 643 B**, the fixture's OWN GET
  tallies ending where the crawl left them (**51 GETs, 17 of them recipe-file GETs** — ZERO added,
  still the locality proof) and the census at recipe **12** / item 6 / copies 4 / `vendor_buy`
  **0** / `price_trend_normal` **0**. The exit map on fresh 9407 answers **31 doors at rc `0`×19 ·
  `2`×5 · `4`×7 — the SAME totals as v0.12.1's map** (nine card doors rc 0, FOUR rc-`4` doors in
  the SAME words, rc-`2` usage at the same words 48 B / 40 B, `recipe 132 --fixture <DEAD>` rc 0
  because the door sends ZERO asks and rc `3` is UNREACHABLE from it, `recipe 132 extra` rc 0 — the
  door reads `words[0]`). Money over the card doors' OWN answers: **0 pairing defects**, counts
  carried 2 / 2 / 2 / 2 / 2 with NO money beside them. TWO driver-side slips are recorded rather
  than hidden: the block parser counts the fixture-start/sleep/env lines as "other" (so THIS
  driver's total is 48 commands, while the earlier eras' drivers replayed fixture-start + sleep +
  plant and reported 51 — the SAME block, both measured), and the fixture server logs requests on
  STDERR via `BaseHTTPRequestHandler`, so a driver that sends stderr to /dev/null measures ZERO
  GETs — fixed by capturing stderr, and the 51/17 above is the FIXED driver's measure.
* **CI ladder, owed row MEASURED:** the v0.12.1 fold push's own run **45102837594 = SUCCESS**,
  5/5 jobs green in **9 min 31 s** (02:54:26Z → 03:03:57Z, run #45, attempt 1, exactly ONE run id,
  no CANCELLED sibling, head `290da81` = the fold record) — quality **7/7** with the Lint step
  completed/success (0 errors / 811 advice on GitHub's own build), `test` **8/8** with the CLI
  smoke step GREEN (the 48-step block, the four card steps included), linux 8/8, macos 7/8, windows
  7/8 (their Ubuntu-libs step `skipped`, as designed). The era's OWN chain+tag run is measured in
  the NEXT record (ADR 0011's ladder), never chased here.
* **Limits kept:** schema `8`; tags stay put (`v0.10.0` → `e4d3447` · `v0.10.1` → `09c8d77` ·
  `v0.11.0` → `59b0185` · `v0.11.1` → `3f20b51` · `v0.12.0` → `ace683a` · `v0.12.1` → `d2c26de`),
  `v0.13.0` rides THIS record; the era's diff is TWO code-touching files (app.css + the banner pair)
  and ZERO behaviour delta, which the byte-identity above measures rather than asserts; the user's
  file was never written, never crawled, read read-only.

### Chain

`d2f532a` feat(ui) (the card window's sizes: ONE CSS file, 21 scoped rules, ZERO JS line) ·
`1f4b08f` chore (the banner rides 0.13.0 — a FEATURE era moves it, toml + lock together) ·
this record + annotated tag `v0.13.0`. ZERO fixture commit (ZERO behavior delta — and the byte-identity
pair above IS the witness), ZERO ui JS commit (app.js draws the SAME classes), ZERO ci commit
(the smoke block is the same 48 step-lines, unchanged).

### Post-tag verification — MEASURED on FRESH ports/DBs: the PUSHED tree's mirror 48/48 GREEN, the map at the SAME 31-door totals, the pinned v0.12.1 pair BYTE-IDENTICAL **AGAIN** (22/22 + 13/13 md5), and the era's own chain+tag run measured **SUCCESS 5/5**, the Lint step GREEN

All FRESH ports/processes/DBs; the real host saw NOTHING (every crawl door in the mirror carried
`--fixture`; the card door is LOCAL and ZERO-request); the user's OWN file was read through `sqlite3
-readonly` and through LOCAL doors only, ZERO requests, NO crawl, NO write. The era's own chain+tag
push is what these probes read — the tree at `b927109` = the tag `v0.13.0`'s target, verified through
the remote's OWN `^{}` resolution (`b09d646` → `b927109`).

| probe | the PUSHED tree's answer |
| ----- | ---------------------- |
| mirror replay, POST-tag — the driver re-PARSES the committed block (48 `cargo run` step-lines + plant one; the era moved NO step) and replays it VERBATIM on a FRESH port + a FRESH DB dir, exec-ing the ALREADY-BUILT binary (the driver-side rule: no compile in flight during an audit, and CI runs on GitHub's host, never this one) | **48/48 commands rc `0`, ZERO NONZERO** (fresh port **9471**, fresh DB dir, the tree at `b927109` = the era's own pushed head) — the SAME 48 the pre-tag mirror measured on 9391 / 9421 / 9461, measured AGAIN rather than cited. Stage presses UNCHANGED: budget-3 recipes **4 requests / 3 pages / 5 rows / `rate_limited` 1** · resumed-from-3 **5 / 5 pages / 7 rows / complete** · re-pull **8 requests / 8 pages / 8 answered the SAME copy / 0 written** (the reuse discipline still holds for the second host). The FOUR card steps answer **7 333 / 6 673 / 4 107 / 643 B**, all rc `0` — bytes ride the RUN's own seed, the SHAPE does not. The fixture's OWN GET tallies END where the crawl left them — **51 GETs, 17 of them recipe-file GETs** — the four card steps added **ZERO**, which IS the locality proof (and the fixture's access log is captured from STDERR, `BaseHTTPRequestHandler`'s own door, with `-u`; a driver that throws stderr away measures ZERO GETs, which is a driver fact, not a door fact). Census: recipe **12** / item 6 / copies 4 / `vendor_buy` **0** / `price_trend_normal` **0** |
| exit map, POST-tag — FRESH port **9481**, FRESH seed DB + a FRESH EMPTY DB + a DEAD port (9399) | **31 doors at rc `0`×19 · `2`×5 · `4`×7 — the SAME totals as v0.12.1's map and as this era's pre-tag map at 9407** (a paint era moved ZERO door fact; the totals staying is the claim, and they stayed). Nine card doors rc `0` (2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B — this process's own seed). SEVEN rc-`4` doors, all naming what ADR 0021 says WOULD write the row: `999999` bare + `--icons`, `recipe 132` bare + `--icons` over the EMPTY DB, `recipes` over the EMPTY DB, `recipe 132 --fixture <DEAD>` over the EMPTY DB, `item 999999 --cached`. FIVE rc-`2` doors at the SAME words: `recipe` (48 B "recipe wants a spell id, for example: recipe 132"), `recipe abc` + `recipe --icons abc` (40 B "…got abc"), `recipe 132 --fixtures` ("unknown flag" — the typo), `recipe-suggest 132` ("unknown verb" — the neighbour did NOT swallow the new verb). The trio's honest half again: over the SEED DB `recipe 132 --fixture http://127.0.0.1:9481` answers rc `0` at 2 124 B and over the DEAD port rc `3` is UNREACHABLE from this door — it sends ZERO asks, and the map shows NO rc `3` anywhere |
| the pinned pair, POST-tag — the PINNED v0.12.1 binary (worktree at the TAGGED `d2c26de`, its OWN build green ZERO named in 1 m 22 s, banner `0.12.0` PUT) vs the PUSHED tree's binary, over ONE fresh fixture process + ONE fresh DB, and over the user's OWN file | **22/22 BYTE-IDENTICAL, ZERO DELTAs** (fixture doors, the NINE card doors included: 2 116 / 7 322 / 6 668 / 4 104 / 643 / 1 206 / 1 153 / 6 735 / 4 031 B, plus `status` 1 236 · `stats` 1 749 · `item 25/40 --cached` 4 334 / 4 070 · `search` 185 / 182 · `watch list` 8 · the four `recipes` doors 12 952 / 2 178 / 4 631 / 2 168 B) — the lengths differ from the 9451 window because a fixture process is ONE observation window (prices seed per run), the IDENTITY is measured WITHIN this one, and the rc totals do not move between windows. And **13/13 md5-equal** through BOTH binaries over the user's OWN file (cards 25 213 / 7 004 / 9 430 / 3 596 / 3 453 B with md5s `f46fec2d` / `cf60227e` / `4f350ecd` / `00604e5f` / `865aa824`, rc `4` on `8800111` at 113 B, plus `status` 1 098 · `stats` 4 554 · `item 25 --cached` 2 949 · `search sword --limit 2 --offset 1` 979 · `watch list` 707 · `recipes --limit 3` 4 293 · `recipes --profession Blacksmithing --icons --limit 3` 21 090 B). Money over the card doors' OWN answers: **0 pairing defects**, reagent COUNTS **2 / 2 / 2 / 2 / 2** carried with NO money beside them (ADR 0015's count half — the bigger words did not turn a COUNT into MONEY) |
| the era's own chain+tag run, GitHub's OWN words — the FIRST run whose head is the tree at `b927109` | **45199646115 SUCCESS**, 5 of 5 jobs green in **16 min 4 s** (created / run_started 14:56:38Z → updated 15:12:42Z, run **#46**, attempt 1, exactly ONE run id, no CANCELLED sibling, head `b927109` = the TAGGED tree): quality **7/7** with "Lint (informational — clippy's own advice; our named set died at v0.11.1, see docs/DEVELOPMENT.md)" **completed/success** — GREEN, and the local measure of the SAME command on the same tree says 0 errors / the SAME 811 advice — `test` **8/8** with the CLI smoke step GREEN (the 48-step block, the four card steps included, on GitHub's own build), linux **8/8**, macos **7/8**, windows **7/8** (one `skipped` Ubuntu-libs step on each non-linux target, as designed: only the linux target links gtk). The run reads LONGER than the v0.12.x runs (9 min 31 s / 8 min 13 s) because the macos job opened at 15:01:42Z, 5 min 1 s after the run, while the other four opened at 14:56:41Z — a GitHub-side scheduling fact, not a tree fact |
| limits kept | schema stays **`8`** — measured on the user's own file through `sqlite3 -readonly`: `meta` answers `schema_version|8`, and its tallies stay INTACT (19 258 items · 4 602 tooltip copies · **14 656 without** · 1 240 recipe rows AND **1 240 DISTINCT `spell_id`s** · `learned_json` 1 240 set / 0 NULL · `vendor_sell` twin pair `price_min_now` **4 344** + `price_now_missing_at` **258** = the 4 602 each asked ONCE, pending **0** · `vendor_buy` **0** · `price_trend_normal` **0** · tombstoned **0** — ADR 0016's NULL-vs-0 grounds and ADR 0010's copy rule both still true, and the card's live chips stay a TRUTH). Tags stay put (`v0.10.0` → `e4d3447` · `v0.10.1` → `09c8d77` · `v0.11.0` → `59b0185` · `v0.11.1` → `3f20b51` · `v0.12.0` → `ace683a` · `v0.12.1` → `d2c26de` · **`v0.13.0` → `b927109`**, read off the remote's `^{}` lines, SEVEN tags, ZERO moved). CI stays **48 step-lines + plant one** (ZERO step moved by this era), ZERO mutes in the tree, the era's code-touching diff stays `ui/app.css` (49/10) + the banner pair (2/1 each); the leg AFTER the tag is docs-only, so it CANNOT move the build. The fold's OWN push starts the NEXT run — measured in the NEXT record, never chased here (ADR 0011's ladder) |

---

## 2026-11-23 · v0.12.1 — patch: the ONE clippy ERROR the era's own CI run rode dies BY NAME, the `-D warnings` gate stays OFF, and NOTHING moved

### What forced it

The v0.12.0 fold's OWN CI ladder row was a FAILURE, said so: the chain+tag run **45095439290**
(head = the TAGGED tree `ace683a`) measured **4/5 jobs green in 8 min 9 s**, and the failing step was
"Lint (informational — clippy's own advice; our named set died at v0.11.1, see docs/DEVELOPMENT.md)".
Reading that NAME as a plan was the mistake this era refuses: an "informational" step rode the run's
verdict because an **ERROR** rode it, and an ERROR is not advice. The era's own tree was the witness:
`cargo clippy --all-targets` answers **1 ERROR / 811 advice** — `error: this loop never actually loops`
at `src/db.rs:3029:5`, `clippy::never_loop`, `#[deny(…)]` ON by default. Nothing about the DOOR was
wrong (its rc `0` witnesses, its LOCAL rule, its money rule all hold; the smoke block was GREEN on
GitHub's own builds), so the fix is a shape fact, not a behavior question.

### The two gates, and where the ERROR sits (measured 2026-11-23, both sides of the pair)

| gate | pinned v0.12.0 tree (`ace683a`) | the fixed tree |
| ---- | ------------------------------ | -------------- |
| `cargo build` (rustc) | ZERO named warnings (rc 0, 1 m 23 s — the pinned worktree's OWN build is the BEFORE witness) | ZERO named warnings (rc 0, 3 m 11 s with the cache cleared) |
| the lint step, `cargo clippy --all-targets` | **3 `^error` lines** = 1 real ERROR (`this loop never actually loops` at `src/db.rs:3029:5`) + its two "could not compile … due to 1 previous error; 811 warnings emitted" tails; rc `101` | **ZERO `^error` lines**, rc `0`, still **811 advice lines** (the same census: `question_mark`, `needless_return`, `len_zero`, `clone_on_copy`, `single_match`…) and ZERO `never actually loops` hits |
| `cargo fmt --check` | answers a diff (the house style, ADR 0011's measured reason no gate exists) | unchanged: ZERO lines of this era touch formatting |

The ERROR's own claim is a FACT this tree already states elsewhere: the doc comment above `recipe_card`
says `spell_id` is the table's PRIMARY KEY (docs/DATABASE.md's schema block says the same, and the map's
own answers prove the read: `recipe 132` answers ONE card, `recipe 999999` answers rc `4`, never two
rows). So a `for` around a body that always returns could never take a second turn. [ADR 0026](./DECISIONS/0026-a-clippy-error-is-not-advice.md) keeps the decision and its FOUR rejected alternatives (the mute — ADR 0024's rule kept for a NEW gate: an `allow` keeps the lie and hides the ERROR from the next era; a gate — measured 811 advice lines, so `-D warnings` would fail on clippy's OWN style, and it STAYS OFF; letting the run stay red behind an "informational" NAME; a Vec-collect rewrite for a read that states ONE row, noisier than the shape the other single-row readers already ride).

### Measured (the era's own, all fresh)

* **The fix, ONE file:** `git diff 89cab34..56e0356` = `src/db.rs`, **7 insertions / 2 deletions** — the
  loop header becomes `if let Some(one_row) = mapped.next()` (clippy's OWN suggestion) plus a comment
  naming the PRIMARY-KEY fact, and `let mapped` gains its `mut`. ZERO `SQL_` line, ZERO wire struct,
  ZERO `vendor_buy` / `price_trend_normal` producer, ZERO money line, ZERO schema line (schema stays `8`),
  ZERO fixture / ui / ci / Cargo lines (`Cargo.toml` + `Cargo.lock` an EMPTY diff — a PATCH era leaves the
  banner PUT at `0.12.0`, contrast the v0.12.0 FEATURE era that MOVED it).
* **The door answers UNCHANGED, byte-for-byte:** the pinned pair — the worktree at the tagged tree
  `ace683a` (its OWN build green in 1 m 23 s, banner `0.12.0`) vs the fixed binary, ONE fresh fixture
  process (9307) + ONE fresh DB (the crawl ran ONCE, with the new binary, and every compared door is
  LOCAL and read-only) — **22/22 BYTE-IDENTICAL, ZERO deltas**: the NINE card doors (132 bare ·
  132 `--icons` · 202 · 637 · 638 · 683 · 1723 · 4977 · 334 · `999999` bare and `--icons`) AND the
  thirteen pre-existing doors. This is a STRONGER pair than earlier patch eras drew: v0.11.1 compared
  15 pre-existing doors because it added none; here the NEW doors exist in BOTH binaries, so the fix's
  "behavior moved ZERO" claim rides the doors it touched, not only their neighbours.
* **The user's OWN file through BOTH binaries** (read-only, LOCAL, ZERO requests): **13/13 md5-equal** —
  the five card doors 25 213 / 7 004 / 9 430 / 3 596 / 3 453 B (27835, 2171, 2177, and the two REAL
  null-crafted Enchanting entries 7418 `automatic` and 7422 `trainer`), rc `4` on `8800111`, plus
  `status` 1 098 B · `stats` 4 554 B · `item 25 --cached` 2 949 B · `search sword --limit 2 --offset 1`
  979 B · `watch list` 707 B · `recipes --limit 3` 4 293 B · `recipes --profession Blacksmithing --icons --limit 3`
  21 090 B. The file's tallies stay INTACT after the fix (19 258 items / 4 602 copies / 14 656 without /
  1 240 recipe rows / `learned_json` 1 240 set · 0 NULL / `price_min_now` 4 344 + `price_now_missing_at`
  258 = the copies each asked ONCE / pending 0 / `vendor_buy` 0 / `price_trend_normal` 0 / tombstoned 0).
* **The mirror replay and the exit map, UNCHANGED:** the driver re-PARSED the committed block (48 step-lines
  + plant one, the era moved NO step) and replayed it on fresh 9289 → **51/51 commands rc `0`, ZERO nonzero**,
  same stage tallies (budget-3 recipes 4 requests / 3 pages / 5 rows / `rate_limited 1` · resumed-from-3
  5/5/7 · re-pull 8 requests / **8 answered the same copy** / 0 written) and the four card steps answering
  the SAME 7 333 / 6 673 / 4 107 / 643 B as the v0.12.0 fold's run; the fixture's OWN GET tallies END where
  the crawl left them (51 GETs, 17 recipe-file GETs) — the card steps added ZERO, still the locality proof.
  The exit map on fresh 9277 answers **31 doors at rc `0`×19 · `2`×5 · `4`×7 — the SAME totals as
  v0.12.0's map**, the nine card doors at rc `0`, rc `4` on `999999` and on `recipe 132` over an EMPTY DB,
  rc `2` usage at the same words (48 B / 40 B), `recipe 132 --fixture <DEAD>` rc `0` (ZERO asks, rc 3
  unreachable from this door) — byte sizes ride the RUN's own seed pair (2 124 / 7 330 / 6 673 / 4 107 /
  643 / 1 209 / 1 155 / 6 740 / 4 033 B here against the prior run's 2 119 / 7 325 / 6 670 / 4 105 / …),
  which is why the totals and the rc, not the bytes, are this map's claim.
* **CI ladder, owed rows:** the v0.12.0 fold push's own run **45098365666** (head `89cab34`, the
  SAME code tree as the tagged one plus the fold record) measured **FAILURE**, 4 of 5 jobs green in
  **11 min 20 s** (created/run_started 02:07:22Z → updated 02:18:42Z, run #43, attempt 1, exactly
  ONE run id, no CANCELLED sibling): linux build 8/8 · macos 7/8 · windows 7/8 (their Ubuntu-dev-libraries
  step `skipped`, as designed — only the linux target links gtk) · `test` 8/8 with the **CLI smoke step
  GREEN** (the 48-step block, the four card steps included, passes on GitHub's own build) — while the
  quality job answered 6/7 with "Lint (informational — clippy's own advice; our named set died at
  v0.11.1, see docs/DEVELOPMENT.md)" = **failure**, the SAME step its v0.12.0 sibling rode. The tree
  implied that verdict BEFORE it was read (NOTHING about the code moved between the tag and that push,
  so the ERROR still rode the step), and the measure agreed; saying so first, then measuring, is ADR
  0011's ladder form — the run was never waited for blindly. This era's OWN chain+tag run — head = the
  TAGGED tree `v0.12.1`, the FIRST tree since v0.12.0 whose lint step measures 0 errors / 811 advice —
  is measured FORWARD in the next record, never chased here.

### Chain

`56e0356` fix(db) (the ONE row taken, not walked: the ONLY era line) · this record + annotated tag
`v0.12.1`. Patch era: NO chore (the banner stays PUT at `0.12.0` — `Cargo.toml` and `Cargo.lock` answer an
EMPTY diff), ZERO fixture commit (no behavior delta to smoke — and the fixture's door answers ARE the
byte-equality witness above), ZERO ui commit (nothing moved), ZERO ci commit (the step's NAME is true
AGAIN at 0 errors / 811 advice, measured not assumed).

### Post-tag verification — MEASURED on FRESH ports/DBs: mirror 51/51 GREEN, the map at the SAME 31-door totals, the pinned v0.12.0 pair 16/16 md5-equal, the real file's cards HONEST — and the era's own chain+tag run measured **SUCCESS 5/5**, the Lint step GREEN for the FIRST time since v0.11.1

All FRESH ports/processes/DBs; the real host saw NOTHING (the mirror's every crawl door carried
`--fixture`; the card door is LOCAL and ZERO-request); the user's OWN file was read through
`sqlite3 -readonly` and through LOCAL doors only, ZERO requests, NO crawl, NO write.

| probe | answer |
| ----- | ------ |
| mirror replay — the driver PARSES the committed block (48 step-lines + plant 1 + fixture start 1 + sleep 1, so "48" is proven, not retyped) and replays it on a FRESH port + FRESH DB dir, exec-ing the ALREADY-BUILT binary (the era's driver-side rule, no compile in flight) | **51/51 commands rc `0`, ZERO NONZERO** (fresh port 9341, the tree at `d2c26de` = the era's own pushed head) — the era moved NO step, so this is the SAME 51 as v0.12.0's fold, measured again rather than cited. Stage presses UNCHANGED: budget-3 recipes 4 requests / 3 pages / 5 rows / `rate_limited` 1 · resumed-from-3 5 requests / 5 pages / 7 rows / complete · re-pull 8 requests / 8 pages / **8 answered the SAME copy** / 0 written. The FOUR card steps answer **7 333 / 6 673 / 4 107 / 643 B**, all rc `0`; the fixture's OWN GET tallies END where the crawl left them — **51 GETs, 17 of them recipe-file GETs** — the four card steps added **ZERO**, which IS the locality proof. Census: recipe **12** / item 6 / copies 4 / `vendor_buy` **0** / `price_trend_normal` **0** |
| exit map — FRESH port 9373, FRESH seed DB + a FRESH EMPTY DB + a DEAD port | **31 doors at rc `0`×19 · `2`×5 · `4`×7 — the SAME totals v0.12.0's map measured** (this era moved ZERO semantics, so the totals staying is the claim; a PATCH era must not move them). Nine card doors rc `0` (2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B — bytes ride the RUN's own seed pair, rc totals do not), rc `4` on `999999` bare AND `--icons` AND on `recipe 132` over the EMPTY DB bare AND `--icons` — FOUR rc-4 doors, all the SAME words naming the stage ADR 0021 says would write the row; rc `2` usage 48 B "recipe wants a spell id, for example: recipe 132" + 40 B "recipe wants a numeric spell id, got abc" + `recipe --icons abc` also rc `2`, `--fixtures` (the typo) answers 2 "unknown flag", `recipe-suggest 132` answers rc `2` "unknown verb" (the neighbour did NOT swallow the new verb); the trio's honest half again: **`recipe 132 --fixture <DEAD port>` answers rc `0`** (the door sends ZERO asks → rc `3` is UNREACHABLE from this door, and the map shows NO `3`), `recipe 132 extra` answers rc `0` (the door reads `words[0]`, the parser stayed put) |
| the pinned pair POST-tag — the PINNED v0.12.0 binary (worktree `ace683a`, its OWN build green, banner `0.12.0`) vs the pushed tree's binary, over the user's OWN file (read-only, LOCAL doors, ZERO requests) | **16/16 md5-equal, ZERO DELTAs** — the card doors 25 213 (27835 `--icons`) / 7 004 (2171) / 9 430 (2177) / 3 596 (7418) / 3 453 (7422) B plus `recipe 27835` BARE at 3 705 B, and rc `4` on THREE absent ids in the SAME words: `638` (a FIXTURE id absent from the real set, 109 B), `8800111` (113 B), `999999` (112 B); plus `status` 1 098 / `stats` 4 554 / `item 25 --cached` 2 949 / `search sword --limit 2 --offset 1` 979 / `watch list` 707 / `recipes --limit 3` 4 293 / `recipes --profession Blacksmithing --icons --limit 3` 21 090 B. The file's own tallies INTACT: 19 258 visible items / 4 602 tooltip copies / **14 656 without** / **1 240 recipe rows AND 1 240 DISTINCT `spell_id`s** — the PRIMARY-KEY fact the fix rides, measured ON the user's file, not cited — `learned_json` 1 240 set · 0 NULL, `vendor_sell` 4 231, `price_min_now` **4 344** + `price_now_missing_at` **258** = the 4 602 copies each asked ONCE (ADR 0017's once-EVER rule), pending **0**, `vendor_buy` **0**, `price_trend_normal` **0**, tombstoned **0**. Money over these very answers: **0 pairing defects**; the reagent COUNTS 6 / 2 / 2 / 1 / 1 carried with NO money beside them (ADR 0015's count half). The cards' live chips: live PAIRS where a row has a copy (27835's scroll + result + 5 of 6 materials; 2177's result + 2 materials; 7418/7422's material), `not refreshed yet` on the rows ADR 0010's copy rule refuses (2171's scroll 2404 and its result among them) — a TRUTH, not a defect |
| the era's own chain+tag run, GitHub's OWN words — the FIRST tree since v0.12.0 whose lint step answers 0 errors | **45101284057 SUCCESS**, 5 of 5 jobs green in **8 min 13 s** (created/run_started 02:37:33Z → updated 02:45:46Z, run #44, attempt 1, exactly ONE run id, no CANCELLED sibling, head `d2c26de` = the TAGGED tree): quality **7/7** — the step "Lint (informational — clippy's own advice; our named set died at v0.11.1, see docs/DEVELOPMENT.md)" is **completed/success**, GREEN for the FIRST time since v0.11.1, so the NAME is true OF this tree again (measured locally on the SAME tree: 0 errors / the SAME 811 advice) · `test` **8/8** with the CLI smoke step GREEN (the 48-step block, the four card steps included, on GitHub's own build of the tagged tree) · linux 8/8 · macos 7/8 · windows 7/8 (the Ubuntu-dev-libraries step `skipped` on both non-linux targets, as designed: only the linux target links gtk). The verdict moved **FAILURE → SUCCESS with ZERO workflow lines moved**: the step is the same text, the gate is still off, and the ONLY thing that changed is the tree's own shape — which is what ADR 0026 claims, measured |
| limits kept | schema stays `8` (the scratch DBs, the fixture's and the user's file all answer 8), tags stay put (`v0.10.0` → `e4d3447` · `v0.10.1` → `09c8d77` · `v0.11.0` → `59b0185` · `v0.11.1` → `3f20b51` · `v0.12.0` → `ace683a` · **`v0.12.1` → `d2c26de`**, the docs-record commit the tag rides, verified through the remote's OWN `^{}` resolution), the era's code diff stays ONE file / 7 ins / 2 del (`src/db.rs`) and the leg AFTER the tag is docs-only (7 files / 253 ins / 1 del, `src/` + `tests/` + `.github/` + `Cargo.toml` + `Cargo.lock` an EMPTY diff — the docs record CANNOT move the build), ZERO mutes in the tree (`never_loop` answers ONE hit, and it is the comment that names the ERROR BY NAME), CI stays 48 step-lines + plant one; the fold's OWN push starts the NEXT run, measured in the NEXT record, never chased here (ADR 0011's ladder) |

---

## 2026-11-22 · v0.12.0 — a recipe row OPENS ITS OWN CARD: one click reads ONE local door, and the card shows the entry as the GAME shows it

### What forced it

The user's ask: "when looking at a recipe, clicking it in the list should open a detailed window
showing it like it would appear in the game." Read as decoration, that ask is a UI job; read as it
must be for this tool, it is a JOIN question — and a window that guesses which item a recipe belongs
to is a window that lies. So the join was MEASURED before one line of code was written, on the user's
own file, read-only, ZERO requests. What the game shows for a recipe is a SCROLL you can read, an
ITEM it crafts, the MATERIALS it asks for and the SKILL band it sits in — four things, and only TWO
sources exist here: the fetched classic recipe set (the second source, ADR 0021) and the item rows
this DB owns. A card is therefore only honest if every cell is a BY-VALUE link one of those two
sources ACTUALLY states, and every gap names which source is silent.

### The join censuses, measured on the user's own file (read-only, ZERO requests, 2026-11-22)

Base: **1 240** fetched `recipe` entries · **19 258** visible item rows · **4 602** tooltip copies ·
**14 656** rows with none. What can be linked, and what cannot — the rejected two are rejected on
MEASUREMENT, not on looks:

* `spell_id` used as an item id (the tempting "the spell IS the scroll row"): **651** of 1 240 spell
  ids land on a visible item row, and only **20** of those rows carry `item_type Recipe` — the other
  **631** land on rows that are not scroll items at all. **REJECTED**: spell ids and item ids are two
  namespaces, and 631 wrong cells would ride a card that looks right.
* the name link `"Recipe: " + crafted_item_name`: only **123** of 1 240 entries reach an item row
  (case-sensitive and case-folded answer the SAME number), while **716** items carry `item_type Recipe`
  and only **250** items anywhere are named `Recipe:*` — and, the sharper witness measured the same
  pass: of those 123 reaches, **ZERO** land on the item id this very entry states as
  `crafted_item_id`; **all 123** attach to a DIFFERENT item. **REJECTED**: not merely under-reaching,
  it silently MIS-attaches (the DB's OWN names differ from the set's — the detail stage overwrites,
  deprecated prefixes, non-localisation).
* `learned_json` read BY VALUE — the accepted scroll link: **804** entries state
  `{"item_id":…,"item_name":…,"type":"schematic_item"}` and name an id, **413** state the `trainer`
  word, **23** the `automatic` word (sums to 1 240: the fetched shape leaves NO entry silent about
  learning). Of the **804** stated ids **747** have a visible item row, **477** of those carry the
  scroll's OWN tooltip copy. The set itself states the scroll: no invention anywhere on this path.
* `crafted_item_id` (ADR 0021's join, REUSED untouched): **1 105** distinct ids → **1 095** have a
  row, **897** carry a copy, **198** have a row but no copy, **10** have none; **133** entries name
  NO crafted item at all (all `Enchanting`: an enchant is a slot effect, not an item in a bag).
* reagent ids (ADR 0021's join, REUSED untouched): **3 778** mentions over **416** distinct ids →
  **415** have rows (**393** with a copy, **22** row-no-copy), **1** id has no row.
* the DB's own scroll words are a THIRD census: **716** `item_type Recipe` rows, ALL 716 with a copy,
  but only **122** of them are named `Recipe:*` — which is exactly why the name join above cannot find
  them (631 of the 716 rows the set would need are not named that way at all).

### The decision (ADR 0025 — five parts, one per truth)

1 — **ONE door, ONE verb, ONE code path (ADR 0014).** The click does not grow a second answer shape:
`cargo run -- recipe SPELL_ID [--icons]` is the door and the webview calls exactly that argv through
`cli_ok`. A card is auditable from a shell, and the click's answer IS the CLI's answer. Rejected: a
flag on the list door (`recipes --id N`) — one argv answering two shapes is the drift ADR 0014
refuses — and a webview-only shape no audit can witness.
2 — **The door is LOCAL, ZERO requests (ADR 0009/0012's local-door rule).** It reads the rows the
recipe stage wrote and joins BY VALUE through the item rows this DB owns; nothing upstream is asked
for a card. `--icons` rides because a copied icon is about 3.1 K chars once encoded (ADR 0014's
measured reason the bytes stay opt-in); the door is bounded by ONE spell id, but the rule stays the
same rather than silently shipping a fat answer.
3 — **Cells REUSE the item card's and the list's readers, so a card cannot drift from a row.**
`live_name_price_of` answers name + stored min + the live twin + BOTH stamps, `money_of` writes the
words, `icon_b64_of`/`icon_at_of`/`icon_missing_at_of` and `tooltip_text_of`/`tooltip_at_of`/
`tooltip_missing_at_of` answer the copy and icon states, and the MATERIALS ride `join_recipe_reagent`
UNTouched — a card's material cell is the same code that draws a list row's material cell. ONLY the
FOUR columns the list door never reads (quality, item level, required level, vendor sell) come from a
SELECT of this door's own (`SQL_CARD_CELL`), and it carries the SAME `deleted_at IS NULL` guard.
The tombstone rule is MADE TRUE in the door, not assumed: measured, the six stamp SELECTs
(`SQL_TOOLTIP_TEXT_OF` / `SQL_TOOLTIP_AT_OF` / `SQL_TOOLTIP_MISSING_AT_OF` / `SQL_ICON_BLOB_OF` /
`SQL_ICON_AT_OF` / `SQL_ICON_MISSING_AT_OF`) carry NO `deleted_at` clause, so the door asks them ONLY
for a row it saw alive — a row the user buried keeps its stamps buried.
4 — **Money keeps ADR 0015's contract on the NEW surface, and ADR 0016's NULLs stay unwritten.**
Measured over the fixture's own answers: **0 pairing defects** (vendor sell, stored min and the live
twin each carry their money word beside their copper) and **13 counts** carried with **NO** money
beside them. `vendor_buy` / `price_trend_normal` read **0 / 0** in the fixture census: this door READS
the live twin, it never grows a producer for the other two homes.
5 — **`learned_json` gets its FIRST reader, and ADR 0021's debt closes where it was written.** The
column is read BY KEY, never by field name: its `type` key is a reserved word, so the door uses
`serde_json.Value` accessors (`word_at` / `id_at` take the KEY as a string) — the rule ADR 0021 states
next to that very struct, kept here rather than renamed into a serde `rename` (models.rs carries ZERO
serde attributes, and one renamed field would put a wire key's spelling on a type that answers other
doors). ADR 0021's bullet now points here; its measured tables stay history (tags never move).

### Measured (pre-tag)

* builds: exit `0` at EVERY commit. The named tally rode **ZERO → NINE → ZERO** — `feat(db)` PLANTED
  nine names whose reader lands in the very next commit, all nine named BY NAME (`constant SQL_CARD_CELL`,
  `constant SQL_RECIPE_CARD_ROW`, `struct RawRecipeCard`, `struct RecipeCard`, `struct RecipeCell`,
  `function word_at`, `function id_at`, `function recipe_cell`, `function recipe_card`) and the build
  stayed `rc 0`; `feat(cli)` — the door that reads them — built with ZERO named, as did the final tree
  (**4 m 19 s**, prints `Compiling wow-goldtool v0.12.0`, ZERO warnings of any kind). Nine planted and
  killed by the next commit is a fact about THIS era, recorded not muted: ADR 0024's rule, and the
  named set stays ZERO at this release. The `hard linking … incremental cache` notice answered ZERO
  times across this era's own cache-clear builds — where it rode today it rode a DRIVER's `cargo run`,
  and that line is recorded below as a driver-side defect, not a fifth named warning.
* diff scope (`git diff 0b717a4^..aa70092`): **NINE files, 725 insertions / 6 deletions** —
  `src/db.rs` +371 (the card row + `SQL_CARD_CELL` + the three shapes + the two Value readers + the
  two cell doors), `src/cli.rs` +45 −2 (the verb, its JSON, its usage words, its dispatch line,
  two import names), `tests/fixture_api.py` +27 (four rows grown with `learned_from`, plus the block
  that names the three shapes), `ui/index.html` +23 (the ninth tab button + the panel), `ui/app.js`
  +188 −2 (the renderer, the click, the delegated handler), `ui/app.css` +33 (TWELVE new classes),
  `.github/workflows/ci.yml` +36 (the card paragraph + four steps), `Cargo.toml` +1 −1 and `Cargo.lock` +1 −1 (the banner MOVES together with its
  lock twin, `0.11.0 → 0.12.0` — the v0.11.0 shape kept: a FEATURE era takes BOTH files, the shape
  the v0.9.1 / v0.10.1 / v0.11.1 patch eras left PUT, honestly contrasted). `src/` answers an EMPTY
  diff across the fixture / ui / ci commits, so those three commits CANNOT move the build. ZERO SQL
  or wire line moved in `feat(cli)` (the diff answers ZERO matches for `SQL_` / `vendor_buy` /
  `price_trend_normal` / `SCHEMA`), ZERO schema or
  migration line (schema stays **8** — `learned_json` ALREADY existed, which is what makes this door
  need no bump), ZERO money-formatter line, ZERO new quality palette (quality rides the frame's OWN
  `.chip c-<quality>` classes), ZERO new live-state chip (the v0.11.0 trio is REUSED), ZERO `vendor_buy`
  / `price_trend_normal` producer, ZERO per-row listener in the webview (ONE delegated handler).
* CI: **44 → 48 step-lines**, plant line stays one (measured with the same parse the prior eras used:
  **14 `- name:` steps unchanged**, raw `cargo run` mentions **46 → 50**) — the four new steps are
  `recipe 132 --icons` / `recipe 202 --icons` / `recipe 637 --icons` / `recipe 638`, the four learned-from
  shapes. The door's rc `4` is NOT committed as a green step (ADR 0011's split rule, kept from v0.10.0):
  it is measured on scratch, as is the pinned BEFORE witness that this verb is NEW.
* the fixture door, over its OWN 12 rows (fresh port 9159 + fresh scratch DB, ONE observation window,
  driver log `/tmp/opencode/card012-witness.log`): the crawl's own presses stayed the era's — budget-1
  list 3 written, resumed-from-2 list complete; detail 4 fetched + 1 checked-removed, targeted 2 fetched
  + 2 unchanged; icons 3 copied + 1 checked-gone; tooltip 3 copies + 1 checked-none; price 2 live + 1
  checked-price-less; recipes budget-3 3 of 8 files / 5 rows / 4 requests / `rate_limited 1`, resumed-from-3
  5 of 8 files / 7 rows / 5 requests / complete → **12 rows / 8 files**. The door then answered
  **8 rc 0** (7 330 · 1 155 · 6 673 · 4 033 · 4 107 · 643 · 1 209 · 6 740 B) and **rc 4** on `999999`.
  The fixture's own census: `learned_json` **8 NULL / 1** schematic naming 60 / **1 trainer / 1 automatic
  / 1** schematic naming 2589 · 12 recipe rows · 5 item rows · 3 tooltip copies · 1 checked-no-tooltip ·
  2 live set · 1 checked-price-less · 3 icon copies · 1 checked-no-icon · 4 vendor_sell · `vendor_buy`
  **0** / `price_trend_normal` **0**. The four shapes, named: 132's scroll cell for item 60 answers
  IN GAME (`in_db true`, "Fixture Detail 60" beside the set's own words, `Common`, levels 1/1, vendor
  `500 ↔ 5s`, stored `17624 ↔ 1g 76s 24c` with live `15624 ↔ 1g 56s 24c` at 1795394930 — the RUN's pair —
  the copy's 44 chars, and row 60's icon answering CHECKED-GONE: `icon_at` NULL with `icon_missing_at`
  1795394929 SET and ZERO bytes, the third icon state, not empty bytes); 202 answers `learned_type
  "trainer"` with `learned_scroll_id` NULL and `scroll` NULL — the WORD alone — while its crafted 40
  answers `price_now_missing_at` 1795394930 SET with EVERY live field NULL; 637's scroll cell answers
  `in_db false` with the SET's name beside it and `result` NULL (the enchanting shape); 638 answers
  `learned_type` NULL. Money: **0 pairing defects**, **13 counts with no money**. The fixture's own
  GET tallies END where the crawl left them — the door asked NOTHING, which is the proof it is local.
* the real-host door, the user's OWN file, read-only, ZERO requests: `recipe 27835 --icons` rc `0` /
  25 212 B — scroll 22390 answers IN GAME (`epic`, `item_level 63`, `vendor_sell 20000 ↔ 2g`, stored
  `309600 ↔ 30g 96s`, live `420000 ↔ 42g` at 1795393867, a 176-char copy, 3 524 icon chars) and the
  result "Persuader" answers `required_level 58`, vendor `80554 ↔ 8g 5s 54c`, stored `8870000 ↔ 887g`
  with live `8670000 ↔ 867g`, a 244-char copy, 2 928 icon chars; 6 materials, ALL in the DB, 5 with
  money AND live. `recipe 2171 --icons` rc `0` / 7 003 B — scroll 2404 has a row, and EVERY price / copy
  / icon field is NULL (never asked: the honest chip-less state). `recipe 2177 --icons` rc `0` / 9 429 B
  — the `trainer` word, `learned_scroll_id` NULL, NO scroll cell, and a complete result cell with a
  208-char copy and live `230 ↔ 2s 30c` beside stored `250 ↔ 2s 50c`. `recipe 8800111 --icons` rc
  `4` / 112 B naming the stage. Usage rc `2` measured THREE ways: 48 B "recipe wants a spell id, for
  example: recipe 132" · 40 B "recipe wants a numeric spell id, got abc" · 50 B "recipe wants a numeric
  spell id, got 27835 --icons" (a GLUED token — a real rc 2 of the door's numeric guard, found by the
  driver's own quoting slip and recorded as such). The parser stayed as it was: `recipe abc` answers rc
  `2` 51 B "unknown verb recipe abc (run help to see the verbs)" and `recipe 27835 extra` answers the
  same shape at 59 B — the door is ONE word. The usage line names `recipe SPELL_ID [--icons]`.
* the BEFORE witness (the v0.11.1 tree, built green first): `recipe 132` answered rc `2`
  "unknown verb recipe (run help to see the verbs)" — the verb is NEW, not a renamed one.
* the era's price press on the user's OWN file (the WRITER that makes the card's live cells answer
  something, and the honest record of an ABORT): pending basis (ADR 0017's rule: `tooltip_at` set AND
  both price columns NULL AND no `deleted_at` / `manual_at` stamp) started at **4 602**; chunk 1 wrote
  939 + 61 checked-price-less → **3 602** left · chunk 2 976 + 24 → 2 602 · chunk 3 952 + 48 → 1 602 ·
  chunk 4 934 + 66 → **602** · chunk 5 answered **rc 101** and the pass ABORTED there, the rest staying
  a later run's question. The DB now answers `price_min_now` **3 801** / `price_now_missing_at` **199**
  / pending **602**. Chunk 5's rc 101 is an incremental-cache ICE — rustc panicked
  `index out of bounds: the len is 620587 but the index is 1073741824` at
  `rustc_query_system/src/dep_graph/serialized.rs:243:34`, next to the `hard linking … incremental
  cache` notice — caused by the press's own `cargo run` rebuild colliding with the era's `rm -rf
  target/debug/incremental`: a DRIVER-side defect, NOT a door fact (the same tree built green minutes
  earlier, and this era's audit drivers were rewritten to exec the ALREADY-BUILT binary so no compile
  is EVER in flight during an audit). The pass must RESUME later on a PINNED binary, never `cargo run`;
  and until it does, the 602 pending plus the **14 656** rows that have no tooltip copy (ADR 0010's rule
  refuses those) keep answering `live price not refreshed yet` — the card's live chips are a TRUTH on
  this host, not a defect.

### Chain

`0b717a4` feat(db) (the read + the cells: NINE names planted, build rc 0) · `29ac737` feat(cli)
(the verb: ZERO named, rc 4 + the THREE rc 2 usages measured) · `a03008b` test(fixture) (the THREE
`learned_from` shapes + the explicit NULL) · `53d5977` feat(ui) (the ninth tab, the panel, the row
CLICK, the renderer) · `2b1784f` ci (44 → 48 step-lines, ZERO requests, the rc-4 half on scratch) ·
`aa70092` chore (banner 0.11.0 → 0.12.0 — a FEATURE era moves it) · record + tag `v0.12.0`. The
post-tag audits (the 48-step mirror replay on fresh ports, the exit map GROWN with the card doors'
rc `0` / `2` / `4`, the pinned v0.11.1 binary refusing the new argv as the BEFORE witness, the user's
file read read-only) land FORWARD in this era's fold, never chased here. The ladder line the previous
era owed lands HERE: the v0.11.1 fold push's own run measured **45059534810** success 5/5 in **10 min
48 s** (19:13:33Z → 19:24:21Z, exactly ONE run id, no CANCELLED sibling) — its `cargo build` (dev +
release) jobs printed NO named warning on three targets while its lint step answered its own **0
errors / 817 warnings** (clippy's advice, ADR 0011's rule that the gate stays off). That run was
waited for BEFORE this chain pushed, per ADR 0011's ordering rule.

### Post-tag verification — MEASURED: mirror 51/51 GREEN, the map GROWN to 31 doors at NEW rc totals, the pinned pair 15/15 BYTE-IDENTICAL + the BEFORE witness, the real file's cards HONEST before AND after the resumed price pass — and the chain+tag run's own verdict measured **FAILURE 4/5**, with ONE clippy ERROR named BY NAME

All FRESH ports/processes and FRESH DBs; the real host saw NOTHING (every crawl door carried
`--fixture`; the card door is LOCAL); the user's OWN file was read through `sqlite3 -readonly` and
its OWN LOCAL doors, ZERO requests, NO crawl.

| probe | answer |
| ----- | ------ |
| mirror replay — the driver PARSES the committed block (48 step-lines + plant 1 + fixture start 1 + sleep 1 + other 0, so "48" is proven, not retyped) and replays it on a FRESH port + FRESH DB dir, exec-ing the ALREADY-BUILT binary (no compile in flight, the era's own driver-side rule) | **51/51 commands rc `0`, ZERO NONZERO** (`TOTAL_STEPS=51`, rc tally `0×51`) — measured THREE times (9219 / 9231 / 9243; the last keeps the stage reports). Stage presses UNCHANGED: budget-3 3 files / 5 rows / 4 requests / `rate_limited 1` · resumed-from-3 5 files / 7 rows / 5 requests / complete · re-pull 8 requests / **8 answered the same copy** / 0 written. The FOUR card steps answer **7 333 / 6 673 / 4 107 / 643 B**, all rc `0`, and the fixture's OWN GET tallies END where the crawl left them — 51 GETs, 17 of them recipe-file GETs (alchemy 3 + the other seven 2×): the four card steps added **ZERO**, which IS the proof the door is LOCAL. Census: recipe 12 / item 6 / 4 copies / `vendor_buy` **0** / `price_trend_normal` **0** |
| the driver's OWN slips, named honestly (they are DRIVER facts, not door facts — the record keeps them rather than hiding them) | (a) the naive `line.split()` broke the `add-item`'s QUOTED words → that step answered rc `2` "name is not editable…" and the row was NEVER made, which cascaded into `item 900000000 --cached` rc `4` — fixed with `shlex`; (b) leaving the step-lines at the fixture's hardcoded 8765 while the server bound a FRESH port gave rc `3` + rc `4` (a DEAD port) — fixed by replacing the port in BOTH the server AND the argv. The green 51/51 is the FIXED driver's measure; the broken driver answered 22/0×1/3×13/4×15, recorded so the numbers are traceable |
| exit map GROWN with the card door's THREE states — fresh fixture 9213 + fresh seed DB + fresh EMPTY DB + a DEAD port | **31 doors: rc `0`×19 · `2`×5 · `4`×7** (the map GREW here, and honestly so: this era ADDED doors, so the totals moving is the fact — the prior eras' 89-door totals stay in their own records). Nine card doors rc `0` (132 bare 2 119 B · 132 `--icons` 7 325 · 202 6 670 · 637 4 105 · 638 643 · 683 1 207 · 1723 1 154 · 4977 6 737 · 334 4 032 B); rc `4` on `999999` (bare and `--icons`) AND on `recipe 132` over an EMPTY DB — the SAME words, naming the stage ADR 0021 says would write the row (ADR 0004's no-invented-answer rule); rc `2` usage 48 B "recipe wants a spell id, for example: recipe 132" + 40 B "recipe wants a numeric spell id, got abc" + `--fixtures` (a typo) answers 2 "unknown flag"; `recipe 132 extra` answers rc `0` (the door reads `words[0]` — ONE word, the parser stayed put), `recipe-suggest 132` answers rc `2` "unknown verb" (the neighbour verb did NOT swallow the new one); and the trio's honest half: **`recipe 132 --fixture <DEAD port>` answers rc `0`** — the card door IGNORES the flag because it sends ZERO asks, so rc `3` is UNREACHABLE from this door, which is WHY the map shows no `3` at all |
| the pinned pair — the worktree at the tagged tree `3f20b51`, whose OWN build is the BEFORE witness (green, ZERO named, banner `0.11.0` PUT — the PATCH era's shape) vs the new binary, ONE shared fresh read-only DB | **15/15 pre-existing doors BYTE-IDENTICAL, ZERO deltas** (`status` 1 223 B · `stats` 1 773 · `stats --icons --limit 4` 17 384 · `item 25/40 --cached` 4 341 / 4 076 · `search sword` ×3 · `watch list` 8 B · the FIVE recipes doors 12 993 / 2 185 / 4 634 / 7 398 / 2 174 · `refresh-twins` 223 B). ONE shared DB is ASKED here where prior eras used TWO: every compared door is LOCAL and read-only, so nothing writes during the compare — the stronger probe, and no stamp can move. BEFORE witness: the PINNED binary answers `recipe 132` / `recipe 132 --icons` / `recipe 999999` all **rc `2` "unknown verb recipe (run help to see the verbs)"** while the new answers rc `0` 2 124 B / rc `0` 7 330 B / rc `4` — the verb is NEW, not a re-aimed old one |
| the user's OWN file, read-only, measured BEFORE and AFTER the resumed price pass (the card door is LOCAL, so both sides are the SAME doors over the SAME rows) | **BEFORE** (pending 602, `price_min_now` 3 801 / `price_now_missing_at` 199): `27835 --icons` rc `0` 25 213 B — scroll 22390 answers IN GAME (`epic`, level 63, `vendor_sell 20000 ↔ 2g`, stored `309 600 ↔ 30g 96s` with live `420 000 ↔ 42g`, a 176-char copy, 3 524 icon chars) and the result "Persuader" answers `required_level 58` / `80 554 ↔ 8g 5s 54c` / `8 870 000 ↔ 887g` with live `8 670 000 ↔ 867g`; `2171 --icons` 7 004 B answers a scroll row (2404) whose EVERY price / copy / icon field is NULL — never asked, the honest chip-less state; `2177 --icons` 9 430 B answers the `trainer` WORD with NO scroll cell and a complete result cell (live `230 ↔ 2s 30c` beside stored `250 ↔ 2s 50c`); the REAL null-crafted witnesses `7418` rc 0 3 595 B (`automatic`, NO scroll cell, NO result cell, 1 material) and `7422` rc 0 3 452 B (`trainer`, BOTH cells absent — the 133 Enchanting entries' shape); rc `4` on `638` — a FIXTURE spell id absent from the real set, named not hidden — and on `8800111`. **AFTER** the pass (pending 602 → **0**): the SAME doors, sizes 25 213 / 7 004 / 9 430 B UNCHANGED, and 7418 / 7422 rode **3 595 → 3 596 B / 3 452 → 3 453 B** — the ONE-byte delta is the material's live pair FILLING a NULL, not a wording change (the honest delta, recorded not smoothed); the live states now answer **live pairs** where a row has a copy (27835: scroll + result live + 5 of 6 materials; 2177: result live + both materials; 7418/7422: their material live) while the rows ADR 0010's copy rule refuses (14 656 of them, `2171`'s scroll 2404 and its result among them) STILL answer `live price not refreshed yet` — the chip is a TRUTH, not a defect. Money over these very answers: **0 pairing defects**, counts carried 6 / 2 / 2 / 1 / 1 with NO money beside them (ADR 0015's count half). The resumed pass: **pending 602 → 0 in ONE chunk** (602 requests, 543 written, 59 checked price-less, 258 s, rc `0`) on a PINNED already-built binary — the era's ABORT at chunk 5 rc `101` (the incremental-cache ICE) is GONE once the driver execs the binary instead of `cargo run`, which IS the driver-side fix measured true; file after: `price_min_now` **4 344** + `price_now_missing_at` **258** = **4 602** copies each asked ONCE (ADR 0017's one-request-per-record rule), `vendor_buy` / `price_trend_normal` **0 / 0**, tombstoned 0 |
| the era's chain+tag run, GitHub's OWN words — measured FORWARD, the honest verdict | **45095439290 FAILURE**, 4 of 5 jobs green in **8 min 9 s** (01:35:45Z → 01:43:54Z, run #42, attempt 1, exactly ONE run id, no CANCELLED sibling, head `ace683a` = the TAGGED tree). linux 8/8 · macos 7/8 · windows 7/8 (their Ubuntu-dev-libraries step `skipped`, as designed: only the linux target links gtk) — the three `cargo build` (dev + release) jobs are GREEN, and `test` is 8/8 with the **CLI smoke step GREEN**: the 48-step block, the four card steps included, passes on GitHub's own builds. The quality job answers 6/7 with the step "Lint (informational — clippy's own advice; our named set died at v0.11.1, see docs/DEVELOPMENT.md)" = **failure**. Measured on THIS tree, the SAME command the step runs: **1 ERROR / 811 advice warnings** — `error: this loop never actually loops` at `src/db.rs:3029:5`, the `for one_row in mapped {` inside `recipe_card` whose body ALWAYS returns, so the iterator can never take a SECOND turn: `clippy::never_loop`, **deny-by-default** (so an ERROR, not advice — the other 811 lines are advice, 758 of them duplicates, on the bin and its test twin). Established doors avoid this shape by COLLECTING into a Vec (`history_of` / `recipes_rows` / `watch_list`), and `spell_id` is the table's PRIMARY KEY, so AT MOST ONE row answers — clippy's OWN suggestion `if let Some(one_row) = mapped.next()` is the honest repair, and ADR 0024's rule refuses a mute. **THE DOOR ITSELF IS NOT BROKEN**: rc `0` witnesses above, nine rc `0` on the map, the smoke block green on GitHub — this is NOT a door fact, and the fix lands FORWARD in the v0.12.1 patch era. The step's NAME is now a LIVE LIE: "informational" was true OF the tree at v0.11.1 (0 errors / 817 advice), but riding an ERROR the step GATES the run (GitHub's own verdict moved success → FAILURE), so the step's name and the header comment at `ci.yml:18-24` move WITH the fix — ADR 0011's gate stays OFF for clippy's ADVICE, and an ERROR is not advice |
| limits kept | schema stays `8` (the scratch DBs, the fixture's and the user's file all answer 8), tags stay put (`v0.10.0` → `e4d3447` · `v0.10.1` → `09c8d77` · `v0.11.0` → `59b0185` · `v0.11.1` → `3f20b51` · `v0.12.0` → `ace683a`), the era diff stays NINE files / 725 ins / 6 del with `src/` an EMPTY diff across the fixture / ui / ci commits, CI stays 48 step-lines + plant one; the fold's own push starts the NEXT run (the SAME tree, so the lint step is EXPECTED to answer the SAME error — the record says so rather than hoping) and its verdict is measured in the NEXT record, never chased here (ADR 0011's ladder) |

---

## 2026-11-22 · v0.11.1 — patch: the FOUR named build warnings die BY NAME, the set shrinks FOUR to ZERO, and NOTHING was muted

### What forced it

The user's ask: "lets work on those warnings, fix them and if not fixable mute them." For
sixteen milestones the set was a CONTRACT, not a surprise: `docs/DEVELOPMENT.md`, `docs/DEVLOG.md`
and ADR 0011's `-D warnings` bullet named the same FOUR on every build — `PendingSql`'s privacy,
`Tooltip`'s unread fields, `open_db`, `catalogue`. A patch era that leaves them standing while
the docs keep calling them planned is the drift this era must not ship, so all FOUR were taken
down BY NAME. The ask's second branch never had to be used: each of the four is a name the tree
no longer needs, and rustc itself proves which name that is.

### The four warnings, rustc's own words, off the last green build

* `type 'PendingSql' is more private than the item 'pending_plan'` — `src/stats.rs:199`, "reachable
  at visibility `pub(crate)`", note "type `PendingSql` is only usable at visibility `pub(self)`"
  at `src/stats.rs:191` (`private_interfaces`, on by default).
* `multiple fields are never read` — `src/models.rs:142:9`, fields in `pub struct Tooltip`: `id`,
  `kind`, `name`, `quality`, `version`, `build`, `icon_url`, `links`, `attribution`; the read two
  are `html` and `fields`; rustc's own note says derived `Debug`/`Clone` are "intentionally
  ignored during dead code analysis".
* `function 'open_db' is never used` — `src/db.rs:490:8`.
* `function 'catalogue' is never used` — `src/sync.rs:909:8`.

The `hard linking … incremental cache` notice that rode the PREVIOUS era's builds answered
**ZERO times** across this era's builds — every build here still starts by removing
`target/debug/incremental`, so the FS artifact's absence is measured, not silenced, and it was
never a fifth named warning.

### The decision (ADR 0024 — four fixes, zero mutes, every rejected mute recorded)

Each fate is the NARROWEST true thing the tree can say, and each rode with its claim where the
survivor lives. 1 — `pending_plan`'s claim was the lie (both `use crate::stats` lists leave the
name out, all FOUR callers are this module's own queue doors), so the function stops claiming
`pub`; widening `PendingSql` instead would put a queue-plan type on a surface nothing consumes.
2 — `Tooltip` narrows to the two keys a reader reads (`html` for the copy stage, `fields` for the
price stage's `price.min`); the mirror already types ONLY what has a reader TWICE in this file
(`TooltipFields` types only `price`, the recipe shapes the same) and ADR 0021 called a
stored-without-reader field dead code, so the eight unread keys ride the wire the way serde had
always read them; `docs/API.md` keeps the whole measured key list, which is where probe records
live. The cost is recorded in the struct's comment, not hidden: `id` was serde's required-key
check, and NOTHING compares a tooltip's own id against the one in the request URL. 3 — `open_db`
died as a wrapper whose contract ("every entry point goes through this") stopped being true when
the entry points moved to `db_path` + `open_db_at`; its body was character-for-character the live
door's, so the schema rules, the refusal and the fresh-file write did not move. 4 — `catalogue`
died AS its import names: `sync.rs`'s `use crate::stats` line named `coverage` and `Coverage`
ONLY for that one-line delegation, so leaving either behind would have traded a dead-code warning
for an unused-import one — the swap ADR 0011's "the set is expected" rule refuses. `coverage`'s
own comment now carries the promise the removed wrapper owned ("used by `status`, the `stats` verb
and the UI footer"), so no claim was buried with a grave.

### Measured (pre-tag)

* builds: exit `0` at EVERY commit, and the named tally SHRANK one per commit — FOUR at the base
  (`e1dc5c6`), **THREE** after `523dfcc`, **TWO** after `7b3bdb0`, **ONE** after `6333783`,
  **ZERO** after `316bb0e` (each measured with the era's cache-clear mechanic; the tally is the
  honest delta of this era, and the FOUR-named contract that held every era since v0.8.0 DIES
  HERE — later builds answer FEWER than FOUR by construction, not by accident).
* diff scope (`git diff e1dc5c6..316bb0e`, measured with the last fix in hand, this era's Rust):
  FOUR files, 40 insertions / 49 deletions —
  `src/stats.rs` (the visibility claim + `coverage`'s moved promise), `src/models.rs` (the mirror
  narrowing), `src/db.rs` (the wrapper + its dangling doc reference), `src/sync.rs` (the wrapper +
  its orphaned import names). `Cargo.toml` / `Cargo.lock` answer an EMPTY diff (a PATCH era — the
  banner stays put at `0.11.0`, the v0.9.1 / v0.10.1 shape), `tests/` an EMPTY diff (no fixture
  delta: there is no behavior delta to smoke), `ui/` ZERO lines, ZERO SQL constant moved
  (`SCHEMA_SQL` itself never changed — the two `SCHEMA_SQL` / `meta_set` hits inside the diff are
  the REMOVED wrapper's own body copy), ZERO schema or migration line, ZERO money-formatter line,
  ZERO request-rule line.
* CI: **ZERO steps moved** — the block stays 44 step-lines + plant one (14 `- name:` steps and
  46 raw `cargo run` mentions measured on both sides of the edit, the same parse the prior eras
  used rather than the naive grep). Two claims in the workflow were LIVE lies after this era and
  are corrected in the same era: the header comment ("this build's four dead-code warnings … are
  intended") and the lint step's NAME ("this build's warnings are the plan"). Both keep their
  history (FOUR through v0.11.0, ZERO from v0.11.1) and ADR 0011's reason the gate stays off —
  clippy answers 0 errors and hundreds of its OWN advice lines, so `-D warnings` would still lie.

### Chain

`523dfcc` fix(stats) · `7b3bdb0` fix(models) · `6333783` fix(db) · `316bb0e` fix(sync) · `092ab61` ci
(the two warning claims, ZERO steps) · record + tag `v0.11.1`. Post-tag audits (the 44-step mirror
replay on fresh ports, the exit map re-run grown at 89 doors, the pinned v0.11.0 binary vs the new
one, the user's own file read read-only) land FORWARD in this era's fold, never chased here. The
ladder line the previous era owed lands HERE: the v0.11.0 fold push's own run measured **45049266077**
success 5/5 in **8 min 2 s** (17:24:24Z → 17:32:26Z, one run id, no CANCELLED sibling) — waited for
BEFORE this chain pushed, per ADR 0011's ordering rule.

### Post-tag verification — MEASURED on fresh ports: mirror 47/47, the map still 89 doors at the same rc totals, the pinned pair BYTE-IDENTICAL, the real file HONEST

All FRESH ports/processes and FRESH DBs; the real host saw NOTHING (every crawl door carried
`--fixture`; the paging/live/recipe doors are LOCAL — ZERO requests); the user's OWN file was read
through its OWN doors only (opens + LOCAL, ZERO requests, NO crawl).

| probe | answer |
| ----- | ------ |
| mirror replay — driver generated FROM the committed block, port-stripped compare EQUAL **44/44 step-lines** (the era moved no step, proven not claimed), fresh fixture process on 8765, fresh DB dir | **47/47 commands rc 0, ZERO NONZERO** (`TOTAL_STEPS=47`); stage presses UNCHANGED: budget-3 3 files / 5 rows / 4 requests / `rate_limited 1` · resumed-from-3 5 files / 7 rows / 5 requests / complete · re-pull 8 files / **8 answered the same copy** / 0 written; the Blacksmithing step in THIS RUN reads BOTH live states in ONE answer — crafted 40 `crafted_now_missing_at` 1795374142 SET with its OWN copied bytes (b64 2 596 chars, icon stamp 1795374121), crafted 60 live `13151 ↔ 1g 31s 51c` DIFFERENT from stored `15151 ↔ 1g 51s 51c` (the RUN's pair) with `crafted_now_at` 1795374142 and its icon keeping the honest "no copied bytes"; reagent 25 answers live `14151 ↔ 1g 41s 51c` |
| mirror builds, named tally | **ZERO named warnings** on every one of the era's builds; the `hard linking … incremental cache` notice rode SOME mirror builds ("`generated 1 warning`" = that notice ALONE) — the FS artifact of the cache mechanic, and the era's own fix-commit builds (cache cleared each time) answered it ZERO times; the driver's own `rm -f …fixture.sqlite*` line hit zsh's nomatch because NOTHING existed to remove — freshness confirmed by the glob's abort, not by a silent pass |
| exit map RE-RUN GROWN — fresh fixture 9631 + fresh seed DB + fresh EMPTY DB + DEAD 9996 + the scratch 404-file server 9561 | **89 doors `0`×60 · `2`×17 · `3`×6 · `4`×6, ZERO mismatches** — the SAME rc totals as v0.11.0's map, because ADR 0024 moved NO door semantics at all; the live-state counts ride the map's own answers unchanged (seed table: matching 12 / crafted live 5 / crafted missing 1 / crafted not-refreshed 4 / reagent price-less 4; Blacksmithing 2 rows carry 1 + 1 side by side) |
| the PINNED pair — the v0.11.0 binary (worktree at the tagged tree `59b0185`, whose OWN build is the BEFORE witness: it answers the FOUR named warnings, verbatim, in 1 m 08 s) vs the new binary, ONE shared fresh fixture process (9633), TWO fresh DBs | **25 DB-only doors, ZERO content delta**: 22 BYTE-IDENTICAL in the alternating run, and the three whose answers carry STAMPS (`watch add 25 40` 37 B · `set-item 25 vendor_sell=606` 76 B · `item 25 --cached` 4 399 B) are raw-EQUAL on a same-second re-measure with **ZERO non-stamp field** differing — their only difference is `at` / `*_at` landing on different seconds (timing rides honestly, ADR 0011's rule). Unlike v0.11.0's pair, NO strip was needed: the recipes doors are byte-identical TOO, because ADR 0024 added no key anywhere. Both DBs agree: schema 8 / 5 items / 4 priced / 2 `price_min_now` / 1 price-less / 12 recipe rows / `vendor_buy` 0 / `price_trend_normal` 0 |
| the user's OWN 19 258-row file, BOTH binaries over the SAME file (opens + LOCAL doors, ZERO requests) | **10 of 10 doors md5-equal**: `status` 1 098 B · `stats --icons --limit 6` 37 648 B · `item 25 --cached` 2 950 B · `search sword --limit 2 --offset 1` 979 B · `search sword --limit 4 --offset 10` 1 729 B · `watch list` 707 B · `recipes --limit 3` 4 248 B · `recipes --profession Blacksmithing --icons --limit 3` 21 061 B · `search Chambers --limit 3` 188 B · `stats` 4 554 B; tallies INTACT schema 8 / **19 258** items / **4 231** vendor_sell / **4 287** samples / **1 240** recipe rows; the live twin HONEST on this host: `price_min_now` 0 / `price_now_missing_at` 0 (the price stage never re-pressed here) so the REAL joined rows answer `live price not refreshed yet` (3/3 crafted, 3/3 reagents; Blacksmithing answers **matching 250** REAL rows) — byte-equality is what keeps those words identical to the pinned era's record rather than a fresh claim; `vendor_buy` / `price_trend_normal` **0/0** (ADR 0016's grounds hold) |
| the CI run's OWN compile log, external witness | the chain+tag run (head = the TAGGED tree `3f20b51`) **45058168649** success 5/5 in **9 min 25 s** (18:59:38Z → 19:09:03Z — quality 2 m 2 s · linux 5 m 27 s · test 5 m 50 s · macos 9 m 2 s · windows 9 m 23 s), exactly ONE run id, no CANCELLED sibling; its `cargo build` (dev + release) jobs printed NO named warning AT ALL — the four died on GitHub's own builds, not just on this filesystem — while the lint step answered its own **0 errors / 817 warnings** (clippy's advice; ADR 0011's earlier ~600 measurement stays put, the count rode `--all-targets` and duplicates, not our named set) |
| limits kept | no schema move (stays `8` — scratch DBs and the user file all answer 8), ZERO ui line moved (so NO parse witness was owed: `ui/` answers ZERO lines in the era diff — there is nothing NEW to parse), ZERO fixture change, ZERO CI step (44 step-lines + plant one), tags stay put (`v0.10.0` → `e4d3447`, `v0.10.1` → `09c8d77`, `v0.11.0` → `59b0185`, `v0.11.1` → `3f20b51`); the fold's OWN push starts the NEXT run, measured in the NEXT record — the ladder never chases a tag (ADR 0011) |

---

## 2026-11-22 · v0.11.0 — the LIVE price IN the recipe cell: the stored min keeps its cell and hover, the live pair rides beside it, the two NULLs stay two honest chips

### What forced it

The user's fifth ask ("the crafted cell should show what the AH offers NOW, not just
the stored min") and the question v0.9.9 + ADR 0020 recorded as DEFERRED — "showing
the live price IN the cell = next release's honest question" — because the SEARCH-hit
cell would have needed a wire-shape change (`HIT_COLUMNS` + a per-row live money pair).
The deferral found its producer-side answer where the row was already owned: the
RECIPE door joins by value through the item rows its own DB holds (ADR 0021), and
ADR 0017's price stage ALREADY writes `price_min_now` beside the stored min (schema
7's OWN columns, its own stamp pair). BEFORE witness (measured, the v0.10.1 era's
own mirror): the `recipes --limit 40` answer carried `crafted_price_copper` + money
ONLY — no live fields anywhere, and the fixture already offered the states: id 25
stored 17520 / live 16520, id 40 live NULL WITH `price_now_missing_at` SET, id 12809
BOTH stamps NULL (the price stage never asked it — `tooltip_at` NULL rides its pending
rule). The two NULLs mean DIFFERENT things: showing one answer for both would be the
lie ADR 0006 refuses.

### The decision (ADR 0023, four parts)

One read carries the twin (ADR 0014: `SQL_LIVE_NAME_PRICE` widens to `price_min_now,
price_now_at, price_now_missing_at` in the SAME SELECT — both by-value joins already
call `live_name_price_of` ONCE, so the joins cannot meet a row a write moved mid-read;
no second query). The two NULLs stay TWO answers named by the STAMPS: `price_now_missing_at`
set = "looked, the AH offers NOTHING right now" (ADR 0017's third state); both stamps
NULL = "live price not refreshed yet" (this record never met the price stage — ADR 0016's
never-written pattern restated). The cell rule: money IN the cell stays the STORED min
with its exact-copper hover (ADR 0015 + the hover rule untouched); the LIVE money span
rides beside it ONLY when the two DIFFER (live == stored adds NOTHING — a second identical
number is noise; the hover keeps the exact live copper, the words come from Rust's ONE
formatter); NO live noise at all for an id the DB has no row for (the joins wrote none —
the caller guards). ONE conditional note sentence in the ADR 0020 pattern (NO numbers —
numbers ride the cells) names cell = STORED vs live = LAST ASK, shown ONLY when a live
part fired. What it does NOT do: no schema hop (schema stays `8`, the columns are schema
7's), `vendor_buy` / `price_trend_normal` stay NULL UNTOUCHED (ADR 0016's grounds hold —
this release READS `price_min_now`'s twin, never fills the other two), the SEARCH-hit
cell stays stored-only (ADR 0020's deferral measured UNCHANGED — `HIT_COLUMNS` did not
move).

### Measured (pre-tag)

* builds: exit 0, the FOUR named warnings (`PendingSql` privacy, unread `Tooltip` fields,
  `open_db`, `catalogue`) every commit — the incremental-clear hard-link line the FS
  artifact of the clear-the-cache mechanic, never a fifth named warning (the mirror's 44
  cargo builds print FOUR + the FS line each; cargo counts it in its summary).
* AFTER (fixture set, fresh DB + fresh fixture process, ZERO real-host requests — every
  crawl door carried `--fixture`, the doors are LOCAL): `recipes --limit 40` answers 12
  rows with stored `18490 ↔ 1g 84s 90c` (the RUN's seed) and live `17490 ↔ 1g 74s 90c`
  (id 25) / `16490 ↔ 1g 64s 90c` (id 60) — live differs 5, `no auction right now` 1 (id 40,
  `price_now_missing_at` set), `not refreshed yet` 4, and the TWO enchanting crafts
  (crafted id NULL) carry NO live noise at all; reagents 11 of 19 join a row (4 live pairs
  beside the stored money, 4 price-less states, 3 not refreshed yet). Fixture RUN pairs
  moved honestly (20622 / 18490 / 17520 / 18908 seen across the RUNs — the fixture seeds
  prices per RUN, the RULE (ADR 0015) did not).
* the Blacksmithing door carries BOTH states side by side in 2 rows (the shape the new
  CI step reads): spell 202 crafted 40 stored `20622 ↔ 2g 6s 22c` with `crafted_now_missing_at`
  1795366692 SET and its OWN copied bytes beside it (b64 2 596 chars, icon stamp 1795366661);
  spell 683 crafted 60 live `18622 ↔ 1g 86s 22c` DIFFERENT from stored `20622` with
  `crafted_now_at` 1795366692, and its crafted icon stays the honest "no copied bytes"
  state (the DB never copied 60's icon) — four field kinds ride in ONE answer, none merged.
* CI grew ONE step (**43 → 44 step-lines** + plant one, verified by step-line parse —
  the naive grep counts the comments' own `cargo run` mentions, 46 raw, not used):
  `cargo run -- recipes --profession Blacksmithing --icons --limit 3`, ZERO requests
  (the door is LOCAL, ADR 0012's rule). Fresh mirror measured BEFORE the ci block landed,
  driver generated FROM these very steps (port-stripped compare EQUAL): **47/47 commands
  rc 0, ZERO NONZERO** (`TOTAL_STEPS=47` = 44 step-lines + plant + fixture start + sleep,
  fresh fixture process on the block's own port, DB born 8); the stage presses kept their
  measured answers UNCHANGED (budget-3 4 requests / 3 of 8 files / 5 rows / `rate_limited 1`
  · resumed-from-3 5 / 5 / 7 / complete · re-pull 8 / 8 files / 8 answered the SAME copy /
  0 written) — the new step spends ZERO GETs, the tallies cannot move.
* UI witnesses ride the launch-audit rule (no display in this build room): `node --check`
  PARSES the grown app.js (a PARSE witness, not a rendered claim), grep witnesses name the
  EXACT live trio drawn — `class="live"` span + the two `chip_word` chips (`live-missing` /
  `live-none`) — and app.css styles the SAME three class names; the conditional note fires
  ONLY when a live part fired (`live_shown` reset per door run, measured by grep of the 6
  sites). The rendered tab stays never-claimed.
* limits kept: the SEARCH-hit shape did not move (the mirror's `search sword --offset 4`
  step answers `total 0` + `hits []` IDENTICALLY to the v0.10.1 era's mirror; the diff
  touched no `HIT_COLUMNS` line — grep of the chain), money words never ride an auction
  COUNT (no count moved), `vendor_buy` / `price_trend_normal` NULL untouched (the CI card
  step answers them null — the trio's NULL-vs-0 split rides put), schema answers `8` in
  the mirror DB (born 8 — no hop), the real host saw NOTHING, `tests/__pycache__/` stayed
  out of every commit (explicit paths, the .gitignore gap honored).

### Chain

`ff99bce` feat(db) (the SELECT widens, both joins fill copper + money + the two stamps) →
`0690f38` feat(ui) (`live_cell` + the conditional note + the CSS trio `.live` / `.live-none`
/ `.live-missing`) → `b03be1b` ci (the 44th step reads both live states, ZERO requests) →
`e1500fd` chore (banner 0.10.0 → 0.11.0 — a FEATURE release: new cell shape + new door fields
READ + new step; contrast the v0.10.1 and v0.9.1 patch eras that LEFT it put — this release
earns the move, recorded not slipped) → this record carries the annotated tag `v0.11.0`.

### CI ladder

The v0.10.1 verification-fold push measured WHILE this chain built: **45045533751** —
**success, 5/5 in 11 min 4 s** (2026-11-22T16:44:44Z → 16:55:48Z — quality 2 m 6 s · linux
5 m 32 s · test 5 m 58 s · macos 5 m 39 s · windows 11 m 1 s), exactly ONE run id for that
head, no CANCELLED sibling, no rerun. The era's own mid-chain feature push (head `0690f38`)
started **45046599075**, IN FLIGHT at record time — its verdict lands FORWARD in this era's
verification fold, NEVER chased here (ADR 0011's ordering rule; every push in this era waited:
the chain pushed only with no run in flight, and the record's chain+tag push waits for
`45046599075` to complete). This record's chain+tag push starts the NEXT run, measured in
the fold — the ladder never chases a tag, tags stay put (`v0.10.0` → `e4d3447`, `v0.10.1` →
`09c8d77`, `v0.11.0` → this record's commit).

### Post-tag verification — MEASURED on fresh ports: 47/47 green, the map grown 89 doors, the pinned strip-proof, the real file answers HONEST

All FRESH ports/processes and FRESH DBs; the real host saw NOTHING (every crawl door carried
`--fixture`; the paging/live doors are LOCAL — ZERO requests); the user's OWN file was read
through its OWN doors (opens + LOCAL, ZERO requests), never crawled.

| probe | answer |
| ----- | ------ |
| mirror replay — driver generated FROM the GROWN committed block (port-stripped compare EQUAL **44/44** step-lines + plant one), fresh fixture PROCESS on 8765, fresh DB `post111/fixture.sqlite` born 8 | **47/47 commands rc 0, ZERO NONZERO** (`TOTAL_STEPS=47` = 44 step-lines + plant + fixture start + sleep); the stage presses UNCHANGED: budget-3 **4 requests / 3 of 8 files / 5 rows / `rate_limited 1` / `complete false`** · resumed **`from 3` / 5 / 5 / 7 rows / `complete true`** · re-pull **8 / 8 files / 8 answered the SAME copy / 0 written**; this RUN's stored pair `16181 ↔ 1g 61s 81c` beside the live pair — the new step rc 0 reading BOTH states in one answer: crafted 40 `crafted_now_missing_at` 1795367781 SET + its OWN copied bytes (b64 2 596 chars, icon stamp 1795367748), crafted 60 live `14181 ↔ 1g 41s 81c` DIFFERENT from stored `16181` with `crafted_now_at` stamped; mirror builds print the FOUR named warnings + the hard-link FS line (44/44 — the incremental-clear mechanic's artifact, never a fifth named) |
| exit map RE-RUN GROWN — fresh fixture 9622 + fresh seed DB + fresh EMPTY DB + DEAD 9997 + the scratch 404-file server 9559 | **89 doors: `0`×60 · `2`×17 · `3`×6 · `4`×6**, ZERO mismatches; the TWO live doors answer rc 0 (the era moved NO rc semantics — the answer GREW fields instead); over the seeded table `recipes --limit 40` reads crafted live-pair 5 / crafted missing-state 1 / not-refreshed-yet 4 / reagent price-less 4, and the Blacksmithing door 2 rows carry crafted live 1 + missing 1 side by side; the empty-table pair still 4/4, the bogus filter still 4 (`Jewelcrafting` not in the set), the whole-door refusals still 2/2, the 404-FILE abort 3/3 no summary line; the tombstone-ordering door (`item 900000000 --cached`) rides rc 0 HONESTLY: an earlier door tombstoned the id, the card answers the row the DB OWNS (ADR 0010) — the gone-upstream pair 127555 still answers 4/4 |
| pinned pair — the v0.10.1 binary (worktree at the tagged tree `09c8d77`, built 4-warnings-green) vs the new binary, ONE shared fresh fixture process 9623, TWO fresh DBs | **19 non-recipes DB-only doors BYTE-IDENTICAL** (rc + md5 both sides); **every SIX recipes door shows the exact proof** `new answer MINUS the live keys ≡ pinned answer` (rc 0/0 each): spell/row ORDER, names, dataset words, stored copper + money, icon states (copied bytes AND the honest no-bytes state) all did not MOVE — the release ADDED fields where and ONLY where it said (`price_now_copper/price_now_money/price_now_at/price_now_missing_at` + crafted mirrors); both DBs answer identical tallies (schema 8 / 5 items / 2 `priced_now` / 1 `price_less` / 12 recipe rows — no schema delta, no value drift behind the fix) |
| the user's OWN file, read by the new binary (opens + LOCAL doors, ZERO requests, NO crawl) | two `status` opens rc 0/0 **BYTE-IDENTICAL**; tallies intact: schema 8 / **19 258** rows / **4 231** `vendor_sell` / **4 287** samples / **1 240** recipe rows; the live twin measured HONESTLY on the real file: `price_min_now` **0**, `price_now_missing_at` **0** — the price stage never re-pressed THIS host, so the columns are the THIRD state (never-refreshed-yet), NOT a price-less state — and the REAL doors answer exactly that: `recipes --limit 3` joined rows carry `live price not refreshed yet` (crafted 3/3, reagents 8/8), `--profession Blacksmithing --icons --limit 3` answers `matching 250` real rows with the same honest chips (fixture NEVER invented here); `vendor_buy` / `price_trend_normal` still **0/0** (ADR 0016's grounds hold on the real file unchanged); the PINNED v0.10.1 binary vs the new one OVER THAT FILE: **6 read-only doors BYTE-IDENTICAL md5-equal** (`status` 1 098 B · `stats --icons --limit 6` 37 648 B · `item 25 --cached` 2 950 B · the two search pairs 979 B / 1 729 B · `watch list` 707 B — the search/card shape did not move AT ALL), and the real `recipes` answers are struct-equal-after-strip (the live keys are ALL the delta, on real 1 240-row data) |
| limits kept | no schema move (stays 8: scratch DBs and the user file answer 8), no UI line moved SINCE the feat(ui) commit (nothing new to parse — the rendered look never claimed, `node --check` was the PARSE witness), the real host saw NOTHING but the deliberately recorded witnesses of earlier eras, `tests/__pycache__/` stayed out of every commit (explicit paths), every fixture/scratch server killed after its run and the ports re-verified free; one harness-noise note recorded honestly: the wait loop's `gh run watch --exit` on 45046599075 exited **1** while that run itself COMPLETED success — the harness quirk, the ladder lost nothing, chased nothing |

**Ladder where it stands (all measured, never chased):** **45046599075** (the era's mid-chain
feature-push head `0690f38`): **success, 5/5 in 10 min 42 s** (2026-11-22T16:56:24Z → 17:07:06Z —
quality 1 m 43 s · linux 5 m 13 s · test 5 m 45 s · macos 7 m 45 s · windows 10 m 40 s), exactly
ONE run id for that head, no CANCELLED sibling, no rerun — it landed WHILE the record + chore +
docs grew (nothing pushed while it flew). **45048079898** (this release's chain+tag push
`0690f38..59b0185`, head = the TAGGED tree `59b0185…`): **success, 5/5 in 7 min 28 s** (17:11:55Z
→ 17:19:23Z — quality 2 m 3 s · linux 6 m 37 s · test 6 m 29 s · macos 7 m 24 s · windows 6 m 35
s), exactly ONE run id, no sibling. The fold's own push starts the NEXT run — measured in the
NEXT record: the ladder never chases a tag, tags stay put (`v0.10.0` → `e4d3447`, `v0.10.1` →
`09c8d77`, `v0.11.0` → `59b0185`).

## 2026-11-22 · v0.10.1 — the paging doors ride one rule: `--offset` alone answers, twice

### What forced it

v0.10.0's post-tag exit map (ADR 0021's post-tag section, the map's own line) measured a
defect class that lives on TWO doors, not one: `--offset` WITHOUT `--limit` composes
`OFFSET ?` alone and SQLite REFUSES that — `search sword --offset 3` and `recipes --offset
10` each answered rc **4** carrying the raw refusal `near "OFFSET": syntax error … OFFSET ?`.
The recipe twin was recorded fresh; the search twin shows the class was BORN with v0.9.0's
door and never measured until the map ran it today. Two truths shaped the fix: a door that
says "skip N and show the REST" still has to emit a LIMIT token, and SQLite's OWN token for
"no bound" is a NEGATIVE limit value — measured the same day on the DB this tool links
(sqlite 3.54.1): `LIMIT -1 OFFSET 3` on a 10-row scratch table answered rows 4..10,
IDENTICAL to the bare SELECT's tail. A patch era: no feature, no schema move, no new door.

### The decision (ADR 0022, three parts)

One `paging_clause(limit, offset)` in `query.rs` — ADR 0014's one-code-path rule restated
(a door growing its own copy is a second, drifting truth); the negative limit rides SQLite's
measured semantics rather than inventing a sentinel row-count (a second SELECT asking the DB
how big the store is would add a query the door never needed) or an arbitrary cap (it would
hide rows under a door that claims "the rest"); and the raw-SQL message class is killed at the
COMPOSITION, not by a redaction filter invented for text the tool can no longer produce
(ADR 0004's message rule, measuring-first).

### Measured (pre-tag)

* builds: exit 0, the FOUR named warnings (`PendingSql` privacy, unread `Tooltip` fields,
  `open_db`, `catalogue`) every commit; the incremental-clear hard-link line is the FS
  artifact of the clear-the-cache mechanic, not a fifth.
* BEFORE, both doors, same machine same day (scratch DBs, both doors LOCAL — ZERO requests):
  `search sword --offset 3` rc 4 with the raw clause; `recipes --offset 10` rc 4 with the raw
  clause (12 recipe rows landed by `sync --stage recipes --fixture`, the fixture's own set).
* AFTER, the same doors: `search paging --offset 4` rc 0 answering the 8-row TAIL (rows 5..12
  of a 12-row store), `--offset 11` 1 row, `--offset 40` an EMPTY set rc 0 (never a 4 — the
  door's no-row-past-the-end shape); `recipes --offset 10` 2 rows of `matching 12` (spell ids
  3024/3023), `--offset 2` 10 rows, `--offset 12` empty rc 0, `--profession Alchemy --offset 1`
  1 row of matching 2.
* the PINNED pair (the v0.10.0 binary, worktree at the tagged tree `e4d3447`, vs the new binary
  — ONE shared fresh fixture process, TWO fresh DBs): **22 DB-only doors BYTE-IDENTICAL**, and
  the two paging doors show EXACTLY the rc delta **4 → 0**; BOTH DBs answer the same tallies
  (schema 8, 5 items, 4 priced, 12 recipe rows — no schema delta behind the fix). The working
  pair is byte-stable, md5-equal both sides: `recipes --limit 4 --offset 10` **1 629 B**,
  `search sword --limit 4 --offset 4` **182 B**, `search sword --limit 4` **185 B**.
* CI grew TWO steps (**41 → 43 step-lines** + plant one, verified by step-line parse and not by
  the naive grep, which counts the new comment's own `cargo run --` mentions): `recipes --offset
  10` + `search sword --offset 4`, ZERO requests added (both doors LOCAL). Fresh mirror measured
  BEFORE the ci commit, driver generated from these very steps: **46/46 commands rc 0, ZERO
  NONZERO** (`TOTAL_STEPS=46` = 43 step-lines + plant + fixture start + sleep), fresh fixture
  process, fresh DB born 8; the two steps' rc delta measured against the pinned binary over the
  SAME CI DB (pinned answers rc 4 with the raw clause, new answers rc 0 — `search sword --offset
  4` reading `total 0` `hits []` because the store holds no sword row at that point, an empty
  tail is the door's own shape and never a 4; `recipes --offset 10` reading 2 rows of matching
  12). The fixture RUN's money pair `18908 ↔ 1g 89s 8c` rode the SAME steps (yesterday's `17371
  ↔ 1g 73s 71c` moved because the fixture seeds prices per RUN — the rule (ADR 0015) did not).
* limits kept: the real host saw NOTHING (every crawl door carried `--fixture`, the paging doors
  are local); no UI line moved (ZERO webview lines across the chain — the rendered look stays
  never-claimed by construction, no display here and nothing new to parse); no schema move, no
  new dependency; `tests/__pycache__/` stays out of every commit (the .gitignore gap honored by
  explicit paths).

### Chain

`9f4da68` fix(query) (the rule + the search door rides it) → `2a41ce3` fix(db) (the recipe
door rides the SAME helper) → `9e009b8` ci (the two door steps) → this record carries the
annotated tag `v0.10.1`. NO chore commit: the Cargo banner stays put at `0.10.0` through a
patch chain — the same honest shape the v0.9.1 patch era left put, recorded not slipped.

### CI ladder (folded as measured)

**45042291654** (the v0.10.0 verification-fold push, head `c94f6b0`): **success, 5/5 in 9 min
32 s** (16:10:04Z → 16:19:36Z — quality 1 m 57 s · linux 5 m 38 s · test 5 m 44 s · macos 2 m
27 s · windows 9 m 29 s), ONE run id for that head, no CANCELLED sibling, no rerun — it landed
WHILE this patch chain built (nothing was in flight at `9f4da68`'s first commit; the ordering
fact recorded where it stands). Earlier run folded in v0.10.0's own fold: 45040598595. This
record's chain+tag push starts the NEXT run, measured FORWARD in the NEXT record — the ladder
never chases a tag, tags stay put (`v0.10.0` → `e4d3447`, `v0.10.1` → this record's commit).

### Post-tag verification — MEASURED on fresh ports: 46/46 green, the map grown, the real file's doors answer the REST

All FRESH ports/processes and FRESH DBs; the real host saw NOTHING (every crawl door carried
`--fixture`, the paging doors are LOCAL — ZERO requests); the user's OWN file was read through
its OWN doors, never crawled.

| probe | answer |
| ----- | ------ |
| mirror replay — driver generated FROM the GROWN committed block (port-stripped compare EQUAL **43/43** step-lines + plant one), fresh fixture PROCESS on 8765, fresh DB `post102/fixture.sqlite` born 8 | **46/46 commands rc 0, ZERO NONZERO** (`TOTAL_STEPS=46` = 43 step-lines + plant + fixture start + sleep); the stage presses re-measured: budget-3 **4 requests / 3 of 8 files / 5 rows / `rate_limited 1` / `complete false`** · resumed **`from 3` / 5 requests / 5 files / 7 rows / `complete true`** · re-pull **8 requests / 8 files / 8 answered the same copy / 0 written**; the RUN's money pair `17520 ↔ 1g 75s 20c` beside all 12 reagent slots; the two paging steps rc 0 — `recipes --offset 10` answering 2 rows of `matching 12` (first spell 3024) and `search sword --offset 4` answering `total 0` + `hits []` (no sword row there: the door's own no-row-past-the-end shape, never a 4) — the mirror's builds print the FOUR named warnings plus the hard-link FS line (the incremental-clear mechanic's artifact, never a fifth named warning) |
| exit map RE-RUN GROWN — fresh fixture 9617 + fresh seed DB + fresh EMPTY DB + DEAD 9997 + the scratch 404-file server 9558 | **87 doors: `0`×58 · `2`×17 · `3`×6 · `4`×6**; the SIX paging doors answer rc 0 exactly as ADR 0022's contract says (`recipes --offset 10/2/12`, `--profession Alchemy --offset 1`, `search sword --offset 3`, `search map102 --offset 1`); the empty-table pair still answers 4/4, the bogus-profession filter answers 4 (`Jewelcrafting` is not in the set's words), the whole-door refusals still answer 2/2, the 404-FILE abort answers 3/3 (no summary line); one ordering fact recorded honestly: `item 900000000 --cached` answered rc 0 because an EARLIER map step tombstoned that id (the card answers the row the DB OWNS) — the gone-upstream pair `item 127555` / `--cached` still answers 4/4 |
| the user's OWN file, read by the new binary (opens + LOCAL doors, ZERO requests, NO crawl) | two `status` opens rc 0/0, status **BYTE-IDENTICAL**; tallies intact: schema 8, **19 258** rows / **4 231** `vendor_sell` / **4 287** samples, `recipe` + `recipe__crafted` present with the **1 240** rows the v0.10.0 era's recorded real-host witness wrote (nothing re-fetched here); the fix's own doors over the REAL data: `search sword --offset 3` answers the REST — **220 hits of `total 223`** rc 0 — `search shortsword --offset 26 --limit 5` 2 rows of total 28, `recipes --offset 1238` 2 rows of `matching 1 240`, `recipes --offset 1240` an **EMPTY set rc 0** (never a 4), `recipes --profession Enchanting --offset 150` 2 rows of `matching 152` |
| the pinned pair over the REAL file (the v0.10.0 binary at the tagged tree `e4d3447` vs the new binary, read-only doors, ZERO requests) | **8 doors BYTE-IDENTICAL, md5-equal both sides**: `status` 1 098 B · `stats --icons --limit 6` 37 648 B · `item 25 --cached` 2 950 B · `search sword --limit 2 --offset 1` 979 B · `search sword --limit 4 --offset 10` 1 729 B · `recipes --limit 40` 81 428 B · `recipes --limit 4 --offset 1236` 4 802 B · `watch list` 707 B — the fix moves NOTHING that does not page past an OFFSET without a LIMIT, on the user's own 19 258-row file |
| the fixture-pair pinned audit cited from the chain (the v0.10.0 binary vs the new binary, ONE shared fresh fixture, TWO fresh DBs) | **22 DB-only doors BYTE-IDENTICAL**, the two paging doors show exactly the rc delta **4 → 0**, the working pair md5-equal both sides (1 629 B / 182 B / 185 B), and BOTH DBs answer the same tallies (schema 8 / 5 items / 4 priced / 12 recipe rows — no schema delta behind the fix) |
| limits kept | no schema move (stays 8, both scratch DBs and the user's file answer 8), no UI line moved (nothing new to parse — the webview stayed untouched, so the rendered look is never claimed), the real host saw nothing but the deliberately recorded witnesses of earlier eras, `tests/__pycache__/` out of every commit, every fixture/scratch server killed after its run |

**45044360477** (this release's chain+tag push `c94f6b0..09c8d77`, head `09c8d77…` = the TAGGED
tree): **success, 5/5 in 9 min 45 s** (16:31:58Z → 16:41:43Z — quality 2 m 27 s · linux 6 m 38 s ·
test 7 m 18 s · macos 7 m 20 s · windows 9 m 42 s), exactly ONE run id for that head, no CANCELLED
sibling, no rerun. The fold's own push starts the NEXT run, measured in the NEXT record — the
ladder never chases a tag, tags stay put (`v0.10.0` → `e4d3447`, `v0.10.1` → `09c8d77`).

## 2026-11-22 · v0.10.0 — the recipe doors: every job with every recipe, the second source fetched, read locally ever after

### What forced it

The user's third ask ("pull every job with every recipe … a new tab … Icons, descriptions,
items needed"), plus the question underneath it: "search for APIs that give this for wow
forever". Measured, that question has a SPLIT answer, and the split is what shaped the
release (all facts below measured the SAME day, docs/API.md carries them):

* TheWoWDB has **no professions door**: `/api/v1/forever/{professions,profession,jobs,skills,
tradeskills}` all answer `404`; its developers page still names exactly FOUR doors.
* Its recipe CARDS exist (716 rows of `item_type "Recipe"` in the mirror's own copy, 699 of
  716 tooltip copies say "Teaches you how to…") but a card names NO profession, NO skill
  number, NO materials — **0** copies anywhere contain "Requires Materials" — and the
  `?q=Recipe` search door answers NAMES only (259 ids, undercounting the 716): the names-only
  trap again, and names cannot fill a tab.
* Blizzard's own Game Data API answers `404` TODAY (`data/wow/recipe/2424` with the exact
  namespace a 2025 bug report names, `static-11.1.5_60179-us`; the 11.2.0 guesses also 404).
  Recorded as a dead end: inventing its shape would invent upstream.
* ONE measured carrier states job + skill + exact reagents: **ReagentRoute** (MIT, text
  measured at the repo), plain files at `raw.githubusercontent.com/Nyte-fyre/ReagentRoute/main/
  data/<profession>_recipes.json` — EIGHT classic files (NO Jewelcrafting: JC rides the set's
  `*_tbc_recipes.json`, another era), 2 843 484 B fetched whole, **1 240 entries**,
  `profession` per file, 16 keys on ALL entries (`acquisition_note` on 745), `source` values
  `cmangos-classic-db` / `cmangos-tbc-db+wago-tools-db2`. The id spaces MEET the DB (classic
  ids: `spell_id 2339` ↔ `crafted_item_id 2455` Minor Mana Potion). Join measured against the
  user's OWN file the same day: 1 105 distinct crafted ids → **1 095 have an item row, 10 do
  not**; **133 entries carry NO crafted id at all, all 133 Enchanting** (an enchant is a slot
  effect, not an item in a bag); 416 distinct reagent ids → **415 have a row, 1 does not**,
  **392 already answer a `price_min`**.
* The other candidate carrier was rejected on measured grounds: Napalmsteak's WoW-Classic-Item-
  Caches answers `404` at its LICENSE path (no license → not a carrier), and its
  `Forever Cache.json` keys are `sourceKeys/sourceLabels/sourceTypes` — acquisition words,
  not reagents.

### The decision (full text in ADR 0021)

Three parts, one per truth. **Fetch stage:** `sync --stage recipes` fetches the eight files
(one request each, conditional GETs, `meta` key `recipe_etag_<file>`, resume cursor = files
done, `--budget` bounds FILES) — the dataset is NOT vendored (2.6 MB of an external set in the
repo would be a second, drifting copy the day the source moves; the docs carry the counts).
**Local door:** `recipes [--profession] [--limit] [--offset] [--icons]` reads the new `recipe`
table (schema **7 → 8**, `SQL_MIGRATE_TO_V8` = CREATE + index, ladder case `7 =>`) and joins
BY VALUE through the item rows this DB already owns — ONE read, ZERO requests (ADR 0012's
local-door rule restated), empty table `not_found` rc 4 (ADR 0004). **Eighth tab:** job chip,
recipe name, skill + tier band, one material cell per reagent (DB's OWN icon bytes, `money_cell`
hover-copper, state chips for what the DB has no row for), the cooked item through its own card;
the Sync tab grows `Recipes: fetch the set` on the SAME `runSync(stage)` loop, deliberately NOT
chained into pull-all, and `Watch the cooked items` batches one `watch add` over the cooked ids
of the shown rows, skipping the two un-watchable kinds rather than inventing them.

The honesty of the tab is the point: the words come from the FETCHED set, the icons/prices come
THROUGH the item rows this DB owns, and the two ride SIDE BY SIDE — an unjoined reagent keeps
the set's words + "not in your DB", an enchanting craft says it has NO crafted item, and the
tab's note names which source owns which word.

### Bends and defects found on the way (recorded, not swept)

* **A money word without its number was a bug the compiler caught**: `RecipeRow.crafted_price_
copper` existed in the struct but the builder had no start and the join site filled only the
words — E0063 fired before anything could ship. `fix(db) be9e934` lands the start (`None`) and
the join's clone of `live.price_min` beside the words.
* **A targeted ask on a whole-door stage did nothing the report said**: measured on the REAL
  host BEFORE the fix, `sync --stage recipes --ids 25` answered rc 0 with the WHOLE 8-file pull
  (1 240 entries written — the real set's own number). `fix(cli) 24e02bc` makes `--ids`/`--watch`
  on this stage answer 2 (ADR 0004's contract: a wrong ask answers 2, it never quietly does
  something else), and the refusal sits before `open_at`/`build_client` so it spends ZERO
  requests (measured on a scratch DB — the fixture saw nothing).
* **`feat(sync)`'s `Reagent`/`RecipeEntry` typed with only the keys the door reads**, serde
  ignoring the set's other measured keys — v0.8.0's "cannot break a build" promise keeps its
  teeth for a second source.
* **The stage's own first REAL-host run wrote 1 240 rows in 8 requests (rc 0)** — the tool's
  witness of the real set, recorded in ADR 0021; CI still runs the fixture's tiny rows (ADR 0011).

### Measured (pre-tag)

* eight files fetched live today: sizes 241 981 · 638 864 · 82 407 · 348 098 · 296 531 · 26 372
· 518 352 · 690 879 B; per-file `recipe_count` matching the array 113 · 250 · 81 · 152 · 161 ·
15 · 232 · 236 = 1 240 entries; join facts above against the user's own file the same day;
* the hop on a schema-7 scratch copy born by the PINNED v0.9.9 binary (list/detail/tooltip
stages): `7 → 8` additive, 5 item rows / 4 priced intact, `recipe` table + `recipe__crafted`
index land, 0 recipe rows (the stage never ran), a SECOND open re-migrates NOTHING, and the
pinned v0.9.9 binary refuses the 8-file `database on disk uses schema 8, this build wants 7`
rc 4, file untouched;
* the stage on a fresh mirror (fresh fixture process, fresh DB, driver generated FROM the
committed 41-step block, 44/44 step-lines rc 0): press 1 budget 3 → 3 files / 5 rows / 4
requests / `rate_limited 1` / `complete false`; press 2 → `resumed_from 3`, 5 files / 7 rows /
5 requests / `complete true`; press 3 → 8 files / **8 answered the same copy** / 0 rows / 8
requests (the reuse discipline kept for the second host);
* the door (ZERO requests): `matching 12` with 12 rows, money beside every copper answer (8
pairs `19603 ↔ 1g 96s 3c`), 11 reagent slots riding DB rows (3 `price_copper: null` → money
null → "no price"), 8 slots `in_db: false` + "not in your DB", 12 crafted slots → 10 ride DB
rows (`Fixture Detail 25/40/60/12809`, `Conjured Cocoa Choco-Frogo`) and 2 answer `crafted_item_id:
null` → "no crafted item — enchanting craft"; `--profession Alchemy` matching 2, `--profession
Enchanting --icons` showing the mirror's OWN copied icon (2 596 base64 chars + `icon_at`) riding
BESIDE the null-crafted rows, `--profession Cooking` matching 1 through the 127555 join;
* the two states CI CANNOT run green, measured on scratch instead (ADR 0011's split): `recipes`
on an empty table → rc 4 BOTH with and without a filter, naming the stage that would run; the
404-FILE abort on a scratch audit server answering one file 404 → rc 3 naming the file ("the
second source moved"), no summary line printed;
* `node --check` parses the grown `app.js` (PARSE witness only), grep witnesses: 8th tab button
+ `data-tab="recipes"` + 3 button ids + 3 wirings, and the CSS names the EXACT classes app.js
draws (`jobchip skillchip tierchip mat matname mat-none mat-not-in-db cooked cooked-none
cooked-not-in-db`) — the rendered tab stays never-witnessed (no display, the launch-audit rule);
* builds: exit 0, FOUR named warnings (`PendingSql` privacy, unread `Tooltip` fields, `open_db`,
`catalogue`) every commit, the incremental-clear line a FS artifact not a fifth warning.

### Chain

`af864b4` feat(db) (schema 8 fresh + hop 7→8 + ladder case) → `615c561` feat(db) (writer, by-value
read, typed `Reagent`) → `8061749` feat(sync) (the stage) → `656c116` feat(cli) (the verb +
`--profession` + stage whitelist) → `be9e934` fix(db) (the copper twin) → `40e97c7` feat(ui)
(the eighth tab) → `24e02bc` fix(cli) (the whole-door refusal) → `fbb2721` test(fixture) (the
tiny `/data/` grounds, 12 entries over 8 files) → `c9efa43` ci (41 step-lines) → `c0de045`
chore (banner 0.10.0) → this record carries the annotated tag `v0.10.0`.

### Post-tag verification — MEASURED on fresh ports: 44/44 green, byte pairs identical, the map grown

All FRESH ports and FRESH DBs, every number read off the run that produced it; the real host
saw NOTHING (every crawl door carried `--fixture`, every bare-verb door rode a DEAD port).

| probe | answer |
| ----- | ------ |
| mirror replay — driver generated FROM the GROWN committed block (port-stripped compare EQUAL **41/41** step-lines + plant one), fresh fixture PROCESS on the block's own 8765, fresh DB `post10/fixture.sqlite` born 8 | **44/44 commands rc 0, ZERO NONZERO** (`TOTAL_STEPS=44` = 41 step-lines + plant + fixture start + sleep) |
| the stage's three presses, post-tag, fresh DB | budget-3: **4 requests / 3 of 8 files / 5 entries / 5 rows / `rate_limited 1` / `complete false`** · resumed: **`resumed_from 3` / 5 requests / 5 files / 7 rows / `complete true`** · re-pull: **8 requests / 8 files / 8 answered the same copy / 0 rows written** — the reuse discipline held for the SECOND host |
| the local door over the fixture's 12 rows, post-tag | `matching 12` · money pair **`17371 ↔ 1g 73s 71c`** beside 11 reagent slots · 8 `in_db:false` + "not in your DB" · 2 `crafted_item_id:null` + "no crafted item — enchanting craft" · the crafted id whose detail 404ed answers price `null` → "no price" · `--profession` answers 2/2/1 · the `--icons` press rides the DB's OWN bytes: b64 **2 596 chars**, stamp `1795362824` |
| pinned pre-hop byte pairs — the PINNED v0.9.9 binary (worktree at `b2cec23`, born 7) vs the new binary, ONE shared fresh fixture (9601), TWO DBs | **18 DB-only doors BYTE-IDENTICAL**; the 4 recipes steps: pinned **rc 2** (unknown verb — the documented new-door delta) vs new **rc 0**; pinned DB born 7 = 5 items/4 priced/**no** recipe table, new DB born 8 = 5/4/**12** recipe rows |
| the user's OWN file by the new binary (opens only, NO crawl) | before: schema 7 / **19 258** rows / **4 231** `vendor_sell` / **4 287** samples / no recipe table → open1 **rc 0** → schema 8 + `recipe` + `recipe__crafted`, **0** recipe rows → open2 **rc 0**, status BYTE-IDENTICAL; the PINNED v0.9.9 binary on that same 8-file answers **rc 4** `wants 7`, file untouched AT 8 |
| exit map GROWN — fresh fixture 9612 + fresh EMPTY DB + DEAD 9997 + the scratch 404-file server 9557 | **86 doors: `0`×55 · `2`×17 · `3`×6 · `4`×8**; six measured deviations named below (this era's map is GROWN, so its totals are stated as THIS era's — the prior era's 101-door identity stands on the prior folds) |
| extension drivers — abort where pending EXISTS, fresh DB `ext10.sqlite`, fixture 9613, DEAD 9999 (the STRONG proof the drained map pass could not give) | **12 doors: `0`×7 · `3`×5** — detail abort rc 3, pending **5→5** identical, stamps ZERO → live re-press rc 0 (6 requests / 4 written / 1 checked removed); icons rc 3, pending **4→4** → re-press 0 (5 requests / 3 copied / 1 no-icon); tooltip rc 3, pending **4→4** → re-press 0 (5 / 3 copies / 1 no-tooltip); price rc 3, pending **3→3** → re-press 0 (3 requests / 2 written / 1 price-less, `rate_limited 0`); the SECOND host: budget-2 partial `2 of 8 files, 4 rows, complete false` → dead abort **rc 3 with etag rows 2→2 and `sync_state` IDENTICAL — resumable, nothing marked** → re-press RESUMES `resumed_from 2`: 6 requests / 6 files / 8 entries / 8 rows / `complete true` |
| the 404-FILE abort, fresh scratch server 9556 (one file 404 — "the second source moved") | **rc 3**, names `cooking_recipes.json`, NO summary line printed; the map's two abort doors rode the fresh 9557 server and answer 3/3 |
| the map's four DEAD-port crawl doors (detail/icons/tooltip/price) answered **rc 0** | recorded AS THE WEAK PASS IT IS: pending was ALREADY EMPTY after the live presses (the same shape v0.9.6/v0.9.7/v0.9.9 eras recorded) — the strong proof is the line above |
| `search` bare (no TEXT) answered rc 0 | contract-FREE, measured not assumed: `search` is the LOCAL door, the empty term reads the DB's own LIKE — no request spent, nothing invented |

**A defect the post-tag map found, recorded not swept (fix lands FORWARD, next release):**
`recipes --offset 10` WITHOUT `--limit` answers **rc 4** with the raw clause in its message —
`near "OFFSET": syntax error … FROM recipe ORDER BY … OFFSET ?` — SQLite's `OFFSET` requires
`LIMIT` and `recipes_rows` emits the clauses independently. Its **twin already existed at
birth**: `search sword --offset 3` answers the SAME rc 4 with the same raw clause (born with
v0.9.0's search door, never measured until today). The fix FORWARD composes `LIMIT ? OFFSET ?`
with the door's DEFAULT limit when `--offset` rides alone — BOTH doors fixed in one place, and
the raw-SQL message never belongs in a door's answer either (ADR 0004's message rule).

**45040598595** (this release's chain+tag push `b2cec23..e4d3447`, head `e4d3447…` = the TAGGED
tree): **success, 5/5 in 10 min 32 s** (15:51:29Z → 16:02:01Z — quality 2 m 12 s · test 5 m 27 s ·
linux 5 m 26 s · macos 5 m 42 s · windows 10 m 29 s), exactly ONE run id for that head, no
CANCELLED sibling, no rerun. The fold's own push starts a run that lands FORWARD in the NEXT
record — the ladder's rows stay put, tags never move (`v0.10.0` → `e4d3447`).

Housekeeping: every fixture and scratch server killed after its run; the ports re-verified free;
`tests/__pycache__/` stays out of every commit (the .gitignore gap honored by explicit paths).

Limits kept: the rendered tab is never claimed (no display — `node --check` = PARSE witness), the
content embedding is compressed (filename map), the real host saw nothing but the two deliberately
recorded real-host witnesses of the pre-tag era (ADR 0011's split).

## 2026-11-21 · v0.9.9 — the live-sort knob: order what the AH offers NOW, the shown cell stays the stored min

### What forced it

The user asked for it right after the stage buttons ("a then b"): the hunt list is ordered by
what the DB once STORED (`price_min`), while ADR 0017's price stage now keeps a LIVE minimum
per copied record (`price_min_now`) — what the AH offers NOW. "Cheapest first" by the stored
column answers with the crawl-era order; the knob orders by the live pair the button refreshed.

### The decision (full text in ADR 0020)

ONE new sort key, `price-now`: query.rs `NOW_ORDER` = `ORDER BY (price_min_now IS NULL) ASC,
price_min_now ASC, name ASC` (the SAME NULL-last pattern every other price sort uses; the tail is
BOTH kinds of NULL — never refreshed AND checked price-less at ask time, their stamps keep them
apart, the sort does not care and does not guess; name tiebreak = stable across runs) + the
`order_for` branch + cli.rs usage string; UI grows ONE select option whose value rides the EXISTING
`--sort` argv. The note grows one conditional sentence ONLY when the knob is on — naming which
column the ORDER reads (live) vs which the CELL shows (stored), and that the NULL tail sorts last
— because the live order can make the column LOOK out of order (measured: 60 live 16 106 before
25 live 17 106, both stored 18 106). Deferred, said so: the live price IN the cell (wire shape
change + its own money pair — next release's honest question); `--priced` stays the stored-min
filter (the knob is an ORDER). NO schema hop: `price_min_now` is already a schema-7 column.

### Chain

`b8abbd8` feat(search) (NOW_ORDER + branch + usage) -> `536ea1d` feat(ui) (option + conditional
sentence) -> `b700a57` ci (34th step, both-binaries shape quoted from measurement) -> `8ed805d`
chore (banner 0.9.9, lock rides) -> this record carries the tag.

### Pre-tag verification — the witnesses (measured, not assumed)

| probe | answer |
| ----- | ------ |
| knob on the mirror DB | live order answers 60 BEFORE 25 (16 106 < 17 106) while stored `price_min` TIES at 18 106; the four NULL-live rows tail by name (CI Test Mug, Conjured Cocoa Choco-Frogo, Fixture Detail 12809, Fixture Detail 40) — matching sqlite's own rendering of the same clause exactly |
| v0.9.8 binary (`wgt-v98-pre-sort`, saved pre-release) vs v0.9.9 | default-order doors BYTE-IDENTICAL: `search sword --where description --sort price --icons --limit 5` 3 359 B pair + `search detail --where all --limit 8` md5-equal — the knob rides ONLY the new key; on the OLD binary `price-now` falls back to default (unknown key — the door's documented rule), hits portion IDENTICAL |
| bogus sort key | hits IDENTICAL vs default (fallback behavior unchanged since before this release) |
| CI block | 34 step-lines + plant one (was 33); the grown step's paragraph quotes the BOTH-BINARIES measured shape — green BECAUSE the knob works; yaml parses; ZERO requests added, GET tallies cannot move |
| UI | `node --check` parses (PARSE witness; no display — the rendered panel stays never-witnessed, ADR 0011/0018's line); grep `price-now` x1 option + x1 clause |
| schema | stays 7 (grep `SCHEMA_VERSION: i64 = 7` — NO hop: the column was born at 6 with ADR 0016, this release READS it) |
| builds | exit 0, FOUR named warnings every commit (incremental-clear mechanic every build) |

### Post-tag verification — MEASURED on fresh ports: 34 steps green, wire pairs byte-for-byte

| probe | answer |
| ----- | ------ |
| mirror replay — driver generated FROM the GROWN committed block (port-stripped compare EQUAL **34/34** + plant one), fresh fixture port 9241 (bind-verified), fresh DB mirror099a.sqlite born 7 | **34/34 rc 0, plant rc 0** — the grown `--sort price-now` step itself rc 0 inside the replay (its live order printed in the log lines); GET tallies **34 = 200x18 / 304x9 / 404x3 / 429x4** — IDENTICAL to the v0.9.8 era: the new step reads the DB, adds ZERO requests, the tallies cannot move; price press THIS process: live 18 460/17 460 vs stored 19 460 (fixture RUN), row 40 stamped 1795315035 with COUNTED missing 1; trio NULL |
| wire byte pairs — v0.9.8 binary (`wgt-v98-pre-sort`, saved pre-release) vs v0.9.9, SAME mirror099a DB | `item 25 --cached` **BYTE-IDENTICAL 4 394 B**, `stats --icons --limit 6` **17 473 B**, default-sort `search sword --where description --sort price --icons --limit 5` **3 361 B** — the knob moved ZERO bytes on the OTHER doors |
| live door shape, post-tag, same DB, BOTH binaries | NEW: 60, 25, 12809, 40 (live 17 460 < 18 460 then NULL tail by name, sort echo `price-now`); OLD binary on the same ask: 12809, 25, 40, 60 — the DEFAULT fallback (unknown key, its documented rule) — the knob is exactly the delta |
| exit map — 90-door driver remapped to fresh 9242-9250 + fresh DBs (exit099p), v0.9.9 binary | **90/90 rc pairs IDENTICAL**: 0x59 / 2x14 / 3x6 / 4x11 held |
| extension doors — both drivers remapped to fresh 9251/9252 + DEAD 9497/9498 | ext1 0x6/2x2 (press 3 reqs / 2 written / missing 1; drained dead-port pass recorded AGAIN as weak); ext2 abort where pending EXISTS: rc **3**, pending 25/40/60 identical AFTER, stamps ZERO at that moment, 3 only after the re-press; list-only DB 0 requests complete true. **101-door totals 0x67 / 2x16 / 3x7 / 4x11 — identical for the FOURTH era in a row** |
| pinned pair on the fresh 7-file | v0.9.5 rc 4 "wants 6", v0.8.1 rc 4 "wants 4", files untouched |
| the user's own file by the v0.9.9 binary (two opens) | schema 7, 19 258 / 4 231 / 4 287 samples, AVG 54 366.05243532976 — **BYTE-IDENTICAL to the v0.9.8-era snapshot**: NO NEW drift this era (the earlier +1 sample is fully explained in THAT era's row); five price-home/stamp columns still NULL on every row (0 with any value); second status BYTE-IDENTICAL |
| limits kept | rendered look never claimed (no display), content embedding compressed (filename map), node --check = PARSE witness — same honest line as v0.9.8's era |
| housekeeping | all fixture servers killed after the runs; ports 9241-9252 + 9497/9498 re-verified FREE (survivor check run separately — the self-match lesson honored) |

**44976892186** (this release's chain+tag push `c4fac31..53f9221`, head `53f9221…` = the TAGGED tree): **success, 5/5 in 9 min 39 s** (02:36:38Z -> 02:46:17Z — quality 1 m 49 s · test 4 m 36 s · linux 4 m 25 s · macos 6 m 22 s · windows 9 m 37 s), exactly ONE run id for that head, no CANCELLED sibling, no rerun (matched by id/head_sha). The fold waited LOCAL while it flew and GREW this verdict by amend before pushing — only the fold's OWN push run lands FORWARD in the next record. (Honest probe note: the wait loop's first line issued one invalid run-id API call that answered 404 — harness noise, not a run; the ladder above lost nothing and chased nothing.)

### CI ladder (folded as measured)

**44976027751** (v0.9.8's verification-fold push, head `c4fac31…`): **success, 5/5 in 6 min 21 s**
(02:25:34Z -> 02:31:55Z — quality 2 m 10 s · test 5 m 01 s · linux 5 m 36 s · macos 5 m 44 s ·
windows 6 m 18 s), exactly ONE run id for that head, no CANCELLED sibling, no rerun — it landed
WHILE this chain built (nothing was in flight at v0.9.9's first commit; the ordering fact recorded
where it stands). Earlier run folded earlier (in v0.9.8's fold): 44975077893. v0.9.9's own
chain+tag push starts a run that lands FORWARD in this release's verification fold, never chased.
Tags stay put (`v0.9.8` -> `9aaa3f6`, `v0.9.9` -> this record's commit).

## 2026-11-21 · v0.9.8 — the Sync tab grows its stage buttons: icons / descriptions / price refresh, same argv, one honest note each

### What forced it

The v0.9.6 and v0.9.7 records both kept the crawl buttons at FIVE (list / pull-all / details:watch /
details:hits / Cancel) and said, honestly, that icons/tooltip/price stayed CLI-side — "said so, not
slipped in". The user asked, one milestone after the other, to undo exactly that scoping: the panel
should have doors for the stages it already prints notes about. The engine needs NOTHING new —
`sync --stage icons|tooltip|price` have been the CLI's own verbs since v0.6.0/v0.8.0/v0.9.6; only the
panel lacked buttons.

### The decision (full text in ADR 0019)

Three buttons in the Sync tab's action row — `Icons: copy the files` / `Descriptions: copy` /
`Price: live refresh` — ALL THREE wired to the SAME `runSync(stage)` loop the panel already had: same
chunk of 25, same cancel-at-chunk-end, same `sync_state` resume, same no-progress guard, same
`cli_ok(["sync","--stage",X,"--budget",chunk])` argv — ADR 0014's rule restated: a click and a cargo
line are ONE code path down to the report JSON. The Price button is a REFRESH, deliberately NOT chained
into Pull-all-info: the chain's contract is "the catalogue is complete" (list→detail→icons→tooltip),
the refresh re-asks a LIVE pair under ADR 0017's once-per-copied-record-EVER rule, and chaining it would
make a second pull-all ask zero and say nothing new — recorded where the chain's note lives, not
silently added.

Notes moved the SAME breath so none can lie beside the new buttons: the card's static note names the
door ("the Sync tab's Price button or the CLI stage", one live ask per copied record ever), its API-touch
claim gets its honest card-scope ("on the card itself 'Re-check online' is the only button…"); the
card's dynamic sentence names the door that exists now; the live-row comment replaces the v0.9.6
"the panel has no button for this stage" claim with what is TRUE SINCE this release — the gui.rs facts
stay (the Rust shell builds no card, reads no column; the button is webview-side).

The completion note grows ONE clause ONLY when the report carries `missing > 0`:
", N answered the stage's own third state (checked once, never asked again)" — generic words on
purpose ("the stage's OWN third state", ADR 0006's pattern): detail = gone upstream, icons/tooltip =
file/copy 404, price = the AH offers nothing right now. No stage's meaning guessed; reports without the
field keep their note BYTE-IDENTICAL — no missing-0 noise invented.

### Chain

`058d21b` feat(ui) (buttons + the four honest note moves + the generic clause) -> `d60bc84` chore
(banner 0.9.8, Cargo.lock rides) -> this record carries the annotated tag. Zero Rust lines across the
chain (diff names only ui/index.html + ui/app.js + Cargo.toml/lock).

### Pre-tag verification — the witnesses this machine HAS (ADR 0011's rule for a webview)

| probe | answer |
| ----- | ------ |
| builds through the chain | exit 0 every commit, the FOUR named warnings (incremental-clear mechanic run every time this release) |
| `node --check` app.js | parses (a PARSE witness — the webview is NOT claimed launchable here: no display, recorded as such) |
| grep witnesses | 3 button ids (`btn-icons`/`btn-tooltip`/`btn-price`) in index.html, 3 wirings in app.js, the third-state clause x1; the old "panel has no button" comment claims are GONE (replaced same breath) |
| `git diff --stat` across the chain | ui/index.html + ui/app.js + Cargo.toml + Cargo.lock — ZERO .rs; SCHEMA_VERSION stays 7; fixture/CI block untouched (still 33 step-lines + plant one: the block ALREADY runs the very stage verbs the buttons reach) |
| CI ladder (folded as measured) | **44973516964** (v0.9.7's verification-fold push, head `0236479…`): **success, 5/5 in 10 min 58 s** (01:53:08Z -> 02:04:06Z — quality 2 m 09 s · test 5 m 19 s · linux 6 m 28 s · macos 5 m 31 s · windows 10 m 55 s), exactly ONE run id for that head, no CANCELLED sibling, no rerun (matched by id/head_sha; the run's own .attempt reads null — the ladder's habit). It was IN FLIGHT while this chain built (the observed rule: pushes waited, local building did not); its verdict folds HERE, FORWARD, this record. v0.9.8's own chain+tag push starts a run that lands FORWARD in the next record, never chased — same for this release's verification fold |

### Post-tag verification — replays on FRESH ports, real numbers (the verification fold)

### Post-tag verification — MEASURED: the buttons moved ZERO bytes on the wire

_(the fold grew these numbers off the replays that produced them — fresh bind-verified ports, fresh
scratch DBs, the v0.9.8 binary that carries the buttons; nothing here is hoped, every row names its
own port/DB/driver. The fold waits LOCAL while this release's chain+tag push run flies, GREWS with
that waited verdict before pushing, and only its OWN push lands FORWARD — the discipline since the
v0.9.6 era.)_

| probe | answer |
| ----- | ------ |
| mirror replay — python driver generated FROM the committed block (port-stripped compare EQUAL 33/33), fresh fixture port 9229 (bind-verified free), fresh DB mirror098a.sqlite born 7 | **33/33 rc 0, plant rc 0**; GET tallies **34 = 200x18 / 304x9 / 404x3 / 429x4** — IDENTICAL to the v0.9.6/v0.9.7 era (block untouched); price press on THIS process: live 17 106/16 106 vs stored 18 106 (the fixture's own RUN moves numbers per process — recorded since the v0.9.6 era), row 40 stamped 1795313703 price-less-at-ask with the COUNTED missing 1; `vendor_buy`/`price_trend_normal` NULL on every row; schema meta 7 at birth |
| wire byte pairs — v0.9.7 binary `/tmp/opencode/wgt-v97-pre-panels` (saved pre-release) vs the v0.9.8 binary, SAME DB mirror098a, same verbs | `item 25 --cached` **BYTE-IDENTICAL at 4 389 bytes** (live pair still rides it: 17 106 beside `1g 71s 6c`), `stats --icons --limit 6` **BYTE-IDENTICAL at 17 461**, `search sword --where description --sort price --icons --limit 5` **BYTE-IDENTICAL at 3 359** — the buttons moved ZERO bytes on the wire, proven not argued. Cross-process the pair SIZES track the fixture RUN (the v0.9.7-era pair was 4 394/17 473/3 361 — same shape, different numbers: honest restatement where sizes are quoted) |
| exit-code map — the 90-door driver REMAPPED to fresh ports 9230-9238 + fresh DBs (exit098p...), v0.9.8 binary | **90/90 rc pairs IDENTICAL**: 0x59 / 2x14 / 3x6 / 4x11 held |
| extension doors — both drivers REMAPPED to fresh fixture ports 9239/9240 + DEAD ports 9397/9398 (upstream sees nothing) | ext1: 8 doors 0x6 / 2x2 (price press 3 requests / 2 written / missing 1 counted; the drained dead-port ask rc 0 recorded AGAIN as the weak pass it is — pending already empty). ext2: abort RE-MEASURED where pending EXISTS (dead 9398, pending 25/40/60): rc **3**, pending AFTER identical, stamps ZERO at that moment, THREE only after the re-press; bogus 2s; list-only DB rc 0 / 0 requests / complete true (5 rows / 0 copies). **Totals over 101 doors: 0x67 / 2x16 / 3x7 / 4x11 — identical to the v0.9.7 AND v0.9.6 era maps** |
| pinned refusal pair on the fresh 7-file | pinned v0.9.5 (`wgt-v95-prehop`): rc **4**, "database on disk uses schema 7, this build wants 6", file stays 7; pinned v0.8.1: rc **4**, "wants 4", untouched |
| the user's own file by the v0.9.8 binary (two opens) | schema 7 · 19 258 rows / 4 231 vendor_sell / **4 287 samples**, AVG 54 366.052435329765 — and the DRIFT recorded honestly: +1 sample vs both era snapshots (seq 4287, item 10940 = Strange Dust, 20c min / 150c market, born 1795312676 ≈ 19 min before the read): NOT my doing — every audit run points WGT_DB at scratch DBs (nothing opened the user's file; `status` sends no request and writes no sample), so the user's own app run re-sampled the details chart. Everything ELSE byte-identical to the era snapshots; the five price-home/stamp columns stay NULL on ALL rows (0 rows with any value); second status BYTE-IDENTICAL. The file is the user's: their run moved it, this zero-Rust release cannot and did not |
| limits recorded, not claimed | the window's rendered look (no display — the buttons are GREP + PARSE witnesses only), content embedding compressed (filename map x14/x1), webview runtime never run (`node --check` = PARSE witness) — ADR 0018's honesty line transfers to the buttons UNCHANGED |

**44975077893** (this release's chain+tag push `0236479..9aaa3f6`, head `9aaa3f6…` = the TAGGED tree): **success, 5/5 in 10 min 45 s** (02:13:34Z -> 02:24:19Z — quality 2 m 03 s · test 5 m 03 s · linux 5 m 25 s · macos 4 m 54 s · windows 10 m 42 s), EXACTLY ONE run id for that head, no CANCELLED sibling, no rerun (matched by id/head_sha; .attempt reads null — the ladder's habit). The fold waited LOCAL while this run flew (the observed rule), GREW this verdict by amend, and pushed — only the fold's OWN push run lands FORWARD in the next record. Every recorded push through this release's chain+tag is green measured: 44973516964 · 44972713823 · 44975077893 (and the era's earlier, straight or resubmitted).

### Bends, recorded where they stand

* none new so far this release: every build ran the incremental-clear mechanic (FOUR named warnings
  the whole way); the note-move rewrites are the SAME BREATH rule holding, not a bend;
* the fold stays LOCAL while this release's chain+tag push run flies, waits, grows with its verdict
  (amend) and only the fold's OWN push lands FORWARD — the pattern recorded since the v0.9.6 era.

## 2026-11-21 · v0.9.7 — the AH frame and the tabs: the shape the user asked for, the same verbs under it

### What forced it — the user's own words

"I want this app to look like the blizzard default wow auction house frame. Multiple tabs for
different activities instead of cramming everything into one tab." Today the panel IS crammed:
EIGHT cards stacked in one page (status · search · sync · item card · the DB's own writes · twin
refresh · live look · analytics) with a two-column >=980 px grid for three of them, and a flat
round-cornered dark-card skin the AH silhouette never had. The ask is a LOOK plus a STRUCTURE:
borrow the AH's shape, split the activities.

### The decision (full text in ADR 0018)

SEVEN activity tabs on a nameplate strip: Overview · Search · Sync · Item card · Writes · Live look
· Analytics — eight cards map to seven because the twin-refresh button JOINS the Writes tab (the
four DB verbs and the twin refresh are ONE activity: both send ZERO requests, ADR 0013/0010).
`show_tab` in app.js is the ONE visibility truth (one section visible, the rest `tab-off`
display:none — hidden buttons get no events, the layout enforces the rule); the strip rides the
delegated-handler pattern the hits table already used; index.html PAINTS the default overview
statically so first paint never waits on script (boot re-asserts — recorded, not leaned on). The
frame is CSS-ONLY: bronze trim + gold inner hairline + corner ORNAMENTS as two border Ls
(main::before/::after), nameplate tabs (uppercase letter-spaced, lit tab gold, gold underline strip),
parchment-gold headers, deep blue-black room. Rejected and said so: Blizzard ART (their property —
only the silhouette is borrowed), raster/generated assets and webfonts (offline rule), tab state in
Rust (the engine never learns the window exists, ADR 0002/0014), moving the watch toggle off the hit
rows (marking IS the search activity), growing panel stage-buttons this release (the verbs stay the
five the v0.9.6 record scoped; stage buttons are future work, not slipped in).

### Chain

`35342b9` feat(ui) (the page splits: nav strip, data-tab on all 8 sections, static paint, show_tab/
tab_click/handler/boot re-assert) -> `24971fe` fix(ui) (the live note's dangling fragment, below) ->
`5c7eb09` feat(ui) (the AH frame skin) -> `c6d2b18` chore (banner 0.9.7, Cargo.lock rides) -> this
record carries the tag. Zero Rust lines across the chain (measured: git diff --stat names 5 files).

### A product defect found reading, fixed FORWARD, measured both ways

The live card's success note ended with `+ hits_link` - an identifier with NO definition anywhere
(grep counted ONE occurrence: the use site), so building the note ALWAYS threw a ReferenceError and
the card answered raw error text instead of the door's note. node (this FS) measures both ways: the
ORIGINAL expression throws `ReferenceError: hits_link is not defined`; after the fix the REAL line is
extracted from app.js, run through new Function, and RETURNS the note with no throw; grep now counts
hits_link ZERO times and node --check parses. The fix states the truth the fragment promised and never
delivered: every live hit row already carries its own upstream `view` link. No link invented, ONE ask
unchanged. An identifier scan surfaced only this code-level hit — recorded honestly: the scanner is
heuristic (prose apostrophes mispair), its value is the confirmed hit, NOT a proof of absence.

### Pre-tag verification — the witnesses this machine HAS (ADR 0011's rule for a webview)

| probe | answer |
| ----- | ------ |
| builds through the chain | exit 0 every commit, the FOUR named warnings; one bend recorded: the first fix(ui) build skipped `rm -rf target/debug/incremental` and the FS's spurious hard-linking cache line made that summary say "5 warnings" — FOUR named, re-measured with the mechanic the same breath |
| `node --check` app.js | parses (a PARSE witness — the webview's runtime is NOT claimed: no display here, recorded) |
| class contract old-vs-new app.css | EVERY class app.js emits still styled: ZERO dropped (the crude scan's new list = frame/tab classes + comment noise); brace-balanced; money rules UNTOUCHED (money_cell, tabular, cursor:help, hover contract), quality chips + bars keep the GAME's hues byte-for-byte |
| `git diff --stat e4c6f3f..c6d2b18` | 5 files: ui/ x3 + Cargo.toml + Cargo.lock — zero `.rs`, SCHEMA_VERSION stays 7 (grep), fixture/CI untouched (no ci commit: the block stays 33 step-lines + plant one) |
| embed witness, honest form | the binary maps the ui FILENAMES in plaintext (`index.html` x14, `app.css` x1) but tauri v2 COMPRESSES the content (greps for the NEW strings answer 0) — this CORRECTS the v0.9.6-era phrasing "re-embedded the edited files" where this record quotes it: greppable = filename map + green build; content embedding is not greppable; the RENDERED window is only observable at launch (no display — recorded, never claimed) |
| refusal/doors/exit maps | stay the v0.9.6 facts BY CONSTRUCTION (zero Rust); re-measured post-tag on FRESH ports (mirror replay 33/33 + plant, exit-map re-runs) landing FORWARD in the verification fold |

### Bends, recorded where they stand — nothing swept

* the build-mechanic count above (five LINES vs FOUR named: the FS's own cache warning line rode a
  summary that skipped the incremental-clear);
* no fixture/CI commit was planned or made (the block cannot see the webview — ADR 0011 — and
  the release proves the wire stayed via POST-tag replays instead of growing the block);
* the v0.9.6-era "re-embedded" phrasing became measurable and got corrected HERE, not retro-tagged
  anywhere (tags stay put; the era's own record keeps its words).

### Post-tag verification — replays on FRESH ports, real numbers, the wire did not move

| probe | answer |
| ----- | ------ |
| mirror replay, driver python-generated FROM the committed block (port-stripped compare EQUAL 33/33), fresh fixture port 9213 (bind-verified free), fresh DB mirror097a (born 7) | **33/33 rc 0, plant rc=0**; fixture GET tallies **34 = 200x18 / 304x9 / 404x3 / 429x4** — IDENTICAL to the v0.9.6 era's 34-tally: the block is unchanged and the wire did not move (the price press adds its own three 200s, the idempotent press nothing, exactly as before); price press live 16 645/15 645 against stored 17 645 (the fixture's own RUN 645 THIS process), row 40 stamped 1795311783 price-less with the COUNTED missing 1; trio NULL on every row |
| wire byte pairs, v0.9.6 binary (`/tmp/opencode/wgt-v96-pre-ui`, saved pre-release) vs the v0.9.7 binary, SAME mirror DB, same verbs | `item 25 --cached` **BYTE-IDENTICAL at 4 394 bytes** (live pair still riding: `16645` beside `1g 66s 45c`), `stats --icons --limit 6` **BYTE-IDENTICAL at 17 473**, `search sword --where description --sort price --icons --limit 5` **BYTE-IDENTICAL at 3 361** — the frame moved ZERO bytes on the wire, proven not argued |
| the v0.9.5-era 90-door exit map REMAPPED to fresh ports 9220-9228 + fresh DBs (exit097p...) run with the v0.9.7 binary | **90/90 rc pairs IDENTICAL**: totals `0`x59 / `2`x14 / `3`x6 / `4`x11 held — money doors 2/2/0/0, pinned trio 65/66/67 = 0/0/4 with door 66 chaining 4->5->6->7 in one open, meta 7 |
| the two extension drivers REMAPPED to fresh ports 9214/9215 + DEAD ports 9297/9298 (upstream sees NOTHING) | ext1 eight doors 0x6 / 2x2 (press on pending rc 0 written 2 + missing 1 counted; the drained dead-port ask rc 0 recorded AGAIN as the weak pass it is — pending was already empty); ext2 abort RE-MEASURED where pending EXISTS (dead 9298, pending 25/40/60): rc **3**, pending AFTER IDENTICAL, stamps ZERO — resumable, nothing marked; re-press rc 0 (requests 3, written 2, missing 1 — stamps moved to 3 only AFTER the press); list-only DB (5 rows, ZERO copies): rc 0, 0 requests, `complete true`. **Totals 101 doors: `0`x67 / `2`x16 / `3`x7 / `4`x11 — identical to v0.9.6's map** |
| pinned refusal ladder on the fresh 7-file | pinned v0.9.5 (`wgt-v95-prehop`): rc **4**, `database on disk uses schema 7, this build wants 6`, file stays 7; pinned v0.8.1: rc **4**, `wants 4`, untouched — the ladder keeps its teeth under a release that never touched Rust |
| the user's own file, first door by the v0.9.7 binary | schema 7, **19 258 rows / 4 231 vendor_sell / 4 286 samples**, `price_min_now`/`price_now_at`/`price_now_missing_at`/`vendor_buy`/`price_trend_normal` NULL on ALL (rows with any value: **0**) — second `status` BYTE-IDENTICAL, and BYTE-IDENTICAL to the v0.9.6-era audit snapshot (no drift: the file did not move since) |
| what CANNOT be witnessed, recorded as such | the window's rendered LOOK (no display on this FS — never claimed); content embedding (tauri v2 compresses: content greps 0, filename map x14/x1 is the greppable witness); the webview's runtime (node --check is a PARSE witness only) — the release's honesty line says the CONTRACT survived, not the pixels |

### CI ladder

**44970667747** (v0.9.6's CI-fold push `0c450ba..e4c6f3f`, head `e4c6f3f...`): **success, 5/5 in 7 min
30 s** (01:16:24Z -> 01:23:54Z - quality 2 m 00 s · test 4 m 42 s · linux 5 m 24 s · macos 7 m 12 s ·
windows 7 m 28 s), EXACTLY ONE run id for that head, no CANCELLED sibling, no rerun (matched by
id/head_sha; the run's own .attempt reads null on this API). **44972713823** (v0.9.7's chain+tag push
`e4c6f3f..dfb8826`, head `dfb8826...` - the TAGGED tree): **success, 5/5 in 8 min 44 s** (01:42:32Z ->
01:51:16Z - quality 2 m 19 s · test 4 m 55 s · linux 4 m 29 s · macos 4 m 42 s · windows 8 m 42 s),
exactly ONE run id for that head, no CANCELLED sibling, no rerun submitted. The verification fold waited
(staying LOCAL while that run flew - the observed rule) and GREW with this verdict before pushing -
honest: only the fold's OWN push lands FORWARD in the next record. Every recorded push through v0.9.7 is
green now, straight or after an honest resubmission of the SAME head.
Tags stay put (`v0.9.6` -> `4ced794`, `v0.9.7` -> this record's commit).

## 2026-11-21 · v0.9.6 — the live min buyout: ADR 0016's first producer, and the door that answers NOW

### What forced it — the door that was always live

ADR 0016 carved `price_min_now` and REJECTED writing it "because no door has EVER answered those
words". True of the detail door — and it stopped being true the moment the TOOLTIP door was
re-asked. Measured live 2026-11-21 with the tool's own UA: id 25 answers `fields.price` `{min
149, market 1200, available 5→7, realms 3, scope "WoW Forever Normal Alliance"}` TWICE in one day
at its own `updated_at 16:25:03Z → 22:25:10Z` — the door moves its own numbers; ids 202/237/763,
whose STORED copies carried NO price at all, answer `5000/3500/4425` — the live min moves APART
from `price_min` ON THE SAME record, which is ADR 0016's two-columns rule with evidence instead
of an assertion; and id 200 answers `fields.price` as **literal NULL** while its own copy says
"Lowest buyouts" — the door's OWN third state: "we looked, the AH offers NOTHING right now".
Upstream's OWN developers page (docs/API.md's source of truth) lists exactly FOUR doors and states
tooltips cache ~5 min "because they carry a live Auction House price". The tooltip door IS the
live door — and the crawl already owns it (ADR 0010's copy stage), so the refresh rides the SAME
door the copy came from, not the detail door at its own cache rhythm.

### The decision, and what was rejected

`sync --stage price`, a FIFTH stage mirroring every crawl discipline (full text in ADR 0017):
pending = COPY EXISTS + never written + never checked-empty, once per record EVER; the third
state marks `price_now_missing_at`; `price_now_at` is OUR ask time while the door's `updated_at`
TEXT is NOT stored (two stamps of one fact — ours says when we looked, recorded); NO
If-None-Match (a 304 would answer "unchanged since a read that NEVER saw this field" — the door's
server cache is what keeps it polite); fresh writes sample NOTHING (ADR 0007's sample stays on
the detail write — re-asking must not mint history points); the value reads through the MODEL
mirror `Tooltip.fields → Option<TooltipFields>` with ONLY `price` typed (serde ignores the other
25 measured keys — the v0.8.0 promise keeps its teeth). Rejected and recorded: re-asking the
detail door; storing the door's own timestamp; sampling on refresh; inventing producers for the
OTHER two homes — measured NEGATIVES keep them reserved: no vendor-buy key or door anywhere
(4 602 copies' 28-key set holds `vendor_sell` only, four doors upstream), the only upstream
"can be bought" signal is its own flavor prose on exactly ONE copy (id 247786), ZERO trend words
in 19 258 rows (all 20 "volume" hits are book titles). Money rides the live copper (ADR 0015),
auction COUNTS never get money, and the search/stats doors stay UNCHANGED (sort stays `price_min`).

### Chain

`9f794e4` feat(db,cli) (hop 6→7 AND the whole stage — pending rule, `SQL_SET_PRICE_NOW`/`SQL_MARK_PRICE_NOW_MISSING`
writes, `TooltipFields`, dispatch, usage: every constant has its consumer in this commit; the
bent "one logical change" rule here, SAID so: the planned separate `feat(db)` hop commit was
never made) → `55f58f8` test(fixture) (the tooltip door answers the MEASURED `fields.price`
shapes: 25/60 a full block moving with the fixture's RUN, 40 the whole block literal null — the
fixture twin of live id 200) → `d3d004a` test(fixture) (the first fix moved ONLY the comment —
the rerun died with the same `NameError`: recorded, not swept) → `5c77769` test(fixture) (the
move lands; file compiles; TOOLTIP_PAGES follows the definition) → `1a281e3` fix(sync) (the
null-price third state COUNTS — see below) → `4cfd7a2` feat(db,cli) (card door reads the pair
back + money sibling, getter+caller same commit else FOUR would break) → `b1b2c8f` feat(ui)
(the card's live money row, note reworded; gui.rs READ before the claim: the window gains
nothing) → `51121c9` ci (31 → 33 steps + plant one, replayed green BEFORE committing) →
`b920e7d` chore (banner 0.9.6, Cargo.lock rides) → this record carries the tag.

### Bends, recorded where they stand — nothing swept

* **The report lied, and the product with it** (`fix(sync)`): the first mechanism run STAMPED the
  null-price row (measured: `price_now_missing_at 1795307172` on disk) but answered `missing:0`
  and lost its note — the stamp existed, the counter did not move. Fixed the same breath,
  re-measured `missing:1` + "; 1 answered no auction price now and are checked as
  price-less-at-ask-time".
* **The fixture broke its own mechanism** (two commits, both honest): giving the tooltip door
  `fields.price` answers referencing `RUN` died at import (`NameError`) because a dict LITERAL
  evaluates where the v0.3.0 detail handler's REQUEST function never did — and the first fix
  commit moved only the comment/sentence while the rerun died with the SAME error; the second
  carried the actual move. The chain carries both, tags stayed put, nothing was ever pushed
  broken (the local mech measurement is the only witness).
* **The mirror grew a field with no reader**: the first build's FIFTH warning named `vendor_sell`
  unread in `TooltipFields` — trimmed the same breath (house rule: a mirror entry without a
  reader is dead code). Every build through the chain answers the FOUR named warnings.

### Verification, pre-tag — the hop, the stage, the reader

| probe | answer |
| ----- | ------ |
| hop 6 → 7, schema-6 scratch copy | additive ALTER pair, stamps NULL everywhere, rows/copper intact, SECOND `status` BYTE-IDENTICAL; fresh DB BORN at 7, pair BEFORE `raw_json`; pinned v0.8.1 refuses the 7-file honest (exit 4 "wants 4", file untouched) |
| stage, fresh fixture port 9204 (fresh DB mech096d) | rc 0, requests 3, written 2 (`price_min_now 16365/15365` against stored `price_min 17365` on the SAME rows — the live value differs from the stored copy's, measured), row 40 → `price_now_missing_at 1795307172`, value NULL; second press requests 0, `complete true` — once-per-record holds end to end |
| stage after `fix(sync)`, port 9205 | `missing:1` + the note the counter now feeds — the lie above is GONE in the answer |
| pending rule on the mirror's rows | 25/40/60 pending; 12809 (tooltip 404, no copy) NOT pending; 127555 (no detail stamp) not pending — the rule points where the copy exists, measured |
| card door, pre-reader binary vs new, SAME DB | added keys EXACTLY `price_min_now`/`price_min_now_money`, removed NOTHING, zero drift: live `16190 ↔ 1g 61s 90c` beside stored `17190 ↔ 1g 71s 90c` — the money formatter rides the live copper; `price_now_at` stays off the wire (recorded) |
| CI replay, driver generated FROM the committed block (port-stripped compare EQUAL 33/33), fresh port 9207 + fresh DB | **33/33 rc 0, plant rc 0**: press 1 requests 3 / written 2 (live 16 175/15 175 vs stored 17 175) / missing 1 + note; press 2 ZERO asks; `item 25 --cached` reads the pair (`16175` + `1g 61s 75c`), the manual card answers null/null; `vendor_buy`/`price_trend_normal` NULL on EVERY row; fresh CI file BORN at 7 |
| builds through the chain | exit 0 every commit, the FOUR named warnings the whole way; the fixture RUN-order break above is fixture-side, not build-side |

### Post-tag verification — replays on FRESH ports, real numbers, honest weak measurements

| probe | answer |
| ----- | ------ |
| mirror replay, driver generated FROM the committed block (python extraction; port-stripped compare EQUAL 33/33) — fresh port 9208, fresh DB mirror096c.sqlite | **33/33 rc 0, plant rc=0**: press 1 requests 3 / written 2 at live `15119/14119` against stored `16119` (RUN 1119 THIS process — the fixture's live answers move with it by design) / missing 1 + note; press 2 requests 0, `complete true`; card step 22 reads the pair back |
| fixture GET tallies | **34 = 200×18 / 304×9 / 404×3 / 429×4** vs the v0.9.5-era **31 = 200×15 / 304×9 / 404×3 / 429×4** — the price press adds EXACTLY three 200s on the tooltip door (25/40/60), the idempotent press adds NOTHING, and 304/404/429 cannot move: no conditional ask was sent |
| pre-hop twins: the COMMITTED 33-step block MINUS the price steps (31), the v0.9.5 binary BUILT FROM THE TAGGED TREE `23ceb4e` (worktree `wt-v95base`, build green FOUR) vs this binary — fresh ports 9209/9210, fresh DBs pre095a (born 6) / pre096a (born 7) | both **31/31 rc 0**; structural diff over the 31 pairs: ONLY the card doors grew keys — `price_min_now` + `price_min_now_money` present as **null/null** (steps 5/22/23, +50/+52 bytes) — every other delta is the fixture's OWN per-process RUN (stored prices `18444` vs `20238`: two crawl processes) and wall-clock stamps 7 s apart, RECORDED as such, not pinned away |
| the hop read back over those files | the v0.9.5-era 6-file opened by the v0.9.6 binary: `status` rc 0, meta 7, 6 rows, stamps NULL, second `status` BYTE-IDENTICAL; the v0.9.5 binary on a 7-file: rc **4** `database on disk uses schema 7, this build wants 6`, file stays 7; pinned v0.8.1 on the 7-file: rc **4**, `wants 4`, untouched |
| the v0.9.5-era 90-door exit map, RE-RUN on fresh ports with the v0.9.6 binary | **90/90 rc pairs IDENTICAL** — totals `0`×59 · `2`×14 · `3`×6 · `4`×11 unchanged (money doors 87/88 answer 2, 89/90 answer 0 — NO flag for money OR the trio; pinned trio 65/66/67 = 0/0/4, and door 66 now rides **4→5→6→7 in ONE open**, meta says 7, 6 items, zero price/vendor values on it) |
| the map GROWN by eleven doors (fresh DBs, fresh ports 9211/9212, a DEAD port — upstream sees NOTHING) | press on pending rows rc 0 (written 2 + note, missing 1); idempotent press rc 0 (0 requests); **the drained dead-port ask answered rc 0 because pending was ALREADY EMPTY — recorded as the WEAK measurement it was, NOT the abort proof**; bogus stage rc 2; unknown flag rc 2; second fresh DB press rc 0; card doors over the pair rc 0/0 (`14869 ↔ 1g 48s 69c` beside a stored `15869`, the price-less row answers null/null); THEN the abort RE-MEASURED where pending EXISTS (dead port 9998, DB still pending 25/40/60): rc **3**, pending AFTER IDENTICAL, stamps ZERO — resumable, nothing marked; re-press rc 0 (requests 3, written 2, missing 1); list-only DB (5 rows, ZERO copies): rc 0, 0 requests, `complete true`. **Totals now 101 doors: `0`×67 · `2`×16 · `3`×7 · `4`×11** |
| the user's own file, first door by the v0.9.6 binary | 6 → 7 in place: **19 258 rows / 4 231 vendor_sell intact**, `price_history` **4 286** — the drift from this ADR's own v0.9.5 number (4 285) is the FILE'S OWN fresh detail writes (newest sample id 22384 @ 23:51:43Z UTC today, one more at 22:50:11Z; ADR 0007's one-sample-per-fresh-write rides, RECORDED not chased), `AVG(price_min)` moves WITH them (54 391.705896 → 54 366.070845956652); `price_min_now`/`vendor_buy`/`price_trend_normal` NULL on ALL 19 258 and BOTH stamps NULL on ALL — no producer ever touched the live catalogue (the price stage ran against fixture mirrors only), and a SECOND `status` is BYTE-IDENTICAL |
| pinned-window growth through the new era | pinned v0.9.5 binary refuses the 7-file honest (the PINNED refusal ladder keeps its teeth: v0.7.0→4, v0.8.1→4, v0.9.5→6), the CURRENT build chains everything stale into 7 |

### CI ladder — the pending verdict folds here



**44962724652** (head `8e79e97…`, v0.9.5's CI-history fold push): **success, 5/5 in 8 min 37 s**
(23:37:44Z → 23:46:21Z), attempt 1, no supersession — the verdict ADR 0016 left pending, folded
FORWARD here. **44968797491** (v0.9.6's chain+tag push `8e79e97..4ced794`, head `4ced794…` — the
TAGGED tree): **success, 5/5 in 9 min 15 s** (2026-11-22T00:53:13Z → 01:02:28Z — quality 1 m 50 s
· test 4 m 40 s · linux 5 m 24 s · macos 6 m 25 s · windows 9 m 12 s); matched by `id`/`head_sha`
(the run's own `.attempt` field reads null on this API — the ladder's measured habit), EXACTLY ONE
run id for that head and no CANCELLED sibling: no supersession event, no rerun submitted. **44969661861**
(v0.9.6's verification-fold push `4ced794..0c450ba`, head `0c450ba…`): **success, 5/5 in 9 min 17
s** (01:04:00Z → 01:13:17Z — quality 1 m 40 s · test 4 m 37 s · linux 5 m 14 s · macos 6 m 39 s ·
windows 9 m 14 s), one run id for that head, no supersession — the CI fold GREW on the same breath
it pushed (honest: this fold waited for the branch to be clear, and the wait's verdict landed
green, so it folded HERE, and only the CI fold's OWN push stays FORWARD). The fold itself stayed
LOCAL while 44968797491 was in flight (the observed rule: a fresh push cancels the branch's in-flight
run). Every recorded push through v0.9.6
is green, straight or after an honest resubmission of the SAME head. Tags stay put (`v0.9.5` →
`23ceb4e`, `v0.9.6` → `4ced794`).

## 2026-11-21 · v0.9.5 — the price homes: FOUR columns own FOUR questions, carved now, filled later

### What forced it — the user's own test

The user tested the app and said: "the prices shown right now are not the vendor prices —
there should be a vendor sell value and a vendor buy value (if the item can be bought), and
down the line two more: current auction-house min buyout and the historic normal the live
trend answers. How we'll get to those values can be ignored for now, but the columns should
be allocated now." The answer until today really does carry ONE price (plus the vendor sell):
`price_min` is the STORED copy's cheapest AUCTION-house buy — an aging payload's number, not
a live one, and not a vendor's. The honest shape is FOUR homes, and the user's sentence
settled the scope: carve NOW, fetch LATER.

One thing the test exposed that the record must keep: `price_min` answering "the gold column"
in docs/DATABASE.md was always the stored copy's number — users read `2g 5s 27c` on the
money cell and assumed the AH OFFERS that now. It never promised now-ness. The hop names the
live question as its OWN column instead of quietly overloading the old one.

### The decision, and what was rejected

The schema hop 5 → 6 adds `vendor_buy`, `price_min_now`, `price_trend_normal` — names that
are OURS: no upstream door has EVER answered these words (docs/API.md's measured keys hold
nothing near them), so writing a producer now would INVENT an upstream shape (rejected twice:
the user deferred fetching, AND the house rule forbids guessing at payloads). Computing live
values at answer time was rejected too — unlike money (ADR 0015: a pure rendering of copper
that already exists), "what the AH offered at 14:05" is an OBSERVATION that AGES: it cannot
be re-derived from what is on disk, so its home IS the DB. Overloading `price_min` for the
live value was rejected: one column answering two questions over time is two truths in one
word — `price_min` stays what the payload said, `price_min_now` is what the AH says NOW.
Carving nothing and documenting a wish was rejected by the user's own words. Chosen:
additive ALTERs, `check_schema` chains v1..v5 up to 6 — the staircase v0.9.0's comment
promised since its own hop. NULL in the trio means **"no producer has ever written it"** —
deliberately NOT yet the fetched/not-there triple the price columns split: a checked-missing
state for a door that does not exist is ADR 0011's rejected plan; the third state arrives
WITH the first producer.

### Chain

`cb80fcc` feat(db) (`SCHEMA_VERSION` 6, trio in `SCHEMA_SQL` before `raw_json`, `SQL_MIGRATE_TO_V6`,
arms 1–5 in `check_schema`) → `5fa6b23` ci (money paragraph gains its tense note, schema claims
truthful, 31 steps stay 31) → `e5acc83` chore (banner 0.9.4 → 0.9.5, Cargo.lock rides) →
this record carries the tag.

### Verification, pre-tag — the hop, ZERO requests

| probe | answer |
| ----- | ------ |
| fresh DB born at 6 | `.schema item`: trio BEFORE `raw_json` in CREATE; `schema_version|6` |
| scratch schema-5 copy, first door | `status` rc=0 → meta 6; rows 6 / 4 priced / 4 vendor_sell intact, copper unchanged (25 keeps its stated `606`), trio NULL on all; SECOND `status` BYTE-IDENTICAL, nothing re-migrated |
| the user's own file, first door | 5 → 6 in place: **19 258 rows / 4 231 vendor_sell / 4 285 samples intact**, trio NULL on ALL (`COUNT ... IS NOT NULL` = 0), money doors re-answer (`item 14138`: `vendor_sell 20042 → 2g 42c`, no money key inside `item`) |
| door pairs, BOTH binaries (v0.9.4 tree `4875222`, scratch worktree; this one) over two copies of ONE fixture DB | **12 of 13 BYTE-IDENTICAL** at equal byte counts: search 1 941 / 763 / 755 / 3 363, stats 1 533 / 1 850 / 17 462, cards 4 336 / 1 103 / 3 868 / 4 022, `refresh-twins` 223; `status` differs ONLY in its `db.path` echo — pinned, IDENTICAL |
| pinned v0.8.1 binary on a migrated file | exit **4**, `database on disk uses schema 6, this build wants 4`, file untouched after |
| build through the chain | exit 0, the FOUR named dead-code warnings the whole way — `SQL_MIGRATE_TO_V6` had its consumer in the SAME commit (no dead window, unlike v0.9.4's `money_of_avg`) |

This release's own push verdicts land in the post-tag section below, joining the ladder
(`v0.9.4` → `4875222`, `v0.9.3` → `78426d1`, `v0.9.2` → `46ee524`, `v0.9.1` → `5631168`,
`v0.9.0` → `174460f`).

### Verification, post-tag — fresh fixture ports 9190 / 9191 / 9192..9199 + 9200

* **Mirror replay from the committed block** (the sed-generated driver, fresh port 9190 —
  verified free by a python socket bind — fresh scratch DB): BEFORE running, the driver's step
  list was compared to the committed block after stripping the `cargo run --` prefix —
  **EQUAL, 31 steps, zero diffs** (an early comparator pass printed noise from its own prefix
  slicing; the honest filter was the one used). Result **31/31 steps rc 0, plant line rc=0**,
  fixture tally **31 GETs = 200×15 / 304×9 / 404×3 / 429×4** — identical to the v0.9.4 and
  v0.9.3 replays: the hop asks nothing. DB tail `6|4|4|1|1|0`, icons `3|1`, twins `4|4|4|0`,
  made row **ZERO** asks, committed fixture untouched — and schema reads **`schema_version|6`**
  now: the fresh file is BORN at 6 (the replay driver's own summary header still says "must
  stay 5" — that header line is the v0.9.4 claim the sed driver carried along; the ANSWER the
  replay prints is 6, and the comment in the committed ci.yml names the truth). Trio
  `COUNT(...) IS NOT NULL` = **0** on the mirror: reserved homes, NULL everywhere.
* **Pre-hop replay**: the whole 31-step block run with the **v0.9.4-era binary** (tree `4875222`,
  scratch worktree) on fresh port 9191 — 31/31 green, same GET tallies — then the v0.9.5 build
  READ that schema-5 file: 9 doors over two copies, **8 BYTE-IDENTICAL** at equal byte counts
  (`status` aside, whose 1-byte delta is its own `db.path` echo — with that string pinned the
  JSON is IDENTICAL), `item 25 --cached` answers on the MIGRATED file exactly as on the old one
  (`4 336` bytes, money pair `vendor_sell 606 → 6s 6c` rides), stamps equal row-by-row, trio NULL
  on all — **no refresh door**: the reservation reaches old files on the next read.
* **The full exit-code audit re-run** on fresh ports 9192..9199 + 9200 and fresh DBs: **90 doors**,
  the v0.9.4 map's **90/90 rc pairs IDENTICAL, zero changed** — money doors 87/88 still answer
  usage `2` (still NO flag for money or the trio), 89/90 still `0`; totals `0` ×59 · `2` ×14 ·
  `3` ×6 · `4` ×11 — byte-for-byte the v0.9.4 distribution. Fixture A **29 GETs = 200×16 /
  304×3 / 404×6 / 429×4**, B **2**, C **4 = 200×2 / 404×1 / 429×1** — identical to the v0.9.4
  audit: the trio sent no ask anywhere.
* **One driver glitch found and fixed, honestly recorded**: the audit's FIRST pass let door 65
  open a scratch DB the pinning was NOT supposed to see — `migr095b.sqlite` sat there from an
  earlier pre-tag measurement, already migrated to 6 — so door 65 printed rc **4** (the pinned
  binary's HONEST refusal of a schema-6 file, product behaving perfectly) while its "FRESH
  file" precondition was quietly broken. The DRIVER lied, not the product; `rm -f` for the pinned
  scratch DB (and its WAL sidecars) restored the precondition, and the re-run measured door
  65 rc=0 (pinned crawl WRITES schema 4: 3 requests — 2 pages + one 429 — matching C's 4-GET
  tally with the liveness probe), door 66 rc=0 with the CURRENT build migrating **4 → 5 → 6**
  in one open (5 items intact, meta `schema_version|6`), and door 67 rc=4 honest refusal on the
  migrated file (`database on disk uses schema 6, this build wants 4`), file untouched. The
  stale-chain hop chain is proven from the PINNED era all the way to 6.
* **CI run 44960088009** (chain+tag push `1bc4e50..23ceb4e`, `head_sha 23ceb4e…` — the TAGGED
  tree): **attempt 1 CANCELLED** (created 23:05:59Z, ended 23:12:07Z) — GitHub's branch
  management superseded it the moment this verification record's push landed a new head on
  `main`: a run-management fact, not a build failure, not the app's. Resubmitted through the
  rerun endpoint: **attempt 2 SUCCESS, 5/5 jobs green in 6 min 12 s** (jobs 23:14:47Z →
  23:20:59Z — quality 1 m 57 s · test 4 m 40 s · linux 5 m 23 s · macos 6 m 10 s · windows
  5 m 59 s). Same head, re-measured: the TAGGED tree is green.
* **CI run 44960596846** (verification record's push `23ceb4e..62a7501`, head `62a7501…`):
  **attempt 1 CANCELLED** (jobs started 23:12:09Z, ended 23:12:09Z → 23:14:3x–4x — only
  quality finished green) — superseded by the rerun submission above, landing on the SAME
  branch while this run was still in flight; the observed rule stated where it bit: a rerun
  or a fresh push on the branch cancels that branch's in-flight run, which is why NOTHING
  was pushed while these runs were underway and the two rerun requests were the only
  submissions. **Attempt 2 (rerun): SUCCESS, 5/5 in 9 min 35 s** (23:22:03Z → 23:31:38Z —
  quality 1 m 56 s · test 4 m 47 s · linux 5 m 14 s · macos 5 m 48 s · windows 9 m 35 s), on
  the verification tree.
* **The v0.9.4 ladder is now COMPLETE**: its verdict-fold push run **44957300299** (head
  `1bc4e50…`) landed **success, 5/5 in 10 min 50 s** (22:32:48Z → 22:43:38Z), beside chain+tag
  **44954802500** (10 min 36 s) and verification push **44956129051** (10 min 44 s) — every
  recorded push through this release is green, either straight or after an honest resubmission
  of the SAME head.
* **This CI-record commit** starts its own run when pushed; its verdict lands FORWARD the
  same way — never chased with extra commits — and the tags stay put (`v0.9.5` → `23ceb4e`,
  `v0.9.4` → `4875222`, both re-verified after this push).


## 2026-11-21 · v0.9.4 — the money words: prices read like the GAME writes them, and copper stays the number

### What forced it — the unit was an open question, and the DB's OWN copies settle it

Until today the app printed raw integers next to a column comment that says `-- copper`: the
DB holds `20628`, the game writes `2g 6s 28c`. Nothing in this repo states the base, so the
base got MEASURED before any code changed — and the DB already carries the answer, because a
copied tooltip states its sell price in money words. One read of the user's own file:

```
rows                                                19 258
rows with a vendor_sell INTEGER                      4 231
rows whose copy states a "Sell Price:" line           4 231   the SAME rows
money_text(vendor_sell) EQUALS the copy's own words   4 230   of 4 231
rows that DISAGREE with their own copy                    1   → item 25: column 500 ("5s")
                                                            beside its copy "Sell Price: 7c"
rows at copper 0 in ANY copper column                     0   MIN over each column is 1
```

The copies also state the zero rule, so the shape is upstream's, not our invention:

```
  810  vendor_sell  45 267  copy "Sell Price: 4g 52s 67c"   all three parts
 2528  vendor_sell  10 367  copy "Sell Price: 1g 3s 67c"    100 c = 1 s, 100 s = 1 g
 4499  vendor_sell  25 000  copy "Sell Price: 2g 50s"       zero copper DROPPED
12720  vendor_sell  20 000  copy "Sell Price: 2g"           zero silver AND copper
  733  vendor_sell     100  copy "Sell Price: 1s"
   36  vendor_sell       7  copy "Sell Price: 7c"
14138  vendor_sell  20 042  copy "Sell Price: 2g 42c"       the MIDDLE zero: silver dropped
                                                            between gold and copper
```

Money classes over `vendor_sell`: 1 413 rows are copper-only, 810 whole silver with zero
copper, 219 carry all three parts, 73 whole gold, and exactly **1** is middle-zero — while
`price_min` has 19 middle-zero rows and `price_market` 17, so the rule is per value, not an
accident of one column. Spans and the money each column reaches:

```
price_min      n 4 291   1 .. 14 006 500   →  "1c" .. "1400g 65s"  longest 12 chars "799g 99s 98c"
price_market   n 4 291   1 .. 5 000 001 000 → "1c" .. "500000g 1s" longest 13 chars "1334g 99s 49c"
vendor_sell    n 4 231   1 .. 1 000 000    →  "1c" .. "100g"       longest 10 chars "8g 42s 91c"
AVG(price_min) = 54 391.705896 copper      →  floored "5g 43s 91c", rounded would print "5g 43s 92c"
```

### The decision, and what was rejected

Storing money in the DB was rejected before it was considered seriously: money is a rendering
of a number that already exists, and a `*_money` column would split one truth in two, break
the numeric contract docs/DATABASE.md names (the price sorts read INTEGER copper, and so do
scripts), and would drag a refresh door — which is exactly the shape v0.9.2's `refresh-twins`
had to be because a twin is STORED. Formatting money in JavaScript was rejected for the house
rule ADR 0014 restates with `b64_text`: the CLI and the GUI would then hold two formatters of
the same words and could drift. Always-visible three parts (`2g 0s 0c`) was rejected BY THE
COPIES: upstream drops a zero denomination, so printing `0s` puts words in its copy. Money
only in the UI was rejected because ADR 0004 made the CLI and the GUI one code path down to
the same JSON; a hidden money in the CLI would mean the two doors answer different shapes.
So: ONE Rust formatter, `money_text` in `src/db.rs` beside the price accessors, riding as a
**sibling fragment beside every copper number**, derived at answer time, with `money_of`
keeping an absent price absent (ADR 0004's made-up-zeros rule again: `null` beside `null`,
never `0c`) and `money_of_avg` FLOORING (money has no fraction of a copper, and rounding up
would print `5g 43s 92c` for a mean that never reached it). Money sits OUTSIDE the `item`
object on the card because `ItemDetail` and `Price` mirror the wire (docs/API.md): a field
inside them would claim upstream answered a word it never answered — the same ground ADR 0014
gave `icon_b64`. And money reads the MERGED detail, ADR 0010's merge-last rule applied to
money, which the fixture proves is not a harmless choice: row 25's `vendor_sell` was STATED by
a user at `606`, and the card prints `6s 6c` next to it, not the payload's own number.

Money needs NO flag, and that is the honest contrast with ADR 0014: an icon costs ≈ 3.1 K chars
once base64'd, money costs about 56 bytes per priced row on the mirror and the longest money
string in the user's whole DB is 13 chars. So money rides every answer, always.

### Honest non-gains and print glitches, recorded not swept

* **the money disagreement is upstream's and stays upstream's**: item 25 is the ONE row of 4 231
  whose column (`500`) disagrees with its own copy (`Sell Price: 7c`). The committed fixture
  carries the same shape (`"vendor_sell": 500` beside a copy saying `7c`), and after the CI's own
  `set-item 25 vendor_sell=606` plus the block's plant line, ALL THREE mirror rows that state
  Sell Price words disagree with their own column (`606 → 6s 6c` and `500 → 5s` beside `7c`).
  Money is money OF THE COLUMN, never a copy of a tooltip's words; nothing in this release
  reconciles the two, and no commit tries to — recording the disagreement is the honest answer.
* **the chain printed FIVE dead-code warnings for a window, then FOUR**: `money_of_avg` had no
  consumer between feat(db) and feat(stats), so the builds in that window said 5; feat(stats)
  landed its consumers and the set came back to the four names docs/DEVELOPMENT.md keeps. The
  feat(db) body says so; nothing swept the note.
* **the feat(stats) body printed `status 1 078 -> 1 125`; the measured pair is 1 035 → 1 125.**
  The commit bodies cannot move and the tag never moves, so the correction lives HERE, as the
  measurement, and every audit below re-reads 1 035 → 1 125.
* two fixes landed BEFORE any commit, so nothing shipped with them: the formatter's `parts.join(" ".to_string())`
  answered E0308 (Rust wants a `&str` separator), and one edit to `cli.rs`'s import line duplicated
  five names (E0252). Both were fixed and re-built before the commits that carry them.
* the money rows read `no price` on a row the user MADE (its `column_card` has no price to money) —
  honest, and it is the same wording ADR 0014's glyph carries: the card also says the row was made.

### Chain

`2cfb977` feat(db) (the formatter + `PriceSample`'s money pair, `history_of` fills it) →
`edbdfc5` feat(query) (`Hit`'s money pair, `HIT_COLUMNS` grows NOTHING) → `680e3e1` feat(stats)
(`Coverage`/`QualityStat`/`Extreme` money, `money_of_avg` consumed) → `21b316b` feat(cli)
(`card_money_parts` at all four card sites) → `8ae2939` feat(ui) (`money_cell`, the money rows,
trend hovers, the grid, the bars, the note, `.moneycell`, the three theads) → `f87e98f` ci (money
rides the block, NO new step, 31 stays 31) → `261b2d9` chore (banner 0.9.3 → 0.9.4, Cargo.lock rides,
schema stays `5`) → this record carries the tag.

### Verification, pre-tag — the money doors, ZERO requests

No network was touched by any money probe: every door below is `--cached` or DB-only, so the
mirror's fixture never saw an ask. Measured against the mirror DB and the user's own DB:

| door | answer |
| ---- | ------ |
| `item 3850 --cached` (user's DB) | `price_min 10075` beside `1g 75c` (the middle-zero class), `price_market 12500` beside `1g 25s`, `vendor_sell 7304` beside `73s 4c`; `available 8`, `realms 9` — no money sibling |
| `item 14138 --cached` | `vendor_sell 20042` beside `2g 42c` — and its OWN copy says `Sell Price: 2g 42c`, so formatter and copy agree with no code comparing them |
| `item 25 --cached` (mirror) | `20628 → 2g 6s 28c`, `17628 → 1g 76s 28c`, stated `606 → 6s 6c`, its ONE history sample `20628 → 2g 6s 28c` |
| `item 900000000 --cached` (the row the user made) | `price null` and money `null` beside `null` — the card answers from the DB's own columns |
| `search --limit 6` (mirror) | four priced rows `20628 → 2g 6s 28c` / `17628 → 1g 76s 28c`; the two unpriced rows (900000000, 127555) answer `null` beside `null`, NOT `0c` |
| `stats --limit 3` (mirror) | coverage `min/max/avg 20628 / 20628 / 20628.0` beside three `2g 6s 28c`; the Rare facet with ZERO priced items keeps `avg_price null` beside `avg_price_money null` |
| the UI cells (node runs the REAL `ui/app.js` over the REAL door JSON) | hits cells print `<span class="moneycell" title="20,628 copper exact">2g 6s 28c</span>`, unpriced rows print `no price`; the card prints `min price 2g 6s 28c` / `market price 1g 76s 28c` / `vendor sell 6s 6c` and keeps `available 12` un-moneied; a trend bar's hover reads `2g 6s 28c · 20,628 copper` |
| `schema_version` after money | `5` on BOTH the mirror's DB and the user's file; `refresh-twins` answers 4 copies / 0 rewritten / 4 matching / ZERO requests — money needed no door |

The v0.9.3 window's own verdict, recorded here so the record holds the whole ladder: the v0.9.3
record's push `e2db5a9..1b56f45` started GitHub run **44946961792: success, 5/5 jobs in 10 min 15 s**
(20:34:06Z → 20:44:21Z — quality 2 m 11 s · test 5 m 02 s · linux 5 m 27 s · windows 10 m 13 s ·
macos 6 m 36 s), and the tags stayed put (`v0.9.3` → `78426d1`, `v0.9.2` → `46ee524`, `v0.9.1` →
`5631168`, `v0.9.0` → `174460f`). This release's own push verdicts land in the post-tag section below.

### Verification, post-tag — fresh fixture ports 9177 / 9178..9185 + 9186 / 9187

* **Mirror replay from the committed block** (the sed-generated driver, fresh port
  9177 — verified free by a python socket bind — fresh scratch DB): BEFORE running, the
  driver's step list was compared to the committed smoke block — the SAME 31 steps, the
  only diffs being the fixture-port substitution. Result **31/31 steps rc 0, plant line
  rc=0**; fixture tally **31 GETs = 200×15 / 304×9 / 404×3 / 429×4**, byte-identical to
  the v0.9.3 replay — money sent no ask, so the tally could not move. DB tail `6|4|4|1|1|0`
  (rows|detailed|tooltips|tooltip_missing|manual|deleted), icons `3|1`, twins `4|4|4|0`,
  schema `schema_version|5`, the made row 900000000 saw **ZERO** asks, the icon encoder still
  answers **1** distinct shape. This replay's OWN copper differs from the v0.9.3 replay's
  (the fixture re-randomizes prices each run): `20527 → 2g 5s 27c`, market `17527 → 1g 75s 27c`
  — every money number in this section is read off the replay that produced it.
* **Money audit over the replay's 31 step JSONs**: **53 money keys** — `price_min_money` 28,
  `price_market_money` 12, `avg_price_money` 6, `min_price_money` 2, `max_price_money` 2,
  `vendor_sell_money` 3 — with **zero drift**: each money equals money of the very copper in
  the SAME answer (the float `avg 20527.0` answers `2g 5s 27c` — the floor holds), and **zero
  money keys inside any `item` object**. Honest probe glitch, recorded: the audit driver's FIRST
  pass stripped 5 characters (`money`) instead of 6 (`_money`) when looking up the money's
  sibling, which printed every sibling as "absent"; fixed and re-measured — the product never
  changed, the driver did.
* **Money against the fixture's OWN copies**: **3 of 3** rows that state Sell Price words
  disagree with money OF THE COLUMN (25: stated `606 → 6s 6c`; 40 and 60: `500 → 5s` beside
  `7c`), and 12809 carries 500 while its copy states no Sell Price line. Same shape the user's
  DB shows on exactly 1 of 4 231 rows — money reads the column and the record names the
  disagreement, nothing reconciles a column against a copy.
* **The full exit-code audit re-run post-tag** on fresh ports 9178..9185 + 9186 and fresh DBs:
  **90 doors**, and the v0.9.3 map's **86 pairs BYTE-FOR-BYTE identical**, zero changed. Money
  adds only four doors: an invented flag answers usage — `search sword --where description
  --moneyx` → **2**, `stats --money` → **2** (money has NO flag, ADR 0015) — while the money-
  bearing fixture door `search sword --where description --fixture` → **0** and `status` →
  **0**. New totals `0` ×59 · `2` ×14 · `3` ×6 · `4` ×11; the pinned migration trio 65/66/67
  stays `0/0/4`. Fixture A tallied **29 GETs = 200×16 / 304×3 / 404×6 / 429×4**, fixture B
  **2**, fixture C (the pinned window) **4 = 200×2 / 404×1 / 429×1** — money doors sent
  nothing, the tallies moved exactly as far as the doors ABOVE money did, nowhere further.
* **Money over a DB that never saw money**: the whole 31-step block replayed with the PRE-money
  v0.9.3 binary (built in a detached v0.9.3 worktree, since removed) on fresh port 9187 — 31/31
  green, same tally 200×15 / 304×9 / 404×3 / 429×4 — then the money binary read that DB:
  `schema_version` reads **5 before and after** (money wrote nothing to it), `search --limit 6`
  answers `16702 → 1g 67s 2c` / `13702 → 1g 37s 2c`, coverage `16702 / 16702 / 16702.0` beside
  three `1g 67s 2c`, the card prints its stated `vendor_sell 606 → 6s 6c` and the sample rides
  money, and the DB still has **zero** `*_money` columns. This is the opposite of the twin: the
  twin needed v0.9.2's `refresh-twins` because it is STORED; money answers on the next read of
  a file the OLD binary wrote, so there is nothing to refresh.
* **Money over the pinned-era DB** (the schema-4→5 migration hop, 5 rows / 0 detailed / 0
  priced): every money answer is `null` beside `null`, ZERO requests — money invents no zero
  on a migrated file, ADR 0004's rule survives the hop.
* **CI**: the chain+tag push `1b56f45..4875222` started GitHub run **44954802500: success,
  5/5 jobs in 10 min 36 s** (22:04:26Z → 22:15:02Z — quality 2 m 15 s · test 4 m 23 s · linux
  4 m 33 s · macos 7 m 16 s · windows 10 m 32 s), and it ran on the TAGGED tree
  (`head_sha 4875222…` — exactly what the tag holds). The verification record's own push
  `4875222..90cb963` started **run 44956129051: success, 5/5 jobs in 10 min 44 s**
  (22:19:17Z → 22:30:01Z — quality 2 m 16 s · test 4 m 40 s · linux 5 m 33 s · macos 6 m 19 s ·
  windows 10 m 41 s) on head `90cb963…`, the verification tree. This verdict commit starts its
  own run the same way — the ladder's rule keeps it honest: verdicts land FORWARD, the tags
  never move (`v0.9.4` → `4875222`, joining `v0.9.3` → `78426d1`, `v0.9.2` → `46ee524`, `v0.9.1` →
  `5631168`, `v0.9.0` → `174460f`).


### What forced it — one read of the user's own DB

Measured before any code changed, and it is the whole decision:

```
rows with icon_blob                       4 030  of 19 258 rows
checked as gone upstream (icon_missing_at)     1
never asked (no icon_at, no blob)         15 228
detailed rows                             4 602  ALL carry an icon_url, 4 030 carry bytes
blob bytes    min 1 385 · avg 2 327 · max 3 268   total 9 380 048  → ≈ 3.1 K chars each
```

(a) `icon_url` on 4 602 rows but bytes on 4 030: hotlinking the address would show an image
the DB never claims to hold — ADR 0008 already says the opposite of that ("nothing is
hotlinked, the DB gets the bytes"), and the hosts were measured serving `image/jpeg`. (b)
3.1 K chars per icon: the UI asks 40 hits (≈ 124 K chars) and 6 extremes (≈ 19 K), while a
CLI `search` with NO `--limit` would carry every visible row — tens of MB. So icons ride
when ASKED, and the ask is bounded by the caller.

### The decision, and what was rejected

Always-inline was rejected by that size measurement; a separate `icon ID…` verb was rejected
because the hits are already collected and the icon columns sit on the same rows in the same
DB — a second door would split one truth in two. A flag with no stamps was rejected because
an empty cell would read as "this item has no icon in the game", a lie for 15 228 rows. So:
`--icons`, a no-value flag beside `--watch`/`--cached`, and BOTH row shapes (`Hit`, `Extreme`)
grew the trio `icon_b64`/`icon_at`/`icon_missing_at` — keys always present, payload only when
asked, stamps always. The encoder moved out of `cli.rs` into `db.rs`, beside the bytes it
encodes, and `icon_b64_of` became the ONE helper the card and the lists call: the house rule
is one truth, one encoder, and the move was measured byte-identical (2 596 chars before and
after, sha256 prefix `c67a63484dc91767` both times, and the emitted base64 decodes back to the
DB's own 1 945 bytes). ZERO requests leave either door, so ADR 0004's `3` is unreachable there.

### Three honest non-gains, recorded not swept

The card needed NO code — it has drawn an `<img>` data URI since v0.6.0; the lists were the
gap. The watch list has no panel at all (measured in `ui/index.html`), so nothing was invented
to hang an icon on: watched rows gain icons where they appear, in the hits table. The live-search
table keeps no icon column, because that door reads and writes NO DB (ADR 0009) and upstream
names have no local copy. And 15 228 of 19 258 rows still answer the glyph — honest, because
those rows genuinely hold no bytes.

### Chain

`770b5bf` refactor(db) (the encoder moves) → `4de3b0d` feat(query) (the `Hit` trio) →
`d03e2f2` feat(stats) (the `Extreme` trio) → `618fae3` feat(cli) (`--icons`) →
`7db9502` feat(ui) (the icon columns + the glyph) → `41cc1f0` ci (smoke 29 → 31 cargo steps) →
`7fcd90c` chore (banner 0.9.2 → 0.9.3, schema stays `5`, Cargo.lock rides) → this record carries the tag.

### Verification, post-tag — fresh ports 9165 / 9166..9174 / 9176 (2026-11-21), nothing moved the tag

The tag stays exactly where it was: `v0.9.3` → `78426d1` (the record commit that carries it),
`v0.9.2` → `46ee524`, `v0.9.1` → `5631168`, `v0.9.0` → `174460f`. 9144..9164 were burned by
earlier windows, so every fixture below sits on a port that was verified free first (a python
socket bind) - 9165, 9166..9173, 9174 and 9176.

| probe (post-tag, v0.9.3) | measured answer |
| ------------------------ | --------------- |
| mirror replay on **9165**, driver generated FROM the committed ci.yml smoke block (31 cargo steps + the plant line) | **31/31 rc 0** (`MIRROR: 31 steps, 0 non-zero`), `plant rc=0`; the icon doors land LAST in the block: `search sword --where description --icons` → row 25 **2 596 chars + icon_at**, the manual row **none of the three** · `stats --icons` → both lists filled and row 60 answers **no bytes with `icon_missing_at` set** · `stats` without the flag → the trio present, payload null. Fixture tallies **GETs 31 = 200×15 / 304×9 / 404×3 / 429×4**, byte-identical to the v0.9.2 replay, so the icon doors asked it NOTHING. Mirror DB tail: rows 6 · detailed 4 · copies 4 · checked-gone 1 · manual 1 · **icons 3 copied + 1 gone** · twins 4\|4\|4\|0 · schema **5** (`900000000` asks ZERO, committed fixture untouched) |
| the ONE encoder across the card AND the doors (same replay) | every `icon_b64` any step emitted collapses to **one shape**: 2 596 chars → 1 945 bytes → **EQUALS the mirror DB's own blob** (`distinct shapes: 1`) - the card's row and the list doors cannot encode one copy two ways |
| exit-code audit on **9166..9173 + 9174** | **86 doors**: 01–77 keep the identical rc map (usage `2` ×11, network `3` ×6, not-found `4` ×11, rest `0`) incl. the pinned migration trio 65/66/67 = 0/0/4; the icon doors 78–86 answer **0/0/0/0/2/0/4/0/0** - `search sword --iconsx` the usage `2`, a missing DB the honest `4` - and with a fixture URL both doors still answer 0 with **A GETs 29 before = 29 after**: the flag asks nothing |
| the three icon states as three answers (door 80, `search --icons --limit 6`) | 12809/40/25 → **2 596 chars + `icon_at`** · 60 → **no bytes, `icon_missing_at` 1795292160** · 900000000/127555 → **neither stamp** ("no copy and never asked for one") - so the glyph's two states are distinguishable, not guessed |
| a DB whose icon stage never ran (door 83, a scratch copy with the blobs blanked) | states `{(None,None,None)}` plus the gone row's `{(None,None,stamp)}` - exactly what the glyph means on the user's 15 228 never-asked rows |
| **the icon doors over a DB BORN under the pinned v0.8.1 snapshot** (fresh port 9176, the pinned binary rebuilt from `a160ffa`) | the pinned era crawls list+detail+**icons** and leaves `schema_version 4` with 3 icon blobs and NO v5 columns; the current build opens it (`3\|1\|5` after the v4→v5 hop) and **both icon doors read those pinned-era bytes**: 3 rows × 2 596 chars + `icon_at`, row 60 gone-stamped, 127555 neither; every emission **decodes EQUAL** to the DB's own 1 945-byte blob (3/3, one shape); the fixture saw **15 GETs, all from the pinned crawl - ZERO from the doors** |
| GitHub | run **44944988663: success, 5/5 jobs in 10 min 42 s** (20:11Z → 20:22Z) for the chain push `ff1d174..78426d1` + the tag push - so this tag HAS a green run on its own tree (the v0.9.0 tag never did); this record's own push run gets its verdict recorded as soon as it lands |

The push of THIS record (`78426d1..e2db5a9`) started run **44946023051: success, 5/5 jobs in
9 min 44 s** (20:23Z → 20:33Z) — green on the record's own tree too, joining the chain push
`44944988663` (5/5 in 10 min 42 s) on the tagged tree. The tags stayed put throughout
(`v0.9.3` → `78426d1`, `v0.9.2` → `46ee524`, `v0.9.1` → `5631168`, `v0.9.0` → `174460f`).

Two honest print glitches in MY OWN drivers, recorded not swept: the audit's decode line fell
into its own "no bytes for row 25" fallback (my `sqlite3.connect` read of the audit DB returned
nothing while the door's answer clearly carried 2 596 chars - the clean decode proofs are the
mirror replay's `distinct shapes: 1 / EQUALS the DB's own blob: True` and the gap probe's 3/3
EQUAL against the pinned blob); and the gap probe's regex-based emission counter printed 0 while
the same answers parsed as objects show 3 emissions of 2 596 chars each. Both were re-measured
with the parsed JSON (the numbers above are the re-measurements), and neither touches the product.

## 2026-11-21 · v0.9.2 — the refresh door: old copies get their twin re-derived, ZERO requests

### What forced it — one measurement on the user's own file

ADR 0012 made the twin DERIVED and v0.9.1 corrected the derivation, but the pending rules
refuse a stamped copy (ADR 0007) and a `304` writes nothing — so the corrected rule only
ever reaches NEW copies. Measured the day this door was written, on the one file that
finished a crawl: **4 602 copies, ZERO matching the corrected rule, and the stored twin
equal to the recomputed v0.9.0 GLUED shape for all 4 602** (`Worn ShortswordItem Level
2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9(1.1 damage per second)Sell Price: 7c…`).
ADR 0012's "honest gap" (a migrated row with a copy has no twin until a write touches it)
has the SAME cause, and on this file the gap is every copy.

### The decision, and what was rejected

Re-asking upstream would spend 4 602 requests to re-fetch bytes the DB already holds — and
the pending rules would not even schedule it. Re-deriving at read time would put two
answers on one copy and let the column drift, against ADR 0012's invariant. Folding it
into the next sync pass is measured dead (the queues answer ZERO pending for stamped copies).
So: an explicit door the user presses — `refresh-twins`, no arguments, the whole DB, the
SAME `readable_tooltip_text` every write path calls (no second derivation to disagree),
only differing/missing rows written, rows collected before any write, tombstones and manual
rows riding, idempotent, and exit `0` with a report for the nothing-to-do door (`4` would
mean "the DB does not have it"). ZERO requests leave it, so ADR 0004's `3` is unreachable.

### The honest size of the gain

The card's words row becomes readable on every copy and the invariant `twin == derivation(copy)`
holds for the whole DB — those are the real wins. The description door is a SUBSTRING door,
so a phrase inside one span was always findable: measured on the user's 4 602 copies,
`damage speed` +358, `hand one` +90, `sword 1` +14, `1 main` +3, while `main hand` is 75
before AND after and `sell price: 7` is 291 both ways. The search gain is the phrases that
span an element boundary, not a general rewriting of what search can do.

Chain: `a5da052` feat(db) → `4cd95b3` feat(cli) → `d1703e4` feat(ui) → `cb8eee9` ci (24 → 29
cargo steps + one plant line) → `bb9b2c2` chore (banner 0.9.0 → 0.9.2, schema stays 5) →
this record carries the tag.

### Verification, post-tag — fresh ports 9161/9153..9160+9162/9163, and the door run over the user's own file (2026-11-21)

Nothing here moved the tag (`v0.9.2` stays on the record commit that carries it; `v0.9.1` →
`5631168`, `v0.9.0` → `174460f`). Every number was measured after the tag on FRESH ports —
9144..9152 and 9145..9160 were burned by the v0.9.1 window and no replay reuses them.

| probe (post-tag, v0.9.2) | measured answer |
| ------------------------ | --------------- |
| mirror replay, fresh port **9161**, driver generated from the committed ci.yml smoke block | **29/29 cargo steps rc 0** (`MIRROR: 29 steps, 0 non-zero`) + the plant line (`plant rc=0`); the refresh doors land IN ORDER: 0 rewritten / 4 unchanged → `search "damage speed" --where description` **total 0** → refresh **1 rewritten** → same door **total 1** → third press 0 rewritten; fixture tallies **GETs 31 = 200×15 / 304×9 / 404×3 / 429×4**, byte-identical to the v0.9.1 replay, so the refresh asked the fixture NOTHING; DB tail rows 6 · detailed 4 · copies **4** · checked-gone 1 · manual 1 · twins **4\|4\|4\|0** · schema **5** · `900000000` asks **ZERO** · committed fixture untouched |
| exit-code audit, fresh ports 9153..9160 + 9162 | **77 doors**: 01–67 rc map identical to the v0.9.1 run (usage 2 ×10, network 3 ×6, DB/not-found 4 ×10, rest 0), the migration trio still 0/0/4; the refresh doors 68–77 answer 0/0/0/0/0/0/0/**4**/**2**/0 — a missing DB is the honest 4 and `refresh-twins extra` the usage 2 — and **A GETs 29 before / 29 after** the refresh doors: the door asks nothing |
| ADR 0012's honest gap, measured the way it really happens (fresh port 9163, the pinned v0.8.1 snapshot rebuilt from `a160ffa`) | the pinned era crawls list+detail+tooltip and leaves `schema_version 4` with **3 copies and NO twin columns at all** (`.schema item` counts 0 of them); the current build opens that file (the `v4→v5` hop rides, rc 0) and reports `3\|0`; **refresh-twins answers `filled 3`** → `3\|3`; the card then reads the twin with ZERO requests (`item 25 --cached` → `Fixture Detail 25 Item Level 1 Main Hand One-Handed Sword 1 - 3 Damage Speed 1.9 Sell Price: 7c`); press 2 answers 0 rewritten; the fixture's 15 GETs all belong to the pinned crawl |
| **the door over the user's own file** — the residue v0.9.1 named, closed | BEFORE: 19 258 rows, 4 602 copies, the FIRST twin `Worn ShortswordItem Level 2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9(1.1 damage per second)Sell Price: 7cLowest buyoutsAlliance…`; the phrase doors answer `damage speed` 0 · `hand one` 0 · `sword 1` 0 · `1 main` 0 · `main hand` 75 · `sell price: 7` 291. **Press 1: rc 0 in 114 s, `copies 4602, rewritten 4602, filled 0, requests 0`** — the FIRST twin now reads `Worn Shortsword Item Level 2 Main Hand One-Handed Sword 1 - 3 Damage Speed 1.9 (1.1 damage per second) Sell Price: 7c Lowest buyouts Alliance …`; the same phrase doors answer **358 · 90 · 14 · 3 · 75 · 291** — the predicted gains exactly, and `main hand`/`sell price: 7` unchanged exactly as the substring rule says. **Press 2: rc 0, 0 rewritten / 4 602 unchanged.** File tail after: 19 258 rows · 4 602 copies · 4 602 twins · 4 602 `tooltip_lc` · schema **5** · icons 4 030 + 1 checked-gone · 4 284 price samples · manual 0 · deleted 0 · html-without-twin **0** — the file is intact and the ONLY thing that changed is the twin's wording |
| GitHub | the chain push `51a8d24..46ee524` + the tag push = run **44939561521: success, 5/5 jobs in 9 min 17 s** on the tagged tree, so this tag HAS a green run (the v0.9.0 tag never did); this record's own push run gets its verdict recorded as soon as it lands |

The push of THIS record (`46ee524..e746c4a`) started run **44940485574: success, 5/5 jobs in
10 min 36 s** (19:21Z → 19:32Z) — green on the record's own tree as well, and the tag stayed
exactly where it was (`v0.9.2` → `46ee524`, `v0.9.1` → `5631168`, `v0.9.0` → `174460f`).

Two honest print glitches in MY drivers, not in the product: `mirror092.sh`'s rows query named
a column this DB does not have (`detail_at`) and `refresh-user.sh`'s tail query named
`price_samples` — both tallies were re-measured with the real column names above, and the
drivers keep the glitch in their text so the record says which lines were re-measured.

## 2026-11-21 · v0.9.1 — the v0.9.0 defects die here: the scope door answers, the twin keeps its spaces

### What forced it — two measurements, no theory

1. **A hang**: both v0.9.0 replays (ports 9124 / 9125..9133) froze **2 h 01 m** on
   `search sword --where description`, the child in **state R** with no output; a scratch `timeout 5`
   repro answers **rc 124**. Read off `parse_invocation`: the value-flag path advances at the loop
   TAIL (`index = index + 2`); the v0.9.0 `--where` block answered VALID values with a bare
   `continue` — skipping the advance, re-reading the same pair forever. The house pattern (assign
   `out.scope`, fall through to the shared advance like `--sort`/`--quality`) lands; the bogus
   scope still answers the usage door `2`.
2. **Glued twins**: the user file's first live tooltip pass (4 602 copies) shows live tooltip html
   is minified — spans adjacent, no whitespace between them — and the v0.9.0 derivation DROPPED
   spans: `Worn ShortswordItem Level 2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9…`. The
   fixture's stand-in card carries spaces AT the tag boundaries, which is exactly why this survived
   every fixture-shaped audit. Measured lesson written down: **fixture-shaped data cannot disprove
   claims about the live shape; only the live pass can.** The rule now: every `<…>` span leaves
   ONE space, collapse whitespace runs, trim.

`ee8c06d` `fix(cli)` + `5a36d19` `fix(db)`, this record carries the tag. The v0.9.0 tag stays put
(tag `v0.9.0` = `174460f`, its verification record `05d073d`); the user file's 4 602 glued twins
stay until a copy's content changes (pending rule + 304-writes-nothing) — a refresh door is a
named plan.

### Verification, post-tag — fresh ports, the tag stays put (2026-11-21)

Nothing in this section moved the tag (`v0.9.1` = the record commit that carries it, `v0.9.0`
still `174460f`). Every number below was measured after the tag, on FRESH fixture ports — the
ports the earlier windows burned (9124..9143) stay BUSY/defunct and no replay reuses them.

| probe (post-tag) | measured answer |
| ---------------- | --------------- |
| mirror replay, fresh port 9144 | **24/24 steps rc 0** (`MIRROR: 24 steps, 0 non-zero`) — the three v0.9.0-added steps land: `search sword --where description` → rc 0, `"total": 2`, `--where all` → rc 0, `"total": 2`, and `item 900000000 --cached` answers the made row from the DB (`"cached": true, "reused": false`, twin `a plain CI sword mug`); fixture tallies **GETs 31 = 200×15 / 304×9 / 404×3 / 429×4**, stages detail 18 / icon 5 / tooltip 5 / search 3; rows 6, twins **4\|4\|4\|0** (`html-without-twin` = 0), schema stamp **5**, `900000000` asks **ZERO** |
| exit-code audit, fresh ports 9145..9152 | **67 doors**: the v0.8.1 map is byte-identical (05/06 = 4, 08 = 2, 10/11 = 2, 12–14 = 3, 19 = 2, 20/25/30 = 3, 29 = 2, 32–34 + 36 = 2, 35 = 4, 48–51 + 54 = 4), the v0.9.0 doors hold (58/59/60 = 0/0/0 with hits **1/1/0**, bogus scope 61 = **2**, twin doors 62/63/64 = 0/0/0 with the stated `<b>Probe Sword stated</b>` → twin `Probe Sword stated` in the SAME write, counts **3\|3\|3**), and `900000000` asks stayed **ZERO** across the stages |
| the schema chain, REAL now (fresh port 9143 + a scratch DB, the user's file never opened) | 65 rc **0**: the pinned v0.8.1 snapshot rebuilt from `a160ffa` BIRTHS its own shape → `schema_version\|4`, 5 rows; 66 rc **0**: the current build opens that schema-4 file and the `v4→v5` hop rides → `schema_version\|5`; 67 rc **4**: the pinned snapshot on the bumped file answers `database on disk uses schema 5, this build wants 4; delete the file to rebuild it` and leaves the file untouched (meta still 5, rows 5\|0) — the pre-tag run had to SKIP these doors (its scratch copy had been wiped by the harness restart); this one answers them |
| GitHub runs | the fix-chain push `174460f..5a36d19` = run **44934053776: success, 5/5 jobs in 8 min 11 s**; the record + tag push = run **44934961058: success, 5/5 jobs in 7 min 17 s** on the tagged tree — so this tag HAS a green run, which the v0.9.0 tag never did; the hung run **44863357406** stays `completed/cancelled` at 18:07Z, exactly as recorded under v0.9.0 |
| the user's own file opened by the v0.9.1 build | rc 0, `19 258 rows · 4 602 detailed · 14 656 checked-removed · tooltips 4 602 · twins 4 602 · twin_lc 4 602 · schema 5` — rows intact through the additive migration, and the 4 602 twins keep their glued v0.9.0 shape (the pending rule never re-asks a stamped copy, and a 304 writes nothing): the refresh door is still a named plan, not code |
| the committed fixture | `git status --porcelain tests/fixture_api.py` empty — the drivers generate their own scratch copies, `tests/fixture_api.py` untouched |

The push of THIS record (`5631168..b6d19d1`) started run **44935854879: success, 5/5 jobs in
10 min 9 s** (18:30Z → 18:40Z) — green on the record's own tree as well, and the tag stayed
exactly where it was (`v0.9.1` → `5631168`, `v0.9.0` → `174460f`).

## 2026-11-21 · v0.9.0 — the copies are READABLE and FINDABLE: the readable twin of every copied tooltip

### What forced the milestone

v0.8.0 stored each record's description copy as upstream's OWN tooltip `html`, and its own docs
state the two honest gaps that came with it: the card prints that markup as escaped TEXT (and
`search` would grep TAGS), and the local search door reads NAMES only — so the DB HOLDS
descriptions it can never FIND, and the card shows markup where the words belong. The user chose
BOTH together: one milestone ("the copies are readable and findable"), one ADR (0012), the code
chain split into `feat` commits.

### The measurements — every number probed, none invented

| probe (2026-11-21) | measured answer |
| ------------------ | --------------- |
| `GET /api/v1/tooltip/forever/item/25` | 200 json 3060 b, `html` 1404 b, `ETag W/"2b111cbcd3bbb4d7078f6d05"`, `cache-control public, max-age=300, s-maxage=900, stale-while-revalidate=3600` |
| tags inside that html | `<div class="twdb-…">`, `<span class="twdb-…">`, `<img class="twdb-icon" src=… width height alt loading decoding>` |
| `<img>` hotlinks INSIDE the copy | `render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg` + `thewowdb.com/assets/pvp/faction/alliance.jpg` — TWO hosts, pixels: the card must NEVER render upstream markup (the icon stage's rule, aimed at the same pixels, says the DB gets the bytes) |
| entities in TWO bodies | **ZERO** `&[A-Za-z#0-9]+;` matches in both — 25 (plain stats prose) AND 728, picked off the USER's DB as a row whose `raw_json` carries flavor (728/1132/1357 answer it; 728's `twdb-quote` shows `"Teaches you how to cook Westfall Stew."`, 1281 b of html) → the derivation is EXACTLY drop-spans/collapse/trim, and NO entity decoder exists — inventing one is not our call to make; if upstream ever ships entities they show VERBATIM and the words still search |
| fixture detail server CODE (`tests/fixture_api.py`) | `body["name"] = "Fixture Detail {}".format(tail)` for EVERY detailed row → the list door's row named "Worn Shortsword" (id 25) is OVERWRITTEN before the smoke's search steps run; the only row keeping its list name is the checked-404 127555 → `search sword --where description` AND `--where all` BOTH answer 2 (row 25 + the manual row's stated "a plain CI sword mug"); the name scope answers 0 |
| `next_manual_id` on a fresh DB | `MANUAL_ID_BASE` = 900000000 → the smoke's mug IS 900000000; and `cmd_item` answers manual-row cards from the DB's columns BEFORE the `--cached` branch → `item 900000000 --cached` sends ZERO requests, exits `0`, and prints the twin its fields earned at birth |
| the user's own file, decision date | schema `4`, 19 258 rows, **ZERO** tooltip copies (`tooltip_at` count 0) → the honest twin gap (a migrated row that ALREADY had a copy shows no twin until its next tooltip write — SQLite `ALTER` cannot run Rust, and duplicating the reader across languages is the drift DATABASE.md exists to forbid) only bites DBs that copied at v0.8.x; for this file EVERY copy will be born with a twin, because its tooltip pass runs AFTER this release |

### Own fixes, owned mid-chain (nothing swept under the rug)

- a python-style `.format(` leaked onto a parser line — `cargo build` answered, `format!` landed;
- the derivation arm missed a `;`, and the tooltip write double-wrapped an `Option` into
  `optional_text` — E0308 + parse errors, three small honest fixes;
- the `--where all` two-needle loop moved `needle` and rebound an immutable `seen` —
  E0382/E0384 → `needle.clone()` + `let mut seen`;
- the twin reader's regex function-clone failed twice (anchor drift) → exact-source insert
  after `tooltip_html_of`'s block instead;
- the smoke comment's FIRST draft claimed "all answers 3", grounded only on the LIST payload —
  measured against the fixture's OWN server BEFORE the tag (fa4311f rewrote it): a comment is
  trusted text for the next reader, so the lie died pre-tag, not post-record;
- the rustc ICE + "hard linking … failed" pair on this FS again: the recorded workaround
  (`rm -rf target/debug/incremental`) applied first, then `cargo build` rc 0 with the four
  documented warnings.

### The crawl sequencing — this release does NOT touch the running crawl

The pinned v0.8.0 snapshot crawl keeps working the user's file (measured mid-milestone: the
detail stage ≈ 63.5 % done — 12 235 of 19 258 rows PROCESSED, i.e. 3 580 answered + 8 655
checked-404). A schema-5 build stamps that file the MOMENT it
opens it, and the pinned snapshot would then REFUSE it (exit 4 — ADR 0004's honest refusal, and
the file would still be intact). House rule for this milestone: while the pinned crawl runs,
NO 0.9.x binary opens the user's DB; every audit of this release works on scratch DBs. When the
crawl finishes, the tooltip pass rides a v0.9.0 snapshot — and every copy it ever makes is born
with a twin, so the honest gap never shows on this file.

### What shipped

`89ff23d` schema 5 + the v4→v5 hop chained from EVERY stale shape → `cafe3af` `readable_tooltip_text`
+ `SQL_SET_TOOLTIP` grows both columns (value count 4→6) derived from the MERGED text, same write
→ `5c1341b` the manual doors birth the twin (`set-item tooltip_html=`, `add-item`) → `96f9f7a`
`Filter.scope` + `text_clauses`/`text_needles` (tombstone clause stays first) → `e3f7f8f` CLI
`--where` + usage line + `Invocation.scope` default `name` → `dd1599d` `tooltip_text_of` reader +
card fragment (BOTH shapes on the card) → `48b16a8` UI where select + rendered twin row →
`0e0f0ac` CI smoke 21→24 → `dbb699a` banner 0.9.0 → `fa4311f` the smoke comment's measured truth
→ docs record + ADR 0012 (this commit carries the tag).

### Verification, post-tag — what the replays ACTUALLY did (2026-11-21, kept not papered over)

The two replays this tag's own entry promised ("the post-tag mirror replay… recorded in the
verification record") ran — and did NOT land green. The truth, every row measured:

| probe (2026-11-21) | measured answer |
| ------------------ | --------------- |
| mirror replay (port 9124), steps 1–16 | all rc 0 — the v0.8.1-shaped steps still pass (the `add-item` now states a description) |
| step 17 / exit-audit door 58: `search sword --where description` | child process **R (running), 2 h 01 m, no output** — BOTH replays froze on this exact door; mirror stopped at 16/24, the audit at 57/66 |
| reproduction on a scratch DB | `timeout 5` → **rc 124** killed at 5 s — the door never lands: not slowness, a loop |
| the cause, read off `parse_invocation` | the v0.9.0 `--where` block used a bare `continue` on VALID values; the parser's value-flag path advances at the loop TAIL (`index = index + 2`) — the `continue` skipped it, re-reading the same pair forever |
| GH run for the tag-push tree (run 44863357406) | created 03:53Z, sat queued overnight, **cancelled — never ran**; a rerun was fired at 11:04 EST and reached the SAME smoke door: **in_progress 2 h 03 m, cancelled by hand at 18:07Z** (measured) — the tree at this tag never had a green run |
| the user file's crawl this window | the file read **schema 5 at ≤10:25** (a 0.9.x build opened it EARLIER than the sequencing rule assumed), and the pinned 0.8.0 snapshot then refused it (exit-4 door) and died with the harness restart. The crawl RESUMED on pinned v0.9.0 snapshots (pending rule = resume point): detail COMPLETE (990 fetched + 4 663 checked-404 → 4 602 answered / 14 656 checked-removed of 19 258), icons COMPLETE over 4 tries (a 403 icon door retried in place; 1 393 copied; DB: 4 030 icons + 1 checked-gone), tooltip COMPLETE in ONE try: **4 602 requests → 4 602 copies in 2 256 s (~2 rec/s)** — and `tooltip_text`/`tooltip_lc` both count **4 602**: every copy on this file was born with a twin; rows 19 258 intact through the additive migration |
| the FIRST twin on the user file | `Worn ShortswordItem Level 2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9(1.1 damage per second)...` — **words glued**: live tooltip html is MINIFIED (no whitespace between spans) and the v0.9.0 derivation DROPPED spans instead of leaving a space. The fixture never showed this — its stand-in card carries spaces AT the tag boundaries — so fixture-shaped data cannot disprove claims about the live shape: only the live pass can |

Both defects are fixed FORWARD, and this tag stays put: `ee8c06d` `fix(cli)` (the shared
advance) and `5a36d19` `fix(db)` (span→space) open the v0.9.1 chain; its green replays
(fresh ports 9134 / 9135..9142: mirror **24/24**, GETs 31 = 429×4/304×9/404×3, twins
4|4|4|0, hits 2/2/0, ZERO asks for the manual id; audit **64 doors**, the v0.8.1 map byte
identical + `0,0,0,2` scope + `0,0,0` twin doors) and its CI runs are recorded there. The
user file's 4 602 GLUED twins stay in their v0.9.0 shape until a copy's content changes
(the pending rule will not re-ask stamped copies, and a 304 writes nothing by ADR 0007) —
a refresh door is a named plan, not invented code.

## 2026-11-20 · v0.8.1 — CI says only what it can prove: the public project is green on three targets

### What forced the milestone

The repo got published — public `wow-goldtool`, `main` at `ebee66b` plus 11 annotated
tags, all verified on the remote (`git ls-remote` answers 24 refs, every tag object
`^{}` resolves to its commit). Publishing a "documented milestone project" carries one
promise it had NEVER made good: its CI. GitHub started its own run and it went red —
and the release discipline forbids leaving it red with a lie on top, so v0.8.1 is the
milestone that makes the workflow assert only what was measured. No `src/` line moves;
the discipline of the repo's OWN infrastructure earns its own ADR (0011).

### The measurement rounds — four runs, every error quoted

GitHub's runs are the witness. The local CI mirror replays the smoke block green (it
did since v0.8.0, 21/21), which proves WHAT THE SMOKE BLOCK DOES — it cannot prove the
WORKFLOW IS TRUE: it hardcodes its own DB path, it runs on one Linux box, and it never
evaluates env expressions or resolves action names. Only GitHub can say those things,
so every round of this milestone is "ask the runner, read the annotation, fix exactly
that line".

| run | commit | measured result |
| --- | ------ | --------------- |
| #1 | `ebee66b` | **failure after 7 s**, 5 annotations, ALL of them `Unable to resolve action`: `actions/rust-action`, `ubuntu-setup-action/install-ubuntu-system-dependencies`, `windows-setup-action/install-windows-system-dependencies` — `gh api orgs/…` answers **HTTP 404** for the two orgs; and the committed file has **no `checkout` step at all**, so even a resolvable action would have handed `cargo build` an empty directory |
| — | local | the same file's two gates measured on this tree: `cargo lint --help` answers `error: no such command: lint` (cargo 1.94.0) — the command does not exist; `cargo fmt --check` answers a **diff** on our own files (hand-written house style: long `use` lists, explicit `return`); `cargo clippy --all-targets` rc 0, **0 errors / 600 warnings** |
| #2 | `c29e45c` | **cancelled** — the workflow's own `concurrency` block cancelled it when #3 landed (fix round 1: `actions/checkout@v7` + `dtolnay/rust-toolchain@stable`, apt for the dev libs, clippy informational, header comment states the two limits) |
| #3 | `5c888b9` (fix round 2: apt set = `libgtk-3-dev libwebkit2gtk-4.1-dev`, the majors measured off `ldd target/debug/wow-goldtool` — the template pair was wrong on TWO majors: gtk 2 vs gtk 3, webkit-4.0 vs 4.1) | **failure**: build (linux) + quality **success**, three NEW lines answered: test died at the FIRST smoke step `cannot open /fixture.sqlite: unable to open database file` → rc 4, because `runner.tmpDir` is NOT a context property and evaluated to the EMPTY string; build (macos) rc 101 `error[E0463]: can't find crate for std` + `the x86_64-apple-darwin target may not be installed` while the SAME run's toolchain step answers `host: aarch64-apple-darwin` — macos-latest IS arm64; build (windows) rc 1 `` `D:\a\wow-goldtool\wow-goldtool\icons/icon.ico` not found; required for generating a Windows Resource file during tauri-build`` — the repo's `icons/` was EMPTY, and `tauri.conf.json`'s `bundle.icon` already named `icons/icon.png` for a file that never existed |
| #4 | `59b8379` (fix round 3: icon assets landed as `f29a2b2`, then `WGT_DB: fixture.sqlite` + matrix row `aarch64-apple-darwin` as `59b8379`) | **SUCCESS, 5/5 jobs in 9 min 13 s**: quality 2 m 04 s · build (macos, aarch64) 2 m 27 s · test 4 m 19 s · build (linux) 5 m 46 s · build (windows) 9 m 12 s — the whole sequence green on the public account |

### Decisions (one line each, full text in ADR 0011)

* Only resolvable action names — the two orgs answer 404, inventing similar names is
  not our call to make; `actions/checkout` and `dtolnay/rust-toolchain` are the
  documented, marketplace-real ones, and the checkout step is ALSO the fix for the
  structural half of run #1 (a cargo build with no checked-out tree).
* apt, not third-party setup actions: the package SET is measured off our own binary's
  NEEDED lines, so the workflow stops owing correctness to an unpublished org.
* clippy informational, no fmt gate: the gates would contradict measured truth (fmt
  answers a diff; the four dead-code warnings ARE the plan this very file names).
* No matrix narrowing: three targets still ride, but each now names the triple its
  image actually builds — macos-latest is arm64, measured.
* The icon assets are product work: `bundle.icon` already lied once (it names a file
  that never existed); the honest fix is to HAVE them — drawn by hand in PIL, our own
  card/coin/W, `file` confirms PNG 512x512 and a real 7-entry ICO.
* The smoke block stays verbatim through every round: its 21 `cargo run --` lines diff
  EMPTY against the audited sequence each time (asserted before each commit).

### The audit (executed, not asserted)

| probe | expectation | observed |
| ----- | ----------- | -------- |
| the old workflow's claims | resolvable + true, or the run is red | run #1 red in 7 s, quoted above |
| `cargo lint` / `cargo fmt --check` / `cargo clippy` on this tree | the gates that survive | lint DOES NOT EXIST / fmt answers a diff / clippy 0 errors + 600 warnings |
| `ldd target/debug/wow-goldtool` | which dev packages Linux needs | `libwebkit2gtk-4.1.so.0 libgtk-3.so.0 libgobject-2.0.so.0 libsoup-3.0.so.0 libpango-1.0.so.0` → apt `libgtk-3-dev libwebkit2gtk-4.1-dev` |
| the icon assets | real files, not ignored, our own | `file`: PNG 512x512 + ICO 7 entries; drawn by hand; `git check-ignore` says no |
| every ci.yml rewrite | smoke block preserved | parsed `cargo run --` list diffs EMPTY, 21 steps; YAML: build 5 / quality 4 / test 5 |
| `cargo build` after the banner bump (`fe691c9`) | everything still compiles | rc 0, 8.94 s incremental, exactly the four documented warnings, 0 errors |
| GitHub's verdict | green | **run #4: 5/5 jobs, 9 m 13 s** — and run #2 cancelled by the concurrency block, honestly |

The post-tag replays (CI mirror on fresh port 9114, the 57-door exit audit on 9115..9122
+ 9123, fresh DBs) follow in the verification record under this tag, same as at v0.8.0.

### What this milestone is NOT

It is not a code release: no door, no schema, no UI element moved. It IS the release
banner (Cargo.toml `0.8.1` at `fe691c9`) because the promise it fixes is the repo's own,
and the USER_AGENT line built from `CARGO_PKG_VERSION` now says `wow-goldtool 0.8.1` —
every upstream ask announces which build of the discipline it rode.

### Verification at the tag (executed AFTER the tag, on ports nothing had touched)

GitHub's own witness at the tagged tree and both local replays re-run against it:

| probe at `v0.8.1` | expectation | observed |
| ----------------- | ----------- | -------- |
| run #5 (`a160ffa`, the commit the tag rides) | green on the tagged tree | **SUCCESS 5/5 in 8 min 01 s**: quality 2 m 18 s · test 4 m 23 s · build (linux) 5 m 37 s · build (macos, aarch64) 6 m 30 s · build (windows) 8 m 01 s — five measured runs total, the last two green |
| `cargo build` at the tag (after the documented `rm -rf target/debug/incremental` workaround for the hardlink-free FS) | still compiles | rc 0, **0 errors, exactly the four documented warnings** |
| 21-step mirror replay GENERATED from the committed ci.yml (fixture port 9114, fresh DB) | the smoke sequence still green | **21/21 rc=0** · **31 GETs** (list 5 · detail 13 · icons 5 · tooltip 5 · search 3) · `4 × 429` retried IN PLACE · `9 × 304` · `3 × 404` · **ZERO asks for the manual id 900000000** · DB ends `6|4|3|1|1|0|4` · four `sync_state` rows `complete` |
| 57-door exit audit (ports 9115..9122 + 9123, fresh DBs) | the contract unchanged | **57/57 door/rc pairs identical** to the v0.8.0 record — usage `2`, network `3`, not-found `4` on the same doors; manual-row card 940 b DB-only; fresh write `requests 2, written 1, rate_limited 1`; keeps your `vendor_sell 606` while the crawl-owned columns refresh; ledger 4 → 5 |
| the harness hiccup, honestly owned | the mirror is GENERATED, so it cannot drift | my regeneration for v0.8.1 first pasted the ci.yml's literal `8765` into the step lines (fixture bound at 9114, steps knocked a dead port: 14 non-zero). The fix was IN THE HARNESS — steps re-pasted with `$PORT` — and the second replay is the one recorded. The repo's ci.yml stayed byte-identical throughout |
| only figures that moved between audits | prices, by design | the fixture gives each process its own RUN offset (ADR 0007's sampling rule) — a fresh fixture process IS a fresh observation window, so price fields cannot and must not reproduce |

## 2026-11-20 · v0.8.0 — the DB holds every forever item's description copy, and takes edits by hand

### What forced the milestone

The user's milestone asks for something the last seven releases never promised: a DB that holds
**every forever item with its vendor price, its icon and its description copy**, and that can be
**edited by hand**. Two of the three were already reachable — the vendor price rides the detail
record, the icon bytes live in the DB since v0.6.0 — and the third is not behind the door the
crawl already reads. That is not a nicety to design around: it is a fact about the wire, and it
decides the shape of the release. And the manual half is forced by the crawl's own shape: the list
stage UPSERTs every id upstream still lists, so a row a user HARD-deleted would be resurrected by
the next list pass. Both halves were measured before anything was written
([ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md) carries the full record).

### The grounding, measured 2026-11-20 (live probes, then read back off the user's own file)

```
# the record the crawl ALREADY reads (raw_json of id 25, read back off the user's own file):
attribution, icon_url, id, item_level, item_subtype, item_type, links, name,
price, quality, required_level, unique, vendor_sell, weapon
    -> NO description, NO html, NO `fields`: a description column on the detail door
       would have invented a field upstream never answers

# the door the description lives behind (one live request, UA wow-goldtool 0.8.0):
GET https://thewowdb.com/api/v1/tooltip/forever/item/25
  -> 200 application/json 3060 b
  etag W/"2b111cbcd3bbb4d7078f6d05"
  cache-control: public, max-age=300, s-maxage=900, stale-while-revalidate=3600
  x-ratelimit-limit: 120   x-ratelimit-remaining: 62
  x-wgf-build: wow_classic_beta-1.60.1.70532
  top keys: attribution, build, fields, html, icon_url, id, kind, links, name, quality, version
  `html` = 1404 b of upstream's OWN card markup: <div class="twdb-card twdb-k-item">… its own
           <img class="twdb-icon" src="https://render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg">,
           its own wording ("Item Level 2", "Sell Price: 7c")
  `fields` = 26 keys mirroring the detail record's own column set (armor, binding, build, classes,
             effects, faction_prices, flavor, … sockets, socket_bonus, stats, unique, vendor_sell,
             version, version_label, weapon)
  links.api = the door itself, links.html = https://thewowdb.com/wow-forever/item/worn-shortsword-25/

# an id the door has not (one live request):
GET https://thewowdb.com/api/v1/tooltip/forever/item/999999999999
  -> 404 application/json 79 b, no ETag offered, x-ratelimit-remaining: 119
  body: {"error":"not_found","message":"No item 999999999999 in forever.","status":404}
        (upstream's OWN words — the stage reads the STATUS and gives that record its own
         checked third state, exactly as v0.5.3 gave a dropped detail record)

# the user's own file, read-only: 19 258 rows, MAX(id) below the manual block = 288 340
```

Three measured facts decide the shape, none of them invented: the description is not on the record
the crawl reads, so a FOURTH door is a fact (≈ 19 258 tooltip requests ≈ 2.7 h at the measured 2/s
— the same order as the detail and icon stages, so it belongs to the crawl, budgeted/resumable/
cancellable, and not to a live ask); the door revalidates (`ETag` + `max-age=300`), so the stage
stores its own `tooltip_etag` and a targeted re-ask costs a `304` and writes nothing; and its `404`
answers upstream's own JSON, so the third state gets ADR 0006's treatment. The reserved manual id
block at `900 000 000` is provably clear of every id the walk can bring back (measured max upstream
id on the user's file: `288 340`), and a tombstone — not a hard delete — is the only shape that
survives the next list pass.

### The decision

[ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md) puts the description copy in as **stage
4 of a pull** with the plumbing every stage already has (its own pending rule, its own `sync_state`
row, the shared `RateBudget`, cancel at chunk granularity, `429` retried IN PLACE so a rate limit
never aborts a stage) and puts the user's edits in as **one rule: overrides merge LAST**. The
rejected alternatives are recorded there in full — inline-in-stage-2 (the reuse path would leave
the `2 919` records the user already fetched permanently description-less, a stage whose output
depends on which build fetched the record), copying the whole tooltip body (two sources of truth
for one record), rendering upstream's markup (that lets upstream write the card), a hard delete
(resurrected by the next UPSERT), a user-picked id (it can collide with a FUTURE forever record),
a separate `manual` table (a second place "the user already said" could hide), editing the auction
columns (a permanent lie about CURRENT prices, for a gold tool), and making a manual row crawlable
(upstream has no such id: the queue would ask a record that cannot exist and stamp "gone" on the
user's own row).

### Schema 4, and the migration that had to chain

`SCHEMA_VERSION 3 → 4`, eight additive columns (`tooltip_html`, `tooltip_etag`, `tooltip_at`,
`tooltip_missing_at` + `manual_at`, `manual_overrides`, `edited_at`, `deleted_at`), and `SQL_MIGRATE_TO_V4`
is eight `ALTER`s so the chain is now v1→v2→v3→v4 — the user's own 19 258-row catalogue comes up
in ONE call, and ALTER appends, so a migrated file's column order differs from a fresh file's and
every reader names columns explicitly.

The direction that cannot be guessed at was MEASURED at this release: the moment a v0.8.0 build
first opened the user's file it moved 3→4, and the pinned v0.7.0 crawl that was still walking it
died with `database on disk uses schema 4, this build wants 3` — which is why the real pull-all had
to be re-pinned on a v0.8.0 snapshot mid-release. A file newer than this build is still refused;
that is the honest answer, not a guess.

### What the audits caught in the code

* **a fresh DB was writing the wrong shape.** `SCHEMA_SQL` lost a comma INSIDE the Rust string —
  `"… detail_missing_at INTEGER, tooltip_missing_at INTEGER"` glued onto `"icon_blob BLOB"` — so
  SQLite read `INTEGERicon_blob` as a type name: the `CREATE` **succeeded**, and BOTH columns came
  out ABSENT. The chain migration masked it (its `ALTER`s name the columns), the fresh write did
  not: a fresh file had no icon bytes and no tooltip third state, and every reader naming them
  would have died. The witness is `sqlite3 -readonly .schema item` read back off a fresh DB —
  `tooltip_missing_at` present at slot 44, `icon_blob BLOB` at 45 — and `fix(db)` landed the comma.
  The family is now named three times in this file: a schema string that BUILDS is not a schema that
  WORKS.
* **the coverage answer's read slots moved.** Five new counts pushed the price columns from slots
  7-9 to 12-14; rusqlite reads rows by position, so an un-guarded read would have averaged the
  wrong numbers into `stats`. The expected-count guard in the audits is what catches that drift,
  and the CLI, the UI and the audits read ONE answer.
* **the card was asked for a row upstream cannot know.** `item 900000000 --cached` answered the v0.7.0
  `not-found` — "fetch it with `sync --stage detail`" — and every pending queue then REFUSED that
  exact ask. The promise was a lie in one direction, so the row's own columns answer it instead
  (`column_card`/`SQL_COLUMNS_CARD`, 14 columns), and the live door short-circuits BEFORE building a
  client: ZERO requests, witnessed as ZERO asks for 900000000 in the fixture log before and after
  all three queue stages ran.
* **the fresh schema's comma fix had to land before CI grew the steps.** Chronology this release got
  right, and v0.7.0 got wrong: the fixture grounds the tooltip door (`test(fixture)` 413e472) and
  the schema fix (`fix(db)` 13cd1e6) land BEFORE the six new CI steps (`chore(ci)` b1df47a), so the
  mirror stayed green at the commit that grew the sequence.

### What the release grew

Eighteen commits between `944b510` and the tag: `feat(db)` schema 4 + chained migration → the manual
layer (overrides merge last, CRUD writers, tombstone) → the derived search columns in the edit door
→ `feat(cli)` the four CRUD verbs and the usage line → `feat(stats)` the pending queues refuse
tombstones and manual rows → `feat(query)` `visible_clause()` as the FIRST WHERE clause → `feat(stats)`
coverage grows five counts → `feat(wowdb,models,db)` the tooltip door's URL, its `Tooltip` mirror and
its writers → `feat(stats)` the tooltip queue → `feat(sync,cli)` `sync_tooltips` as stage 4 →
`test(fixture)` the door's measured shape → `fix(db)` the fresh schema's comma → `chore(ci)` twenty-one
mirror steps → `feat(db,cli)` the card carries the copy/marks/refuses tombstones → the manual row's
card answers from its own columns → `feat(ui)` the status grid + the fourth pull-all leg → `feat(ui)`
the write panel + the card's tooltip/marks/tombstone note → `chore` the version banner rides 0.8.0
(both `Cargo.toml` and the pinned `Cargo.lock`, the v0.7.0 precedent).

### The numbers, not adjectives (every audit re-run against the shipped tree)

| audit | expectation | observed |
| ----- | ----------- | -------- |
| CI mirror (the committed twenty-one steps, fixture process, fresh DB, budget 8) | green with the fourth stage and the four manual doors inside | **21/21 steps exit 0**; the fourth stage budgeted `requests 5, items 3, written 3, missing 1, rate_limited 1, complete true` — asked ids `25, 40, 60, 12809`, and `12809` is the checked third state — then drained `requests 0, complete true`; the three manual answers are their own: `set-item 25 vendor_sell=606` → `{"id": 25, "field": "vendor_sell", "value": "606", "edited_at": 1795218573}`, `add-item "name=CI Test Mug" quality=Rare` → `{"id": 900000000, "manual_at": …, "fields": ["name","quality"]}`, `delete-item 60` → `{"deleted": [60], "at": …}`, `undelete-item 60` → `{"restored": [60]}`; `stats` after the mirror `items 6, named 6, priced 4, detailed 4, missing 1, icons 3, icon_missing 1, tooltips 3, tooltip_missing 1, edited 2, manual 1, deleted 0, history 4`, min/max/avg `18136`; the DB ends `6\|4\|3\|1\|1\|0\|4` (rows\|detailed\|tooltips\|tooltip_missing\|manual\|tombstones\|samples); `sync_state` FOUR rows `complete` (`list page 2`, `detail 0/2/2`, `icons 0/4/4`, `tooltip 0/0/0`); fixture log **31 GETs** — detail 13, list 5, icons 5, tooltip 5, search 3 — with `4 × 429`, `9 × 304`, `3 × 404`, and **ZERO** asks for 900000000 |
| exit-code audit (**57 doors**) | the old doors byte-for-byte identical, the new families land where ADR 0004 says | the eighteen of v0.6.0 `0,0,0,0,4,4,0,2,0,2,2,3,3,3,0,0,0,0`; the seven of v0.7.0 `2,3,0,0,0,0,3`; the tooltip stage `0,0,0,2,3` (drained queue, targeted `--ids 12809` on the gone tooltip → **0** `requests 1, items 0, missing 1, complete false` and the DB says `tooltip_missing_at` set / `tooltip_html` NULL, a re-ask `--ids 25` → **0** `requests 1, items 1, written 0, unchanged 1` = the door's own ETag revalidation, `--stage tooltipx` → **2** `sync wants --stage list, detail, icons or tooltip, got tooltipx`, dead host → **3** naming the door `…/api/v1/tooltip/forever/item/25`); the manual layer `0,2,2,2,2,4,2,0,0` with the messages `an editable INTEGER field wants a plain number` / `nose is not editable (…)` / `set-item states exactly one FIELD=VALUE per ask` / `no item 999999999 in the DB (add-item creates one)` / `add-item wants at least a name (name=...)`, and BOTH card doors of the made row → **0**, 940 b, `status null`, `detail_synced_at null`, name `Probe Mug`, quality `Rare`; three queue doors after the create → `0,0,0` with `requests 0` each and the fixture's ask count for 900000000 still **0**; the edit doors `0,0,0,0` — `set-item 40 name=ProbeRenamed` then `search ProbeRenamed` → **0** `total 1`, and the DB says `name` "ProbeRenamed" with `name_lc` "proberenamed" in ONE write; the tombstone five `0,4,4,4,4` (bare search total **6 → 5** across `delete-item 60`, double-delete → **4** `item 60 is already tombstoned`, `item 60 --cached` → **4**, `item 60` live → **4** while the fixture's ask count for `forever/item/60` stays **2** — refused BEFORE any request goes out — and `set-item 60 …` → **4** `… (undelete-item first)`); the restore doors `0,0,4,0` (`{"restored": [60]}`, again → **4** `item 60 is not tombstoned`, search total back to **6**, `stats` `deleted 1` → `deleted 0`); and one fresh-write door against a SECOND fixture process → **0** `requests 2, written 1, rate_limited 1`: the row KEEPS the user's `vendor_sell` 606 while the crawl-owned `price_min` moves 15528 → 19913, `tooltip_html` stays in the row, and `price_history` grows 4 → 5 with exactly one new sample (`seq 5, item 25, 19913`) — ADR 0007's rule holds under the merge. No DB-only door reached **3**: the manual verbs send no request, so they cannot be rate limited |
| GUI launch audit (scratch DB seeded by a fixture crawl: 5 rows, 4 samples) | the shipped page boots FOUR verbs; the new panel fires nothing itself | run 1: exactly `status, watch, stats, status` and **ZERO** requests to the fixture; temp probe (the four write verbs plus the made row's LIVE card, rebuilt into the page then reverted): the ipc log grows exactly `add-item (2 words), set-item (2 words), delete-item (1 words), undelete-item (1 words), item (1 words)` and the fixture saw **0** requests during that run — the manual row's card asked nothing — the DB ends 6 rows, manual 1, edited 2, deleted 0, samples 4, the made row `id 900000000, manual_at …, name "GUI Probe Mug", quality "Rare"`; run 3 re-audits the clean page after the revert (four verbs again, `git status --porcelain ui/` clean) |
| the schema chain on a real file | a fresh DB and a migrated one answer the SAME columns | a fresh DB's `status` writes schema `4`, and `.schema item` names `tooltip_missing_at` (slot 44) and `icon_blob BLOB` (45); a genuine schema-3 file migrated by this build lands at `schema_version` 4 with `icon_blob` present and its rows intact — the comma bug above is what the fresh half of that row caught |
| `cargo build` at the tag | warnings are the plan, not surprises | **0 errors**, FOUR warnings: `catalogue`, `open_db`, `pending_plan`'s private `PendingSql` and `Tooltip`'s unread fields (the measured key list rides the model, only `html` is read) — the set grew by exactly one, ADR 0009's rule; plus the hardlink notice when the DB-path doc recompiles on this filesystem |
| syntax of the three embedded UI files | they parse before the build embeds them | `node --check ui/app.js` parses (v22.23.5); `ui/index.html` tag-balanced (7 sections, 17 divs, 4 selects, 16 buttons); `ui/app.css` braces 80/80 |
| post-tag verification (fresh ports 9111 and 9102..9109 plus the second window 9099, fresh DBs) | the recorded numbers reproduce, or explain themselves | mirror **21/21 exit 0** with the identical counts (31 GETs, `4 × 429`, `9 × 304`, `3 × 404`, **0** asks for 900000000, DB `6\|4\|3\|1\|1\|0\|4`, four `sync_state` rows `complete`); exit audit **57 doors** with the SAME code line, card 940 b on both doors, fresh write `requests 2, written 1, rate_limited 1` keeping 606 and growing the ledger 4 → 5. Only the price figures moved — min/max/avg 18136 → 20814 and 15 528 → 17 923 — because the fixture gives every process its own `RUN` offset (ADR 0007's rule: a fresh process is a fresh observation window), so a re-run cannot reproduce a price and is not meant to |
| the user's own DB | never written by an audit; the authorized crawl walks it | read-only (`sqlite3 -readonly`) at the tag: schema `4`, `19 258` rows, max upstream id `288 340`, `tooltip_at` set on **0**, `detail_synced_at` set on `2 919` — the tooltip pass cannot start before the tag, because the pinned crawl snapshot has no fourth stage; the authorized real pull-all (detail → icons) kept walking on a pinned v0.8.0 snapshot, and the tooltip pass runs as a THIRD crawl pass with the released binary after the tag |

The audits themselves never touched the user's file: every run pointed at a scratch DB through
`WGT_DB` and the file was read hashed / `sqlite3 -readonly`. The ONE authorized write to it is the
real pull-all the user asked for, and the schema migration 3→4 that any v0.8.0 build does on first
open (measured consequence above: the pinned v0.7.0 crawl died refusing it).

---

## 2026-11-20 · v0.7.0 — the live-search door: one human ask, one request

### What forced the milestone

v0.6.0 shipped three names compiled but never used — `search_url`, `SearchResponse`,
`ApiResponse.remaining`: the client already built the URL, the mirror already shaped the
grouped answer (`groups` as a `Value`), the response struct already carried the burst
counter — and none had a consumer, because the door docs/API.md had promised ("we use it
for the live search panel") was never opened. Six milestones of dead code are a promise the
next release makes to itself; opening the door is this milestone. WHAT it does is decided
only by measurement — inventing a shape for a door nobody had opened is not our call to make.

### The grounding, measured 2026-11-20 (live, one request each)

Three live probes (UA `wow-goldtool …; data source: thewowdb.com`), verbatim answers:

```
GET /api/v1/forever/search/?q=Worn%20Shortsword -> 200 application/json 1288 b
    etag W/"76db61a78b8398bc089262d2"
    x-ratelimit-limit: 120  x-ratelimit-remaining: 119
    access-control-expose-headers: ETag, Retry-After, X-RateLimit-Limit, X-RateLimit-Remaining, X-WGF-Build
    top keys: attribution, groups, q, version ("forever"); q echoes DECODED "Worn Shortsword"
    groups exactly item/npc/quest/sound/spell/zone, each { has_more, more, results }
    the one item hit's keys: id, item_level, kind, links{api,html,tooltip}, name,
    quality, required_level, version
    every group's `more` is upstream's own paged list endpoint, e.g.
    https://thewowdb.com/api/v1/forever/item/?page=1&per_page=50&q=Worn+Shortsword
    — a space rides as `+` INSIDE the answer's own URLs, not %20

GET /api/v1/forever/search/?q=Ring              -> 200 application/json 7271 b
    item/quest/spell: has_more TRUE, five results each
    npc 4 / zone 2 / sound 0: has_more false

GET /api/v1/forever/search/?q=socket            -> 200, every group EMPTY
    (the names-only trap: socketed items exist in the catalogue, the door matches NAMES,
     so the word answers nothing — docs/API.md measures it)
```

`SearchResponse` already matched this shape EXACTLY — the mirror needed nothing. Three
measured facts decide the door: an empty answer is a SUCCESS (the names-only trap is a
word upstream never said, not a failure); every group's `has_more` + `more` means chasing
it would ask the paged per-group endpoint, one request per group per page — a human ask
becomes a crawl; and the answer is upstream's own JSON printed as it came (serde pas-through),
so the `+` rides verbatim while OUR request URL uses `%20` because the measured ask asked
with `%20` and answered 200 (a raw space is not a URL character).

### The decision

[ADR 0009](./DECISIONS/0009-live-search-door.md) keeps the door an ASK, not a sync: ONE
request like `item`, exit codes riding ADR 0004's contract (`2` no text, `3` network or
`429`-with-`Retry-After`, `0` even for all-empty groups, `4` only if the door itself answers
`404` for the shape), NO DB write and NO DB read. The load-bearing rejection is the write-from-
an-ask one: ADR 0007 made every fresh write an OBSERVATION in `price_history`, and an answer
that is not a record's own detail body would put ledger rows that never came from a detail
write. Chasing `more` was rejected on the measured `?q=Ring` answer (three-plus extra requests
and more per page: that is a crawl, not an ask; a later patch could take `--page` honestly);
retry-in-place was rejected because that dance belongs to a stage that owns a queue and a
budget, so the door mirrors the `item` door — and the fixture's `search` stage is UNGATED for
exactly this reason: the committed CI step must stay green.

### What the door grew

* `src/cli.rs`: verb `live-search` (usage line 30 between `item ID [--cached]` and the watch
  verbs, dispatch one line); the WHOLE verb tail joined with spaces is the query (`join_tail`,
  so `live-search Worn Shortsword` asks one name, not two words); one `get_json` with an empty
  stored ETag — the door revalidates nothing; payload `{status, retry_after, remaining, answer}`
  with `answer` the serde pas-through.
* `src/wowdb.rs`: `encode_query` — letters/digits/`-._` ride through, a space is `%20`,
  everything else its UTF-8 bytes as `%XX`: the shape the measured ask proves.
* `tests/fixture_api.py`: the grouped answer from the measured transcript — six groups each
  `{has_more, more, results}` by fixture-name substring matching (names only: `socket` answers
  empty), `+` inside its own `more` URLs, `version "forever"`, `etag W/"fixture-search-etag"` —
  AND the `429` gate no longer names the `search` stage.
* `ui/index.html` + `ui/app.js`: a live-search card, one field, one `Ask once` button, a
  `group · id · name` mini table, a `view upstream` link per hit at each hit's own `links.html`,
  and a note printing `remaining` (honestly `null` when the host sends no burst header — the
  fixture does not on this stage), the names-only fact, and whether any group said `has_more`.
  The wiring grew twice and got deduped: a second `btn-live` listener would have made one
  click two asks, and one request per click is the whole promise of the panel.
* CI grew the smoke sequence to fifteen steps: the door's one-word ask, the two-word ask
  (riding `%20`, the fixture's own GET log is the witness), and the names-only trap
  (`tooltip` answers all-empty groups — the step passes at exit `0`).
* The chain order deserves honesty: `chore(ci)` landed the door's steps BEFORE the fixture
  grounds them — between those two commits the v0.2.0 gate answered the door's first request
  `429` (the gate named every stage) and the stub shape was the v0.2.0 placeholder, so that
  intermediate sequence was red. Every commit in this series only guarantees a build, never
  a green CI run mid-series; the mirror replay below runs the committed fifteen steps against
  the committed fixture, both landed.
* `Cargo.toml` and the `User-Agent` banner moved 0.6.0 → 0.7.0 (the UA names both the version
  and the source, per the developer terms).

### The numbers, not adjectives (every audit re-run against the shipped tree)

| audit | expectation | observed |
| ----- | ----------- | -------- |
| CI mirror (fixture process A, fresh DB, budget 8: the committed fifteen steps + the v0.5.3 gone-id door + a drained icon queue) | green with the door inside; the door touched no DB state | fifteen steps exit `0`; the icon stage's numbers unchanged `requests 5, items 4, written 3, missing 1, rate_limited 1, complete true`; `stats` after CI `items 5, named 5, detailed 4, missing 1, icons 3, icon_missing 1, history 4` — the door wrote no sample; extra `item 127555` answers exit `4` and re-stamps once, the crawl after it `requests 0, complete true`; drained icons `requests 0`; fixture A log **28 GETs, 3 of them the door** (`sword`, `Worn%20Shortsword` — the two-word ask rides `%20` on the wire, the answer echoes `q` decoded — and `tooltip` with its all-empty 200), `3 × 429`, `4 × 404`; second process (fresh window): `detail --ids 25` `requests 2, written 1, rate_limited 1` grows the ledger 4 → **5** (`seq 5, item 25`), its log 3 GETs `1 × 429` |
| exit-code audit (twenty-seven doors) | the eighteen of v0.6.0 stay identical, the new verb adds its doors | old eighteen `0,0,0,0,4,4,0,2,0,2,2,3,3,3,0,0,0,0`, byte-for-byte v0.6.0's audit; new: `live-search` alone → **2** (`live-search wants text, for example: live-search Worn Shortsword`), dead-host ask → **3** `error sending request for url (http://127.0.0.1:9974/api/v1/forever/search/?q=ring)` — the URL in the message proves the verb's own URL building, two-word on a dead host → **3** with the percent-shape `…q=Worn%20Shortsword` in the message; all-empty groups → **0**, two grounded fixture asks → **0** `0` (the answer's `more` rides the `+` form `…q=Worn+Shortsword`), the no-DB door `live-search sword --db …/exit70-nodb-dir/none.sqlite` → **0** with the answer while `exit70-nodb-dir` is NEVER created — opening that path would have answered `4`; and the retry branch: a SCRATCH copy gates `search` once (the committed fixture untouched, `git status` clean after) → **3** `rate limited by the API: the API asked us to back off`, the scratch log one `429` — witnessed WITHOUT touching the committed one; the fresh-window write door → `0` grew the ledger |
| this build's own live asks | upstream's answer as it came, exit 0 | `live-search "Worn Shortsword"`: `status 200, remaining 119`, item group **1 hit** `has_more false`; `live-search Ring`: **5 hits** `has_more true` with `more` = `https://thewowdb.com/api/v1/forever/item/?page=1&per_page=50&q=Ring` — the `+` rides verbatim; `live-search zzzqqqnothere` → **exit 0**, all six groups `results: []` |
| GUI launch audit (the shipped page) | boot stays four verbs, nothing crawls | exactly `status, watch, stats, status` and NO `live-search` line — the panel moves only on its own button; scratch DB `0` items, `0` samples, empty `sync_state`, schema `3` |
| GUI live probe (temporary, rebuilt into the page, reverted right after) | the panel's ask reaches the SAME ipc guard | a 5th line `gui: ipc verb live-search (1 words)` + one fixture GET `/search/?q=sword … 200`; probe then reverted with `git checkout -- ui/app.js` (`grep probe` answers `0`), rebuilt clean, and the four-verb boot above re-audited against the reverted tree |
| `cargo build` at the tag | warnings are the plan, not surprises | exactly THREE dead-code warnings — `catalogue`, `open_db`, `pending_plan`'s private `PendingSql` — plus the hardlink notice: the set SHRANK from v0.6.0's six, the door consumed the three names it was owed |
| the user's own DB | never written by any of this | audited read-only before and after every run: hash `7442814b…c3d3b8`, mtime `2026-11-19 21:42:54`, `19 258` rows — identical throughout |

The user's DB (`HOME/.local/share/wow-goldtool/wow-goldtool.sqlite`) was never written by
any of this: every run above pointed at a scratch file through `WGT_DB`, and the file
itself was only ever read hashed / `sqlite3 -readonly`.

---

## 2026-11-20 · v0.6.0 — price history, local icons and the card that shows both

### What forced the milestone

Two questions the schema on disk cannot answer, both asked by the app's own purpose:

* **A trend.** The `item` row keeps ONE price per item (`price_min` …), overwritten by the
  next write. "What does this cost?" answers; "is it getting cheaper?" has no answer in the
  file, and nothing upstream does either: a `forever/item` payload carries the CURRENT
  auction-house state, and a record that did not move answers `304` with no body at all —
  a revalidation says "nothing moved", not "here is a price". Any growth of the ledger had
  to come from **our** fresh observations, which makes the sampling rule a cost question
  about the one request per record the detail stage already pays.
* **Icons offline.** v0.5.x's card deliberately never hotlinked `icon_url` (the README and
  the card notes say why). So the bytes have to come from the DB, and where they live and
  how they arrive is a decision, not a detail.

### The grounding, measured 2026-11-20

The live catalogue answers exactly two `icon_url` shapes on answered records, and both
serve JPEG bytes; the same records also answer an EMPTY `icon_url` sometimes:

```
https://render.worldofwarcraft.com/<region>/icons/56/<name>.jpg   (/us/, /forever-eu/, /forever-us/)
https://thewowdb.com/assets/icons/<name>.jpg
.../us/icons/56/inv_mace_03.jpg      -> 200 image/jpeg 1945 b
.../forever-eu/icons/56/133749.jpg  -> 200 image/jpeg 2265 b
thewowdb.com/assets/icons/inv_axe_11.jpg -> 200 image/jpeg 1936 b
"icon_url": ""                       -> 4 of the 29 detailed rows of the scratch catalogue
                                        (Peace Tea, Craftsman's Writ, Scroll of Imbue
                                         Iceknife, Lodestone)
```

An ABSENT icon path answers differently per host — measured, and it decided the third
state's rule: `thewowdb.com/assets/icons/<bogus>.jpg` → **404 `text/html`** (the site's own
"Page Not Found" body: an answer about the file), while
`render.worldofwarcraft.com/us/icons/56/<bogus>.jpg` → **403 `application/xml`**
(243 b of S3-style `AccessDenied`: an answer about a CDN, not about an icon). `icon_missing_at`
is written for the `404` and the `403` stays a stage failure (exit `3`, resumable, nothing
marked, icon pending) — a fact the upstream never stated stays unstated. The card's `stats`
pairs ride the same discipline: across the scratch catalogue's 29 detailed rows, `stats`
carried values on 2 (`Cold Basilisk Eye`: `[{"name":"Frost Resistance","value":9}]`,
`Tough Scorpid Bracers`), and `sockets`/`socket_bonus`/`classes`/`effects` were empty on
**all** 29 — so `stats` renders as pairs and the others get a raw row only when they carry
something. No shape was invented anywhere.

### The two decisions

[ADR 0007](./DECISIONS/0007-price-sampling.md) put the ledger in a second table and sampled
**once per fresh detail write**, prices present: a `304`, a list write, a check-mark, a
re-insert write nothing, so the mirror's `written 0, unchanged 2` steps must leave the count
untouched and a second crawl session must grow it exactly by its fresh writes. Five
alternatives rejected there: sampling `304`s (the ledger becomes a function of click count),
a dedicated sampling pass (doubles the request cost and spends the budget on records the
write visit could not refresh), a "numbers moved" rule (hides repeated observations and lies
about flat markets), a timer binary (ADR 0004: the headless CLI is the integration path; no
second binary has ever shipped), and stuffing samples into the `item` row (the point of
`docs/DATABASE.md` is that the UI reads SQLite). The FK probe measured the guard's shape:
SQLite 3.54.1 **accepts and does not enforce** an FK clause on `INSERT` — an
`INSERT INTO price_history` with `item_id = 999` and no such item row succeeded — so no
clause ships, and the guard is the Rust order inside `upsert_detail`.

[ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md) made the icon a **third crawl
stage** (`sync --stage icons`) with the detail stage's whole plumbing — own pending queue,
budget, cancel, resume, third state — copying each answered record's own icon file into an
`icon_blob BLOB` exactly once. The load-bearing rejection: fetching icons INLINE inside the
detail stage would double a chunk's requests AND leave every record fetched before this
build permanently icon-less, because those details now answer `304` and a `304` re-fetches
nothing. A file sidecar next to the DB got the same treatment: the DB is the one artifact
the user manages (the README names its path), and a second place for bytes is a second
place for "gone upstream" to hide. No icon ETag is stored anywhere, because the pending
rule excludes copied icons and the copy path is asked once.

### Schema 3, and the migration that chains v1→v2→v3

`SCHEMA_VERSION 2 → 3`: three `ALTER`s append `icon_blob`/`icon_at`/`icon_missing_at`, a
`CREATE` adds `price_history` plus its index, and the stamp moves in one call. The file on
the user's disk says `schema_version = 1`, so `check_schema` now **chains**: v1 first gets
v0.5.3's `detail_missing_at`, then both v1 and v2 get the v0.6.0 batch. Refusing either
would have thrown away a 19 258-row catalogue on the excuse of a schema bump; a file
**newer** than this build is still refused (the audit's schema-9 file answers exit `4` with
`database on disk uses schema 9, this build wants 3`). A migrated file's column ORDER
differs from a fresh file's (`ALTER` appends), and every reader in the code names columns
explicitly, so nothing reads positions.

### What the audits caught in the code

Two fixes landed because the audits read what the code actually did, not what the notes
claimed:

* `SCHEMA_SQL`'s v3 rewrite had dropped `CREATE INDEX item__name_lc` — a migrated file
  keeps its copy (the user's file has `item__detail item__name_lc item__price item__rank`
  and the migration re-issues only `price_history__item`), but a FRESH v3 file would have
  had none, and local search drives `name_lc LIKE` on every query the UI issues. Restored
  in the fresh `CREATE` list, deliberately NOT re-issued by the migration (a migrated file
  already has that index, and a duplicate `CREATE INDEX` is a hard parse error).
* `raw_json_of` inferred a plain `String` from the row cell, so a NULL `raw_json` answered
  as rusqlite's type error instead of "no stored copy". The v0.6.0 mirror was the first run
  to exercise `item <checked-gone id> --cached`, and it printed `Invalid column type Null
  at index: 0` — the same exit `4` as not-found by the contract's own sharing, but the wrong
  answer. Bound `Option<String>` like every stamp reader in the file, so the honest door
  prints `item 127555 has no stored copy: fetch it with sync --stage detail --ids 127555`.
* The icon queue's pending rule gained the NON-EMPTY url terms from the start: under a
  NULL-only rule the four measured empty-`icon_url` records would have kept pending forever
  (the stage fetches nothing for them, marks nothing), which is v0.5.2's price trap again.

### The fixture grew the same grounding

`tests/fixture_api.py` now embeds the two REAL JPEGs the grounding above measured (the
`inv_mace_03.jpg` and `inv_axe_11.jpg` bytes, fetched once and committed base64'd — CI
never asks a host for pixels), answers `/icon/<name>.jpg` on its own server, and makes one
record's `icon_url` name a FILE it answers `404` — the icon stage's third state inside CI.
Detail prices carry a per-process `RUN` offset (the ETag salt moves with it): one fixture
process is one observation window, so a fresh write in a SECOND process grows the ledger by
exactly one sample while a re-ask inside one process still writes none — ADR 0007's rule
becomes a measurable CI claim instead of a promise.

### CI grew the third stage

The committed smoke sequence gains the icons step (twelve steps now) before the card and
stats reads, and the header comment says what each new step proves.

### The numbers, not adjectives (every audit re-run against the shipped tree)

| audit | expectation | observed |
| ----- | ----------- | -------- |
| CI mirror (fixture process A, fresh DB, budget 8, twelve steps) | copy lands, gone file checked once, ledger counts fresh writes | icons `requests 5, items 4, written 3, missing 1, rate_limited 1, complete true`, message `icon crawl copied 3 icons; 1 answered 404 for the icon file and are checked as no-icon-upstream`; samples seq 1..4; the three reuse steps each `written 0, unchanged 2` and the count stayed 4 |
| fixture process B (new observation window) | one fresh write, one new sample | `detail ids 25`: `requests 2, written 1, rate_limited 1` → ledger 4 → **5**; the card answers `history` with the old 20271 and the new 15311; a targeted `--ids 25` icon visit re-copied (a human ask, two requests, pending queue unaffected) |
| icons run on a drained queue | asks nothing | `requests 0, complete true`, `icons stage: 0 icons copied (0 requests)` |
| DB witnesses after the twelve | three states visible in rows | 5 items, 4 detailed, 1 checked-gone; `icon_at` on ids 25/40/12809 with `1945 b` blobs (magic `0xff 0xd8 0xff … 0xff 0xd9`, the grounded JPEG's own ends), `icon_missing_at` on id 60; both queues pending **0**; `sync_state` three rows `complete` |
| exit-code contract (18 doors) | the new doors land on the codes | `usage 2 · 2 · 2` (`sync --stage detailx` quotes `detailx`; `item` without id; unknown verb), `3` on a dead host for detail/list/icons (the icons line names `http://127.0.0.1:9923/icon/inv_mace_03.jpg`, which also proves `icon_target`'s base-relative resolution), `4` twice (`no item 127555 upstream`, and the cached door's honest text), `0` for status/search/card/idle stages; a bare `sync` runs the list stage (v0.2.0 default, unchanged by this release) |
| migration, user's own v1 file (copied; hashed read-only before/after) | the chain lands, the file on disk untouched | v1 → `schema_version 3`, `19 258` rows, `7` detailed, `price_history` 0, `sync_state` list row `stopped … page 170` intact, index set gains only `price_history__item`; second `status` re-reads 3 without re-migrating; hash `7442814b…` and mtime `2026-11-19 21:42:54` identical before/after |
| migration of a v2 scratch catalogue | same legs, one step | 19 257 rows, 29 detailed, 57 checked-gone, `price_history` 0, `sync_state` intact |
| refusal door kept closed | a newer file still refuses | schema-9 scratch file → exit `4`, `database on disk uses schema 9, this build wants 3` |
| FK probe (SQLite 3.54.1) | the guard is Rust's order, not a clause | `item_id INTEGER REFERENCES item` accepted, `INSERT … item_id = 999` with no such row **succeeded** |
| GUI launch audit, shipped binary (probe removed) | nothing crawls by itself | exactly four verbs `status/watch/stats/status`, zero fixture hits, fresh DB gains schema 3 and nothing else |
| GUI pull-all launch audit (boot probe, fresh fixture + DB) | one click chains three stages | three `sync` verbs with ticker `status` polls interleaved; fixture log 14 requests, **3 × 429** (one per stage) and **2 × 404** (the removed record + the gone icon file); DB 3 icon copies, 1 checked no-icon, ledger 4, pending 0 both queues, `sync_state` three rows `complete`; probe removed and the tree rebuilt before the docs commit |
| `cargo build` at the tag | warnings are the plan, not surprises | exactly the six dead-code warnings (`search_url`, `SearchResponse`, `catalogue`, `open_db`, `ApiResponse.remaining`, `PendingSql`'s privacy) plus the incremental-cache hardlink notice on this filesystem |

The user's DB (`HOME/.local/share/wow-goldtool/wow-goldtool.sqlite`, 19 258 rows) was
never written by any of this: every run above pointed at a scratch file through `WGT_DB`,
and the file itself was read hashed/`sqlite3 -readonly` only.

---

## 2026-11-20 · v0.5.3 — patch: a record the upstream dropped is a state, and the crawl survives it

### The measurement, not a hypothesis

The v0.5.2 build run against the **live** API on a scratch DB whose list walk had
already stored 3000 ids:

```
$ cargo run -- sync --stage detail --budget 3
fetching item details: this is the slow stage, about two records per second
item 41 answered 404
$ echo $?
3
```

The detail stage read every non-200/304/429 status as `HttpKind::Failed`, and a failed
record is a network error: the run aborts, exits `3`, and leaves the rest of the queue
untouched. The ids that answer that way are not exotic — the live catalogue reports
`build: wow_classic_beta-1.60.1.70532`, and `41` is `OLDRecruit's Belt`, `42`
`OLDSquire's Belt`, `46` `OLDFootpad's Belt`; the same `OLD…` / `Deprecated …` /
`… (TEST)` name shapes sit inside the user's 19 258 stored rows. The upstream answers
with a body, and the body is its own words:

```json
{"error":"not_found","message":"No item 5828 in forever.","status":404}
```

So the "Pull all info" flow shipped in v0.5.2 — one click that is supposed to retire
every pending record — **cannot finish on the real catalogue**: it dies at the first
removed record. The fixture never showed this because it answers every id. Fixing the
crawler has to land **before** v0.6.0's price history and icon copies, which are the
things that read what a finished crawl produced.

### The decision: a third state, not a fourth retry

[ADR 0006](./DECISIONS/0006-checked-missing-third-state.md) is the record; the shape is
one column and one rule. `classify` grows `HttpKind::Missing`, the crawl marks the id
(`SQL_MARK_MISSING`, guarded by `SQL_ROW_OF_ID`) and counts it in the report (`missing`),
then walks on. `SQL_PENDING` becomes `detail_synced_at IS NULL AND detail_missing_at IS
NULL`, so a removed record costs **one request per crawl** rather than one per run, and
"nothing pending" stays literally true. Four alternatives were rejected in the ADR:
staying fatal (the real DB would never finish), skipping silently (every later run re-asks
the same dead ids and the crawl spends budget on records that cannot arrive), a staleness
cadence ("nothing pending" becomes a lie; nothing measured says a removed record returns),
folding the fix into v0.6.0 (ships a crawl that cannot finish next to the features that
depend on it).

The mark never invents a row: `SQL_MARK_MISSING` is an `INSERT … ON CONFLICT` in the
codebase's idiom, so the existence check in front of it is what stops an un-listed id from
becoming a name-less phantom record. And the stage-2 upsert now writes
`detail_missing_at = NULL` — a record that answered is not a missing record, so the two
stamps cannot both be set.

### The schema step that was not optional

The column is a schema change, and `check_schema`'s honest refusal would have refused the
user's file. Refusing is right for a file this build cannot understand, but here it would
mean asking a human to delete a 19 258-row catalogue because one column was added. So
`SCHEMA_VERSION` moves `1 → 2` **with** a migration: when the file says `1`, `ALTER TABLE
item ADD COLUMN detail_missing_at INTEGER` runs in place and the stamp in `meta` moves in
the same call. A file **newer** than this build is still refused, because that direction
cannot be guessed at. Both fresh-DB stamps now write `format!("{}", SCHEMA_VERSION)` —
without that, a v2 file written by this build would have refused itself on the next open.

### Where the pending arithmetic got checked

Numbers, not adjectives. After the first live run on the migrated DB: `3000` listed =
`10` fetched + `3` checked-gone + `2987` pending, and the report read `requests 6, items
3, missing 3` with **exit 0**. A second run asked four ids and got `missing: 1` — the newly
fetched ids were `47, 48, 49`, i.e. the three already-marked ids were **not** re-asked, and
the new 404 is a fourth removed record, not a re-ask of an old one. That is the difference
between "keeps crawling" and "silently wastes budget", and the arithmetic proves it.

### The UI says what the third state is

The status card gained a row (`checked as gone upstream · N answered 404, not re-fetched`)
and the pull-all note gained its clause: the intro explains that a dropped record is
checked once and counted as gone, and the completion note names how many of the listed ids
were checked as gone, beside the honest sentence about NULL prices. The numbers come from
`stats`/`status` coverage, so the card and the CLI report read one aggregate.

### Audit

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| v1 scratch DB made by the v0.5.2 build (3000 rows) migrated by this build | rows survive, stamp moves | `status` exits `0`, `3000` rows, `schema_version 2`, `detail_missing_at` column present, the old `list` cursor (`stopped`, page 30) intact |
| fresh DB written by this build | does not refuse itself | `schema_version 2`, second open exits `0` |
| live detail run, budget 6 | a 404 does not abort | `requests 6, items 3, missing 3, complete false`, **exit 0**; DB `10` detailed, `3` missing, pending `2987 = 3000 − 10 − 3` |
| second live run, budget 4 | a marked id is not re-asked | `requests 4, items 3, missing 1`; newly fetched ids `47, 48, 49`; the three marked ids absent |
| `item 41` on the live API | the exit-code contract's answer for "no record" | exit `4`, message `no item 41 upstream: the forever catalogue has no record for it`, mark re-stamped |
| fixture mirror (virgin port, thirteen steps) | the third state inside the mirror, pacing intact | detail run `requests 6` (one forced `429` retried in place), `items 4, missing 1, complete true`, message ends `; 1 answered 404 and are checked as removed`; `item 127555` → **exit 4**; the crawl that follows `requests 0, complete true`; DB `5` items, `4` detailed, `1` missing, pending `0` |
| exit-code audit, seven probes | the new case lands on `4` | `2 · 2 · 3 · 4 · 4 · 4 · 0`, and `detail_missing_at` set for `127555` |
| committed CI sequence with the new fixture | still green | 11/11 `exit=0`, DB carries `1` missing |
| GUI pull-all launch audit (virgin port, temp boot probe) | the flow completes with a removed record inside it | verbs `status/watch/stats/status` then `sync` × 2 with the ticker's `status` polls interleaved, then the probe's `item`; fixture log: crawl's single `404` for `127555` plus the probe's targeted `404`; `sync_state` detail row `complete … 1 answered 404 and are checked as removed`; DB `4` detailed, `1` missing, pending `0`; probe removed afterwards |
| warnings on a clean build | the planned dead-code set only | `PendingSql` privacy, `SearchResponse`, `search_url`, `remaining`, `open_db`, `catalogue` plus the filesystem's hardlink notice — the three new SQL constants and both new helpers are used, so nothing new goes quiet |
| build from a cleared incremental cache | green, and it is what the notice says | the filesystem under the project (`/run/media/w/...`) cannot hard link, so the incremental cache came back with a corrupt dep-graph and rustc 1.94.0 ICEd in `setup_dep_graph`; `rm -rf target/debug/incremental` then a fresh `cargo build` exits `0` with the planned warnings. The workaround is environmental, not a code change: every audit below ran on that binary |

### Deliberately not done in this tag

No re-check cadence for removed ids ([ADR 0006](./DECISIONS/0006-checked-missing-third-state.md)
rejects it); no price-history table and no icon copies (those are v0.6.0 and they read what
a finished crawl produced); no index for the new pending filter. Also still open for v0.6.0:
the live API has only answered `[]` for `sockets`, `socket_bonus`, `classes` and `effects` on
every id probed so far (the scratch walks found no level-60 epic gear in the first 3000 ids,
and the live id space has shifted since the user's crawl — `40` is `Recruit's Boots` there,
`Mossborer` in the DB), while `stats` is grounded as
`[{"name":"Intellect","value":4},{"name":"Spirit","value":3}]`. The item card will render the
grounded shape and print the raw payload for the rest, rather than invent a shape.

---

## 2026-11-20 · v0.5.2 — patch: pending's real rule, the client-build stall, the one-click catalogue

### The ask

"instead of the 'sync pending details' make it into a pull all info… basically
run the 'Re-check online' for every item that doesn't have all its information
in the DB yet… one big run and then never have to touch it again." A button that
retires pending work is only honest if pending means *work that exists*.

### Pending: the rule had to carry the truth

`SQL_PENDING` asked `price_min IS NULL OR detail_synced_at IS NULL`. The first
clause was never a pending rule: `docs/DATABASE.md` distinguishes fetched-with-
NULL-price (the item exists, it is **not on the AH**) from never-fetched on
purpose, and `price_min` is read field-by-field out of an optional object, so a
fetched record can legitimately carry a NULL price. Keeping such an item pending
means the detail stage walks it again forever — the stage never terminates on the
real catalogue, and no pull-all flow can promise "nothing pending". The user's DB
(read-only query: 19258 items, 7 with details, 0 fetched-and-unpriced at this
hour) cannot expose the trap yet, because only 7 records have ever been fetched:
the clause is latent, and it becomes loud the moment a real catalogue item answers
without AH data. It is fixed **before** the flow that depends on it ships, not
after that flow has lied. `detail_synced_at` already encodes "has a detail record",
so pending is now that one predicate.

### The GUI never reached the API

Driving a full `runPullAll()` from the boot probe made the next failure loud: the
stage verb entered Rust and then nothing happened, and the fixture access log stayed
empty for the whole session. Markers at each verb step pinned the stop point: the
verb logged the DB-open marker and never reached the request site — it stalled inside
`build_client()`. The DB was never the lock: status polls completed normally between
those marker lines, and the page stayed interactive throughout.

Mechanism read from the crates: reqwest's blocking `Client::new` does not build a value
on the spot. `ClientHandle::new` (`blocking/client.rs:1424` spawns a thread named
`reqwest-internal-sync-runtime` that builds its **own** single-threaded tokio runtime
(`runtime::Builder::new_current_thread()` at `:1428`) and `rt.block_on`s the request loop
(`:1468`); the thread that asked for the client then parks while waiting for that
thread's startup handshake (`wait::timeout(spawn_rx, None)` at `:1476`), and that wait
polls the future and `thread::park()s` the caller (`blocking/wait.rs:58`). Headless that
is exactly the CLI's shape, and every CI mirror has always worked; in the GUI the verb
body sits on a tokio worker of tauri's async manager (v0.5.1's `async fn cli_call`), and
parking there while the client's private runtime has to start is the nested-runtime case:
measured as the stall at `build_client`, zero requests issued.

Fix: the command stays `async` — v0.5.1's wrapper decision is untouched, the IPC handler
still answers immediately and the page keeps ticking — and the verb body hops to a fresh
`std` thread, the runtime-free home the headless CLI proved good. `join` occupies one
tokio worker, which the webview never borrows. A verb that panics on that thread becomes
an `AppError` instead of leaking `Result`'s `String` payload, so the envelope stays the
one shape the UI knows. The helper sits at the crate root because that is where both doors
reach it without an import cycle.

### The flow itself

`btn-detail` becomes `btn-pull` ("Pull all info") and `runPullAll()` chains the two
stages to their ends: the list walk resumes from `sync_state` and runs to completion,
then the pending queue is drained to completion. Orchestration only — ADR 0005's one-
command door, the chunk budgets, the cancel flag and the DB's progress truth are
untouched. For the chain to know a stage finished, `runSync` returns an outcome
(`complete`/`cancelled`/`stalled`/`failed`/`already-running`): Cancel stops the stage in
flight and the flow stops with it rather than jumping ahead, because the next click
resumes the list from the stored cursor and the pending queue recomputes which details
remain. The same change re-orders the chunk's two checks (`complete` before `requests
=== 0`): a stage with nothing left to fetch answers `complete: true, requests: 0`, which
is a legal completion, not a stalled crawl — the old order called it "made no progress".

The completion note refuses to overpromise. "Nothing left to pull" means every listed
item has a detail record; a NULL price on a fetched item is *not on the AH*, not missing
work; and prices are live data, so later syncs still make sense — they revalidate cheaply
(a finished walk restarts at page 1 and every page answers `304` from the stored ETags).

### Audit

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| pull-all launch probe (shipped code + boot probe, fresh fixture server + fresh DB) | both stages crawl, page alive throughout | `sync` verb → two ticker `status` polls → second `sync` verb → three more polls |
| fixture access log for that run | one forced `429` per stage, retried in place, then payloads | page 1 `429` then `200`, page 2 `200`, item 25 `429` then `200`, items 40/60/12809/127555 `200` |
| scratch DB after the pull | everything fetched, nothing pending | 5 items, 5 detailed, **0 pending**, both `sync_state` rows `complete` |
| boot audit on the shipped binary (probe removed) | nothing crawls by itself | exactly four verbs `status/watch/stats/status`, zero fixture hits |
| CI mirror | eleven steps | 11/11 `exit=0`, and both stages report `rate_limited: 1` with the wait inside the run (`seconds: 1`) |
| exit-code audit | unchanged | `2 · 2 · 3 · 4 · 4 · 0` |
| stop → resume → resync | the cursor contract stays | `--budget 1` stops (`complete: false`), the next run reads `resumed_from: 2` without refetching page 1, and a finished walk restarts at page 1 with both pages `304` (`unchanged: 2`, `written: 0`) |
| clean build | warnings only | the dead-code set already planned for v0.6.0 (`PendingSql` privacy, `SearchResponse`, `search_url`, `remaining`, `open_db`, `catalogue`) plus the filesystem's hardlink notice |

### Deliberately not done in this tag

No engine change for pull-all: it is UI orchestration over the existing stages, so ADR
0005 and the CI mirrors stay untouched. No new progress channel — the rejected `emit`/
`event.listen` note in ADR 0005 still applies, and the ticker remains the mid-chunk
feedback. No price history, no icon copies, no item-card rendering of `sockets`/`stats`/
`classes` (that stays v0.6.0).

---

## 2026-11-19 · v0.5.1 — patch: the crawl no longer owns the page

### The report

During a sync list run the whole app was frozen: hovering a button stopped
highlighting, clicking anything took several minutes. The measurements agree
on the length: one GUI chunk is 3.5 s (fixture) / ~5 s (real) at the 2/s
pacing — and a 429 with a `Retry-After` sleep inside the chunk stretches it
to minutes. The "several minutes" is that wait, charged to the page rather
than to the crawl.

### The cause, read from the crate source

`#[tauri::command]`'s wrapper kind decides it. A bare `fn` gets the macro's
blocking body, and that body's last acts are `kind.block(result, resolver)`
(`tauri-macros/src/command/wrapper.rs:425`) then `return true` (`:428`): the
verb runs synchronously **inside** the webview's IPC handler — and that
handler is the page's message loop. So hit-testing (CSS hover), click
dispatch and any later IPC message can only run when the verb returns. A
hover that stays dead for exactly one chunk is that stall.

### The fix: the command itself

`cli_call` becomes `pub async fn`. `body_async` hands the future to
`respond_async_serialized` (`wrapper.rs:~372`): tauri's tokio-backed async
manager — tokio `rt-multi-thread` is a required dependency of tauri 2.x,
not a feature switch, so the manager exists in this build — and the IPC
handler returns immediately. Same argv, same dispatch in `cli.rs`, same
envelope: only the wrapper kind flips (sync → async). The crawl still runs
synchronously inside its own future (pacing blocks one tokio worker and
none of the webview); tokio logged no complaint about the blocking calls,
stderr stayed all `gui:` lines.

Cancel granularity deliberately unchanged (chunk ends, ADR 0005): this
release is about the page, not the crawl — and "always honour `Retry-After`"
is policy, so the wait still delays Cancel; what must not delay is the page.

### Liveness made visible

The UI gains a ticker: while a chunk's await is pending, every second the
status card re-reads the DB (coverage grows per page in list, per item in
detail) and the sync note carries the chunk's elapsed time. Honest about
position on purpose: `sync_state` moves only at chunk ends (`finish()` in
src/sync.rs), and ADR 0002 keeps progress truth in the DB — so the note
prints elapsed seconds, not a mid-chunk cursor that would be a lie.

### Audit

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| boot audit after the fix | unchanged from v0.5.0 | four verbs `status/watch/stats/status` |
| probe launch: 12-id targeted chunk, timestamps added for the run (probe removed before commit) | status polls **inside** the chunk await | `sync` verb at t=…268, ticker polls t=…269…293 one per second — the Retry-After stretch with the page alive |
| tokio slow-task warnings | none | stderr still only `gui:` lines |
| exit-code audit | unchanged | `2 · 2 · 3 · 4 · 4 · 0` |
| CI mirror | unchanged | 11/11 `exit=0` |

**Amendment (v0.5.2)**: the probe row above attributes the visible stretch to a
`Retry-After` wait inside the chunk. The v0.5.2 audit shows that probe's crawl never
issued a request from the GUI at all — it stalled at `build_client`, so no wait was ever
reached from the webview. The liveness witness itself stands (the `status` polls
interleave while the verb runs, and that is what the fix is about), and the wrapper
decision it recorded is unchanged; only the cause of the stretch was attributed wrong.

Rejected for mid-crawl feedback: a JS progress channel (`emit`/`event.listen`)
— those are plugin commands and ACL-gated; the exemption recorded in ADR 0005
would stop covering them (an `app:core:event` grant would be needed), and
progress truth is already the DB.

---

## 2026-11-19 · v0.5.0 — targets, analytics, the card: what the buttons finally answer

### The question this release answers

v0.4.0's sync buttons could only crawl "everything": the list stage, and the whole pending
queue. The watch list had existed since v0.2.0 as something that only **printed**. The
release starts where the roadmap arithmetic already insisted: 19 k details cost 2.7 hours,
so the only useful question is "which details do I actually need?" — and that question
belongs to the user, not the crawler.

### A latent bug found by the first run that read those columns

`sync --ids 25,40` exited 4 on the first audit, and the message was rusqlite's:
`Invalid column type Null at index: 0, name: detail_etag`. Three single-column lookups in
`db.rs` were written `Ok(value) => Ok(Some(value))`, which makes rusqlite infer a **bare
String** for the getter, so a NULL column is an error instead of a value. The signatures
always claimed `Option<String>`; nothing had ever read a NULL through them, because the
pending stage fetched an item before it ever asked for that item's ETag. The targeted stage
reads `detail_etag_of` for every id, so the latent wrongness surfaced immediately. Asking
for the Option type makes NULL a `None` — which is what the type always promised.

### Chunking targeted ids in JS, not in the engine

The engine's `--budget` truncates from the head of the list, so a second chunk of the same
`--ids` would re-visit what the first chunk already fetched and the UI's chunk loop would
never reach the tail. The UI slices instead (20 ids per call), which also keeps Cancel
meaningful between chunks and one slice far under the 120-request burst. `--budget` still
bounds the pending crawl, where it means "items per run".

### Honest accounting, twice more

A targeted run answers its own question: `remaining` is the target list, not the pending
queue ("left in the list" ≠ "left pending"), and `complete` means the target set was fully
visited. The detail stage now sends the stored `detail_etag` when one exists, so re-asking
for the same ids is a `304` — the audit reads `requests 2, written 0, unchanged 2` on the
second pass over 25,40. docs/API.md counts both sides the same way (a `304` costs rate
budget, saves bandwidth and DB churn) and the sentence about "pending items have no ETag
yet" is now scoped to what it actually is: the pending stage.

### The card reads the DB, and offline stays the point

`item ID --cached` sends no request: the card answers from `raw_json` and prints
`detail_synced_at` so the user can tell today's prices from last month's. A record never
fetched is `not-found`, not an empty card — the UI turns that word into an offer, and the
first use of the `NotFound` error kind retired its dead-code warning. No item icons are
rendered: `icon_url` is a remote path, and hotlinking it would trade the app's whole
purpose (answering offline) for decoration. Quality colour carries identity instead; a
local icon copy is roadmap work.

### Analytics before any crawl

`stats --limit 6` is one verb, and its JSON (coverage, by-quality, extremes) renders as
bars in the same palette as the search chips, cheapest/priciest extremes, and one sentence
about the spread. The question "is this worth hunting" is answerable before the detail
stage has seen a single item.

### boot() as the audit's witness

Startup reads status, the watch list and the analytics, so no card paints empty — and the
Rust log shows four verbs (`status`, `watch`, `stats`, `status`). The fourth line is the
witness that the analytics **render** ran to the end: a JS throw after the awaited verb
would leave three lines, not four. CI still cannot open a window, so the eleven-step CLI
mirror (six new steps: `watch add`, a targeted fetch, the same ids again answering 304s, a
watch-list target, the DB-only card) is what guards this release.

### Audit table

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| `sync --ids 25,40` (first pass) | both written | `requests 2, written 2, unchanged 0, complete true` |
| `sync --ids 25,40` (second pass) | revalidation, nothing rewritten | `requests 2, written 0, unchanged 2` |
| `sync --watch` after `watch add 25 40` | target = the watch list | `visited every targeted record: 2 fetched` |
| `--ids 25,x` | usage | exit 2 |
| `item 999 --cached` | not fetched yet, not a crash | `not-found`, exit 4 |
| GUI launch | window + four verbs from `boot()` | `gui: ipc verb status/watch/stats/status` |
| CI mirror | eleven steps | all `exit=0` |
| exit-code audit | unchanged | `2 · 2 · 3 · 4 · 4 · 0` |

### Deliberately not done in this tag

No price history yet: the DB stores one price per item, so a trend line needs a second
table (roadmap). No icon download. No item-card rendering of `sockets`/`stats`/`classes`
values (they arrive as unshaped JSON, and inventing a shape is not our call to make).

---

## 2026-11-19 · v0.4.0 — the shell: `build.rs`, `window.__TAURI__`, chunked cancel

### The question that shaped the release

v0.3.0 proved the pipeline headless. The next question is the one the IDE asks first: does
`cargo run` open a window at all? Everything in this release is downstream of reading the
crate sources in the registry cache instead of tutorials — every tutorial assumes `create-tauri-app`
plus `tauri dev`, which [ADR 0001](./DECISIONS/0001-flat-cargo-layout-no-cli.md) forbids.

### The trap that cost the most time

`cargo build` failed with `error: OUT_DIR env var is not set, do you have a build script?`,
raised by `tauri-codegen/src/embedded_assets.rs:335` while expanding `generate_context!`. The
obvious fix — an empty `build.rs` — compiles, opens the window, loads the page **and leaves
`window.__TAURI__` undefined**, so every button in the UI fails silently. The reason is one
line up the pipeline: `tauri-codegen/src/context.rs:412` injects the global-API scripts only
when `tauri_utils::plugin::read_global_api_scripts(out_dir)` finds a list file that
`tauri_build::build()` writes. So the fix is not "a build.rs", it is "the build.rs that runs
`tauri-build`", which is why `tauri-build = "2.7.6"` is a build-dependency and why
`tauri.conf.json`'s `withGlobalTauri` means anything at all. A GUI feature that can fail
silently is a design smell, and it is why the audit below ends in Rust log lines.

### ACL: measured, not assumed

The gate is `tauri/src/webview/mod.rs:2080`:

```rust
if (plugin_command.is_some() || has_app_acl_manifest || !is_local)
  && request.cmd != crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND
  && invoke.acl.is_none()
```

Ours is none of the three: `cli_call` is an app command, the page is a local origin, and
`has_app_acl_manifest` is false — `acl/mod.rs:353` defines it as `acl.contains_key("__app-acl__")`
and the build's own artifact `gen/schemas/acl-manifests.json` carries 10 plugin keys with no
`__app-acl__` in it. The app command is reachable **by construction**, and the evidence is a
file the build itself produced. The trap that follows is recorded in ADR 0005: the moment an
app-side ACL manifest lands, `has_app_acl_manifest` flips and a missing grant becomes a hard
failure.

### Cancel without `AppState`, because v0.3.0 already made it possible

ADR 0002 says the progress truth is the `sync_state` row, and v0.3.0 made `sync_state.page` an
absolute cursor with resume legal for `stopped` and `cancelled`. That makes chunked sync free:
the UI issues `sync --stage <stage> --budget 10` (list) or `--budget 25` (detail) in a row until
the report says `complete`, each chunk ends with a `sync_state` write, and a JS flag read at the
next chunk boundary *is* cancel. There is no `state.rs`: nothing mirrors the cursor in Rust, so
nothing can drift. The cost is honest — cancel waits for the chunk, and a `304` still costs one
request of rate budget (the v0.3.0 note), so a chunk of list pages is cheap in bandwidth, not
in budget.

### The icon is generated, so the blob is explainable

Codegen reads `bundle.icon` at compile time (`CachedIcon`), so `cargo build` fails without the
PNG. `tools/gen_icon.py` draws it in PIL from fixed constants — navy plate, gold coin, rising
price line — supersampled 1024 → 256 with LANCZOS. Nothing is downloaded or pasted: the script
sits next to the committed PNG, so a fresh clone builds and a reviewer can see how the icon was
made.

### The version source, once more

`tauri-codegen/src/context.rs:255` computes `package_version` as `config.version` **else**
`env!("CARGO_PKG_VERSION")`. The shipped conf therefore omits `version` entirely: `Cargo.toml`
is the only version source, and the house rule that used to say "bump both files" was amended
rather than broken silently.

### Launch audit — display server: KDE Plasma on Wayland (`wayland-1`), XWayland `:1`

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| `cargo build` with the new files | still one buildable unit | four errors, each fixed at the source of the message: `OUT_DIR` (build.rs + `tauri-build`), `cannot find macro __tauri_command_name_cli_call` and `__cmd__cli_call` (→ `generate_handler![crate::commands::cli_call]`, because `handler.rs:166` rewrites the last path segment), `E0382` partial move of `inv` (→ match on `inv.fixture.clone()`) |
| `WGT_DB=/tmp/opencode/fixture.sqlite cargo run` | window opens, page loads, JS reaches Rust | `gui: wow-goldtool shell starting…` → `gui: the webview asked for tauri://localhost (window main)` → `gui: ipc verb status (0 words)` — the page-load log proves the asset map served `index.html`, the ipc line proves the awaited `core.invoke` round-tripped |
| webview console | not a channel | `console.log` added to `ui/app.js` never appeared; grepping `tauri`/`wry`/`tauri-runtime-wry` for console forwarding found nothing on Linux, so the observable audit channel is Rust's `gui:` lines and the UI states its own rendered-field count in the status card |
| CI mirror + exit codes + v0.3.0 audits after the `cli.rs` refactor | unchanged | smoke mirror all `exit=0` (it is a zsh script: run with `zsh`, `bash` mis-parses the `$=step` split), exit-code audit `2 · 2 · 3 · 4 · 4 · 0`, `resumed_from: 2`, `unchanged: 2`, `item 25 → 304 reused true` |
| `window.__TAURI__` presence | proven by the round trip above | the empty-`build.rs` trap above is exactly the state that produces no `gui: ipc …` line |

### Deliberately not done in this tag

No packaging yet (`bundle.icon` and `identifier` are in place for it). No `state.rs`. No
detail-sync target selection from the UI yet — the buttons run the whole stage with chunk
budgets, and choosing watchlist/ids is v0.5.0. The webview's first paint was not reviewed
pixel-by-pixel: KDE is Wayland, `grim`/`kscreen-shot` are absent and X11 `import` cannot see
Wayland windows, so the visual check is a `cargo run` for the user to run.

### Follow-ups parked for later

* `.deb`/`.AppImage` once packaging is worth doing — the identifier and icon groundwork is done.
* Whether a second window (`app.windows` with `templates`) is worth the config surface.
* `tests/api.rs` / `tests/pipeline.rs` unit tests are still a plan, not code.

---

## 2026-11-19 · v0.3.0 — reuse, resume, cancel: the second crawl

### The question that shaped the release

The v0.2.0 crawler only ever crawled forward from scratch. Re-opening the app would
re-download everything it already knew. The upstream had told us how to avoid that
(`ETag` + `If-None-Match` → `304` **with no body**) — so the release is built around
what a body-less response forces us to admit.

### What a 304 forces, and what the v0.2 code would have done

The old loop fell through to `parse_page(body)` for every non-429 status: a `304` would
have died as a decode error on an empty body. Every reuse path now branches **before**
parsing. Because a list page has no DB row to re-read, each page also caches its
`has_more` and `count` in `meta` (`list-page-<n>-etag/-more/-count`) — the crawl needs
exactly those two facts to decide "walk on?" and keep `total` honest across 304s.

The detail side is opposite: the record is already on disk, so `raw_json` is the stored
copy and `item 25` revalidates the row instead of echoing a fresh payload. A `304`
**without** a stored copy is a real inconsistency — reported as an honest decode error,
never papered over. And a fresh `200` from a live lookup is now written through the
same upsert the sync stage uses, so its ETag can be reused next run.

### Honest accounting, twice

- A `304` still costs one request against the burst: revalidation buys bandwidth and
  DB churn, **not** rate budget. Written into docs/API.md so nobody reads "reuse" as
  "19 k fewer requests".
- `pages` counts a run; the `sync_state` page column must be an **absolute** cursor for
  resume to mean anything. `finish()` took an explicit `cursor` parameter — the two
  numbers were quietly the same value in v0.2 only because every run started at page 1.
- The report grew `resumed_from` and leans on `unchanged`: a run of two 304 pages
  honestly reads `complete: true, items: 0, unchanged: 2`.

### Resume only for pauses, not for completions

`stopped` and `cancelled` resume at stored page + 1. `complete`/`ceiling`/`idle` restart
at page 1 — a completed crawl has nothing to resume from, and the ETag walk makes that
cheap. Unknown status strings fall to the same branch, so a hand-edited DB cannot wedge
the crawler.

### Cancel without signal juggling

No atomics, no FFI `sigaction`: a plain `CancelFlag` struct that the loops check before
every request. The GUI will own one in AppState (v0.4.0) and raise it from the UI; the
CLI hands out a fresh one because Ctrl-C already leaves the DB consistent (every write
is its own statement) and the next run resumes by construction. A cancelled run records
status `cancelled` and exits **0** — the user asked for it, nothing failed.

### Fixture audit — every claim executed, never asserted

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| list `--budget 1` | stopped, 1 page, forced 429 retried | `requests 2, pages 1, rate_limited 1, complete false` |
| list (resume) | `resumed_from 2`, page 1 not refetched | `requests 1, resumed_from 2, complete true` |
| list (full re-sync) | both pages `304`, nothing written | `unchanged 2, written 0, items 0` |
| detail | 429 + retry + 5 records | `requests 6, written 5, rate_limited 1` |
| `item 25` | `304`, record served from stored `raw_json` | `status 304 reused true` |
| exit-code audit | 2 usage · 3 network · 4 DB · 0 healthy | all six cases re-ran green |

The fixture grew conditional-GET behaviour itself (stable per-endpoint ETags, real 304
responses with no body): a server that cannot answer `If-None-Match` would have made the
release untestable, so the test asset learned the protocol at the same commit.

## 2026-11-19 · v0.2.0 — data layer, sync engine, contract checks

### One buildable unit, eight modules

The data layer and the sync engine landed together because they are one compile unit:
`main.rs` declares every module, so a half-written tree does not build. Splitting further
would have broken the "every commit must build" rule, so the code ships as one
`feat(core)` commit and the docs/CI split into separate commits after it.

### rustc as the arbiter, twice

The session had a display quirk: tool output and echoed payloads mangled `::` into `.`
and angle-bracket generics into parentheses. Never patched Rust from displayed text;
byte counters in python verified file contents and rustc decided what was right. A
multi-file "apply rustc suggestions" script also overwrote files **inside `~/.cargo`**
(`rusqlite`'s and `serde_json`'s extracted sources); the fix restricted the applier to the
diagnostic's primary file, and the corrupted crates were restored byte-identical from the
registry cache tarballs.

### Two bugs the compiler cannot see

- **SQL arity drift**: the detail upsert declared 38 columns but its VALUES run had 40
  markers — rusqlite fails at *runtime*, rustc stays silent. A checker script cross-verified
  column names, VALUES markers, SET assignments and the `*_VALUE_COUNT` consts; 40 fixed to
  38.
- **WHERE-chain builder**: it emitted `a AND b AND ` with no keyword — SQLite answered
  `near 'LIKE': syntax error`. Now the builder owns its keyword: `format!("WHERE {} ",
  joined)`.

Verified along the way: bundled rusqlite 0.43 really does accept the
`CREATE INDEX name ON table (cols)` spelling that docs/DATABASE.md documents — the open
question from v0.1.0 is closed.

### The fixture caught a lie in the report

`sync --stage detail` against the fixture reported `done: 4 / total: 5` next to
`"complete": true`: a 429 hit the `continue` of the per-item loop, which dropped that item
from the run, and `complete` was hardcoded. Fixture forced this by answering the **first
request of each stage** with 429 + Retry-After. Fixed the semantics: retry the same item
in place (up to 3 retries after each Retry-After), leave it pending if that still fails,
and derive `complete` from the remaining pending count — otherwise report `partial`.
Re-run: `requests: 6, written: 5, rate_limited: 1, complete: true`, exit 0.

### Exit-code audit (ADR 0004)

| Scenario                     | Observed | Expected |
| ---------------------------- | -------- | -------- |
| unknown verb / unknown flag  | 2 / 2    | 2        |
| unreachable base URL         | 3        | 3        |
| corrupt DB via `--db`        | 4        | 4        |
| unwritable DB path           | 4        | 4        |
| healthy `status`             | 0        | 0        |

### House rule amended mid-flight: the version bump

"Version bumps happen in `Cargo.toml` **and** `tauri.conf.json` together" cannot run yet:
`tauri.conf.json` joins the tree at v0.4.0 (it is what `generate_context!` reads). The
rule now means "every version field that exists today"; `Cargo.toml` bumps to 0.2.0, and
the conf-file half of the bump arrives with the GUI shell.

## 2026-11-19 · v0.1.0 — scaffold, measurements, decisions

### Setting the stage

The project started life as a default RustRover binary (`hello world`, git history empty,
branch `master`). Before writing any application code I pinned down the two things that
decide the whole architecture: **what the data source can actually do** and **what the
toolchain can actually build**.

### Toolchain probe (before writing a line of app code)

| Check | Result |
| ----- | ------ |
| `rustc` / `cargo` | 1.94.0 (2026 snapshot) — comfortably newer than Tauri's 1.81 floor |
| Node | v22 present, but deliberately **not** used (see below) |
| `tauri` crate resolves | 2.12.7 (`cargo add tauri@2` locked 427 packages, incl. webkit2gtk/gtk stack) |
| `tauri` + webkit2gtk + gtk compile on this box | ✅ probe crate built in 61 s |
| `rusqlite` / `libsqlite3-sys` | already in the cargo cache, `bundled` feature available |
| `reqwest` 0.13 | default TLS is rustls (`aws-lc-rs`), so no OpenSSL linkage assumption needed |
| Desktop | KDE Plasma on Wayland + XWayland (`WAYLAND_DISPLAY=wayland-1`, `DISPLAY=:1`) |

The probe crate is thrown away; its only purpose was to prove the GUI layer can build
before I spend effort on it. Lesson learned early and written into the house rules:
**probe the toolchain, then design**.

### The hard constraint that shaped the layout

The app must compile straight out of RustRover. That killed the usual
`create-tauri-app` + `tauri dev` route, which needs the Tauri CLI and a Node toolchain to
generate the config context and the asset map. Reading the actual crate sources in the
cargo registry cache answered the question precisely:

* `tauri-macros` documents `generate_context!` as reading `tauri.conf.json` **relative to
  `CARGO_MANIFEST_DIR`, at compile time**;
* `tauri-codegen/src/context.rs` then reads `config.build.frontend_dist` **from the
  filesystem during compilation** and generates a `phf_map!` of assets by itself
  (`EmbeddedAssets::new(phf_map! { … })`).

So the plain `cargo build` path already embeds config + UI assets. The Tauri CLI is only
needed for hot reload and installer bundling. That made the flat layout
([ADR 0001](./DECISIONS/0001-flat-cargo-layout-no-cli.md)) strictly better for this
project than the `src-tauri/` convention.

### API measurement day (TheWoWDB, all figures below are measured, not assumed)

* `GET /api/v1/forever/item/25` → full record for *Worn Shortsword*: `price.available 11`,
  `price.market 400`, `price.min 290` copper, `vendor_sell 7`, `weapon{1-3 dmg, 1.9 s,
  dps 1.1}`, `quality "common"`, faction price buckets present but empty for that item.
* `GET /api/v1/forever/search/?q=Worn Shortsword` → grouped hits with per-kind `has_more`
  and a `more` URL.
* Paging: `per_page` is clamped server-side to **100** (asked 200, got `per_page: 100`);
  `page` runs to 200.
* Total item count is **not** reported on large lists — only `has_more` + `links.next`.
  Binary search over pages located the last page carrying data: **page 193, 57 records,
  `has_more:false`** ⇒ ≈ **19 257 items**. Ids are sparse (first 25, last 288 340), so
  id-space iteration is useless: crawl by page, not by id.
* Docs confirm caching rules (`ETag`, ~1 h public cache, tooltips 5 min, `304` on
  `If-None-Match`) and the rate model: **120 req burst per IP, refilled at 2/s**, `429` +
  `Retry-After` + `X-RateLimit-Remaining` past that.

Cost arithmetic written down before designing the sync, because it decides the design:
193 list requests ≈ 2 minutes, a full detail pass ≈ 19.3 k requests ≈ **2.7 hours**. A
single "sync everything" button is therefore unusable as designed → stage the sync, make
it resumable, cancellable and opt-in ([ADR 0002](./DECISIONS/0002-two-stage-sync.md)).

### Decisions recorded as ADRs

| ADR | Verdict in one line |
| --- | ------------------- |
| [0001 flat Cargo layout, no Tauri CLI](./DECISIONS/0001-flat-cargo-layout-no-cli.md) | RustRover must build the app with plain `cargo build`; `generate_context!` already does the embedding |
| [0002 two-stage sync](./DECISIONS/0002-two-stage-sync.md) | list crawl is cheap, per-item detail crawl is 2.7 h → stage, resume, cancel, opt-in |
| [0003 rusqlite over the DB](./DECISIONS/0003-sqlite-rusqlite-bundled.md) | one disposable file with sort orders the upstream API cannot express |
| [0004 headless CLI in the same binary](./DECISIONS/0004-headless-cli-mode.md) | the data pipeline must be testable without a display server |

### Deliberately not done yet

No application code in this tag. `src/main.rs` is still the generator's `hello world`, so
`cargo build` passes and CI would be green: the scaffold is documentation-only.

### Follow-ups parked for later

* Packaging: `.deb`/`.AppImage` generation without the CLI (or a documented
  `src-tauri`-compatible shim) — roadmap item after `v0.5.0`.
* Retail version (`/api/v1/retail/...`) as a second catalogue tab; `forever` is pinned for
  now (see [docs/API.md](./API.md)).
* Whether item tooltips (`/api/v1/tooltip/...`) should be fetched lazily on demand instead
  of reconstructed from the detail record.
