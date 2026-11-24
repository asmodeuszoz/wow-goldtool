# ADR 0031 — the Search table OWNS its list surface: the icon cell rides BETWEEN the id and the name, and its icons grow 24 → 32 px under an id-scoped rule

**Status:** decided (v0.17.0, 2026-11-24)
**Refs:** the user's tenth ask ("in the search tab show the icons between ID and name instead. Also
make those icons a little larger"), [ADR 0027](./0027-a-cards-sizes-are-scoped-to-the-window.md)
(a SIZE is a surface fact — the id-scoping mechanism this era rides for the 32 px),
[ADR 0030](./0030-a-list-rows-name-is-its-window-opener.md) (the name cell is the opener button —
this era moves the columns AROUND it, not the click), [ADR 0014](./0014-the-icon-doors.md)
(the `item ID --cached` door and the `--icons` bytes ride PUT),
[ADR 0019](./0019-the-sync-tabs-stage-buttons.md) (note strings move with the surfaces they describe),
[ADR 0011](./0011-ci-asserts-only-what-is-true.md) (the webview NEVER renders in the build room)

## The question

The icon column sat THIRD — after the name. The ask names its position (`between ID and name` — an
absolute placement, not a swap-with-one-neighbour hint that leaves "which neighbour?" open: the ask
states both neighbours) AND its size (`a little larger` — a notch, not a jump). Both are SEARCH-LIST
facts: the census below measured WHERE each lives and what else rides the same class.

## The census, measured off the tree at `fecb6c2`→`5c068d5` BEFORE one line moved

| the fact | measured where | what it meant |
| -------- | -------------- | ------------- |
| thead (`ui/index.html:107`) | `<tr><th class="col-id">id</th><th class="col-name">name</th><th class="col-icon">icon</th>...` | header labels ride the CELLS' order (table layout is positional); the ask's "instead" moves one th |
| `hit_row` (`ui/app.js`) | cells drawn `col-id → namecell(button+view+watch) → iconcell(icon_cell) → qualitycell → pricecell` | the body row answers in the SAME order; the move = the SAME reorder in BOTH places, one era |
| the header CSS rule (`app.css:227`) | `th.col-id, th.col-name, th.col-icon, th.col-quality, th.col-price {` — ONE joined rule, class-keyed | order-INDEPENDENT: the look rides the classes; NO CSS line is needed for the MOVE itself (measured: the rule never mentions position) |
| the icon sizes (frame census) | plain `.listicon` 24 px (hits rows + Recipe material/cooked cells), `table.mini .listicon` 20 px, `#recipe-card .listicon` 72 px | the class is SHARED by four surfaces — growing it moves the Recipe LIST and the mini analytics too; ADR 0027's id-scoping is the frame's OWN mechanism for "grow ONE surface" |
| `td.iconcell` | `width: 34px; padding: 3px 6px` base, `table.mini` 26 px | a 32 px box in a 34 px cell leaves 22 px of content box — the cell must ride the SAME notch (44 − 12 padding = 32 content exactly) |
| the click | `button.pipbtn` sits INSIDE `td.namecell` (ADR 0030's opener) | the column move must NOT touch the button, the handler, the door — the affordance rides its own cell wherever that cell sits |
| the note (`search-note`) | names "the icon column shows the DB's OWN copy..." — no POSITION words | the note stays PUT for the move (it never claimed an order); only the icon-cell comment grew |

## The decision (two facts, two commits)

* **The MOVE** (`feat(ui)`, ONE commit): the thead's `col-icon` th moves before `col-name`, and
  `hit_row` draws its `iconcell` td before the `namecell` td — the SAME reorder in both, because a
  table's header labels sit ABOVE the cells positionally. ZERO CSS rule moves (the header rule is
  class-keyed — measured above); ZERO handler/door/button line moves (the button lives in its own
  cell; ADR 0030's opener, the delegated split BY CLASS and the `item ID --cached` door ride PUT
  word-for-word — the pair audit proves it: search doors BYTE-IDENTICAL).
