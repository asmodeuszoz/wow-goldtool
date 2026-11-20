# ADR 0008 — the icon stage copies upstream's own icon file into the DB once

**Status:** adopted (v0.6.0)
**Related:** `src/db.rs` (`icon_blob`/`icon_at`/`icon_missing_at`, `SQL_COPY_ICON`,
`SQL_MARK_ICON_MISSING`, `SQL_PENDING_ICON` readers), `src/sync.rs` (`sync_icons`),
`src/wowdb.rs` (`icon_target`, `get_bytes`, `BinaryResponse`), `src/stats.rs`
(`SQL_PENDING_ICON`, `icon_pending_ids`), `src/cli.rs` (`--stage icons`, card
`icon_b64`/`icon_at`/`icon_missing_at` + `b64_text`), `ui/app.js` + `ui/app.css`
(the card's `<img>` and its three-state rows), tests/fixture_api.py (the real JPEG
bytes and the gone icon file), docs/API.md ("Icon files" section), docs/DATABASE.md,
[ADR 0002](./0002-two-stage-sync.md), [ADR 0006](./0006-checked-missing-third-state.md)

## The measurement that forced the decision

Every `forever/item` record carries `icon_url`, and the live catalogue answers exactly two
shapes of it — measured 2026-11-20, and these are the ONLY two seen on answered records:

```
https://render.worldofwarcraft.com/us/icons/56/inv_mace_03.jpg    -> 200 image/jpeg 1945 b
https://thewowdb.com/assets/icons/inv_axe_11.jpg                  -> 200 image/jpeg 1936 b
https://render…/us/icons/56/… and …/forever-eu/… and …/forever-us/ variants, same shape
```

and the same walk answered **empty** `icon_url` strings (`"icon_url": ""`, 4 of the 29
detailed scratch rows: Peace Tea, Craftsman's Writ: Mechanical Repair Kit, Scroll of
Imbue Iceknife, Lodestone) — upstream's own words for "no icon here", not a broken link.

What an ABSENT icon path answers, measured 2026-11-20 (these paths are named by no live
record — the probes invented nothing about records, they read answers):

```
render…/us/icons/56/<bogus>.jpg        -> 403 application/xml  243 b  <Code>AccessDenied</Code>
thewowdb.com/assets/icons/<bogus>.jpg  -> 404 text/html      40 147 b  "Page Not Found | TheWoWDB"
thewowdb.com/icon/<bogus>.png          -> 404 text/html      (the bare root-relative form)
```

`404` with the host's own "Page Not Found" body is an answer about the file: the icon stage
gets its third state (`icon_missing_at`) from THAT, one request per icon per crawl and never
re-asked inside one. `403 AccessDenied` states nothing about an icon - a CDN answer, not a
record's - so the stage keeps it a stage failure (exit `3`, resumable, nothing marked, the
icon stays pending): a fact the upstream never stated stays unstated.

The card used to show none of this: v0.5.x renders deliberately *not* hotlinking
`icon_url` (README and the card notes say why: the app's point is answering without the
network). So the icon has to come FROM THE DB — and that decides where its bytes live and
how they get there.

## Options considered

* **Hotlink `icon_url` in the card.** Rejected — not even an option worth half an ADR: the
  README's offline point and v0.5.x's own card note rule it out, the asset host can vanish,
  and the crawl would have quietly stopped being the thing that makes browsing offline.
* **Store the bytes as files next to the DB (`icons/25.jpg`).** Rejected: the DB is the one
  artifact the user already manages (the README names its path, `docs/DATABASE.md` its
  migration rules, `WGT_DB` its location); a sidecar directory adds a second path to get
  right, a second thing to clean up, and a second place where "gone upstream" could hide.
  The card reads through the CLI anyway — a DB `BLOB` rides the row the record already has.
