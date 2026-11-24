# ADR 0034: the LIVE LOOK tab dies — its button, its card, its `runLive` and its handler die in ONE commit, and the live-search DOOR rides PUT word-for-word

**Era:** v0.19.0 (the user's twelfth ask, verbatim: "we don't need that tab, lets get rid of it")

**Status:** ADOPTED — the tab dies COMPLETE, in ONE commit; the door behind it (`live-search TEXT`)
was never the tab and stays. SIX activity tabs answer the strip now (Overview / Search / Sync /
Writes / Recipes / Analytics), ADR 0018's ONE visibility truth kept; ZERO Rust line, ZERO CSS rule,
ZERO CI step, ZERO SQL const, ZERO wire struct, ZERO schema line and ZERO request rule moved.

## The decision

- **A tab dies ALL AT ONCE, in ONE commit.** What a tab IS, in this frame: the strip's nameplate
  button, its `data-tab` section (the h2, the field, the button, the hits table, the note), the
  function that answers its button (`runLive`), and the delegated handler that wires the button.
  Leaving ANY half behind is a lie about the surface: a nameplate to a section that can never show
  (`show_tab`'s loop hides it — ADR 0018's ONE truth), a section no button can turn on (dead DOM,
  18 lines that answer nothing forever), or a handler over ids that no longer exist (`getElementById`
  of a dead id, a click that THROWS). So the removal is ONE logical change and moves all FOUR halves
  together — the commit's own diff is the witness: −19 `index.html` (button 1 + section 18, the
  section rode CONTIGUOUS) and −57 `app.js` (the comment block 8 + `runLive` 48 + handler 1).
