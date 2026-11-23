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

## Post-tag (measured FORWARD, folded in the DEVLOG's v0.14.0 entry)

The tag's tree (`c065815`, read through the remote's OWN `^{}` line `868eff3…` → `c065815`) was measured
AGAIN on FRESH ports/processes/DBs rather than cited, and the rule this ADR made holds on the tree that
landed:

* **GitHub's OWN verdict for the era's chain+tag push reads 45217976658 SUCCESS**, 5/5 jobs green in **18
  min 15 s** (16:34:12Z → 16:52:27Z, run #48, attempt 1, ONE run id, no CANCELLED sibling, head = the TAGGED
  tree). The `test` job pressed exactly **49 `cargo run --` steps** — this era's block with the tab's OWN
  argv INCLUDED — and that step answered **12 rows of matching 12** with 19 reagent cells (7 carrying icon
  bytes, 8 money words, **19 counts carried with 0 money beside them** — ADR 0015's count half), 6 crafted
  cells carrying icon bytes and 2 rows stating NO crafted item, at 46 792 B (my fresh mirrors measured 46
  793 B for the SAME argv: the bytes ride the fixture process's own seed, the row tallies and the rc totals
  do not). The Lint step reads **completed/success** at 0 errors / **858** advice on CI's own build — the
  SAME 858 the v0.13.0 fold's run measured, which is exactly what an era that touched ZERO Rust claims; the
  local `cargo clippy --all-targets` number (0 errors / 811 advice) is a DIFFERENT invocation and is quoted
  apart, as ADR 0011's measured-truth rule requires. The run reads longer than v0.13.0's 16 min 4 s because
  the quality job CLOSED it (macos → 16:40:48Z · linux → 16:40:56Z · test → 16:42:20Z · windows → 16:42:27Z
  · quality → 16:52:27Z), a scheduling fact on GitHub's side; each job answers 12 end-actions, 11 success +
  ONE skipped, and the skipped one is the Rust-toolchain installer's OWN inner action
  (`__dtolnay_rust-toolchain.__run_N`), never a workflow step — the Ubuntu-libs step is simply ABSENT on the
  macos/windows jobs. The era's own sibling measured here too: the v0.13.0 fold push's run reads **45204000066
  SUCCESS**, 5/5 in **9 min 56 s** (head `6fd2e3e`), its `test` job pressing **48** steps — the fold's tree
  carries the block this era had NOT yet grown at that head, which is the honest reading of the two heads.
* **The mirror replay on fresh 9611: 49/49 commands rc `0`, ZERO NONZERO** (the driver re-PARSED the committed
  block: 49 `cargo run` step-lines + plant one, ZERO step moved since the era's own ci commit), the stage
  presses UNCHANGED (budget-3 recipes 4 requests / 3 pages / 5 rows / `rate_limited` 1 · resumed-from-3 5 /
  5 pages / 7 rows / complete · a re-pull 8 / 8 pages / **8 answered the SAME copy** / 0 written), the
  fixture's OWN GET tallies ending where the crawl left them — **51 GETs, 17 of them recipe-file GETs, ZERO
  added** — which IS this era's locality claim (ADR 0012: the door is LOCAL), census recipe **12** / item 6 /
  copies 4 / `vendor_buy` **0** / `price_trend_normal` **0**.
* **The exit map on fresh 9631** (fresh seed + fresh EMPTY DB + a DEAD port): the 31 doors v0.13.0's map
  carried answer the **SAME totals `0`×19 · `2`×5 · `4`×7** (ZERO door fact moved, as this ADR claims), and
  this era's OWN eight shapes answer **rc `0`×7 + rc `4`×1**: `recipes` 12/12 at 12 993 B · `recipes --icons`
  12/12 at 46 793 B · `--limit 0` **0 rows of matching 12** at 52 B with rc `0` (the rejected "0 = all" reading,
  measured true where it was rejected) · `--limit -1` 12/12 at the SAME 12 993 B (the rejected `-1` reading,
  measured true where it was rejected) · `--limit 12 --icons` the **SAME 46 793 B** as unbounded (the claim that
  killed the cap: a bound at the matching count changes NOTHING) · `--offset 10` 2 of 12 at 2 174 B (ADR 0022's
  pair, rode not moved) · `--profession Enchanting --icons` 2 of 2 at 4 634 B · `--limit 100 --icons` over the
  EMPTY DB rc `4` carrying the door's OWN "the recipe table holds no row" words, never a new refusal. Money over
  the door's OWN answers reads **0 pairing defects** with counts 19 / 19 / 19 (Enchanting 3) wearing NONE —
  ADR 0015's count half survives the unbounded tab.
* **The pinned pair on fresh 9671**: the v0.13.0 tree PINNED at the TAGGED `b927109` (a worktree whose OWN
  build is green with ZERO named in 1 m 22 s, banner `0.13.0` PUT, the `recipes-pager` select PRESENT — the
  BEFORE witness this ADR's cap census needs) vs the pushed tree over ONE fresh fixture process and ONE fresh
  DB: **28/28 doors BYTE-IDENTICAL, ZERO DELTAs** — the nine card doors at 2 124 / 7 330 / 6 673 / 4 107 /
  643 / 1 209 / 1 155 / 6 740 / 4 033 B, the recipes doors in BOTH shapes (unbounded `recipes --icons` **46
  793 B on BOTH binaries**, `--limit 0` 52 B, `--limit -1` = the unbounded 12 993 B, `--limit 100 --icons`
  46 793 B) and the pre-existing set (`status` 1 242 · `stats` 1 773 · `item 25/40 --cached` 4 341 / 4 076 ·
  `search` 185 / 182 · `watch list` 8 · the `recipes` doors 12 993 / 3 382 / 2 174 / 2 185 / 4 634 B) — plus
  **13/13 md5-equal** through BOTH binaries over the user's OWN file (cards 25 213 / 7 004 / 9 430 / 3 596 /
  3 453 B, rc `4` on `8800111` at 113 B, `status` 1 098 · `stats` 4 554 · `item 25 --cached` 2 949 · `search`
  979 · `watch list` 707 · the `recipes` doors 4 293 and 21 090 B). And this era's HEADLINE door over the user's
  WHOLE file is byte-identical across the two binaries: **16 696 238 B, md5 `4f1982d6`, 1 240 rows of matching
  1 240** — 3 778 reagent cells (3 514 icon-byte cells / 3 642 money-word cells / **3 778 counts carried with
  NO money beside them**), 897 crafted cells carrying icon bytes / 873 money words / 884 live words, 133 rows
  stating NO crafted item and 10 crafted ids with no row; the same set asked WITHOUT `--icons` costs 3 399 938
  B. The claim this ADR stakes — kill the TAB's bound, keep the DOOR's — moves NOTHING that any pinned binary
  answered, which is exactly what a ZERO-DELTAs pair says.
* **The tab's shape on the pushed tree, quoted** (the launch-audit rule: the webview never renders in the build
  room, so these are parse/grep witnesses and NEVER a rendered claim): `node --check` parses `ui/app.js` at rc
  `0`, `recipes-pager` answers **ZERO** in BOTH UI files, the tab's argv reads `var argv = ["recipes", "--icons"];`
  (line 271), the `recipes-profession` select stays ONE, the Search tab's hardcoded `argv.push("--limit", "40")`
  stays PUT (line 192), the workflow answers **49** `cargo run --` mentions, and `Cargo.toml`'s banner reads
  `version = "0.14.0"` while the pinned tree's still reads `0.13.0`.
* **What stays NOT WITNESSED, stated as such:** the Watch button's reach was measured where writing was possible
  — over the whole set its ONE batched `watch add` argv carries **1 095 ids of the 1 240 rows** (133 Enchanting
  crafts with NO crafted item and 10 crafted ids with no row skipped honestly), answering rc `0` / 6 036 B on a
  **SCRATCH COPY** of the user's file while `watch list` there answers **1 129 ids** (the file's 134 already
  watched + 995 new) — but the user's file was NEVER written and NEVER crawled, and the DOM cost and the Tauri
  IPC cost of painting 1 240 rows stay **NOT WITNESSED**: the era measured BYTES and refuses to claim a smooth
  paint. The user's file still reads, through `sqlite3 -readonly`, schema **8** · 19 258 items · 4 602 tooltip
  copies · **14 656 without** · 1 240 recipe rows / 1 240 distinct `spell_id`s · `learned_json` 1 240 set · watch
  **134** · `vendor_sell` 4 231 · `price_min_now` 4 344 + `price_now_missing_at` 258 · pending 0 · `vendor_buy`
  **0** · `price_trend_normal` **0** · tombstoned **0**, so ADR 0010's copy rule, ADR 0016's NULL homes and ADR
  0017's once-EVER pending rule stay TRUE on this host.
