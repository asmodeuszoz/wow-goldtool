# ADR 0027 — a card window's sizes are scoped to the WINDOW: the type grows ~1.45× and the icons ×3, and the shared classes stay put where the LIST rides them

**Status:** decided (v0.13.0, 2026-11-23)
**Refs:** ADR 0014 (one code path — a card's cell IS the list's cell, so a size that is not
scoped is a second surface, not a bigger card), ADR 0018 (one activity per tab: a card window
is read once, from a distance, alone), ADR 0025 (the card door and its cells: this era moves
their PAINT, not their bytes or their code), ADR 0015 (the money contract rides every bigger
word), ADR 0011 (CI states only measured truths — the webview never renders in the build room,
so the witnesses here are parse/grep, never a rendered claim).

## The ask, and what the frame measured before it agreed to

The ask: "in the recipe card, when I pull up a single recipe, everything needs to be larger,
the font and the icon need a lot larger too." A card is not a list: it is ONE entry, opened on
its OWN tab, read once from a distance. The frame's own numbers (measured off the tree at
`v0.12.1`, `ui/app.css` quoted below) show the card was drawn with LIST sizes:

| the card rode (BEFORE) | its own words | measured |
| ---------------------- | ------------- | -------- |
| its OWN type | `rcardhead` 13 · `rcardname` 13.5 · `rcardlearn`/`rcardmats` 12 · `rcellname` 12.5 · `rcopy` 12 · `rcopyhead` 10.5 · `learn-word` 10.5 · `learn-none` 11 px | small-type numbers copied from the frame's list blocks |
| its ICONS | `.listicon` — the SAME box the Recipes LIST, the search hits and (via `table.mini .listicon`, 20 px) the mini analytics rows ride; the Item card tab owns a separate box, `.iconimg`, at 56 px | 24 px on the card |
| its `no bytes` stand-in | `.noicon` 13 px, riding the 24 px box at a 13/24 ratio | 13 px |
| its PANEL furniture | `h2` 15 · `.note` 12.5 · `.field`/`button.act` 13 px, `.rcardbox` gap 8 / padding 12+14, `.rcell` gap 3 / padding 7+9, `.rcopy` gap 2 / padding 5+8, border-left 2 px | list-grade furniture |

## The decision, and its rejected alternatives