- **The door is NOT the tab, and the ask named the tab.** `live-search TEXT` rides PUT word-for-word:
  the verb name, its usage entry (`src/cli.rs`, the usage line's `live-search TEXT` entry), the
  `cmd_live_search` body, its exit trio (`0` answer / `2` no text / `3` network), and the CI smoke
  block's THREE door witnesses (the grounded ask, the two-word percent-shape ask, the names-only
  empty-groups ask) moved ZERO lines. ADR 0009's promise — one human ask = ONE request, upstream's
  own answer printed as it came, ZERO DB written — is a fact of the DOOR, and a shell ask is a
  contract this tool ships; killing it with the tab would move four fact families (the usage words,
  the rc trio, THREE green CI steps, the fixture's grouped `/search/` answer) that the ask never
  touched. The door is also the pacing proof: it is the ONLY green CI line that reaches the grouped
  search endpoint, and the proof "one ask is one request" rides it.
- **ZERO CSS moved for the death.** Measured BEFORE one line moved: ZERO `#live`-id rules exist in
  `app.css`, and every class the card drew is SHARED (`table.mini hitstable`, `.card wide`, `.field`,
  `.act`, `.source` ride the other tabs and cards) — the card's look came from the frame's rules, so
  the frame's rules still serve the others and ZERO look-rule died with the card. ADR 0027's id-scoped
  census holds: the card windows' id rules (the `#item-card`/`#recipe-card` furniture) rode PUT, and
  the dead tab had never owned a rule.
- **The live-PRICE family stays PUT and SEPARATE — this is not the same word.** ADR 0023's `.live`
  / `live-missing` / `live-none` trio, `live_cell` and `live_shown` in `app.js`, the Sync tab's
  `Price: live refresh` button and the card window's `Re-check online` button are the DB's live
  price LAYER, not the live LOOK tab; the census keeps them apart (measured AFTER the removal:
  `live_cell` ×5, `live_shown` ×8, `live-missing` ×1, `live-none` ×1 in `app.js`; `.live` ×7 and
  still ZERO `#live` id rules in `app.css`). A dead look tab says NOTHING about the live layer, and
  saying so is the point: money's ADR 0015/0023 contract rode PUT.
- **The look the tab offered still exists where a record needs it.** The card window's `Re-check
  online` button (a DIFFERENT door — the item card's own live ask) already answers "look online
  again at this record", and the door answers a fresh name hunt from the shell. The ask's own
  sentence ("we don't need that tab") is answered by removing the surface, not by re-aiming it.

## Measured (this era's own witnesses — FRESH ports 9943 / 9983 / 9995 + fresh processes/DBs, the
real host saw NOTHING, the user's file read read-only)

- **The census BEFORE one line moved** (off the tree at the v0.18.0 record): the tab's SEVEN ids —
  `tab-live`, `live-card`, `live-fields`, `live-text`, `btn-live`, `live-hits`, `live-note` — ×1
  each in `ui/index.html` (304 lines), its TEN JS refs (`btn-live` ×5, `live-note` ×3, `live-hits`
  ×1, `live-text` ×1) in `ui/app.js` (1 545 lines), `runLive` ×2 (definition + handler), `data-tab=`
  ×15 (7 buttons + 8 sections), the strip's SEVEN nameplates (6 `class="tab"` + Overview's `tab
  tab-on`); ZERO of those ids exists in `src/` (the Rust names the DOOR — `cmd_live_search`,
  `join_tail`, `search_url` — and the SEPARATE price family `live_*`).
- **The AFTER:** `index.html` 285 lines, `app.js` 1 488 — every one of the SEVEN ids and the TEN
  refs answers ZERO, `runLive` ZERO, `data-tab=` 13 (6 buttons + 7 sections), SIX nameplates,
  `node --check` PARSES `app.js` (the frame's PARSE witness — the DOM is NEVER WITNESSED, the
  launch-audit rule). Era diff **FOUR files / 2 ins / 78 del** (`ui/index.html` 19 del, `app.js`
  57 del, the banner pair 2/2): ZERO `src/` line, ZERO `tests/` line, ZERO `ci.yml` line.
- **Mirror of the LANDED block (fresh 9943, the driver's OWN parse: 52 step-lines + plant one,
  EXECs the already-built binary):** **53 commands rc 0, ZERO NONZERO**, and the fixture's OWN
  tallies end where the crawl left them — **51 GETs / 17 recipe-file GETs, ZERO ADDED** — with the
  category census naming the door ALIVE: detail 18 · icons 5 · tooltip 8 · **`/search/` live-search
  3** · recipe files 17 = 51. The THREE live-search steps answered; the tab was never the door, and
  the door's pacing proof rides the same THREE lines it rode in the v0.17.0/v0.18.0 windows.
- **Exit map (fresh 9983, fresh seed DB + a SECOND DB with a genuinely EMPTY recipe table + a DEAD
  port nothing binds, 9984):** **41 doors at rc `0`×25 · `2`×5 · `4`×11** — the SAME totals the
  v0.17.0 and v0.18.0 maps carried. Composition stated, not remembered: the BASE 31 (nine card
  doors rc `0` + `recipe 132 extra` + `recipe 132 --fixture <DEAD>` + v0.14.0's seven recipes
  shapes + ONE grounded `live-search sword --fixture` ask = 19; FIVE rc-`2` = bare `live-search`
  + `recipe` usage + `recipe abc` + `recipe 132 --fixtures` typo + `recipe-suggest 132` unknown
  verb; SEVEN rc-`4` = `recipe 999999` bare/`--icons` + `recipe 132` bare/`--icons`/`--fixture
  <DEAD>` over the EMPTY DB + `recipes` over the EMPTY table + `item 999999 --cached`) + the FIVE
  `--costs` shapes (`0`×3: `recipes --costs --limit 3` 3 433 B · `recipe 467 --costs` 1 274 B ·
  `search sword --costs` through-ride) + `4`×2 (`recipe 999999 --costs`, `recipes --costs` over the
  EMPTY table) + the FIVE `--learned` shapes (`0`×3: `recipes --learned --limit 3` 3 568 B · `recipe
  637 --learned` 1 507 B · `search sword --learned`) + `4`×2 (`recipe 999999 --learned`, `recipes
  --learned` over the EMPTY table). The live-search door's own two doors rode PUT: bare
  `live-search` → **rc 2** on its OWN usage words, `live-search sword --fixture` → **rc 0 at 1 017
  B**. Through-rides BYTE-EQUAL: `search sword --costs` == `search sword --learned` == `search
  sword` at **188 == 188 B**, and card `recipe 637 --learned` == `recipe 637` at **1 507 == 1 507
  B** — the flags PARSE and the doors that never asked IGNORE them; no rc moved ANYWHERE.
- **Pinned pair (fresh 9995, the v0.18.0 worktree @ TAGGED `70d2f20` rebuilt against the new tree,
  ONE fresh fixture process + ONE fresh DB):** **28/28 doors BYTE-IDENTICAL, ZERO DELTAs** — the
  nine card doors, the recipes doors in BOTH shapes, `status`/`stats`×3, `item 25/40/12809 --cached`,
  the search doors, `watch list`. Plus **13/13 md5-equal** through BOTH binaries over a BYTE-COPY
  of the user's own file (LOCAL doors, ZERO requests): `stats` `be44e700` · `stats --icons`
  `656f0f3c` · `stats --limit 3` `ce12af50` · `item 25 --cached` `82e349c1` · `item 40` `b16828b5`
  · `item 12809 --cached` `ac1e452f` · `status` `b80d8abb` · `watch list` `02b3e228` ·
  `refresh-twins` `77088fd7` · `search sword --where description` `2b7ebedb` · Blacksmithing
  `c1fe9d13` · `recipe 132 --icons` `d41d8cd9` (rc `4` — the message rides STDERR) · `recipe 638`
  `d41d8cd9` (rc `4`). Grown argv THROUGH-RIDES against the PINNED binary: `recipes --icons
  --costs --learned` **47 433 == 47 433 B**, `recipe 637 --learned` **1 507 == 1 507 B**, `recipe
  467 --learned --costs` **1 274 == 1 274 B** — the flag was BORN at v0.18.0, so THIS era draws NO
  pinned refusal; its BEFORE witness is the UI census instead (the pinned frame carries the SEVEN
  tab ids + 15 `data-tab` attrs + SEVEN nameplates, the new tree ZERO / 13 / SIX). Mechanism said
  honestly: the pinned v0.15/v0.16-era windows pressed the user's file directly; THIS era never
  opened the original with a binary — both binaries pressed a `cp` byte-copy in its OWN directory
  (the original's size 44 023 808 and mtime-ns verified IDENTICAL before and after).
- **CI on this tree:** the parse still answers **52 cargo step-lines + plant one PUT**, and the
  THREE live-search step-lines (with their comment lines) rode PUT — ZERO `ci.yml` line moved by
  this era (the door still needs its witnesses; ADR 0011's rule).
- **Gates:** `cargo build` rc `0` naming ZERO at every commit (the incremental hard-link line is
  the FS-artifact class — this repo sits on a file system without hard links — never a NAMED
  warning); `cargo clippy --all-targets` **0 errors / 824 advice PUT** (823 `-->` arrows to `src/`)
  — the advice SET did NOT move, which IS the claim of an era that moved ZERO Rust line.
- **The user's file, read-only (`sqlite3 -readonly`, NEVER written, NEVER crawled):** schema `8`
  · 19 258 items · 0 deleted · 4 602 copies / 14 656 without · 1 240 recipe rows / 1 240 DISTINCT
  spell ids · `learned_json` 1 240 NOTNULL / 0 NULL · kinds `schematic_item` 804 / `trainer` 413 /
  `automatic` 23 (NONE silent) · 804 name a scroll id · **747** of those have an ALIVE row · **477**
  carry the scroll's OWN copy · **0 of 477** carry a vendor/zone word (the honest zero RE-CONFIRMED,
  the tab's death changed NOTHING about it) · watch 134 · `price_min_now` 4 344 + 258 checked-price-less
  · `vendor_buy` 0 · trend 0.
- **Measured defects: ZERO.** Notes (said so, NOT defects): no-argv `[]` LAUNCHES THE GUI (never a
  CLI audit shape); bare `search` answers rc `0` (the empty word is legal); an unwatched `watch
  drop` answers rc `0` (idempotent) — the map rode TRUE usage/refusal doors for those slots.

## REJECTED alternatives (with their measures)

| rejected | why (measured the same day) |
| -------- | ------------------------- |
| die WITH the tab, delete the `live-search` verb/door/fixture/CI steps | the ask named the TAB. The door is FOUR fact families the ask never touched: its usage entry (`live-search TEXT`, `src/cli.rs`), its exit trio `0`/`2`/`3` (the map's bare-`live-search` rc `2` + the grounded ask's rc `0` re-measured HERE), THREE green CI steps, and the fixture's grouped `/search/` answer (its GETs counted in the 51 — the live-search category answers 3). Deleting it would also bury ADR 0009's one-request proof and its audit table |
| HIDE the tab (drop only the button, keep the section + `runLive` + handler) | dead paint + a dead handler: the section's 18 lines stay in the page showing NOTHING forever, and app.js's handler over `btn-live` would wire a click to a card `show_tab`'s loop can never turn on. "get rid of it" says remove, not park |
| re-aim the ask: move the live ask into the Search tab's card | not asked; and it would put the live door INSIDE the DB-look tab, against ADR 0012's two-doors rule (DATABASE.md states it: the local door and the live door "never share a name or a payload"). A live hunt is not a DB filter with an extra button |
| kill the card window's `Re-check online` button instead | a DIFFERENT door (the item card's live ask, `btn-card-live` + `btn-price` live refresh) — census says it rode PUT (index.html 3 hits / app.js 9), and it already covers the live re-check of a record this tab showed. Killing it would move money's live layer with a look-tab death |
| drop the THREE `live-search` CI steps WITH the tab | the steps are the DOOR's pacing + names-only witnesses, and the door is ALIVE (measured: bare rc `2`, grounded rc `0` at 1 017 B). A green line is a proof of a live contract; killing it deletes the proof, not the surface |
| grow a replacement live-ask box somewhere (the Overview tab, a new tab) | not asked; the door + the card window's own re-check button answer it. A new box is a new surface with no question behind it (ADR 0004's species) — and a new `data-tab` would move ADR 0018's ONE truth's count, which moved 15 → 13 DOWN this era, not back up |
| keep `runLive` for a shell-visible helper or an IPC door | app.js's card function exists ONLY to answer the tab's button (the ONE delegated handler at line 1 493 wired it — measured); `gui.rs` reaches the door through `cli_ok`'s argv, never through a JS function; there is no other caller |

## Pointers

- **ADR 0009 RIDDEN (this era):** the door's promise — one human ask = ONE request, upstream's own
  answer, ZERO DB written — rode PUT word-for-word (the verb, the usage entry, `cmd_live_search`,
  the rc trio and the CI block's THREE steps moved ZERO lines), while the PANEL this ADR's "Related"
  line named (`ui/index.html` + `ui/app.js`, the live-search panel) DIED. The audit table's GUI rows
  (the four-verb boot, the reverted live probe) stay AS HISTORY at v0.7.0 — the panel existed there
  and answers nothing here.
- **ADR 0018 GROWN (this era):** the strip answered SEVEN activities through this truth; SIX answer it
  now — SEVEN nameplates → SIX, `button.tab[data-tab]` SIX, `section[data-tab]` SEVEN, `data-tab=`
  attrs 15 → 13 (the dead button and its dead section carried ONE each). The one-at-a-time rule and
  the window machinery (`open_pip`/`close_pip`, no `data-tab`) stayed PUT.