* **The SIZE** (`feat(ui)`, ONE commit): TWO NEW id-scoped rules — `#hits .listicon { width: 32px;
  height: 32px }` and `#hits td.iconcell { width: 44px }` — the Search table owns its list surface
  (ADR 0027's lesson: a size is a surface fact, and the id-scoped rule is the mechanism already in
  the frame: `table.mini .listicon` at 20 px and `#recipe-card .listicon` at 72 px ARE the precedent
  of scoping a shared class). plain `.listicon` stays 24 px for the Recipe rows' material and cooked
  cells, the mini stays 20, the recipe card's 72 and its 21 id rules ride PUT. 32 px = ONE notch over
  24 ("a little larger"; 72 px would be the CARD's jump — rejected below); the 44 px cell keeps the
  base rule's 3+6/6 px padding around the bigger box exactly (44 − 12 = 32). ZERO new class, ZERO
  new colour token, ZERO new icon renderer (the SAME `icon_cell` draws; only its box changed).

## Rejected, measured before one line moved (SEVEN)

1. **CSS `order:` on the cells** — table layout has no `order`; and thead and tbody rows are laid out
   as ONE column grid: an order trick on the body alone (or the head alone) mislabels the columns —
   the census above shows both halves must move TOGETHER, and CSS could not have moved one half
   without lying about the header anyway. Rejected.
2. **Move the whole column for EVERY table** (edit the shared draw order) — the mini analytics tables
   (`table.mini`, group rows) ride DIFFERENT theads (measured: `group | id | name` and `icon |
   min price` — neither has this column order); the ask names the Search tab. Rejected on scope.
3. **Grow `.listicon` to 32 px unscoped** — the Recipe material cells + cooked cells (3 778 of them
   over the user's file) and the search/stats mini rows would grow too; ADR 0027 refused exactly
   this measure at v0.13.0 and its id-scoping mechanism is the frame's own. Rejected.
4. **`transform: scale(1.33)` on the icon** — paint without layout: the 34 px cell keeps its box, the
   glyphs would overflow their padding and neighbours (the same rejection ADR 0027 recorded for the
   card's 72 px). Rejected.
5. **Grow the icon WITHOUT widening `td.iconcell`** — measured: a 32 px box needs 32 content + 12 px
   padding = 44; leaving 34 px would squeeze the padding to 11 px per side against the row rhythm —
   the frame's cells keep their padding (measured in the base rule). Rejected; the cell rides the
   SAME notch.
6. **56 px (the Item card's `.iconimg` size)** — that is the CARD's icon, more than double the list
   box; the ask says `a little larger`. 32 px = 24 × 1.33, ONE notch; the card sizes (72/39 px) stay
   history. Rejected.
7. **Move the column in `hit_row` only, thead PUT** — the header labels sit above the cells positionally
   (table layout), so `icon` would head the NAME column and `name` head the icon: the header LIES about
   every row. The thead and the row move in ONE commit for exactly this reason. Rejected.

## What grows and what does NOT

* Grows: the thead cell order (ONE th), the `hit_row` cell order (ONE td moved), TWO id-scoped CSS
  rules + their WHY comment, and the moved-with comment sentences in `hit_row` (ADR 0019's note
  discipline — the search NOTE itself stays PUT: it never named an order, measured above).
* Does NOT grow: any Rust line (ZERO — the door's bytes and the exit map both prove it), any `SQL_`
  const, any schema line (`8` stays), any handler/door/button line, the OTHER tables' theads or rows
  (mini stays `id | name` and `icon | price` — measured PUT again in the post-tag census), the Recipe
  rows' 24 px icons, the mini's 20 px, the card's 72 px + 21 id rules, `--limit 40` on the Search tab,
  `data-tab` attrs (STILL 15), nameplates (STILL 7), the joined opener rule (STILL ONE).

## Consequences, kept honest

* Tags never move; ADR 0027 gains a GROWN pointer (its id-scoping mechanism carries a NEW surface's
  size this era); ADR 0030 gains a RIDDEN pointer (the name-opener still answers byte-identically:
  the pair's search doors 188 == 188 B on the fresh window, through-ride measured).
* The webview NEVER renders in the build room (ADR 0011): the 32 px icon's DOM cost is NOT WITNESSED
  and NOT CLAIMED; the era's UI witnesses are parse/grep ONLY (`node --check` PARSE OK, thead/row
  orders grep-true, the TWO `#hits` rules, the 24/20/72 px PUT-siblings grep-true).

## Chain

`f064e2c` feat(ui) (the icon cell rides BETWEEN id and name — ONE file `ui/index.html` + ONE `ui/app.js`
thead/row move, classes and looks PUT) → `fe1b278` feat(ui) (the Search table's icons 24 → 32 px +
its cell 34 → 44 px under TWO id-scoped rules; ZERO new class/token) → `5c068d5` feat(cli,db) (the
cost, see ADR 0032) → `96496cb` feat(ui) (the cost column/line + note + argvs) → `446f85d` feat(ui)
(the cost's paint: JOINED selectors, ONE new `.costnote`) → `4b6c024` chore(ci) (52 step-lines) →
`ef9da01` chore (banner `0.16.0 → 0.17.0`, lock follows in the SAME commit) → this record carries
`v0.17.0` (annotated tag).