**Grow the WINDOW, scope every shared size to it.** The card's OWN classes move in place
(13.5 → 20 · 12.5 → 18 · 12 → 17 · 11 → 16 · 10.5 → 15 px, ≈ ×1.45); its icons take THREE
times the box the list uses (24 → 72 px, larger than ANY icon the frame draws today, the Item
card's 56 px included) and the `no bytes` stand-in rides the SAME ×3 at 13 → 39 px — the frame's
own 13/24 ratio times 72 = 39.0, so the box keeps its SHAPE instead of lying about the space an
icon would take; the panel furniture follows the type (h2 15 → 22 · note 12.5 → 18 · field/act
13 → 19 px, gaps and paddings with them). The shared classes are moved ONLY inside the panel:
ONE id-scoped block `#recipe-card .chip`, `#recipe-card .listicon`, … (21 rules). Scoping is
NOT a new mechanism — `table.mini .listicon { width: 20px }` already scopes a shared class to
one surface today, measured put in the same file.

Rejected, each with its measure:

* **Growing `.listicon` itself (24 → 72 px) for everything.** Measured: `.listicon` is drawn by
  the Recipes LIST, the search hits table and the mini analytics rows (`table.mini .listicon`
  overrides it to 20 px). A global 72 px box triples every list row's height: the hits and the
  recipes list are the doors whose PAGING ADR 0022 measured at their own row heights, and the
  ask named a WINDOW, not the frame. Rejected — and it would have broken the frame's own
  `table.mini` scoping fact, which exists precisely because one box serves many surfaces.
* **A NEW class for the card's icons (`.cardicon` drawn by app.js).** Rejected on ADR 0014's
  one-code-path rule: the v0.12.0 block in this same file already states "listicon/noicon:
  no second copy of a money or icon cell". A second icon class means a second renderer that can
  say something different about ONE id's icon — exactly the drift that file exists to prevent.
  Measured after this era: app.js still draws the SAME `.listicon`/`.noicon`, and the card's own
  class census is still the TWELVE names (`openrecipe rcardbox rcardhead rcardname rcardlearn
  rcardmats rcell rcellname rcopy rcopyhead learn-word learn-none`) — ZERO new class.
* **Inline sizes in the JS markup (`style="…"` per cell).** Rejected — the frame owns the style
  end-to-end; sizes inside the JS that draws the cells is where drift becomes invisible, and it
  would move 40+ JS lines for a paint fact that needs ZERO of them.
* **`transform: scale()` on the card box.** Rejected on measure: a transform scales painted
  output, not layout — the grid gaps, paddings and borders keep their untransformed px values,
  so the cells stay at list sizes while the glyphs overflow them, and the pixelated icon
  (`image-rendering: pixelated`, the frame's own rule for copied bytes) lands on scaled
  sub-pixel boundaries. Every size rule in this file is px (measured census: 142 rule blocks,
  all px) — the frame scales in px, so this era scales in px.
* **A root font-size step (`rem`-based scaling).** Rejected on measure: the frame carries 12+
  `font-size` rules the OTHER tabs ride (status grid, hits, analytics, footer); moving the root
  moves every panel, which is the opposite of the ask ("in the recipe card").
* **Reaching the size through app.js at all** (a per-cell class or an inline `font-size`).
  Rejected — ZERO JS line was needed: the sizes are a paint fact, and the door's bytes (measured
  below) prove the door itself moved ZERO.

## Consequences, kept honest

* Grows: `ui/app.css` ONLY — 49 insertions / 10 deletions, of which 21 rules are id-scoped and
  10 rules are the card's OWN classes moved in place. ZERO `ui/app.js` line, ZERO `ui/index.html`
  line (the section already carries `id="recipe-card"` — measured, the tab mechanism owns it),
  ZERO Rust line, ZERO SQL/`SQL_` const, ZERO schema line (schema stays `8`), ZERO exit-code line
  (the trio measures the SAME 31 doors at `0`×19 · `2`×5 · `4`×7), ZERO money-formatter line,
  ZERO request rule, ZERO fixture line, ZERO CI step-line (48 step-lines stay, plant one).
* The ANSWERS do not move at all: the door's bytes are a fact and CSS is a paint fact, so the
  era draws the STRONGEST claim it can — measured on FRESH ports/DBs, the PINNED v0.12.1 binary
  (worktree at `d2c26de`, whose OWN build is green with ZERO named, banner `0.12.0` PUT) vs the
  new tree answers **22/22 BYTE-IDENTICAL, ZERO DELTAs** on one fresh fixture DB (the NINE card
  doors included: 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B, plus
  `status` 1 242 · `stats` 1 773 · `item 25 --cached` 4 341 · `search` 185/182 · `watch list` 8 ·
  the four `recipes` doors 12 993 / 2 185 / 4 634 / 2 174 B) and **13/13 md5-equal** through BOTH
  binaries over the user's OWN 19 258-row file (card doors 25 213 / 7 004 / 9 430 / 3 596 / 3 453
  B, rc `4` on `8800111` at 113 B). Money over the card doors' OWN answers: **0 pairing defects**,
  reagent COUNTS carried with NO money beside them (ADR 0015's count half, unchanged).
* The census that makes this era honest about ITSELF: rustc's named tally stays **ZERO** and clippy
  stays **0 errors / the SAME 811 advice lines** (rc 0) — this era touched ZERO Rust, so a build
  witness here is not a claim about the era, it is the claim that the era moved nothing else.
* The witnesses are parse/grep, NOT a rendered claim: `node --check` parses app.js (rc 0,
  unchanged), the CSS braces balance 142/142, the TWELVE card classes answer exactly the names
  app.js draws, the 21 `#recipe-card` rules are present, and the PUT sizes are still quoted in
  the SAME file (`.listicon` 24 px, `table.mini .listicon` 20 px, `.iconimg` 56 px, `.note`
  12.5 px outside the card, `h2` 15 px). The webview never launched in the build room (the
  launch-audit rule, kept since v0.9.x) — NOTHING here claims to have seen the grown card render.
* The next fact this makes explicit: **a size is a surface fact, not a class fact.** The frame
  already rode that fact for the mini rows (`table.mini`); this era rides it for a WINDOW, and
  the record says WHICH surfaces stay put, with their px values, so the next era can read the
  frame's sizes rather than re-deriving them.
* **THE FOLD, MEASURED (2026-11-23, post-tag, fresh ports/DBs — this era's own ladder row):** the
  decision's strongest claim was measured TWICE, once BEFORE the tag and once on the PUSHED tree, and
  both times it came back the same: the pinned v0.12.1 binary (`d2c26de`, its OWN build green with
  ZERO named in 1 m 22 s, banner `0.12.0` PUT) vs the new/pushed tree answers **22/22 BYTE-IDENTICAL,
  ZERO DELTAs** (fresh port 9451 pre-tag: card doors 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155
  / 6 740 / 4 033 B · fresh port 9501 post-tag: 2 116 / 7 322 / 6 668 / 4 104 / 643 / 1 206 / 1 153 / 6 735
  / 4 031 B — lengths ride the fixture process's OWN seed, the identity does not, and rc totals do not move
  between windows) and **13/13 md5-equal** through BOTH binaries over the user's own file both times (cards
  25 213 / 7 004 / 9 430 / 3 596 / 3 453 B, rc `4` on `8800111` at 113 B). The mirror replay is **48/48 rc
  0, ZERO NONZERO** on FOUR fresh ports (9391 / 9421 / 9461 pre-tag, 9471 post-tag) at the SAME stage presses
  and the SAME GET tallies (**51 GETs, 17 recipe-file GETs, ZERO added** = the locality proof), and the exit
  map holds the **SAME 31-door totals `0`×19 · `2`×5 · `4`×7** on fresh 9407 AND fresh 9481 — nine card doors
  rc 0, SEVEN rc-`4` doors in the SAME words, FIVE rc-`2` doors at the SAME words, rc `3` UNREACHABLE (the
  door sends ZERO asks). Money over those very answers: **0 pairing defects**, reagent COUNTS 2/2/2/2/2 carried
  with NO money beside them, so the bigger words did not buy a bigger CONTRACT. GitHub's OWN verdict for this
  era: **45199646115 SUCCESS**, 5/5 jobs green in **16 min 4 s** (14:56:38Z → 15:12:42Z, run #46, attempt 1,
  ONE run id, no CANCELLED sibling, head `b927109` = the TAGGED tree) — quality **7/7** with the Lint step
  **completed/success**, `test` **8/8** with the CLI smoke step GREEN (the 48-step block, the four card steps
  included), linux 8/8, macos 7/8, windows 7/8 (`skipped` Ubuntu-libs on both non-linux targets); the run reads
  longer than v0.12.x's because the macos job opened 5 min 1 s after the run, a scheduling fact on GitHub's side,
  not a tree fact. The user's file's own tallies answer 8 / 19 258 / 4 602 copies / **14 656 without** / 1 240
  recipes / 1 240 distinct `spell_id`s / `vendor_buy` 0 / `price_trend_normal` 0 / tombstoned 0 through `sqlite3
  -readonly` (LOCAL, ZERO requests, ZERO crawl), so ADR 0010's copy rule and ADR 0016's NULL-vs-0 grounds stay
  TRUE and the card's live chips stay a TRUTH on this host — the two facts the NEXT era inherits, alongside the
  surface-vs-class rule this ADR states. The fold's OWN push starts the NEXT run, measured in the NEXT record
  (ADR 0011's ladder), never chased here.
**RIDDEN AT v0.15.0 ([ADR 0029](./0029-the-card-window-floats-over-the-tab-its-click-came-from.md)):**
the overlay paints UNDER those rules: the `#recipe-card` id stayed, all **21 id-scoped rules** (`.listicon`
72 / `.noicon` 39 px included) ride the floated panel UNTOUCHED — measured in the era's census, still 21 —
and the pinned pair proves the whole card answered BYTE-IDENTICAL across the move. The id-scoping above is
why the move was free: a window wears what the id paints. ADR 0029 keeps the overlay decision.
