# ADR 0020 — the live-sort knob: order what the AH offers NOW, the shown cell stays the stored min

**Status:** adopted (v0.9.9)
**Related:** `src/query.rs` (`NOW_ORDER` + the `order_for` branch), `src/cli.rs` (usage string),
`ui/index.html` (the select option), `ui/app.js` (the conditional note sentence), `.github/workflows/ci.yml`
(the one grown step), [ADR 0017](./0017-the-live-min-buyout.md) (the live pair this order reads and its
once-EVER pending rule), [ADR 0016](./0016-the-price-trio-homes.md) (the column existed at schema 6;
v0.9.9 READS it for the first time), [ADR 0015](./0015-the-money-words.md) (money stays beside copper,
the note carries no numbers), [ADR 0014](./0014-the-icon-doors.md) (a select value is still one argv),
[ADR 0011](./0011-ci-asserts-only-what-is-true.md) (the block grows a witness CI CAN see — a door's order),
[ADR 0006](./0006-checked-missing-third-state.md) (which NULLs sort last, and why the sort does not care)

## The observation that forced it

The user asked for the live-sort knob right after the stage buttons ("a then b"): the Search tab
orders by what the DB once STORED (`price_min`), while ADR 0017's price stage now keeps a LIVE
minimum per copied record (`price_min_now`) — what the AH offers NOW. Sorting "cheapest first" by
the stored column answers with the crawl-era order; the user wants the hunt list ordered by the
live pair the button (or CLI) refreshed.

## The decision

**Landed:** one new sort key, `price-now`, in three Rust places (query.rs `NOW_ORDER`
`ORDER BY (price_min_now IS NULL) ASC, price_min_now ASC, name ASC` + the `order_for` branch, cli.rs
usage string) and one UI place (a select option `cheapest LIVE price first`, value passed through
the EXISTING `--sort` argv — zero new wiring). The NULL-last pattern rides EXACTLY as every other
price sort already does — and the record says WHICH NULLs sort last and why the sort cannot tell
them apart: rows never refreshed AND rows checked price-less at ask time (`price_min_now` NULL in
both; their own stamps `price_now_at`/`price_now_missing_at` keep the kinds apart — ADR 0006's
pattern, the order does not care and does not guess). Name tiebreak keeps the order stable across
runs.

**The note moves with the knob:** WHEN (and only when) the knob is on, the search note grows the
sentence naming which is which — "this order reads the LIVE min buyout the Sync tab's Price button
(or the CLI stage) wrote, one ask per copied record ever; the price column stays the DB's STORED
min; rows with no live price sort last (never refreshed, or checked price-less at ask time —
their own stamps keep the two kinds apart)". Why that sentence is not decoration: the live order
can make the price column LOOK out of order (measured mirror: 60 live 16 106 before 25 live 17 106,
both stored 18 106) — the note says which is which beside every hit, so a reader never suspects a
drift. Other sorts keep their note BYTE-IDENTICAL.

**Rejected / deferred, recorded where they stand:**
* **Showing the live price IN the cell when the knob is on** — deferred: `HIT_COLUMNS` + the Hit
  money pair would have to grow per-row live fields (a WIRE shape change); the knob's promise is
  the ORDER, the cell stays the stored min the hover-copper belongs to (ADR 0015's money rides the
  copper it derives from — a live cell would need its own pair, next release's honest question).
  The note carries the sentence instead, this release.
* **`--priced` reading the live column** — untouched: the filter is the stored-min rule documented
  since v0.9.1; the knob is an ORDER, not a FILTER.
* **Ties broken by id instead of name** — rejected: every other price sort ties by name already;
  stability across runs is the pattern.
* **A live-sort TOGGLE beside the default** — the select IS one knob; two orders at once would be
  a second, drifting truth.

## What stays true under the knob — measured, not assumed

| rule | how it survived |
| ---- | ------------- |
| ADR 0014 (a click and a cargo line are ONE path) | the UI option rides the SAME `--sort` argv; the CI block runs the SAME key the option sends (the grown step is the door itself) |
| ADR 0015 (money beside copper) | money rules untouched; the grown note sentence carries NO numbers at all (money words never float beside counts); the money cell keeps hover-copper |
| ADR 0006 third states | the NULL tail is BOTH kinds of NULL honestly named; stamps stay the stamps, the sort reads only the column |
| ADR 0004 exit codes | the door's usage/validation paths unchanged (unknown-key fallback behavior re-measured IDENTICAL); the map re-runs post-tag |
| schema | `price_min_now` is already a schema-7 column (born at 6, ADR 0016) — **NO hop this release**: schema stays 7 |
| the wire | default order BYTE-IDENTICAL old-vs-new (`search sword --where description --sort price --icons --limit 5` 3 359 B; `search detail --where all --limit 8` md5 equal) — the knob rides ONLY the new key, proven by the v0.9.8 binary |

## The CI block grew — honestly, one step CI CAN see

34 step-lines + plant one (was 33). The grown step `search detail --where all --sort price-now
--limit 8` runs where the mirror holds 25/60 refreshed and 40 checked price-less: the paragraph
above it quotes the shape measured on the mirror DB with BOTH binaries BEFORE this commit —
v0.9.8 answers the DEFAULT fallback (12809, 25, 40, 60) and v0.9.9 answers the LIVE order (60
before 25) — the step is green BECAUSE the knob works. ADR 0011's line kept: the block grew a
witness of a DOOR's order, not the webview's select option (still a grep+parse witness); ZERO
requests added (search reads the DB), GET tallies cannot move.

## Measured (pre-tag)

* knob on the mirror DB: live order 60 before 25 (16 106 < 17 106), NULL tail by name
  (CI Test Mug, Conjured Cocoa Choco-Frogo, Fixture Detail 12809, Fixture Detail 40) — matching
  sqlite's own rendering of the same clause;
* v0.9.8 binary (`/tmp/opencode/wgt-v98-pre-sort`, saved pre-release): default-order doors BYTE-IDENTICAL
  (3 359 B pair; md5-equal `detail --where all --limit 8`), `price-now` on it falls back (unknown key —
  the door's documented rule, hits portion IDENTICAL on the old binary);
* bogus sort key still IDENTICAL hits vs default (fallback rule unchanged);
* node --check parses; grep: `price-now` x1 option + x1 clause;
* builds: exit 0 FOUR named warnings every commit (incremental-clear mechanic every build).

## Chain

`b8abbd8` feat(search) (NOW_ORDER + branch + usage string) → `536ea1d` feat(ui) (the option + the
conditional sentence) → `b700a57` ci (34th step, shape quoted from measurement) → `8ed805d` chore
(banner 0.9.9, lock rides) → this record carries the annotated tag `v0.9.9`.

## CI row

ADR 0016's ladder, measured: **44975077893** (v0.9.8's chain+tag push, head `9aaa3f6…` — the TAGGED
tree): success 5/5 in 10 m 45 s (02:13:34Z -> 02:24:19Z — quality 2 m 03 s · test 5 m 03 s · linux
5 m 25 s · macos 4 m 54 s · windows 10 m 42 s), one run id, no CANCELLED sibling, no rerun. Its
verdict already folded into v0.9.8's verification fold (that fold waited and grew it). **44976027751**
(v0.9.8's verification-fold push, head `c4fac31…`): **success, 5/5 in 6 min 21 s** (02:25:34Z ->
02:31:55Z — quality 2 m 10 s · test 5 m 01 s · linux 5 m 36 s · macos 5 m 44 s · windows 6 m 18 s),
exactly ONE run id for that head, no CANCELLED sibling, no rerun — it landed WHILE THIS CHAIN built
(nothing was in flight when v0.9.9's first commit was made; the honest ordering fact recorded where
it stands). v0.9.9's own chain+tag push `c4fac31..53f9221` (head `53f9221…` — the TAGGED tree) ran **44976892186**: **success, 5/5 in 9 min 39 s** (02:36:38Z -> 02:46:17Z — quality 1 m 49 s · test 4 m 36 s · linux 4 m 25 s · macos 6 m 22 s · windows 9 m 37 s), exactly one run id for that head, no CANCELLED sibling, no rerun. The verification fold waited (LOCAL while that run flew — the observed rule), GREW this verdict by amend and pushed: only the fold's OWN push lands FORWARD in the next record, never chased. Tags stay put (`v0.9.8` -> `9aaa3f6`, `v0.9.9` -> `53f9221`).
