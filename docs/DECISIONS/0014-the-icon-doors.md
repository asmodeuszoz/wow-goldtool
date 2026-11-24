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
| both `--icons` doors with a fixture on a fresh port | ZERO requests | the fixture's log holds only the driver's own liveness probe (`GET /` → `404`) and ZERO `/api/v1` lines (pre-tag probe, port 9164); post-tag the same proof rides the replay tallies: **A GETs 29 before = 29 after** the icon doors, and the mirror's GETs 31 unmoved |
| the CI runs | green on the tagged tree | run **44944988663: success, 5/5 jobs in 10 min 42 s** (20:11Z → 20:22Z) for the chain push `ff1d174..78426d1` + the tag push - so this tag HAS a green run on its own tree (the v0.9.0 tag never did); this record's own push run gets its verdict recorded as soon as it lands |
| CI smoke (29 → 31 cargo steps) | the two doors run green, GET tally unmoved | **green**: 31/31 cargo steps rc 0 + `plant rc=0` on a fresh port 9165 with a driver generated FROM the committed block; the icon doors land last (bytes + `icon_at`, and the gone row answering no bytes); fixture tallies **GETs 31 = 200×15/304×9/404×3/429×4**, byte-identical to the v0.9.2 replay → the doors asked NOTHING; mirror DB icons 3 + 1 gone, twins 4\|4\|4\|0, schema **5** unchanged, `900000000` asks ZERO |
| the ONE encoder across card + doors | one copy, one shape, no drift | **one shape only**: every emitted `icon_b64` = 2 596 chars → 1 945 bytes and **EQUALS** the mirror DB's own blob (`distinct shapes: 1`) |
| the whole exit-code audit on the release tree (fresh ports 9166..9173 + 9174) | the v0.8.1 map still byte-identical, the new doors answer 0/2/4 only | **86 doors green**: 01–77 identical map (incl. the pinned migration trio 0/0/4); the icon doors 78–86 answer 0/0/0/0/**2**/0/**4**/0/0 - `search sword --iconsx` the usage 2, a missing DB the honest 4 - and both doors with a fixture URL still answer 0 with **A GETs 29 before = 29 after** |
| the three icon states, as three answers | bytes+stamp / no bytes+gone-stamp / neither stamp | door 80 (`search --icons --limit 6`): 12809/40/25 → 2 596 chars + `icon_at` · 60 → no bytes, `icon_missing_at` set · 900000000/127555 → neither. On a DB with the icon stage blanked (door 83): `{(None,None,None)}` plus the gone row's `{(None,None,stamp)}` - the glyph's two states are facts, not guesses |
| the icon doors over a DB **born** under the pinned v0.8.1 snapshot (fresh port 9176) | they read the pinned era's own bytes with ZERO requests | the pinned crawl (list+detail+icons) left `schema_version 4` with 3 blobs; the current build reports `3\|1\|5` after the hop, `search --icons --limit 6` returns 3 rows × 2 596 chars + `icon_at` (row 60 gone-stamped, 127555 neither) and **3/3 emissions decode EQUAL** to the DB's own 1 945-byte blobs; the fixture's 15 GETs all belong to the pinned crawl, ZERO to the doors |

**GROWN AT v0.17.0 ([ADR 0031](./0031-the-search-list-owns-its-icon-position-and-size.md)/[ADR 0032](./0032-the-total-crafting-cost-on-the-recipe-doors.md)):** the ONE-code-path discipline now covers a COST too — both recipe doors fill their cost tallies THROUGH ONE `craft_cost` after their OWN joins, and the tab's cost column and the card's cost line call ONE `cost_cell` in front of both (no drift possible, measured at the call sites). The icon era moved NO door at all: the pinned v0.14.0 pair 28/28 BYTE-IDENTICAL + 13/13 md5-equal on a fresh window (port 9911) says the search/item doors answer the SAME bytes with the icon cell moved and resized in the UI. The era's flag rides PARSED-BUT-IGNORED through the doors that never asked for it — `search sword --costs` == `search sword`, **188 == 188 B** — the same through-ride shape `--icons` has ridden since v0.9.3.

**GROWN AT v0.18.0 ([ADR 0033](./0033-where-a-recipe-is-learned-from-rides-the-doors-own-bytes.md)):** the ONE-code-path discipline rode the LEARNED bytes a THIRD time: the LIST door
now reads `learned_json` THROUGH the card door's OWN machinery — same `from_str` decode shape, same
decode-error WORDS, same `word_at`/`id_at` by-key helpers — so the tab's LEARNED-FROM column and the
card's learned-from block CANNOT drift about one row (measured the same day: the card twins BYTE-EQUAL
1 507 == 1 507 / 1 211 == 1 211 with the flag riding — the card PARSES `--learned` and IGNORES it, it
answered learned-from UNASKED since v0.12.0). The ONE reader the seen fact rides is the crafted join's
OWN `live_name_price_of` (tombstone guard included) — no second shape for the SAME row's fact.
