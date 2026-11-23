# ADR 0028 — a hits tab shows the WHOLE answer: the cap that bounded the Recipes tab dies, because the bound was the TAB's and the door already answers the rest

**Status:** decided (v0.14.0, 2026-11-23)
**Refs:** ADR 0022 (the paging doors ride ONE composition rule — "no `--limit`" means **the rest**,
and this era makes the tab ride that same rule), ADR 0021 (the recipe doors and their tab: the tab
asks the door's OWN argv, so the click and the cargo line stay one code path), ADR 0014 (one code
path — the tab and the CLI stay ONE door, no second renderer), ADR 0012 (the door is LOCAL, ZERO
requests — the fixture's GET tally cannot move and the unchanged tally IS the locality proof), ADR
0015 (money words beside copper, NEVER beside a count — 3 778 reagent counts stay money-free), ADR
0004 (an EMPTY answer is rc `0` on the door's own words, never a `4`), ADR 0011 (CI states only
measured truths; the launch-audit rule: the webview never renders in the build room).

## The ask, and what the frame measured before it agreed to

The ask: "recipes tab, get rid of the limit for how many hits are to be shown and just show all at
all times." Measured off the tree at `v0.13.0` BEFORE one line moved, the cap was found WHERE IT
LIVES — and it never lived in the door:

| the bound lived here | its OWN words | measured |
| -------------------- | ------------- | -------- |
| the tab's control | `<select class="field" id="recipes-pager">` with options `25` / `50` / `100` "a page" | the tab could never show more than **100 of the set's 1 240** rows, and had no way to ask for more |
| the tab's argv | `var argv = ["recipes", "--limit", limit, "--icons"];` | `recipes --limit 100 --icons` answers **100 rows of matching 1 240** at 1 363 515 B — the door HID 1 140 rows |
| the door | `recipes_rows` through ADR 0022's `paging_clause` (no `--limit` = the rest, riding SQLite's OWN negative-limit token) | `recipes --icons` with NO bound answers **ALL 1 240 rows of matching 1 240** at 16 696 238 B (3 399 938 B asked without `--icons`); over the fixture's 12 rows it answers **12 of matching 12** at 46 793 B; and `--limit 1240 --icons` answers the **SAME 16 696 238 B** — a bound AT or above the matching count changes NOTHING |
| the Search tab's control | `argv.push("--limit", "40");` (hardcoded, no select) | a SECOND bound, in a different shape, on a tab the ask never named |

So the ask is a TAB fact: the tab was the thing holding the rows back, and the door already answers
the whole set — ADR 0022's rule, measured rather than remembered.

## The decision, and its rejected alternatives

**Kill the tab's cap; keep the door's bound.** `ui/index.html` loses the `recipes-pager` select
(a control with no remaining question to answer), `ui/app.js` asks `["recipes", "--icons"]` with
`--profession` only when one is picked, and the tab's note is moved the same breath (ADR 0019's
"every note moved the same breath" rule): the frame stops saying "showing 100 of 1240" and says
what the tab now does — **ALL the rows this filter answers**. The door keeps `--limit`: a shell
that prints 16.7 MB for the whole set is a fact the CLI must be able to bound ON REQUEST, and ADR
0022's paging doors ride it.

Rejected, each with its measure:

* **Grow the pager instead of killing it** (a `2 500 a page` option, or an "all" option). Measured:
  a bound AT or above the matching count answers the **SAME bytes** (`--limit 1240 --icons` = 16 696
  238 B = the unbounded answer), so a bigger cap does nothing but stop HIDING rows while still
  offering a choice — and the ask said "at all times", i.e. no choice. A cap ≥ matching is a control
  that cannot do what it claims; rejected, and the measure is why the cap DIED rather than grew.
* **`--limit 0` as "all", measured today**: rc `0`, **52 B, 0 rows of matching 12** — zero means
  NONE, not ALL (the door's stated shape, and ADR 0004's EMPTY half). Inventing "0 = all" would
  flip a meaning the door already owns; rejected.
* **`--limit -1`** (SQLite's negative-limit token): measured rc `0` answering 12 of 12 / 1 240 of
  1 240, i.e. it WORKS — but the tab then sends a token it only understands because of the SQLite
  accident under it, and the fact ("this tab does not bound") becomes invisible at every reader.
  Rejected on legibility: ZERO bound is the same answer with a truth attached to it.
* **Removing the CLI's `--limit` flag.** Rejected — it would move ADR 0022 (the `--offset`-without-
  `--limit` pair measured rc `4` → rc `0` since v0.10.1 and named in the CI comment), FOUR CI steps
  that press bounds (`--limit 40` / `--limit 3` / `--profession … --limit 4` / `--limit 3`), the map
  doors measured with bounds (`--limit 0` answers rc `0` / 0 rows), and the Search tab's own bound.
  The bound is the door's DISCIPLINE for shells; the tab simply stops asking for it.
