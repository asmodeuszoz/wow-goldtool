# ADR 0030 — a list row's NAME is its window opener: the Search row rides the recipe name's click shape, and the separate `card` chip label dies

**Status:** decided (v0.16.0, 2026-11-23)
**Refs:** the user's ninth ask ("make it so in the search results I can click the item name just
like it is for the recipes"), [ADR 0025](./0025-the-recipe-card-window.md) (its `openrecipe` name-cell
button — "the name cell IS the click target" — this era makes that sentence true of BOTH tables),
[ADR 0029](./0029-the-card-window-floats-over-the-tab-its-click-came-from.md) (the item window and its
row opener: the AFFORDANCE moves onto the name, the opener's class, id, handler split and door stay
PUT word-for-word), [ADR 0014](./0014-the-icon-doors.md) (ONE code path: a name click and a `cargo run
-- item ID --cached` line stay one door), [ADR 0027](./0027-a-cards-sizes-are-scoped-to-the-window.md)
(a LOOK is a surface fact — this era moves one by JOINING selectors, never by copying a look),
[ADR 0019](./0019-the-sync-tabs-stage-buttons.md) (the note moved with the surface), [ADR 0018](./0018-the-panel-frame-and-tabs.md)
(the strip and its ONE truth stayed PUT this era — 7 nameplates, 8 data-tab sections, measured again),
[ADR 0015](./0015-the-money-words.md)/[ADR 0016](./0016-the-price-homes.md) (the money census 0 defects,
the NULLs 0/0), [ADR 0011](./0011-ci-asserts-only-what-is-true.md) (the webview NEVER renders in the build room)

## The question

"Just like it is for the recipes." The Recipes tab already answers by its NAMES: ADR 0025 drew each
recipe row's name cell AS the click target. v0.15.0 (ADR 0029) gave the search rows an opener too —
but measured where the recipe shape is NOT: a separate chip LABELED `card` beside a plain-text name.
The ask names the difference: the affordance belongs ON the name, not beside it.

## The two click shapes, measured off the tree at `00f94ba` BEFORE one line moved

| where the name-cell click lived | measured | what it meant |
| ------------------------------- | -------- | ------------- |
| recipe rows (`#recipe-hits`) | `<button class="openrecipe" data-spell>` in `td.col-recipe` — ONE CSS rule (gold, `background: none`, `border: none`, `padding: 0`, `font: inherit`, dotted underline), hover rule **ABSENT**, label with the recipe's OWN name fallback `|| spell_id` | the name IS the button; ADR 0025's sentence, and the frame's honest fallback shape, both already live here |
| search rows (`#hits`) | `td.namecell` = plain `esc(hit.name)` text FIRST, then `view` anchor, `rowact` watch chip, and a separate `button.pipbtn` chip LABELED `card` — its paint = `.rowact`'s chip-box furniture (`--field-2` fill, `--edge` border, 6px radius, 11.5px, 6px margin) + its OWN gold hover, TWO rules | the opener answers BESIDE the name — the row is not clicked BY its name; the ask refuses exactly this |
| the click halves | `#hits` delegated handler: `dataset.id` gate, `if pipbtn → runCard_from_row` else `toggle_watch`; anything else answers nothing; `runCard_from_row` fills `card-id`, `runCard(false)` = door `item ID --cached` | the delegated pattern and the ONE door are already there — this era can leave them PUT entirely |
| the row's other surfaces | `hit_row` has ONE caller (the Search tab); mini analytics rows draw `namecell` as plain text + a quality chip, NO id button | scope: the ask names the Search list; no other list asked for a name click |
| the strip | SEVEN nameplates, 8 data-tab sections (ADR 0029's shrunken frame) | PUT — this era moves no tab, no window, no closer |

## The decision (three parts, one per truth)

* **The NAME cell becomes the button** (`ui/app.js`): the search row draws
  `<button class="pipbtn" data-id=ID>NAME</button>` FIRST in the cell — where the plain name text
  sat — carrying the SAME `data-id` the watch half carries; the label fallback rides the recipe cell's
  OWN shape (`name || id`). The `card` word and the trailing chip die with it. The `view` anchor and
  the watch toggle stay PUT where they sit.
* **The click's other half stays PUT word-for-word** (ADR 0014 kept): class `pipbtn`, the delegated
  handler's split BY CLASS, `runCard_from_row`, the window's OWN `card-id` field, `runCard(false)` and
  the `item ID --cached` door — no line moved. The AFFORDANCE moved; the VERB did not.
* **The look is ONE joined rule, never a second copy** (`ui/app.css`): the `#recipe-hits tr td
  button.openrecipe` rule gains `#hits tr td button.pipbtn` to its selector list — the frame already
  joins selectors (its OWN `th.col-job, th.col-recipe, ...` header rule is the precedent) — so the
  recipe name and the search name are ONE click shape: gold, `font: inherit`, no box, dotted underline,
  and NO hover on either (measured: `button.openrecipe` never HAD a hover rule; the chip's own gold
  hover died with the chip — "just like the recipes" rides the recipe shape EXACTLY, hover absence
  included). ZERO new rules, ZERO new colour tokens. The rule stays table-scoped on BOTH halves.

## Rejected, measured before one line moved (SEVEN)

1. **Keep the `card` chip AND grow a name button** — one row answers its own window twice through
   two affordances; the ask says "just like it is for the recipes" and recipes answer ONCE, by name.
   Rejected.
2. **Reuse the class `openrecipe` on search rows** — measured: openrecipe buttons carry `data-spell`
   and the recipe handler reads `dataset.spell`; a search row would draw a SPELL id under an ITEM row —
   a lie — and the look's rule is table-scoped on `#recipe-hits` so the CSS work would not shrink; the
   `#hits` handler's class split would lose the opener half. The class names a row's half, not a look.
   Rejected.
3. **COPY openrecipe's six-line look into a new `button.pipbtn` rule** — a second copy of one look is
   the drift ADR 0027's id-scoping exists to refuse (its lesson: a SIZE is a surface fact — so is a
   LOOK); the frame joins selectors instead (header-rule precedent measured). Rejected.
