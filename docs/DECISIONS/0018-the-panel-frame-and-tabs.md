# ADR 0018 — the panel frame and the tabs: the AH silhouette in CSS, one activity per tab

**Status:** adopted (v0.9.7)
**Related:** `ui/index.html`, `ui/app.js`, `ui/app.css` (the whole skin lives here — the
release changes ZERO Rust lines), [ADR 0014](./0014-the-icon-doors.md)/[ADR 0015](./0015-the-money-words.md)
(their rules ride the SAME cells this release only re-frames), [ADR 0013](./0013-the-refresh-door.md)
(the twin-refresh button joins the Writes tab), [ADR 0011](./0011-ci-asserts-only-what-is-true.md)
(what CI can and cannot witness about a webview), [ADR 0004](./0004-headless-cli-mode.md)
(a click and a cargo line stay ONE code path — untouched here), [ADR 0010](./0010-manual-layer-and-tooltip-copy.md)
(the twin's zero-request rule), [ADR 0002](./0002-two-stage-sync.md) (the engine never learns
the window exists — still true), `docs/API.md` (the developer-terms attribution rule), `build.rs` +
`tauri.conf.json` (`frontendDist: ui` is the embed path)

## The observation that forced it

The user asked, in their own words: "I want this app to look like the blizzard default wow
auction house frame. Multiple tabs for different activities instead of cramming everything into
one tab." Today the panel IS crammed: EIGHT cards stacked in one page (`status · search · sync ·
item card · the DB's own writes · twin refresh · live look · price analytics`), with a
two-column ≥980 px grid for three of them — every activity reachable at once, nothing separable,
and a flat dark-card skin with round corners the AH silhouette has never had.

## The decision

**Landed:** the page becomes SEVEN activity tabs on a nameplate strip inside the panel's top edge:
`Overview · Search · Sync · Item card · Writes · Live look · Analytics`. Eight cards map to seven
tabs because the twin-refresh button JOINS the Writes tab — the four DB verbs and the twin
refresh are ONE activity (both send ZERO requests: the twin is re-derived from what the DB
already holds, ADR 0013/0010), so splitting them would be decoration, not an activity.

* **One truth for visibility:** `show_tab` in app.js flips `tab-off` (display:none) on sections
  tagged `data-tab`; exactly one tab's worth is ever visible. Buttons inside a hidden section
  keep their listeners but CANNOT receive clicks (display:none gets no events) — the layout
  enforces the rule, no handler rewrites. The strip rides the SAME delegated-handler pattern
  the hits table already used.
* **First paint never waits on script:** index.html PAINTS the default (tab-overview lit, the
  seven other sections class `tab-off` in the static markup); boot()'s `show_tab` re-asserts
  from the one truth — recorded as paint insurance, not leaned on.
* **The frame is CSS-ONLY:** deep blue-black room with a light falling from the top, AH-panel
  fields with bronze trim + a gold inner hairline, the AH's corner ORNAMENTS as three borders
  (`main::before` top-left L, `main::after` bottom-right L, `pointer-events:none` so they cannot
  eat clicks), nameplate tabs (uppercase, letter-spaced, lit tab gold with a gold underline
  strip), parchment-gold section headers, squarer radii. NO Blizzard asset is copied — the game's
  art is Blizzard's property and this is a community tool (the footer already says so): only the
  SILHOUETTE (panel + gold trim + tabs) is borrowed, in plain CSS. No webfonts, no remote assets,
  no generated images — the offline rule of every release keeps its teeth.

**Rejected:** copying Blizzard art (property; and the offline rule); generated or raster assets
(offline rule, and none is needed); tab state in Rust (the engine never learns the window exists,
ADR 0002/0014 — tab state is a class flip in the webview); moving the watch toggle out of the hit
rows (the toggle IS the search activity: "mark what is worth hunting" happens where the hits are);
growing the Sync tab with icons/tooltip/price stage buttons THIS release (the panel's verbs stay
the five the v0.9.6 record already scoped: `list · pull-all · details:watch · details:hits · Cancel`
— the stage buttons are real future work, said so, not slipped in).

## Rules that had to survive the skin — kept, measured, not assumed

| rule | how it survived |
| ---- | ------------- |
| ADR 0014 (a click and a cargo line are ONE path) | no verb, flag or cli_ok path touched: the release's diff is 5 files — 3 ui/, Cargo.toml+lock (banner) — zero `.rs` (measured: `git diff --stat` range) |
| ADR 0015 (money rides copper) | money_cell/moneycell rules UNTOUCHED in app.css (tabular + cursor:help + hover contract); money never recoloured; `available` still a COUNT with no money |
| quality chips are game facts | hues ride byte-identical (Common gray → Heirloom teal): a fact stays a fact under a new frame |
| ADR 0008/0014 icons | still the DB's OWN bytes as data URIs, placeholder glyph unchanged — the skin redraws borders, not sources |
| developer-terms attribution | the footer stays OUTSIDE the tab strip: it must reach EVERY screen, and hiding it behind a tab would breach the rule (docs/API.md) |
| exit-code contract (ADR 0004) | unchanged by construction (zero Rust) — the map's re-run lands FORWARD in the post-tag audits |
| the live row (v0.9.6, ADR 0017) | the card row + its note ride unchanged into the Item card tab — no claim that the panel can run the price stage; the notes still say it is CLI-side |

