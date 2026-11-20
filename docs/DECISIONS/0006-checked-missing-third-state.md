# ADR 0006 — a record the upstream dropped is a checked state, not a failure

**Status:** adopted (v0.5.3)
**Related:** `src/db.rs` (`detail_missing_at`, `SQL_MARK_MISSING`), `src/sync.rs`
(`HttpKind::Missing` branch), `src/stats.rs` (`SQL_PENDING`), docs/API.md, docs/DATABASE.md

## The measurement that forced the decision

A record can be **listed** by the `?page=` walk and still have **no `forever` detail
record**. That is not a hypothesis: the v0.5.2 build, run against the live catalogue
(`build: wow_classic_beta-1.60.1.70532`), died on the first such id.

```
$ cargo run -- sync --stage detail --budget 3      # scratch DB, 3000 listed rows
fetching item details: this is the slow stage, about two records per second
item 41 answered 404
$ echo $?
3
```

Those ids come straight out of the user's own walk: `41` is `OLDRecruit's Belt`,
`42` is `OLDSquire's Belt`, `46` is `OLDFootpad's Belt`, and the same name shapes
(`OLD…`, `Deprecated …`, `… (TEST)`) are visible in the 19 258 rows already on disk.
The upstream's own words for that answer are a body, not a bare status line:

```json
{"error":"not_found","message":"No item 5828 in forever.","status":404}
```

So the v0.5.2 "Pull all info" flow, the one click that is supposed to retire all
pending work, **cannot finish on the real catalogue**: it aborts at the first removed
record and leaves the crawl unpromised. The fixture never showed this because it
answered every id. Inventing a shape is not our call to make — the body above is what
the live API returns, and `404` is the status it returns with it.

## Options considered

* **Stay fatal** (the v0.5.2 behaviour). Rejected: on the user's DB the abort is not
  an edge case, it is the common case — the walk lists thousands of ids upstream no
  longer has records for, so the crawl dies and nothing downstream (v0.6.0's price
  history, icon copies, item cards) ever gets a complete catalogue. A manual
  `--ids` walk-around asks a human to dodge a fact the crawler already knows.
* **Skip silently, remember nothing.** Rejected: the ids stay pending, so every later
  run re-asks the same dead ids. A catalogue with removed records then spends its
  rate budget on records that cannot arrive, and "there is nothing left to pull"
  becomes permanently unprovable.
* **Re-check missing ids on a cadence** (mirror the watch table's staleness rule).
  Rejected for now: it makes "nothing pending" a lie, and nothing measured says a
  removed record ever comes back. One request per crawl per removed id is already
  the cheaper floor.
* **Fold the fix into v0.6.0.** Rejected: v0.6.0 renders and stores what a finished
  crawl produced; shipping a crawl that cannot finish next to the features that
  depend on it makes both broken. This is a patch under the 0.5 line, the same way
  v0.5.1 and v0.5.2 were patches.

## Decision

A detail record has **three states**, and the third one is stored:

| state | column set | what it means | cost |
| ----- | ---------- | ------------- | ---- |
| fetched | `detail_synced_at` | upstream answered with a record | revalidate on later passes |
| checked and gone | `detail_missing_at` | upstream answered `404` for a listed id | one request **per crawl**, never again inside one |
| never looked at | neither column | the only thing that is *pending* | fetched once, then one of the two above |

* `classify` grows `HttpKind::Missing`, so "gone upstream" stops being a failure.
* The detail crawl marks the id (`SQL_MARK_MISSING`, guarded by `SQL_ROW_OF_ID` so
  marking cannot invent a row for an id the walk never listed), counts it in the
  report (`missing`) and keeps going.
* `SQL_PENDING` becomes `detail_synced_at IS NULL AND detail_missing_at IS NULL`.
  A removed record is therefore asked about **exactly once per crawl**.
* `stats`/`status` coverage counts them, and the UI prints them, so the third state
  is visible instead of inferred from a missing number.
* A **targeted** ask may still re-ask: `item <id>` and `sync --ids`/`--watch` are
  explicit human requests, and one human ask per id is not a crawl storm. `item`
  answers with the exit-code contract's `4` (`AppError::not_found`) and re-stamps.

## Schema: migrate, do not rebuild

Bumping `SCHEMA_VERSION` from `1` to `2` without a migration path would have refused
the user's file — the honest refusal in `check_schema` exists to stop a half-applied
schema, and here it is a 19 258-row catalogue the user does not want to lose. So the
one additive step of this shape is applied in place:

* `SQL_ADD_MISSING_COLUMN` = `ALTER TABLE item ADD COLUMN detail_missing_at INTEGER`,
  run by `check_schema` when the file on disk says `schema_version = 1`;
* the stamp in `meta` moves to `2` in the same call, so the migrated file is never
  re-migrated and a file **newer** than this build is still refused — the direction
  that cannot be guessed at stays closed.

Grounded against the pre-v0.5.3 file: the 3000-row scratch DB made by the v0.5.2
build migrates in place, keeps its rows and its `sync_state` cursor, and the next
`status` says `schema_version = 2`.

## Audit (every claim above executed, not asserted)

| probe | expectation | observed |
| ----- | ----------- | -------- |
| migrated v1 DB (3000 listed rows) | rows survive, stamp moves | `3000` rows, `schema_version 2`, `detail_missing_at` column present, old `sync_state` cursor intact |
| live detail run on that DB, budget 6 | a 404 does not abort | `requests 6, items 3, missing 3, complete false`, **exit 0**; DB `10` detailed, `3` missing, pending `2987 = 3000 − 10 − 3` |
| second live run, budget 4 | a marked id is not re-asked | `requests 4, items 3, missing 1` — newly fetched ids `47, 48, 49`, the three marked ids absent; pending arithmetic holds |
| local mirror: the committed CI sequence plus the removed-record probes (thirteen) | the third state inside the mirror | detail run `requests 6, items 4, missing 1, complete true`, message ends `; 1 answered 404 and are checked as removed`; `item 127555` → **exit 4** with the not-found text; later crawl `requests 0, complete true`; DB `5` items, `4` detailed, `1` missing, pending `0` |
| exit-code audit, seven probes | the new case lands on `4` | `2 · 2 · 3 · 4 · 4 · 4 · 0` and the mark recorded in the DB |
| GUI pull-all launch audit (fixture, virgin port) | the flow completes with a removed record inside it | verbs `status/watch/stats/status`, then `sync` × 2 with the ticker's `status` polls interleaved, then the probe's `item`; fixture log shows the crawl's **one** `404` plus the probe's targeted `404`; `sync_state` detail row: `complete … 1 answered 404 and are checked as removed`; DB `4` detailed, `1` missing, pending `0` |
| CI mirror (the committed eleven steps) | still green with the fixture's new `404` | every step `exit=0`, DB carries `1` missing |

The user's own DB (`HOME/.local/share/wow-goldtool/wow-goldtool.sqlite`, 19 258 rows
read-only at audit time) was never written to: every run above pointed at a scratch
file through `WGT_DB`.