* **Tags stay put, read through the remote's OWN `^{}` lines** (30 tags): `v0.1.0` → `8685dea` · `v0.10.0` →
  `e4d3447` · `v0.10.1` → `09c8d77` · `v0.11.0` → `59b0185` · `v0.11.1` → `3f20b51` · `v0.12.0` → `ace683a` ·
  `v0.12.1` → `d2c26de` · `v0.13.0` → `b927109` · **`v0.14.0` → `c065815`** — ZERO tag moved, CI stays **49
  step-lines + plant one**, and the fold leg is docs-only, so it cannot move the build. The run the fold's OWN
  push starts is measured in the NEXT ladder row (ADR 0011's ladder), never chased here.
**RIDDEN AT v0.15.0 ([ADR 0029](./0029-the-card-window-floats-over-the-tab-its-click-came-from.md)):**
the tab's UNBOUNDED argv stayed PUT above — re-measured byte-identical on both binaries of the pinned v0.14.0
pair (46 793 B = ALL 12 of matching 12, `--limit`/`--offset` bounds unchanged at the door) — and the Search
tab's `--limit 40` stays PUT, now measured beside the new window opener (`pipbtn`). ADR 0028's cap census and
its SEVEN rejected alternatives stay history; ZERO bound moved at v0.15.0.