* **Client-side paging / virtual scroll in the DOM.** Rejected — it invents a mechanism the frame
  never had, and any page size is a cap again, which is what the ask refused.
* **Re-aiming `btn-recipes-watch` at a bounded set.** Rejected on ADR 0021's rule, which is stated
  in terms of the SHOWN rows ("the cooked items of the shown rows"): capping it would give "shown"
  a second meaning. The consequence is measured and recorded instead (below), not hidden.
* **A second, unbounded recipe door** (e.g. `recipes-all`). Rejected on ADR 0014's one-code-path
  rule and ADR 0021's tab-door rule: the tab and the cargo line must stay ONE door, or a card click
  and a list row can disagree about what the set holds.

## Consequences, kept honest

* Grows: `ui/index.html` (the select dies, the era's comment names the measure) + `ui/app.js`
  (the argv loses its bound, the note moves) — **19 insertions / 9 deletions, TWO files**. CI grows
  ONE step (**48 → 49 step-lines**, plant one stays, 14 `- name:` steps unchanged): the tab's OWN
  argv `cargo run -- recipes --icons`, ZERO requests.
* Moves NOTHING: ZERO Rust line, ZERO `SQL_` const, ZERO schema line (**`8` stays** — measured on
  the user's own file), ZERO wire struct, ZERO exit code, ZERO money formatter, ZERO request rule,
  ZERO fixture line. The exit map measures it: the 31 doors v0.13.0's map carried keep the **SAME
  totals `0`×19 · `2`×5 · `4`×7** (this era moved ZERO door fact), and the era's own EIGHT shapes
  answer rc `0`×7 + rc `4`×1 (the EMPTY-DB door's own no-row words, never a new refusal).
* The door's answers are **BYTE-IDENTICAL** across the pinned v0.13.0 tree (`b927109`, its OWN build
  green with ZERO named in 1 m 22 s, banner `0.13.0` PUT) and the new tree: **28/28 BYTE-IDENTICAL,
  ZERO DELTAs** over one fresh fixture DB — the nine card doors, the recipes doors in
  BOTH shapes (unbounded `recipes --icons` 46 768 B on BOTH binaries, `--limit 0` 52 B, `--limit
  -1` = the unbounded 12 968 B, `--limit 100 --icons` 46 768 B) and the pre-existing set — plus
  **13/13 md5-equal** through BOTH binaries over the user's own file (cards 25 213 / 7 004 / 9 430
  / 3 596 / 3 453 B, rc `4` on `8800111` at 113 B). And the era's headline door asked over the user's
  whole file is BYTE-IDENTICAL across binaries too: **16 696 238 B, md5 `4f1982d6`, 1 240 rows of
  matching 1 240** (3 778 reagent cells carrying 3 514 icon-byte cells / 3 642 money-word cells /
  3 778 counts carried with **0 money beside a count**; 897 crafted cells carrying icon bytes, 873
  money words, 884 live words; 133 rows state NO crafted item — the Enchanting shape — and 10 crafted
  ids have no row).
* The tab's OWN cost grows, measured as BYTES: 16 696 238 B of door answer and 3 514 + 897 = **4 411
  icon data URIs** for the whole set (vs the ≤100-row cap's ≈1 363 515 B). The webview never renders
  in the build room (the launch-audit rule), so **the DOM and the Tauri IPC cost are stated as NOT
  witnessed** — an honest limit of this era's witnesses, not a claim of a smooth 1 240-row paint.
* The Watch button's reach grows with the rows shown (ADR 0021's rule UNCHANGED, its target set
  not): its ONE batched `watch add` argv carries **1 095 ids of the 1 240 rows** (133 Enchanting
  crafts with NO crafted item and 10 ids with no row are skipped HONESTLY, the row already says so).
  Measured on a **SCRATCH copy** of the user's file (the user's file was NEVER written): rc `0`, 6 036
  B — and the Details door then fetches exactly those records under ADR 0017's pending rule (one ask
  per copied record EVER), so the cost is a user-chosen click, not a click the era grew silently.
* The frame keeps TWO bounds and the record names which is WHICH: the Search tab's hardcoded
  `--limit 40` stays PUT (the ask named the Recipes tab), the Recipes tab has NONE. "Make every
  hits tab unbounded" is a NEXT era's ask, not a surprise slipped in here.
* The next explicit fact: **a bound is a surface fact, and a tab that bounds is lying about its
  own filter.** ADR 0027 said a SIZE is a surface fact; this ADR says the same about COUNTS — the
  door answers, the surface decides, and the surface that hides rows without saying so is a defect
  CLASS, not a design.
