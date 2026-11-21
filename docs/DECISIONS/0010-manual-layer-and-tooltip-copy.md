# ADR 0010 — the manual layer (a user's own words survive every crawl) and the tooltip copy as a fourth stage

**Status:** adopted (v0.8.0)
**Related:** `src/db.rs` (`tooltip_html`/`tooltip_etag`/`tooltip_at`/`tooltip_missing_at`,
`manual_at`/`manual_overrides`/`edited_at`/`deleted_at`, `SQL_MIGRATE_TO_V4`, `editable_kind`,
`set_item_field`, `add_item_manual`, `next_manual_id`, `mark_deleted_at`/`clear_deleted_at`,
`apply_manual_edits_summary`/`apply_manual_edits_detail`, `write_tooltip`, `column_card`,
`SQL_COLUMNS_CARD`, `MANUAL_ID_BASE`), `src/wowdb.rs` (`tooltip_url`), `src/models.rs`
(`Tooltip`, `ManualEdit`), `src/sync.rs` (`sync_tooltips`, `tooltip_wanted_of`,
`tooltip_missing_note`), `src/stats.rs` (`SQL_PENDING_TOOLTIP`, `tooltip_pending_ids`, the five
new `SQL_COVERAGE` counts), `src/query.rs` (`visible_clause`), `src/cli.rs` (`--stage tooltip`,
`set-item`/`add-item`/`delete-item`/`undelete-item`, `card_manual_parts`, `cached_card`,
`cmd_item`), `ui/index.html` + `ui/app.js` (the write panel and the card's marks),
`tests/fixture_api.py` (`TOOLTIP_PAGES`, the tooltip `404` JSON branch), docs/API.md ("Tooltip
copy"), docs/DATABASE.md ("Schema (`schema_version = 4`)", "The manual layer"),
[ADR 0002](./0002-two-stage-sync.md), [ADR 0006](./0006-checked-missing-third-state.md),
[ADR 0007](./0007-price-sampling.md), [ADR 0008](./0008-icon-storage-third-stage.md),
[ADR 0009](./0009-live-search-door.md)

## The measurement that forced the decision

The user's milestone asks for a DB that holds **every forever item with its vendor price, its
icon and its description copy**, and that can be **edited by hand**. Two of those three were
already reachable (price on the detail record, icon bytes via v0.6.0's stage); the third one
is not behind the door the crawl already reads. Measured 2026-11-20 against the live
catalogue, then read back off the user's own file (read-only, `sqlite3 -readonly`):

```
# the detail record's OWN top-level keys (read off raw_json of id 25 on the user's file):
attribution, icon_url, id, item_level, item_subtype, item_type, links, name,
price, quality, required_level, unique, vendor_sell, weapon
                                    -> no description field, no html, no `fields`

# the door the description lives behind (one request, 2026-11-20, UA wow-goldtool 0.8.0):
GET https://thewowdb.com/api/v1/tooltip/forever/item/25
  -> 200 application/json 3060 b
  etag: W/"2b111cbcd3bbb4d7078f6d05"
  cache-control: public, max-age=300, s-maxage=900, stale-while-revalidate=3600
  x-ratelimit-limit: 120   x-ratelimit-remaining: 62   x-wgf-build: wow_classic_beta-1.60.1.70532
  top keys: attribution, build, fields, html, icon_url, id, kind, links, name, quality, version
  `fields` carries: armor, binding, build, classes, effects, faction_prices, flavor, icon_url,
                    id, item_level, item_set, item_subtype, item_type, kind, name, price,
                    quality, required_level, slot, socket_bonus, sockets, stats, unique,
                    vendor_sell, version, version_label, weapon
  `html` is 1404 b of upstream's own card markup: <div class="twdb-card twdb-k-item">…
         <img class="twdb-icon" src="https://render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg"…
         "Item Level 2" … "Sell Price: 7c" …</div>
  links.api = https://thewowdb.com/api/v1/tooltip/forever/item/25 (the door itself),
  links.html = https://thewowdb.com/wow-forever/item/worn-shortsword-25/ (attribution target)

# an id the tooltip door has not (one request, 2026-11-20):
GET https://thewowdb.com/api/v1/tooltip/forever/item/999999999999
  -> 404 application/json 79 b
  cache-control: public, max-age=300    x-ratelimit-remaining: 119    (no ETag offered)
  body: {"error":"not_found","message":"No item 999999999999 in forever.","status":404}

# the user's own file, read-only: 19 258 rows, schema 4, MAX(id) below the manual block
  = 288 340, detail_synced_at set on 2 919 of them
```

Three facts decide the shape of the release, all measured, none invented:

* the description **is not on the record the crawl reads** — a fourth door is a fact, not a
  nicety: `≈ 19 258` tooltip requests at the measured 2/s is `≈ 2.7 h`, the same order as the
  detail and icon stages, so it belongs to the crawl (budgeted, resumable, cancellable), not
  to a live ask;
* the tooltip door revalidates (`ETag` + `cache-control max-age=300`, its own `tooltip_etag`
  column) so a targeted re-ask costs a `304` and writes nothing — the same reuse discipline
  the detail door already promises;
* its `404` answers upstream's **own JSON** ("`No item … in forever.`"), the same shape as the
  detail door's, so the third state gets the same treatment ADR 0006 gave a dropped record —
  one request per record per crawl, and the crawl keeps walking.

The manual half is forced by the crawl's own shape, also measured: the list stage **UPSERTs
every id upstream still lists** (`upsert_summary`), so a row a user HARD-deleted would be
resurrected by the next list pass; and the measured max upstream id on the user's file is
`288 340`, so a reserved block starting at `900 000 000` is provably clear of every id the
walk can bring back.

## Options considered

* **Put the description on the detail stage** (fetch the tooltip inside stage 2). Rejected by
  v0.6.0's reasoning, re-confirmed here: it doubles a chunk's requests, and the reuse path
  means records fetched by an EARLIER build answer `304` forever — a `304` re-fetches nothing,
  so an inline description would leave the `2 919` records the user already has fetched
  permanently description-less while everything fetched later gets one: a stage whose output
  depends on which build fetched the record. A separate stage with its own queue is what makes
  every record's copy reachable regardless of history.
* **Copy the tooltip's whole body** (`fields`, `attribution`, `links`) instead of `html`.
  Rejected for this build: `html` is the thing the user asked for, and `fields` duplicates
  columns the detail row already owns (measured: the two key sets overlap on price/quality/
  vendor_sell/stats), so storing both would be two sources of truth for one record. The model
  keeps `fields` as a `Value` so a key upstream adds cannot break a build, and only `html` is
  read — that is exactly the dead-code notice docs/DEVELOPMENT.md names (ADR 0009's precedent:
  print/keep the measured mirror, read what the door needs).
* **Render the tooltip's `html` as markup in the card.** Rejected: it is upstream's own body.
  Pasting it into the page would let upstream write the card (its `class` names, its `<img>`);
  the card prints it as escaped TEXT, like every other API text (docs/DATABASE.md's rule that
  values are data, not markup).
* **Hard-delete a row the user deletes.** Rejected by the measurement above — the next list
  pass UPSERTs the id back. A tombstone (`deleted_at`) is the only shape that survives the
  walk: `visible_clause()` is one clause pushed FIRST in `where_parts`, so `search`, its
  `count_plan` and the card agree by construction, and the pending queues in `stats.rs` refuse
  a tombstone so no crawl re-asks (or re-stamps) a row the user buried.
* **Let the user pick an id for `add-item`.** Rejected: a user-picked id can collide with a
  FUTURE forever record (measured headroom: upstream's max id `288 340` against a block at
  `900 000 000`), which would overwrite their item on the next crawl. `add-item` takes no id;
  `next_manual_id` is the block's next free slot (`MAX(id) + 1` at or above
  `MANUAL_ID_BASE`), and the row is stamped `manual_at` — the mark is what makes the card
  answer from the DB's own columns instead of promising a fetch upstream cannot serve.
* **A separate `manual` table** (row → override). Rejected: every write path in this DB is one
  row's upsert, and the card, the pending rules and the coverage answer read one row's stamps;
  a second table would be a second place "the user already said" could hide, and a second
  thing a schema migration has to keep in step. The overrides ride as ONE JSON ARRAY per row
  (`field, value, at`) — this codebase never reaches for a map, and `value` stays a wire `Value`
  so nothing invents a shape.
* **Reject every edit while a crawl owns the row.** Rejected by the milestone itself: the user
  must be able to fix a name upstream got wrong. The rule that makes both possible is the
  merge: the crawl refreshes the fields it owns, and the fields the user stated merge LAST.
* **Let the user edit the auction columns.** Rejected: `price_min`/`price_market`/`price_*` are
  live data the crawl refreshes, and an override "fixing" one would be a permanent lie about
  CURRENT prices — the whole point of a gold tool. `id` and the icon columns are out too
  (the id is upstream's, the bytes are v0.6.0's stage's), so the whitelist is
  `name, quality, item_level, required_level, item_type, item_subtype, flavor, vendor_sell,
  tooltip_html` — TEXT or INTEGER columns this DB can bind verbatim. The JSON-shaped columns
  (`sockets`, `classes`, `stats`, `effects`, `faction_prices`) stay out: binding a hand-typed
  blob into a column that carries upstream's JSON would invent a shape inside the DB.
* **Make a manual row crawlable** (pretend the id is upstream's). Rejected: upstream has no
  such id, so the queue would ask a record that cannot exist and stamp "gone" on the user's own
  row. All three pending rules carry `manual_at IS NULL` for that reason, and the live card of
  a manual row sends ZERO requests.
* **Keep the v0.7.0 answer for a manual row** (`not-found`: "fetch it with sync --stage
  detail"). Rejected: that message promised a fetch every pending rule then refused.

## Decision

**One schema bump, four additive groups, chained migration.** `SCHEMA_VERSION` `3 → 4`;
`SQL_MIGRATE_TO_V4` is eight `ALTER`s (four tooltip, four manual) and the chain v1→v2→v3→v4
brings the user's own 19 258-row file up in one call — ALTER appends, so a migrated file's
column order differs from a fresh one and every reader names columns explicitly.

**The tooltip copy is stage 4, with the plumbing every stage has:**

* `sync --stage tooltip [--budget N] [--ids …] [--watch]`; its queue `SQL_PENDING_TOOLTIP` is
  the twin of the other two: a DETAIL stamp is required (the door answers upstream's own
  404 JSON for an id it has not, so asking an unlisted id costs a request AND stamps a state
  about a fact upstream never stated), plus no `tooltip_at`, no `tooltip_missing_at`, not
  tombstoned, not manual.
* Three states like every door: copied (`tooltip_html` + `tooltip_at` + `tooltip_etag`),
  checked gone (`tooltip_missing_at` — one `404` on the tooltip of an EXISTING record, never
  asked again inside one crawl), never looked at. `write_tooltip` upserts, `mark_tooltip_missing_at`
  checks a row exists first (`SQL_ROW_OF_ID`, ADR 0006's guard).
* The stage revalidates with the stored `tooltip_etag`, honours `Retry-After` and retries a
  `429` IN PLACE (a rate limit never aborts a stage — ADR 0002/ADR 0004's contract), and only
  a status that is neither `200/304/404/429` lands on exit `3`.
* The stage shares the same `RateBudget` (measured headroom then `500 ms` per request) and the
  same cancel granularity; a full pass is `≈ 19 258` requests `≈ 2.7 h`, so it is staged,
  resumable and cancellable like the detail and icon stages, and "Pull all info" chains a
  fourth leg.
* ADR 0007's price sample stays on the DETAIL write only: a tooltip re-ask is not a second
  observation of the auction house in this build.

**The manual layer, one rule:**

* **merge last.** `manual_overrides` is one JSON ARRAY per row of `{field, value, at}` entries.
  Every write path applies it AFTER the payload binds — `upsert_summary` (list),
  `upsert_detail` (detail) and `write_tooltip` (tooltip copy) each call
  `apply_manual_edits_*`/re-read the override — so a user-said field survives the next crawl
  while the crawl's own columns keep refreshing. The READ paths use the same rule (ADR 0009's
  one-rule-for-both-doors shape): `cached_card` and `cmd_item` apply the edits to the detail
  they print, so a card cannot show a value the DB does not have.
* **`set-item ID FIELD=VALUE`** states exactly one whitelist field of one existing row. It sets
  the column NOW, replaces that field's override entry and appends the new one (stored order
  stays chronological, the merge is last-wins), stamps `edited_at`, and rides the SAME write for
  the derived search columns (`name_lc` when the field is `name`, `quality_rank` when it is
  `quality`) — so a rename is findable by `search` on the same breath, not after some crawl
  touches it. DB-only: a bad ask answers `2`, an unknown or tombstoned row answers `4`, never
  `3`, because no request leaves the verb.
* **`add-item FIELD=VALUE […]`** creates in the reserved block, stamped `manual_at` (and
  `edited_at`), and wears its fields as overrides FROM BIRTH — so even a future upstream record
  colliding with that id cannot take the user's item. A row without a `name` is refused
  (usage): `add_item_manual` will not create a name-less row.
* **`delete-item ID` is the tombstone** (`deleted_at`), `undelete-item ID` clears it. The row
  keeps its stamps, its bytes, its samples and its overrides, which is why bringing it back
  restores everything. A tombstone is refused by every door — `visible_clause()` in search and
  count, the three pending queues, both card doors (BEFORE reading a column or sending a
  request), `set-item` — and `stats` counts it (`deleted`), because "hidden on purpose" and
  "gone upstream" must stay different answers.
* **A row the user MADE answers its card from the DB's own columns** (`column_card`/`SQL_COLUMNS_CARD`)
  on BOTH card doors, and the live door short-circuits BEFORE building a client — ZERO requests,
  because upstream cannot know the id. Fields only an upstream record can state answer `null`
  rather than being invented: the wire's own labels (`kind`, `version`, `version_label`, `build`),
  its `links` and `attribution`, the auction and weapon columns (crawl-owned, outside the
  whitelist) and the JSON-shaped columns.
* **The card tells which is which.** `card_manual_parts` prints every door's three states
  (`tooltip_html`/`tooltip_at`/`tooltip_missing_at` next to `icon_b64`/`icon_at`/
  `icon_missing_at`) plus the manual layer's own stamps (`manual_overrides` as the stored JSON
  ARRAY verbatim — the same text the merge reads — `manual_at`, `edited_at`, `deleted_at`), so
  "the crawl has not reached it" and "a user already said something about it" stay different
  answers, and the UI marks each row whose field the user stated, from the card's own array
  rather than from what the panel typed.

## Grounding, not invention

```
# what the record the crawl reads carries (read back off raw_json, the user's own file):
detail keys = attribution, icon_url, id, item_level, item_subtype, item_type, links,
              name, price, quality, required_level, unique, vendor_sell, weapon
              (no description, no html, no fields)
# what the tooltip door carries (one request, 2026-11-20): 3060 b, 11 top keys,
  html 1404 b of upstream's own markup, ETag W/"2b111cbcd3bbb4d7078f6d05",
  cache-control public, max-age=300 (…, s-maxage=900, stale-while-revalidate=3600)
# what it answers for an id it has not: 404, 79 b, upstream's own JSON, no ETag offered
# why the reserved block is provably clear: MAX(id) on the user's file = 288 340,
  MANUAL_ID_BASE = 900 000 000
# why a tombstone and not a delete: upsert_summary UPSERTs every listed id —
  the next list pass would resurrect a hard-deleted row
```

The fixture replays the tooltip door's SHAPE with its own records (`TOOLTIP_PAGES` for 25,
40, 60 and 127555; `html` is a small stand-in card in upstream's own class names, NOT upstream
prose — we never copy its text) and answers a `404` with upstream's own JSON body for every
other id, so one answered record has a real twin for the third state. It revalidates the door
like the detail door, because the probe read a real ETag and `max-age=300`.

## Audit (every claim above executed, not asserted)

| probe | expectation | observed |
| ----- | ----------- | -------- |
| schema chain on the mirror DB | a fresh DB and a migrated one answer the SAME columns | fresh `status` writes schema `4`; `sqlite3 -readonly .schema item` names `tooltip_missing_at` (slot 44) and `icon_blob BLOB` (45); a genuine schema-3 file migrated by this build lands at `schema_version` 4 with `icon_blob` present and its rows intact (the fresh-schema comma bug ADR 0010's `fix(db)` commit caught) |
| tooltip stage on the CI mirror (fixture, fresh DB, budget 8) | the copy path lands, the gone tooltip is checked once | `requests 5, items 3, written 3, missing 1, rate_limited 1, complete true`, message `tooltip crawl copied 3 copies; 1 answered 404 for the tooltip and are checked as no-tooltip-upstream`; asked ids `25, 40, 60, 12809` — `12809` is the third state |
| the same stage drained | asks nothing again | `requests 0, items 0, written 0, complete true`, `tooltip stage: 0 copies fetched (0 requests)` |
| tooltip `--ids` on the gone tooltip | a human ask may re-ask once, and it marks rather than invents | `sync --stage tooltip --ids 12809` → **exit 0**, `requests 1, items 0, missing 1, complete false` (message `0 of 1 handled, 1 left in the list; 1 answered 404 for the tooltip and are checked as no-tooltip-upstream`); the DB says `tooltip_missing_at` set, `tooltip_at` NULL, `tooltip_html` NULL — and the queue that follows answers `requests 0, complete true`, so a crawl never re-asks it |
| the tooltip door's own ETag | a re-ask costs a 304 and writes nothing | `sync --stage tooltip --ids 25` again in the same fixture process → **exit 0**, `requests 1, items 1, written 0, unchanged 1` |
| tooltip stage usage door | a mistyped stage is a usage error | `sync --stage tooltipx` → **exit 2**, `sync wants --stage list, detail, icons or tooltip, got tooltipx` |
| tooltip door on a dead host | the network door lands on `3`, and the message proves the URL | **exit 3**, `error sending request for url (http://127.0.0.1:9073/api/v1/tooltip/forever/item/25)` |
| `set-item` on a whitelist field | one field, one row, the answer names the stamp | `set-item 25 vendor_sell=606` → **exit 0**, `{"id": 25, "field": "vendor_sell", "value": "606", "edited_at": 1795218726}`; the DB row keeps `vendor_sell` 606 and `manual_overrides` `[{"field":"vendor_sell","value":606,…}]` |
| a bad edit ask | usage, three shapes, never a silent write | `vendor_sell=six` → **2**, `an editable INTEGER field wants a plain number`; `nose=1` → **2**, `nose is not editable (name, quality, …)`; two pairs → **2**, `set-item states exactly one FIELD=VALUE per ask` |
| an edit of a row that is not there | the fourth family, with the offer | `set-item 999999999 name=X` → **4**, `no item 999999999 in the DB (add-item creates one)` |
| `add-item` with no name | usage | `add-item quality=Rare` → **2**, `add-item wants at least a name (name=...)` |
| `add-item` lands in the reserved block | the DB picks the id, never the user | `add-item "name=Probe Mug" quality=Rare vendor_sell=250` → **exit 0**, `{"id": 900000000, "manual_at": 1795218726, "fields": ["name","quality","vendor_sell"]}` |
| a manual row's card on BOTH doors | the DB's own columns answer, ZERO requests | `item 900000000 --cached` and `item 900000000` (the live door) both → **exit 0**, 940 b, `status null`, `detail_synced_at null`, `name Probe Mug`, `quality Rare`, `price`/`weapon`/`kind`/`links` null; the fixture log holds **0** asks for 900000000, before and after the three queue stages ran |
| the queues refuse a manual row | no crawl walks a row upstream cannot know | `sync --stage detail`/`icons`/`tooltip` after the create each answer `requests 0, complete true`, and the fixture's ask count for 900000000 stays **0** |
| an edit is findable on the same breath | the derived columns ride the same write | `set-item 40 name=ProbeRenamed` → **0**; `search ProbeRenamed` → **0**, `total 1`, hit id 40 named `ProbeRenamed`; the DB says `name` "ProbeRenamed" and `name_lc` "proberenamed" in one write, `manual_overrides` carries the entry |
| a tombstone hides the row everywhere | one clause, every door agrees | bare `search` total **6 → 5** across `delete-item 60`; `delete-item 60` again → **4**, `item 60 is already tombstoned`; `item 60 --cached` → **4** `item 60 is tombstoned at 1795218727 (undelete-item 60 brings it back)`; `set-item 60 name=TombstoneTest` → **4**, `… (undelete-item first)`; `stats` shows `deleted 1` while hidden |
| the tombstoned LIVE card sends nothing | refusing before the client | `item 60` (live) → **exit 4**, and the fixture's ask count for `forever/item/60` stays **2** — unchanged across the tombstone |
| bringing it back | everything it had comes back | `undelete-item 60` → **0**, `{"restored": [60]}`; again → **4**, `item 60 is not tombstoned`; bare `search` total back to **6**, `stats` back to `deleted 0` |
| a crawl cannot fight an edit | the merge survives a FRESH write from a second process | `sync --stage detail --ids 25` against a SECOND fixture process → **0**, `requests 2, written 1, rate_limited 1`; the row keeps `vendor_sell` 606 while `price_min` moves 15528 → 19913, `tooltip_html` stays in the row, and `price_history` grows 4 → 5 with exactly one new sample (ADR 0007's rule: only a fresh detail write samples) |
| the manual layer writes no ledger rows | edits and creates are not observations | after the GUI probe: `edited 2`, `manual 1`, `price_history` still **4** |
| CI mirror (the committed 21-step sequence, fresh DB, fresh port) | green with the tooltip stage and the four manual doors inside | **21/21 steps exit 0** (port 9071); `stats` coverage `items 6, named 6, priced 4, detailed 4, missing 1, icons 3, icon_missing 1, tooltips 3, tooltip_missing 1, edited 2, manual 1, deleted 0, history 4`; the DB ends `6|4|3|1|1|0|4` (rows\|detailed\|tooltips\|tooltip_missing\|manual\|tombstones\|samples); `sync_state` four rows `complete` (`list page 2`, `detail 0/2/2`, `icons 0/4/4`, `tooltip 0/0/0`); the fixture saw `31` GETs — detail 13, list 5, icons 5, tooltip 5, search 3 — with `4` × `429`, `9` × `304`, `3` × `404`, and **0** asks for 900000000 |
| exit-code audit (57 doors: the eighteen of v0.6.0, the seven of v0.7.0, the twelve of the tooltip stage and queues, the fourteen of the manual layer, the fresh write) | the old doors stay identical, the new ones land where ADR 0004 says | old `0,0,0,0,4,4,0,2,0,2,2,3,3,3,0,0,0,0` then `2,3,0,0,0,0,3` then tooltip `0,0,0,2,3` then manual `0,2,2,2,2,4,2,0,0` then queues `0,0,0`, edit `0,0,0,0`, tombstone `0,4,4,4,4`, restore `0,0,4,0,0`, fresh write `0` — no DB-only door ever reached `3` |
| GUI audit (scratch DB seeded by a fixture crawl) | the shipped page boots FOUR verbs; the write panel's doors reach the same ipc guard | run 1: `status, watch, stats, status` and ZERO requests (the new section fires nothing itself); temp probe (four write verbs + the manual row's live card, then reverted): the ipc log grows exactly `add-item (2 words), set-item (2 words), delete-item (1 words), undelete-item (1 words), item (1 words)` and the fixture saw **0** requests during that run, the DB ends `6` rows, `manual 1`, `edited 2`, `deleted 0`, samples `4`; clean re-audit after the revert: four verbs again, `git status --porcelain ui/` clean |
| the user's own DB | never written by any of this | audited read-only (`sqlite3 -readonly`): schema `4`, 19 258 rows, max upstream id `288 340`, tooltips `0` (the copy pass had not started while the detail stage was still walking) |
| the build's dead-code set | the documented one | `cargo build` warns exactly `catalogue`, `open_db`, `pending_plan`'s private `PendingSql` and **`Tooltip`'s unread fields** (the measured key list rides the model, only `html` is read) — plus the hardlink notice on this filesystem when the path doc recompiles |