4. **Style the name as a gold `<span>` and answer on the span** — measured every time: a delegated
   handler's `event.target` is the BUTTON, never the cell or a text span (every handler in this frame
   keys on the button's dataset); a span is not a control (no focus, no enabled state) and the affordance
   would be a click on non-control text. ADR 0029 rejected the same measure for the same reason. Rejected.
5. **Keep the chip's paint, swap only its label to the name** — the name would render as an 11.5 px muted
   chip-box: not the recipe shape, not "just like it is for the recipes" — the ask compares the two tables'
   LOOKS. A LOOK is a surface fact (ADR 0027's lesson); a chip is not a name. Rejected.
6. **Widen openrecipe's rule (drop the table prefixes, style every `button.openrecipe` anywhere)** —
   the scoping above is deliberate (both halves table-scoped; mini rows already carry `namecell` with
   NO opener — a widened rule would reach a list that never asked). Rejected.
7. **Give the mini analytics rows name-buttons too** — the ask names the search results; the mini rows
   ride plain text + chips today (measured) and have no id affordance to ride a door on. Scope creep.
   Rejected.

## The cells and the contract, restated where the name-click ships

Money contract: over the door's OWN answers measured AGAIN on fresh 9941 — **0 pairing defects**,
counts 19/19/19 with **none wearing money** (the search door itself carries counts 0, money 0:
`search sword --limit 4` 185 B, `--offset 1` 182 B, byte-identical on the pinned pair). Exit map:
**SAME 31-door totals `0`×19 · `2`×5 · `4`×7**, ZERO new door shape (the name click rides the door
it already rode — unlike v0.14.0's eight new shapes and v0.15.0's ZERO, this era grew ZERO). The
pinned pair against the TAGGED v0.14.0 tree measured **28/28 BYTE-IDENTICAL, ZERO DELTAs + 13/13
md5-equal** (all nine card doors 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B;
unbounded `recipes --icons` **46 793 B = ALL 12 of matching 12 on BOTH binaries**; user file md5s
`stats` be44e700 · `item 25 --cached` 82e349c1 · `search` 5d8e2281 · `watch list` 02b3e228 ·
`recipes --limit 3` bbc17f2d · Blacksmithing c1fe9d13). The user's file UNCHANGED (`sqlite3 -readonly`,
NEVER written, NEVER crawled): schema **8** · 19 258 · 4 602/14 656 · 1 240/1 240 · 134 · 4 231 · 4 344+258
· `vendor_buy` **0** · trend **0** · deleted **0** · `sync_state` 6.

## What grows and what does NOT

* Grows: the row's name-cell button draw + its comment, the handler comment's v0.16.0 sentence, the
  search note's moved words ("each row's OWN NAME is its window opener"), ONE selector joined to ONE
  rule, and the death notices where the chip died.
