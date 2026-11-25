# ADR 0029 — a card window is an OVERLAY on the tab its click opened on: the card tabs die, the doors stay put

**Status:** decided (v0.15.0, 2026-11-23)
**Refs:** the user's eighth ask ("make it so the recipe card is no longer a tab but a window
in window when clicking a recipe in the recipes tab — and the same for the search: clicking an
item pulls up a picture in picture item card instead of the item card being a whole tab"),
[ADR 0018](./0018-the-panel-frame-and-tabs.md) (the tab strip and the ONE-truth layout — this era
SHRINKS what it owns), [ADR 0025](./0025-the-recipe-card-window.md) (the card of its own, the
delegated row click, the ONE local door — its window keeps every word of that, its WHERE moves),
[ADR 0027](./0027-a-cards-sizes-are-scoped-to-the-window.md) (the window's 21 id-scoped sizes —
they ride the overlay UNTOUCHED), [ADR 0021](./0021-the-recipe-doors.md)/[ADR 0028](./0028-a-hits-tab-shows-the-whole-answer.md)
(the recipe doors and the unbounded tab argv — PUT, measured again on both sides of the pinned pair),
[ADR 0014](./0014-the-icon-doors.md) (one code path: a click and a cargo line — kept, NOT grown),
[ADR 0004](./0004-headless-cli-mode.md) (the exit contract — the carried doors answer the SAME
totals), [ADR 0015](./0015-the-money-words.md) (money beside every copper, NEVER beside a count —
measured 0 defects over the door's own answers again), [ADR 0016](./0016-the-price-homes.md)
(the NULLs stay 0/0), [ADR 0019](./0019-the-sync-tabs-stage-buttons.md) (a note moves with the
surface it describes), [ADR 0011](./0011-ci-asserts-only-what-is-true.md) (the webview NEVER
renders in the build room — this era grows ZERO CI step)

## The question

"Window in window… picture in picture… instead of the card being a whole tab." A card's WHAT
was decided at v0.12.0 (ADR 0025: one local door, the game's own entry, honest gaps). This ask
is about its WHERE: a card that is a TAB competes with the tab whose row was clicked — the ask
names the cure: the window opens ON the tab that is already in front.

## The frame's tab/card structure, measured off the tree at `e449b60` BEFORE one line moved

| where a card window lived | measured | what it meant |
| ------------------------- | -------- | ------------- |
| the strip | **9 nameplates** (`button class="tab"` ×8 + the painted `tab-on` ×1) | TWO of the nine ARE the card windows — a card was an activity beside its list |
| the sections | **10 sections carry `data-tab`** (21 attrs: 9 buttons + 10 sections; `writes` rides two) | ADR 0018's ONE truth: `show_tab`'s loop keeps exactly one visible and wears `display: none` on the rest |
| the recipe window's reach | `runRecipeCard` starts with `show_tab("recipe-card")` — the frame's ONLY JS visibility call for a card (measured: the item card has ZERO — its nameplate alone made it visible) | opening the card tab HID the tab the click came from: `display: none` gets no events — the frame's own comment above (app.js) says it — so the clicked row VANISHES beside the window that answers it |
| the click paths | recipe rows: an `openrecipe` name-cell button + ONE delegated handler over `#recipe-hits`; search rows: a `rowact` watch toggle + ONE delegated handler over `#hits` | the delegated pattern is PUT; measured: the search rows carry **ZERO** open-a-card affordance today — this era draws the row's second half |
| the window's sizes | **21 id-scoped `#recipe-card` rules** (ADR 0027: `.listicon` 72 / `.noicon` 39 px included) | they are id-scoped, NOT tab-scoped: they keep riding whatever surface the id paints — measured before the move, still 21 after |

## The decision (three parts, one per truth)

* **The two card nameplates die, and the two sections stop naming tabs** (`ui/index.html`): the
  `tab-card` and `tab-recipe-card` buttons die, `#item-card` and `#recipe-card` drop their
  `data-tab`. ADR 0018's loop (the ONE truth above) then NEVER touches a card window: it sees
  **7 nameplates + 8 data-tab sections**, and a window's visibility is held by exactly two
  functions, `open_pip`/`close_pip` (class swaps: the section is born wearing `pip-closed` — the
  frame's OWN `display: none` mechanism under a name that is NOT a tab — and `pip` floats it).
  Each window grows its OWN closer button (`Close this window`): closing hides the window back
  where it was born, and the tab underneath stays exactly where it was — a window never eats its
  tab. The ids stay PUT, so ADR 0027's 21 id-scoped rules ride the floated panel byte-identical.
* **The two clicks land the windows ON the tab they came from — ONE door each, ZERO door line
  moved** (ADR 0014 kept, NOT grown): the recipe half stays EXACTLY PUT (the `openrecipe` button,
  the delegated handler, `runRecipeCard`'s door argv `recipe SPELL_ID --icons`) — only its first
  line moves, `show_tab("recipe-card")` → `open_pip("recipe-card")`. The search row grows the
  opener it never had (measured ZERO today): a `pipbtn` button in the name cell carrying the
  SAME item id the watch toggle carries, and the ONE delegated handler over `#hits` splits the
  halves BY CLASS — `pipbtn` fills the window's OWN field and lands on `runCard(false)`
  (`item ID --cached`: the window answers OFFLINE, ZERO requests, like its door), everything
  else keeps its old half. No per-row listener is planted anywhere (ADR 0025's pattern rides).
* **The paint is the frame's OWN silhouette again** (`ui/app.css`, ZERO new colour token): `.pip`
  floats the SAME `.card` panel over the layout — `position: fixed` + a lit-brass trim
  (`--edge-lit`, the frame's hover trim) + the inner gold hairline + a DEEPER drop shadow than a
  laid panel (0.55 vs 0.45, 16 px vs 10 px), so the window visibly sits IN FRONT of the tab
  underneath: picture in picture. `button.pipbtn` rides the furniture of `.rowact` — the SAME
  cell already carries one button.

## Rejected, measured before one line moved (SEVEN)

1. **Keep the nameplates and only restyle** — the ask refuses the tab by NAME; with nine nameplates
   the strip still answers nine activities and ADR 0018's loop still hides the other eight the
   moment the clicked card's nameplate turns on. A paint-only move leaves the LIE the ask names.
2. **Open the window AND switch the tab underneath** (keep `show_tab` beside `open_pip`) — measured:
   the switch sets EVERY other section to `display: none` (the loop above), so the list the click
   came from disappears below its own window, and `display: none` gets no events (the frame's own
   comment): the row you clicked could never be clicked again. A list that eats its own rows is
   not picture in picture. Rejected — the window's reach has exactly ONE visibility call.
3. **A second card renderer for the window** — drift ADR 0014 refuses; the renderers are already
   there (`show_recipe_card` / `render_card`, measured 2 draws + 2 defs) and the window draws
   through them with the SAME argv. A card's cell would be able to say something different in a
   window than in its panel. Rejected.
4. **A new door or flag for the window** (`card N`, `--pip`) — this ask names WHERE a card opens,
   never WHAT answers: the pinned pair measures the doors **28/28 BYTE-IDENTICAL + 13/13 md5-equal**
   against the TAGGED v0.14.0 tree, so the window IS a paint fact, and a second argv answering the
   same truth is the drift ADR 0014 refuses. The era grows **ZERO CI step**: the committed block
   stays **49 step-lines + plant one** (measured by the mirror's own parse).
5. **Make the whole row cell clickable** — measured: a delegated handler's `event.target` is the
   BUTTON, never the `td` (every handler in this frame keys on the button's dataset already), and
   the cell already carries furniture (`view` link, watch toggle): a cell-level handler would
   swallow the watch half's clicks. An affordance you can SEE answers; a cell that answers unseen
   does not. Rejected for the visible `card` button.
6. **Re-parent the two card sections INTO the tab's DOM subtree** — `position: fixed` floats
   without moving a node; moving would rewrite the loop's `section[data-tab]` selector truth
   (ADR 0018's sentence would stop being true of the layout) and ADR 0027's rules are id-scoped,
   so the move carries ZERO gain behind it. Rejected.
7. **Auto-close a window on the next tab switch** — measured: the opener's ONLY closer is today the
   window's OWN button; auto-close would give a click a second, HIDDEN lifecycle (a window that
   vanishes when anything else is touched answers nothing about why). The ask's PiP window rides
   the tab you are LOOKING at; its closer names itself. Rejected.

## The cells and the contract, restated where the window ships

The money contract rides the renderer that stayed PUT: over the door's own answers measured AGAIN
on fresh 9741 — **0 pairing defects**, **counts 19/19/19 with ZERO wearing money** (ADR 0015's
count half), `vendor_buy`/`price_trend_normal` read **0/0** in the fixture census and stay NULL
(ADR 0016), and the user's file still reads schema **8** · 19 258 items · 4 602 copies / **14 656
without** · 1 240 recipe rows / 1 240 distinct spell ids · `learned_json` 1 240 set · **134**
watched · `vendor_sell` 4 231 · `price_min_now` 4 344 + `price_now_missing_at` 258 · pending 0 ·
tombstoned 0 (`sqlite3 -readonly`, NEVER written, NEVER crawled). The exit map answers the
**SAME 31-door totals `0`×19 · `2`×5 · `4`×7** — this era grew ZERO new door shape (unlike the
v0.14.0 era's eight): the window rides the doors it already rode, `item ID [--cached]` and
`recipe SPELL_ID [--icons]`, whose card answers measure BYTE-IDENTICAL on BOTH sides of the
pinned pair (fixture: 2 124 / 7 330 / 6 673 / 4 107 / 643 / 1 209 / 1 155 / 6 740 / 4 033 B;
unbounded `recipes --icons` 46 793 B = ALL 12 of matching 12 on BOTH binaries).

## What grows and what does NOT

* Grows: TWO dead nameplates' comments moving with the truth, TWO closer buttons + TWO closer
  handlers, `open_pip`/`close_pip` (one definition each, two calls each), the search row's `pipbtn`
  opener + the delegated handler's class split, FOUR CSS rules (`.pip-closed`, `.pip`,
  `button.pipbtn` + its hover), notes/note strings moved where the surface moved (ADR 0019's rule).
* Does NOT grow: any Rust line (ZERO — the era's builds answer named 0, clippy the SAME 0 errors /
  811 advice), any `SQL_` const, any wire struct, any schema line (**8** stays), any exit code,
  any money formatter, any request rule, any fixture line, any CI step (**49** step-lines stay
  PUT), any second renderer, any per-row listener, any second bound (the Search tab's `--limit 40`
  stays PUT, measured 1), any `vendor_buy`/`price_trend_normal` producer.
* NEVER claimed (launch-audit rule, ADR 0011): the webview never renders in the build room — the
  window's DOM/IPC cost is NOT WITNESSED; the era's witnesses are parse/grep ONLY: `node --check`
  PARSE OK, 7 nameplates, 15 `data-tab` attrs (7 + 8), ZERO `data-tab="card"`/`"recipe-card"`,
  ZERO `show_tab("card")`/`show_tab("recipe-card")` call sites, 3 definitions+calls each of
  `open_pip`/`close_pip`, 2 closer buttons, the `pipbtn` census, FOUR new CSS rules, and ADR 0027's
  **21 id-scoped `#recipe-card` rules still 21** with `.listicon` 72 / `.noicon` 39 px measured in
  the census.

## Consequences, kept honest

* Tags never move: ADR 0018 gains a CHANGED pointer (the strip shrank NINE → SEVEN, the windows
  left its ONE truth), ADR 0025 gains a GROWN pointer (its window's WHERE moved), ADR 0027 and
  ADR 0021/0028 gain RIDDEN pointers (their rules/doors ride the overlay byte-identical, measured
  on the pair). Their measured tables stay history.
* The window survives a tab switch BY DESIGN (its closer is its own button, R7 above): the ask's
  PiP window rides the tab you are looking at; the note moved with the surface and says so
  (ADR 0019's rule).

## Chain

`b5bcb25` feat(ui) (the two nameplates die, the windows ride the clicks, closers born — TWO files,
89 ins / 15 del) → `6b4f50c` feat(ui) (the overlay's paint: FOUR CSS rules, ZERO new colour token,
32 ins) → `ff84417` chore (banner `0.14.0 → 0.15.0` — a FEATURE era moves it, ZERO Rust line touched)
→ this record carries `v0.15.0` (annotated tag).

**FOLD (measured post-tag on the PUSHED tree at `eaeee89`, fresh ports 9811/9841/9871):** the tag's
tree was verified through the remote's OWN `^{}` line `5f344fce…` → `eaeee89`, and the pushed tree
answers the way every sentence above says, measured AGAIN rather than cited: mirror **49/49 rc `0`,
ZERO NONZERO** (9811 — the driver's OWN parse says **49 step-lines committed, ZERO step moved**, the
window rode doors the block already answers), stage presses UNCHANGED, fixture tallies ending where
the crawl left them (**51 GETs / 17 recipe-file GETs, ZERO added** = the locality proof), census 12/6/4/0/0;
exit map at the **SAME 31-door totals `0`×19 · `2`×5 · `4`×7** with this era's eight shapes AGAIN `0`×7
+ `4`×1 (9841) and money **0 pairing defects**; **the pinned v0.14.0 pair AGAIN 28/28 BYTE-IDENTICAL +
13/13 md5-equal** (9871, pinned @ TAGGED `c065815` with its NINE nameplates = the BEFORE witness vs the
new SEVEN + `pip-closed`/`open_pip` PRESENT, banners `0.14.0` PUT vs `0.15.0` MOVED) — the decision above
("the window IS a paint fact") holds on the pushed tree, not just on the era's pre-tag tree. The ADR's
ladder row is MEASURED now: this era's OWN chain+tag run reads **45253921925 SUCCESS**, 5/5 jobs green in
**10 min 16 s** (run #50, attempt 1, head = the tag's target; the linux job CLOSED it at 20:05:35Z — a
scheduling fact, said so rather than smoothed; each job 12 end-actions = 11 success + 1 skipped `__dtolnay_rust-toolchain`
inner action, never a workflow step; Ubuntu-libs ABSENT on the windows/macos logs; the `test` job pressed
exactly **49 `cargo run --` steps**, its tab's OWN argv step answering **matching 12 at 46 792 B** in GitHub's
own log against my mirror's 46 793 B — bytes ride the seed, tallies do not). Tags stay put: the remote answers
**31 tags / 62 lines, ZERO moved** (`v0.15.0` → `eaeee89`, `v0.14.0` → `c065815`). The pushed-tree re-measures
(still parse/grep ONLY, DOM/IPC NOT WITNESSED): fresh build 49.07 s named ZERO, PARSE OK, SEVEN nameplates,
15 `data-tab` attrs (`writes` rides three), ZERO card attrs/call sites, TWO closers, the `pipbtn` census, FOUR
`.pip` rules, the **21 id-scoped rules STILL 21** (72/39 px), `--limit 40` PUT, `recipes-pager` 0. The fold's
OWN push starts the NEXT run — measured in the NEXT era's record (ADR 0011's ladder), never chased here.
**CHANGED AT v0.16.0 ([ADR 0030](./0030-a-list-rows-name-is-its-window-opener.md)):** the row's
window opener stayed but its TWO surface halves moved: the LABEL (the `card` word died, the row's OWN
NAME became the button - `name || hit.id` riding the recipe cell's fallback shape) and the LOOK (from
`.rowact`'s chip-box to openrecipe's ONE look by JOINED selector, the chip's TWO rules dead, its hover
with it). The click itself, the class, the delegated split BY CLASS, `runCard_from_row` and the `item
ID --cached` door stayed PUT word-for-word - census again: 28/28 BYTE-IDENTICAL + 13/13 md5-equal against
the pinned TAGGED `c065815` tree.

**RIDDEN AT v0.20.0 ([ADR 0035](./0035-a-pin-rides-the-dbs-own-meta-keys-and-the-pinned-tab-is-the-conditional-eighth-nameplate.md)):** the mechanism this ADR locked — a visibility class that `show_tab`'s loop never
reads or writes — is what the v0.20.0 CONDITIONAL nameplate rides, under a name that is NOT a window: the Pinned
recipes nameplate button AND its card section carry ONE class `pin-off` (app.css's `.pin-off { display: none; }`),
written by `set_pin_visible()` from the DB's OWN local door `pin list` and nowhere else, while `show_tab` stayed
PUT word-for-word knowing only `tab-on`/`tab-off`. Unlike a card WINDOW, the pinned section DOES carry `data-tab`,
so it IS an activity this truth hides or shows — the shape kept is the CLASS mechanism, not the window. The window
machinery itself (`open_pip`/`close_pip`, the `Close this window` closer, sections with NO `data-tab`) rode PUT: a
PINNED row's recipe name opens the SAME recipe window as any other row's, over the tab its click came from.
