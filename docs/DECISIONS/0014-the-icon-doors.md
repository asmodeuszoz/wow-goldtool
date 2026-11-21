# ADR 0014 — the icon doors: lists show the DB's OWN bytes, never upstream's address

**Status:** adopted (v0.9.3)
**Related:** `src/db.rs` (`b64_text`, `icon_b64_of`), `src/query.rs` (`Hit`'s trio,
`search(conn, filter, with_icons)`), `src/stats.rs` (`Extreme`'s trio, `extremes(…, with_icons)`),
`src/cli.rs` (`--icons`, `bare_flag`, `card_icon_parts`), `ui/app.js` + `ui/index.html` + `ui/app.css`
(`icon_cell`, the icon columns, `--icons` on both list asks), [ADR 0008](./0008-icon-storage-third-stage.md),
[ADR 0004](./0004-headless-cli-mode.md), [ADR 0011](./0011-ci-asserts-only-what-is-true.md),
[ADR 0010](./0010-manual-layer-and-tooltip-copy.md), `docs/DATABASE.md`, `docs/DEVELOPMENT.md`

## The measurement that forced the decision

One read of the user's own DB, the day this door was written (before any code changed):

```
rows with icon_blob                     4 030        of 19 258 rows
rows checked as gone upstream (icon_missing_at)  1
rows never asked (no icon_at, no blob) 15 228
detailed rows                            4 602   ALL of them carry an icon_url
                                          4 030 of them carry bytes
blob sizes    min 1 385  avg 2 327  max 3 268 bytes
total icon bytes 9 380 048  → about 3.1 K chars per icon once base64'd
```

Two facts decide the shape. (a) `icon_url` exists on 4 602 rows while bytes exist on
4 030 — so hotlinking the address would show an image the DB never claims to hold, and
ADR 0008's rule says the DB's own bytes ARE the image ("nothing is hotlinked, the DB
gets the bytes", measured against both asset hosts). (b) about 3.1 K chars per icon
means an always-inline answer is not free: the UI asks 40 search hits (≈ 124 K chars)
and 6 extremes (≈ 19 K), while a CLI `search` with NO `--limit` over the user's file
would carry every visible row (measured 19 258 rows; an icon per row is tens of MB).

## Options considered

* **Always inline.** Rejected by the measurement above: an unbounded CLI list becomes
  a JSON answer of tens of megabytes, and the door cannot know the caller wants it.
* **A separate `icon ID[,ID…]` verb.** Rejected for the same reason the GUI's second
  round-trip is worse: the hits are already collected and the icon columns sit on the
  very same rows in the very same DB, so a second ask buys nothing and splits one
  truth into two doors that could drift.
* **Only a flag, no stamps.** Rejected: an empty cell would read as "this item has no
  icon in the game", which is a lie for 15 228 rows that were never asked and for the
  1 that was checked as gone upstream. ADR 0008 already insists the three states stay
  three states; the row must say WHICH one it is in.
* **Opt-in `--icons` on the list doors, trio stamps always riding.** Chosen.

## Decision

`--icons` is a no-value flag (it rides `bare_flag` beside `--watch`/`--cached`) on the
two list doors, `search` and `stats`:

* the image is the DB's OWN bytes: the door calls `icon_b64_of`, which reads `icon_blob_of`
  and encodes through the ONE `b64_text` that lives in `src/db.rs` beside the
  bytes it encodes. The card and the lists cannot encode the same bytes two ways
  (the house rule: one truth, one encoder);
* upstream's `icon_url` still rides the row shape unchanged but is NEVER an image src
  (ADR 0008's rule);
* `Hit` and `Extreme` grew the trio `icon_b64` / `icon_at` / `icon_missing_at`. The
  keys are ALWAYS present; the base64 payload rides only when the door was asked with
  `--icons`, and the stamps ride always — so a no-bytes row can still name its state;
* rows are COLLECTED before the per-row icon reads run: a hit list must not straddle a
  write of the table it came from (the same rule the refresh door rides);
* ZERO requests leave either door, so ADR 0004's network door (`3`) is unreachable on
  these paths — the doors answer `0`, `2` (usage) and `4` (missing DB) only, and the
  mirror replay's GET tally not moving is the proof the flag asks the fixture nothing;
* a row with no bytes draws a placeholder glyph (`◇`) and the state's reason in the
  hover, from that row's own stamps: `icon_missing_at` set → "checked as not there
  upstream <when>", neither stamp → "icon never fetched";
* the UI asks `--icons` for its own bounded lists (40 hits, 6 extremes) and prints the
  door's numbers rather than restating them;
* NO schema hop: the doors write nothing. They read `icon_blob`/`icon_at`/
  `icon_missing_at`, the three columns schema `3` grew (ADR 0008's icon stage), and
  `schema_version` stays `5` — measured on the mirror's own DB after a replay that ran
  both icon doors.

## Consequences, measured and not smoothed

* the lists gain the icon where the crawl copied one: 4 030 of the user's 19 258 rows
  can show it, so most rows show the glyph — honest, because those rows genuinely have
  no bytes in the DB;
* **the item card needed no code at all**: it has rendered an actual `<img>` (a data
  URI, never a hotlink) since v0.6.0 (`ui/app.js`'s `icon (local copy)` row). The lists
  were the gap, not the card — recorded here so no one reads this milestone as a card fix;
* **the watch list has no panel to draw**: `ui/index.html` has no watch table (measured:
  the only watch surface is the `Watch these hits` button and the sync scope), and
  `refreshWatched` holds ids for the `unwatch` label only. So the watched rows gain
  icons where they appear — in the hits table — and no panel was invented to hang an
  icon on;
* the live-search table keeps NO icon column: that door reads and writes NO DB (ADR
  0009), so upstream's names have no local copy to show;
* the cost is bounded by the caller, not by the door: `--icons` without `--limit` is
  allowed and honest (the docs bound it), and the UI's asks are bounded;
* the encoder did not change: the move is byte-identical. Measured across the move on a
  scratch mirror DB, `item 25 --cached` answered `icon_b64` of 2 596 chars BEFORE and 2 596
  AFTER with the same sha256 prefix (`c67a63484dc91767`), and the emitted base64 decodes
  back to the DB's own 1 945 blob bytes (`base64.b64decode(...) == blob`).

## Audit

| probe | expectation | observed |
| ----- | ----------- | -------- |
| `search sword --where description` (no flag) | shape gains three keys, payload null | rc `0`, 650 chars, both hits carry `icon_b64:null, icon_at:null, icon_missing_at:null` |
| the same door `--icons` | the copied row fills, the manual row stays empty | rc `0`, 3 250 chars: row 25 → 2 596 chars + `icon_at 1795288450`; row 900000000 → all three null |
| `search --icons --limit 6` (every visible row) | the three icon states answer three ways | rows 25/40/12809 → bytes + `icon_at`; row 60 → NO bytes, `icon_missing_at 1795288450`; rows 900000000/127555 → neither stamp |
| `stats --limit 2` / `stats --icons --limit 2` | nulls / payload | rc `0` both: 954 chars → 11 354 chars, all four extreme rows filled |
| `search sword --iconsx` | the usage door | rc `2`, `unknown flag --iconsx (run help to see the verbs)` |
| a missing DB | the honest `4` | rc `4`, `cannot open …: unable to open database file` |
| both `--icons` doors with a fixture on a fresh port | ZERO requests | the fixture's log holds only the driver's own liveness probe (`GET /` → `404`) and ZERO `/api/v1` lines |
| CI smoke (29 → 31 cargo steps) | the two doors run green, GET tally unmoved | recorded with this tag |
| the whole exit-code audit on the release tree | the v0.8.1 map still byte-identical, the new doors answer 0/2/4 only | recorded with this tag |