## A product defect found reading, fixed FORWARD, measured both ways

The live card's success path ended with `+ hits_link` — an identifier with NO definition anywhere
in app.js. `grep -c hits_link` counted ONE occurrence: the use site. So the success path ALWAYS
threw: build the note → `ReferenceError` → the card's own catch answered with the raw error text
instead of the door's honest note. Measured with node on this FS (the expression itself, no webview
needed): the original line throws `ReferenceError: hits_link is not defined`; after the fix the
REAL line is extracted from app.js and run through `new Function`: it RETURNS the note, no throw —
and `grep -c hits_link` now counts ZERO while `node --check` parses. The fix states the truth the
fragment promised to hand over and never did: every live hit row ALREADY carries its own upstream
`view` link (rendered per row), so the dynamic note says that instead of trailing a dangling colon.
No link invented, no request behaviour changed (still ONE ask, DB untouched). An identifier scan
over the file surfaced only this code-level hit — recorded honestly: the scanner is heuristic
(apostrophes in prose mispair), its value is the confirmed hit, NOT a proof of absence elsewhere.

## What this release can and cannot prove — ADR 0011's rule for a webview

CI and this machine can witness WITHOUT a display:
* `node --check` parses app.js (a PARSE witness; runtime behaviour of the webview is not proven),
* the class contract: every class app.js emits is still styled — old-vs-new app.css selector scan
  with ZERO dropped (the new classes are only the frame/tabs), brace-balanced file,
* zero Rust lines changed (`git diff --stat` names 5 files: ui/×3 + Cargo.toml + Cargo.lock),
* the embed path is the SAME as every release (`frontendDist: ui`, build.rs + tauri.conf.json) and
  the binary maps the ui FILENAMES in plaintext (`index.html` ×14, `app.css` ×1) — but the CONTENT
  is compressed: content greps for new strings answer ZERO. This corrects the v0.9.6-era phrasing
  "the green run re-embedded the edited files" where it is quoted in THIS record: the greppable
  witness is the filename map + green build; content embedding is NOT greppable, and the rendered
  window itself is only observable at launch.
* what is NOT measurable here: the window's look (no display on this FS) — recorded as such, never
  claimed; post-tag audits re-run the CLI doors (mirror replay + exit maps) to prove the release
  moved ZERO bytes on the WIRE, which is what the skin promised.

## Bends recorded where they stand — nothing swept

* the FIRST build after the fix(ui) commit skipped the house's `rm -rf target/debug/incremental`
  mechanic, so the FS's spurious hard-linking cache warning rode the summary to "5 warnings" — the
  FOUR named warnings were always exactly four; re-measured with the mechanic (the pattern of every
  build line in this repo predates this release; recorded because this release counted it).
* the fixture server and the smoke block were NEVER touched (no ci commit this release: the block
  stays 33 step-lines + plant one, and the post-tag replay re-runs it on fresh ports to show the
  wire did not move).

## Chain

`35342b9` feat(ui) (the page splits: nav strip, data-tab on all 8 sections, static default paint,
show_tab/tab_click/delegated handler/boot re-assert) → `24971fe` fix(ui) (the live note's dangling
fragment — above, with its node before/after) → `5c7eb09` feat(ui) (the AH frame skin: class
contract kept, corner pseudos, nameplate tabs) → `c6d2b18` chore (banner 0.9.7, lock rides) → this
record carries the annotated tag `v0.9.7`.

## CI row

ADR 0016's ladder grows measured: **44970667747** (v0.9.6's CI-fold push, `0c450ba..e4c6f3f`, head
`e4c6f3f…`) answered **success, 5/5 in 7 min 30 s** (01:16:24Z → 01:23:54Z — quality 2 m 00 s ·
test 4 m 42 s · linux 5 m 24 s · macos 7 m 12 s · windows 7 m 28 s), exactly ONE run id for that
head, no CANCELLED sibling, no rerun (matched by `id`/`head_sha` — the run's own `.attempt` reads
null on this API, the ladder's habit). It was IN FLIGHT while the v0.9.6 fold waited and pushed
clean, and its verdict folds HERE, FORWARD, this record. v0.9.7's own chain+tag push `e4c6f3f..dfb8826`
(head `dfb8826...` - the TAGGED tree) ran **44972713823**: **success, 5/5 in 8 min 44 s** (01:42:32Z ->
01:51:16Z - quality 2 m 19 s · test 4 m 55 s · linux 4 m 29 s · macos 4 m 42 s · windows 8 m 42 s),
exactly one run id for that head, no CANCELLED sibling, no rerun (matched by id/head_sha; the run's own
.attempt reads null on this API). The verification fold waited - staying LOCAL while that run flew
(the observed rule: a fresh push cancels the branch's in-flight run) - and GREW with this verdict before
pushing, honest: only the fold's OWN push lands FORWARD in the next record. Tags stay put (`v0.9.6` ->
`4ced794`, `v0.9.7` -> `dfb8826`) Tags stay
put (`v0.9.6` → `4ced794`, `v0.9.7` → this record's commit).