* **Fetch the icon INSIDE the detail stage** (each record's icon right after its detail).
  Rejected on a measured fact: it doubles a chunk's requests (one icon per record, on the
  asset hosts — a chunk's budget covers one stage's worth of pacing, docs/API.md), AND the
  reuse path means records fetched BEFORE this build ran answer `304` forever —
  a `304` re-fetches nothing, so an inline icon would leave the ≈ 29 records the user
  already has fetched permanently icon-less while everything fetched later gets an icon:
  a stage whose output depends on which build fetched the record. A separate stage with its
  own pending queue is what makes every record's icon reachable regardless of history.
* **One icon request per detail request, cached forever** (an icon ETag + revalidation loop,
  like the detail row's). Rejected: the copy path is asked **exactly once per icon**
  (`SQL_PENDING_ICON` excludes copied icons — no second visit exists to revalidate), so an
  icon ETag would be a promise we never make and never exercise; the fixture answers icons
  with NO ETag header precisely so no code path may assume one (docs/API.md).
* **Keep only `icon_url` and a fetched-boolean** (no bytes). Rejected: that is v0.5.x's
  position dressed up — the card still shows no icon offline, and the third state
  ("the icon file is gone") is unexpressible, so a changed asset stays invisible.
* **Skip the third state** (icon `404` = failure, exit 3). Rejected by the same measurement
  that made v0.5.3's third state: the detail record EXISTS (its `icon_url` names the file),
  only the file is gone — an abort loses every icon after it and leaves "nothing pending"
  unprovable, exactly like a dropped record lost the crawl in v0.5.2.

## Decision

The third crawl stage, with the detail stage's whole plumbing:

* `sync --stage icons` takes the same `--budget`/`--ids`/`--watch`/cancel shape; its
  pending queue is `SQL_PENDING_ICON` (answered detail + no icon copied + not checked
  gone + a NON-EMPTY url — the empty-url shape above is why the url terms are not
  decoration: without them pending would never drain and "nothing pending" would lie,
  the same trap v0.5.2 fixed for prices).
* `icon_target` resolves only the two grounded shapes (absolute `http…` as measured, or
  a root-relative `/…` form — used by the fixture against the process's own base URL) and
  refuses to guess any other: the stage then fetches nothing and marks nothing, because a
  fact the upstream never stated stays unstated (docs/API.md "Icon files").
* Bytes ride the row: `icon_blob BLOB` + `icon_at` + `icon_missing_at`, mutually exclusive
  stamps like `detail_synced_at`/`detail_missing_at`; `SQL_COPY_ICON` upserts and CLEARS
  the missing stamp (a copied icon is not a checked one), `SQL_MARK_ICON_MISSING` checks a
  row exists first (`SQL_ROW_OF_ID`, v0.5.3's guard) so a mark can never invent a row.
* One `404` on an EXISTING record's icon file → checked state, crawl keeps going; any other
  status is a stage failure (exit 3), and a budget stop leaves the queue resumable — the
  icons stage is ≈ 19 258 requests at 2/s ≈ 2.7 h for a fresh catalogue, so resumable is
  not optional.
* The asset hosts are not the rate-limited API endpoints, but politeness is per host and
  the stage shares the same schedule and honours `Retry-After` the same way (docs/API.md).
* The card: the CLI answers `icon_b64` (the BLOB base64'd — the one door to the webview is
  a JSON verb answer and JSON has no binary; `b64_text` is our own twelve-line standard
  encoder, since the pinned dep graph has no direct base64 crate and Cargo.lock's base64
  entries belong to tauri's own subtree), `icon_at` and `icon_missing_at`; the card shows
  the local copy as `data:image/jpeg;base64,…` (both grounded hosts serve exactly that type)
  or the checked state, and the note line tells all three states.
* `status`/`stats` print the icon pair and the sync panel's grid gets its row; "Pull all
  info" chains list → detail → icons so one click retires everything (ADR 0006's third
  states are what keep that click honest when files or records are gone).

## Grounding, not invention

```
# the shapes live records carry (scratch catalogue, live payloads):
https://render.worldofwarcraft.com/us/icons/56/inv_mace_03.jpg
https://render.worldofwarcraft.com/forever-eu/icons/56/133749.jpg
https://thewowdb.com/assets/icons/inv_axe_11.jpg
"icon_url": ""                                     x4 (Peace Tea, Craftsman's Writ, Scroll of Imbue Iceknife, Lodestone)

# what those paths answer (one request each, 2026-11-20):
.../us/icons/56/inv_mace_03.jpg        -> 200 image/jpeg 1945 b
.../forever-eu/icons/56/133749.jpg    -> 200 image/jpeg 2265 b
thewowdb.com/assets/icons/inv_axe_11.jpg -> 200 image/jpeg 1936 b
# an ABSENT path answers per host (probes only - no live record names these):
render…/us/icons/56/<bogus>.jpg       -> 403 application/xml  243 b  <Code>AccessDenied</Code>
thewowdb.com/assets/icons/<bogus>.jpg -> 404 text/html      40 147 b "Page Not Found | TheWoWDB"
thewowdb.com/icon/<bogus>.png         -> 404 text/html      (the bare root-relative form: live records never use it)
```

The fixture embeds the `inv_axe_11.jpg` and `inv_mace_03.jpg` bytes (the two files above,
1945 b and 1936 b, fetched once and committed base64'd) so CI never asks a host for pixels,
and record 60 answers with an `icon_url` whose FILE the fixture serves as `404`.

## Audit (every claim above executed, not asserted)

| probe | expectation | observed |
| ----- | ----------- | -------- |
| icons stage on the CI mirror (fixture, fresh DB, budget 8) | the copy path lands, the gone file is checked once | `requests 5, items 4, written 3, missing 1, rate_limited 1, complete true`, message `icon crawl copied 3 icons; 1 answered 404 for the icon file and are checked as no-icon-upstream` |
| the DB after that stage | bytes in the row, two stamps apart | `icon_at` on ids 25/40/12809, `icon_missing_at` on id 60 (whose detail answered with `/icon/absent-60.png`); blobs `1945 b` each, magic `0xff 0xd8 0xff … 0xff 0xd9` — the grounded JPEG's own first and last bytes |
| icons run on a drained queue | asks nothing again | `requests 0, complete true`, `icon stage: 0 icons copied (0 requests)` |
| icons `--ids` on a checked-gone record | no request, no mark, no phantom row | `--ids 127555`: `requests 0, items 1, unchanged 1`, **exit 0** |
| icons door on a dead host | the network door lands on `3` | `sync --stage icons --ids 25 --fixture http://127.0.0.1:9923` → **exit 3**, `error sending request for url (http://127.0.0.1:9923/icon/inv_mace_03.jpg)` — the URL in the line proves `icon_target`'s base-relative resolution |
| usage door on the new stage | a mistyped stage is a usage error | `sync --stage iconsx` → **exit 2**, `sync wants --stage list, detail or icons, got iconsx` |
| the card after the copy | the local bytes render, the gone state reads | `item 25 --cached` answers `icon_b64` (the BLOB base64'd) + `icon_at`; the gone record shows the checked state in its own row |
| `stats`/`status` after the copy | the pair is visible, not inferred | `coverage: icons 3, icon_missing 1` on the mirror DB (4 detailed) |
| fixture grounding | CI never touches a host | the fixture's `ICONS` bytes decode to the two measured JPEGs (`1945 b`, `1936 b`, `0xffd8` magic); the fixture log shows `/icon/…` requests answered `200`/`404` by the process itself |
| GUI pull-all audit (fixture, boot probe) | one click chains all three stages | three `sync` verbs with ticker `status` polls interleaved; fixture log `14` requests, `3` × `429` (one per stage), `2` × `404` (the removed record + the gone icon file); DB `3` icon copies, `1` checked no-icon, both queues pending `0`, `sync_state` three rows `complete` |
| CI mirror (the committed twelve-step sequence) | green with the icon stage inside | every step exit 0, `stats` prints the icon pair and the ledger count, the DB carries `1` checked-gone record + `1` checked-gone icon |
| the user's own DB | never written by any of this | audited read-only (`sqlite3 -readonly`), hash and mtime identical before/after every audit |
