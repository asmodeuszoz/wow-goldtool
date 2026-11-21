# ADR 0013 — the refresh door: a derived twin can be re-derived, with ZERO requests

**Status:** adopted (v0.9.2)
**Related:** `src/db.rs` (`refresh_twins`, `SQL_REFRESH_ROWS`, `SQL_REFRESH_WRITE`, `RefreshTally`),
`src/cli.rs` (`refresh-twins`, `cmd_refresh_twins`), `ui/index.html` + `ui/app.js`
(Reword the copies), [ADR 0012](./0012-the-readable-twin.md),
[ADR 0010](./0010-manual-layer-and-tooltip-copy.md), [ADR 0007](./0007-price-sampling.md),
[ADR 0004](./0004-headless-cli-mode.md), `docs/DATABASE.md`

## What forced it

ADR 0012 made the twin DERIVED, and v0.9.1 corrected the derivation (every `<...>` span
leaves ONE space). Both facts sit on the same shelf, and neither reaches what already
exists: the pending rules refuse a stamped copy (ADR 0007's stamp rule) and a `304` writes
nothing. So the corrected rule only ever touches NEW copies. Measured on the user's own
file, the day this door was written:

```
copies read (tooltip_html NOT NULL)                     4 602
twins matching the corrected rule                           0
twins a refresh rewrites                              4 602
stored twin == recomputed v0.9.0 GLUED shape, all 4 602
  "Worn ShortswordItem Level 2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9(1.1
     damage per second)Sell Price: 7c..."
```

ADR 0012's honest gap — "a row that migrated with a copy already in it has NO twin until
the next tooltip write" — has the same cause: nothing re-derives old rows, and the queues
will never ask them. The gap is not rare in the wild; on the one file that finished a
crawl it is every single copy.

## Options considered

* **Re-ask upstream for every copy.** Rejected twice over: it spends 4 602 requests to
  re-fetch bytes the DB already holds (the copy is the source of truth), and the pending
  rules say these rows are NOT pending — a crawl answers ZERO pending for them, so the
  re-fetch would not even be scheduled.
* **Re-derive at read time (the card and the search door compute the twin).** Rejected by
  ADR 0012's invariant: the twin COLUMN matches what the DB SAYS. Deriving only at read
  puts two answers on one copy and lets the column drift — exactly the drift `docs/DATABASE.md`
  exists to forbid.
* **Automatic inside the next sync pass.** Measured dead: the pending rule never re-asks a
  stamped copy and a `304` writes nothing, so a crawl refreshes ZERO of the 4 602 copies.
* **An explicit refresh door the user presses.** Chosen: one ask, the whole DB, and the
  report says what it did.

## Decision

`refresh-twins` — a verb that takes NO arguments and re-derives every twin in the DB:

* the copy is the source of truth: it selects rows with `tooltip_html IS NOT NULL`, calls
  the SAME `readable_tooltip_text` every write path calls (there is no second derivation
  to disagree with it), and writes only the rows that differ or have no twin;
* ZERO requests leave it, so ADR 0004's network door (`3`) is unreachable from this path
  — the doors here are `0` and `4` only;
* tombstones and manual rows ride: ADR 0012's rule is that the twin always matches what
  the DB SAYS, and a stated `tooltip_html` is what the DB says for those rows too;
* a copy whose twin was never born is counted as `filled`, which closes ADR 0012's
  honest gap without re-fetching anything;
* rows are COLLECTED before any write — a sqlite row walk over the table this same door
  rewrites must not interleave with its own UPDATEs;
* it is idempotent: a second press answers `0 rewritten` and still exits `0` (the report
  says so — `4` would mean "the DB does not have it", which is not this case);
* no schema hop: it writes columns schema `5` already has, so a refreshed file keeps its
  schema stamp.

## Consequences, measured and not smoothed

* the card's words row becomes readable on every copy — the visible gain;
* the invariant `twin == derivation(copy)` holds for the whole DB, so the card and a
  search can never disagree with the copy;
* the description door is a SUBSTRING door (`LIKE %text%`), so a phrase inside ONE span
  was always findable in the glued twin: the refresh gains ONLY the phrases that span an
  element boundary. Measured on the user's 4 602 copies: `damage speed` +358 rows,
  `hand one` +90, `sword 1` +14, `1 main` +3, while `main hand` stays 75 before AND
  after and `sell price: 7` stays 291 — the honest size of the search gain;
* a refreshed DB answers 0 rewritten, so the door costs nothing when there is nothing to
  do, and the mirror's GET tallies not moving is the proof that it asks the fixture
  nothing.

## Audit

| probe | expectation | observed |
| ----- | ----------- | -------- |
| `refresh-twins` over twins the corrected rule already wrote (a scratch mirror DB) | nothing to do | rc `0`, `{"copies":4,"rewritten":0,"filled":0,"unchanged":4,"requests":0}` |
| the v0.9.0 GLUED shape planted on row 25 (the shape all 4 602 copies on the user's file are in) | the phrase spanning the boundary comes back | `search "damage speed" --where description` total `0` BEFORE → refresh answers `rewritten 1` → total `1` AFTER; the card's twin row reads `Fixture Detail 25 Item Level 1 Main Hand One-Handed Sword 1 - 3 Damage Speed 1.9 Sell Price: 7c` |
| a third press | idempotent, still success | rc `0`, `rewritten 0` |
| `refresh-twins extra` | the usage door | rc `2`, `refresh-twins takes no arguments (it refreshes every copy in the DB)` |
| a missing DB | the honest `4` | rc `4`, `cannot open ...: unable to open database file` |
| `refresh-twins --fixture URL` | ZERO requests | rc `0`, same report, the fixture's tallies unchanged |
| CI smoke (24 → 29 cargo steps + one plant line) | the sequence above runs green | **green**: 29/29 cargo steps rc 0 + `plant rc=0`, refresh doors in order 0 → phrase 0 → 1 rewritten → phrase 1 → 0, fixture GETs 31 unchanged from the v0.9.1 replay (the door asked nothing), `900000000` asks ZERO |
| the whole exit-code audit on the release tree (fresh ports 9153..9160 + 9162) | the v0.8.1 map still byte-identical, the v0.9.x doors still put, the refresh doors answer 0/4/2 only | **77 doors green**: 01–67 identical map, migration trio 0/0/4, refresh doors 0/0/0/0/0/0/0/4/2/0, A GETs 29 before = 29 after the refresh doors |
| ADR 0012's gap made REAL the way it happens (a DB born under the pinned v0.8.1 snapshot, schema 4 with copies and no twin columns) | the door gives those copies twins without a re-fetch | **filled 3**: the pinned crawl left `schema_version 4`, 3 copies, 0 twin columns; the current build reports `3\|0` after the v4→v5 hop; refresh answers `filled 3` → `3\|3`; the card reads the twin with ZERO requests; press 2 answers 0 rewritten |
| the user's own file, post-tag (the residue v0.9.1 named) | every glued twin re-derived, ZERO requests, idempotent | **rc 0 in 114 s: copies 4602, rewritten 4602, requests 0**; the first twin went from `Worn ShortswordItem Level 2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9(1.1…` to `Worn Shortsword Item Level 2 Main Hand One-Handed Sword 1 - 3 Damage Speed 1.9 (1.1…`; the substring phrase doors moved exactly as predicted — `damage speed` 0→358, `hand one` 0→90, `sword 1` 0→14, `1 main` 0→3, `main hand` 75→75, `sell price: 7` 291→291 — and press 2 answers 0 rewritten / 4 602 unchanged; file tail intact (19 258 rows, schema 5, icons 4030+1, 4 284 samples, html-without-twin 0) |