* Does NOT grow: any Rust line (ZERO — builds named ZERO, clippy the SAME 0 errors / 811 advice), any
  `SQL_` const, any schema line (**8** stays), any exit code, any money formatter, any request rule,
  any fixture line, any CI step (**49 step-lines + plant one stays PUT** — the mirror's OWN parse and
  GitHub's own test-job log both say so), any handler line, any door line, any second look rule, any
  second renderer, any per-row listener, any new window, any `vendor_buy`/`price_trend_normal` producer.
* NEVER claimed (launch-audit rule, ADR 0011): the webview NEVER renders in the build room — the name
  button's DOM/IPC cost is NOT WITNESSED; the era's UI witnesses are parse/grep ONLY: `node --check`
  PARSE OK, the `>card</button>` label count **ZERO**, the name-button draw (class `pipbtn` + `data-id`),
  the ONE joined selector (`button.openrecipe, #hits tr td button.pipbtn` = 1), standalone `button.pipbtn`
  rule blocks **ZERO**, `button.pipbtn:hover` **ZERO** (died with the chip), `.pip`/`.pip-closed` still
  2, nameplates still **SEVEN**, data-tab attrs still **15** (writes rides three), closers still 2,
  `openrecipe` draw still PUT, ADR 0027's 21 id-scoped rules STILL 21, Search tab's `--limit 40` PUT,
  `recipes-pager` 0.

## Consequences, kept honest

* Tags never move: ADR 0025 gains a GROWN pointer (its sentence now covers BOTH tables), ADR 0029 gains
  a CHANGED pointer (the opener's LABEL moved from the `card` chip to the row's name, its LOOK from
  `.rowact`'s furniture to `openrecipe`'s look), ADR 0021/0027 gain RIDDEN pointers (the doors byte-identical
  on the pinned pair; the 21 id rules untouched). Their measured tables stay history.
* The name button answers even on a row whose name cell would be blank: the recipe cell's `|| spell_id`
  fallback shape rides as `|| hit.id` (measured in the recipe draw) — a clickable name is never a blank.
* The strip is UNTOUCHED this era — the ONE truth of ADR 0018 stayed 7 nameplates + 8 sections, measured
  again in the census; this era moved WHERE a row answers, not what the frame shows.

## Chain

`f78a6b0` feat(ui) (the name cell becomes the opener, `card` label dies, handler/door PUT — ONE file
ui/app.js, 14 ins / 7 del) → `239c2a4` feat(ui) (the look rides openrecipe's ONE rule by JOINED selector,
the chip's TWO rules die — ONE file ui/app.css, 15 ins / 14 del) → `fecb6c2` chore (banner `0.15.0 → 0.16.0`
— a FEATURE era moves it; the lock's OWN version line follows in the SAME commit — a house note: the
lock lagged one banner since v0.15.0, this commit settles `0.14.0 → 0.16.0` in one jump) → this record
carries `v0.16.0` (annotated tag).

**FOLD (measured post-tag on the PUSHED tree at `4f3c217`, fresh ports 9921/9951/9981):** the tag's
tree verified through the remote's OWN `^{}` line `d5fab777…` → `4f3c217` (tag object `d5fab777`,
payload 9 559 B), and the pushed tree answers the way every sentence above says, measured AGAIN rather
than cited: mirror **49/49 rc `0`, ZERO NONZERO** (9921 — the driver's OWN parse: 49 step-lines committed,
ZERO step moved, plant one; presses UNCHANGED; **51 GETs / 17 recipe GETs ZERO added** = the locality proof;
census 12/6/4/0/0), the exit map at the **SAME 31-door totals `0`×19 · `2`×5 · `4`×7** (9951, fresh seed +
EMPTY DB + DEAD port; the eight carried shapes re-measure `0`×7 + `4`×1: `--limit 0` answers 0 rows of matching
12 at 52 B rc `0` — EMPTY not 4 — EMPTY-DB rc `4` on the door's OWN no-row words), money **0 pairing defects**
(counts 19/19/19 wearing none), and **the pinned v0.14.0 pair AGAIN 28/28 BYTE-IDENTICAL + 13/13 md5-equal**
(9981 vs the worktree @ TAGGED `c065815`: the item doors `item 25/40 --cached` 4 341 / 4 076 B and the search
doors 185/182 B measured unchanged across the name-click era on BOTH binaries, `recipes --icons` 46 793 B = ALL
12 of matching 12 on BOTH) — the decision above ("the AFFORDANCE moved, the VERB did not") holds on the pushed
tree byte-wise, not just on the era's pre-tag tree. The ADR's ladder row is MEASURED now: this era's OWN chain+tag
run reads **45265336918 SUCCESS**, 5/5 jobs green in **9 min 50 s** (run #52, attempt 1, head = the tag's target;
the linux job CLOSED it at 21:11:08Z — the SECOND linux-closed run in the measured chain, a scheduling fact said
so; the SHORTEST measured run so far — the era grew ZERO CI step; each job 12 end-actions = 11 success + 1 skipped
`__dtolnay_rust-toolchain` inner action, never a workflow step; Ubuntu-libs 7 on the test/quality/linux logs, 0 on
macos/windows; the `test` job pressed exactly **49 `cargo run --` steps**, its `recipes --icons` answer reading
`matching 12` at 46 792 B in GitHub's own log against my mirror's 46 793 B — bytes ride the seed, tallies do not).
Tags stay put: the remote answers **32 tags / 64 lines, ZERO moved** (`v0.16.0` → `4f3c217`, `v0.15.0` → `eaeee89`,
`v0.14.0` → `c065815`). The pushed-tree re-measures (parse/grep ONLY, DOM/IPC NOT WITNESSED): PARSE OK, `>card</button>`
count ZERO, the name draw EXACTLY 1, the ONE joined selector, standalone `button.pipbtn` rule blocks + hover ZERO
(the chip furniture died, measured), SEVEN nameplates + 15 attrs PUT, the 21 id rules STILL 21, `--limit 40` PUT,
`recipes-pager` 0; CI on the pushed tree still 49 step-lines + plant one; the user's file UNCHANGED (schema 8 ·
19 258 · 4 602/14 656 · 1 240/1 240 · 134 · `vendor_buy`/trend 0 — `sqlite3 -readonly`, NEVER written, NEVER
crawled). The fold's OWN push starts the NEXT run — measured in the NEXT era's record (ADR 0011's ladder),
never chased here.
