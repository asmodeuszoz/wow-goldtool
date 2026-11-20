# ADR 0009 — the live-search door: one human ask, one request, a look not a sync

**Status:** adopted (v0.7.0)
**Related:** `src/cli.rs` (`cmd_live_search`, `join_tail`, the usage line), `src/wowdb.rs`
(`search_url`, `encode_query`, `ApiResponse.remaining` via `remaining_of`), `src/models.rs`
(`SearchResponse`, `groups` stays a `Value`), `src/gui.rs` + `ui/app.js` + `ui/index.html`
(the live-search panel), `tests/fixture_api.py` (the grouped `/search/` answer, ungated),
docs/API.md ("Search" section), [ADR 0004](./0004-headless-cli-mode.md),
[ADR 0007](./0007-price-sampling.md)

## The measurement that forced the decision

v0.6.0 shipped three names compiled but never used: `search_url`, `SearchResponse` and
`ApiResponse.remaining` — the door docs/API.md already promised ("we use it for the live
search panel") but never opened. Before deciding what the door does, the door was measured
— live, once each, 2026-11-20 (UA `wow-goldtool 0.6.0 grounding probe; data source: thewowdb.com`):

```
GET /api/v1/forever/search/?q=Worn%20Shortsword      -> 200 application/json 1288 b
   etag: W/"76db61a78b8398bc089262d2"
   x-ratelimit-limit: 120   x-ratelimit-remaining: 119
   access-control-expose-headers: ETag, Retry-After, X-RateLimit-Limit, X-RateLimit-Remaining, X-WGF-Build
   top keys: attribution, groups, q, version ("forever")   q echoes DECODED: "Worn Shortsword"
   groups: exactly item, npc, quest, sound, spell, zone, each { has_more, more, results }
   the one item hit's keys: id, item_level, kind, links {api,html,tooltip}, name,
                            quality, required_level, version
   every group's `more` is upstream's own paged list endpoint, e.g.
   https://thewowdb.com/api/v1/forever/item/?page=1&per_page=50&q=Worn+Shortsword
   — inside the ANSWER's URLs spaces ride as `+`, not %20

GET /api/v1/forever/search/?q=Ring                    -> 200 application/json 7271 b
   item  5 hits has_more TRUE   quest 5 hits has_more TRUE   spell 5 hits has_more TRUE
   npc   4 hits has_more false  zone  2 hits has_more false  sound 0 hits has_more false

GET /api/v1/forever/search/?q=socket                  -> 200 … every group EMPTY
   (the names-only trap: items "with socket" exist in the catalogue, but the door
   matches NAMES, so the word finds nothing - docs/API.md measures it)
```

Three facts decide the door's shape, all measured, none invented:

* the answer is NAMES-only, so an **empty answer is a success** — the door must not treat
  it as a failure (a word upstream never said);
* every group carries its own `has_more` + `more` — chasing it would ask the paged
  per-group endpoint, one request per group per page: a human ask becomes a crawl;
* the answer is upstream's own JSON printed as it came (serde pas-through): the `+` in
  `more` is upstream's shape and rides verbatim; our own request URL uses `%20` because
  the measured ask asked with `%20` and got 200 back (a raw space is not a URL character).

`src/models.rs::SearchResponse` already matched the live shape EXACTLY — `groups` as a
`Value` — so the mirror needed no change: the door consumes what the model already mirrored.

## Options considered

* **Write hits into the DB from a live ask.** Rejected: writes are bounded, paced,
  resumable and stamped; an ask is one human click, and ADR 0007's sampling rule makes a
  write an OBSERVATION — an answer that is not a record's own detail body would pollute
  the price ledger with rows that never came from a detail write. A look wearing a sync's
  clothes is exactly the drift the third states exist to make visible.
* **Chase `more` in v0.7.0** (walk each group's paged pages). Rejected on the measured
  answer: `?q=Ring` says `has_more true` on THREE groups with five results each — chasing
  means three-plus extra requests and more per page, a budget, pacing and a `sync_state`
  row: that is a crawl, not an ask. v0.7.0 stops at one ask, one request; a later patch
  could take `--page` honestly.
* **Retry in place on a 429** (the crawl's Retry-After dance). Rejected: that dance belongs
  to a stage that OWNS a queue and a budget; a human ask has neither, so the door mirrors
  the `item` door — a 429 is the exit-`3` door with `Retry-After` in the payload. The fixture
  leaves its `search` stage ungated for exactly this reason (the committed CI step must
  stay green).
* **Reuse the local verb `search` for the live door too.** Rejected: two doors, one name.
  `search TEXT` reads the DB with filters; a live ask is a different door and the usage
  line says both (`search TEXT […]` and `live-search TEXT`).
* **A local-only panel** (drop the live door and print what the DB has). Rejected by
  measurement: the live door answers kinds the DB never holds (spells, quests, npcs,
  zones, sounds) and NAMES-only is a fact of the endpoint — the panel exists so the
  upstream's own answer is visible, marked as such, next to the local search that
  answers 19 258 rows offline.
* **Reshape `SearchResponse` into typed groups.** Rejected: pas-through printing means
  the door answers what upstream answered, and a new kind upstream (a seventh group)
  must not break a build — the panel renders the kinds it knows about and prints the
  rest of the answer verbatim.

## Decision

* CLI verb `live-search TEXT`: the WHOLE verb tail joined with spaces is the query —
  `live-search Worn Shortsword` asks the API for "Worn Shortsword", not for two flags.
* One human ask = ONE request, like `item`. Exit codes ride ADR 0004's contract:
  `2` no text (usage), `3` network/rate-limited (with `Retry-After` in the payload),
  `0` even when every group answers empty (the names-only trap is a success, not a
  failure), `4` if the door itself answered 404 for the query shape.
* The payload is the door's own answer: `{status, retry_after, remaining, answer}` —
  `answer` is the serde pas-through of `SearchResponse` (`groups` stays a `Value`), and
  `remaining` comes from the burst headers measured above (`x-ratelimit-remaining: 119`
  on a live ask; `null` when the host sends none, as the fixture does not send burst
  headers on this stage — the panel says so honestly).
* NO DB write and NO DB read: a look, not a sync. The DB path is never opened.
* UI: a live-search panel — one field, one button (`Ask once`), one request per click;
  it renders the groups it knows about with a `view upstream` link per hit (attribution
  reaches the ANSWER, not only the footer), and its note prints `remaining`, the names-only
  fact, and whether any group said `has_more` (this build stops at one ask).

## Grounding, not invention

```
# the three answers above are verbatim live answers (2026-11-20);
# this build's OWN live asks (one request each, 2026-11-20, UA `wow-goldtool 0.7.0 …`):
live-search "Worn Shortsword"   -> 200, item group 1 hit, has_more false, remaining 119
live-search "Ring"              -> 200, item/spell groups 5 hits has_more true,
                                   more = …/forever/item/?page=1&per_page=50&q=Ring
live-search "zzzqqqnothere"     -> 200, all six groups EMPTY, exit 0
# the two-word ask rides as `?q=Worn%20Shortsword` (measured by the fixture's own log)
# upstream's `more` URLs carry the `+` form and ride back UNCHANGED in the answer
# the door's percent-encoder: letters/digits/`-._` ride through, a space is `%20`,
# everything else its UTF-8 bytes as `%XX` - the shape the measured ask proves
```

## Audit (every claim above executed, not asserted)

| probe | expectation | observed |
| ----- | ----------- | -------- |
| door with no text | a usage error that names the ask | `live-search` → **exit 2**, `live-search wants text, for example: live-search Worn Shortsword` |
| live grounded asks | exit 0, upstream's answer as it came | `live-search Worn Shortsword`: `status 200, remaining 119`, item group **1 hit** `has_more false`; `live-search Ring`: item/spell **5 hits** `has_more true`, `more` = `https://thewowdb.com/api/v1/forever/item/?page=1&per_page=50&q=Ring` — the `+` inside upstream's own URL rides verbatim |
| names-only trap, live | empty groups still success | `live-search zzzqqqnothere` → **exit 0**, all six groups `results: []` |
| the door on a dead host | the network door lands on `3` | `live-search ring --fixture http://127.0.0.1:9974` → **exit 3**, `error sending request for url (http://127.0.0.1:9974/api/v1/forever/search/?q=ring)` — the URL in the message proves the verb's own URL building |
| two-word ask on a dead host | the percent-shape rides in the message | **exit 3**, `error sending request for url (http://127.0.0.1:9975/api/v1/forever/search/?q=Worn%20Shortsword)` |
| two-word ask against the fixture | percent on the wire, decoded in the answer | fixture GET log line `GET /api/v1/forever/search/?q=Worn%20Shortsword HTTP/1.1" 200`; the door echoes `q` decoded as `Worn Shortsword`; the fixture's own `more` carries the `+` form `q=Worn+Shortsword` |
| the door touches no DB | an impossible `--db` still answers | `live-search sword --db /tmp/opencode/exit70-nodb-dir/none.sqlite` → **exit 0** with the answer; the directory `exit70-nodb-dir` was never created — opening that path would have answered exit 4 |
| the door's 429 branch (a SCRATCH fixture copy gates `search` once — the committed one never does) | `3`, with `Retry-After` in the payload | **exit 3**, `rate limited by the API: the API asked us to back off`; the scratch log answers the ask `429` — the committed fixture's log answers it `200` (its gate names list/detail/icons only) |
| CI mirror (the committed fifteen-step sequence) | green with the door inside | **15/15 steps exit 0** on a fresh fixture DB; the icon stage's numbers unchanged (`requests 5, items 4, written 3, missing 1, rate_limited 1, complete true`); the price ledger stays `4` after CI and grows to `5` ONLY through a FRESH fixture process' detail write — the door wrote no sample (ADR 0007's rule still holds); fixture log `28` GETs, `3` of them the door (`sword`, `Worn%20Shortsword`, `tooltip`); the `tooltip` ask answers all-empty groups and its step still passes at exit 0 |
| exit-code audit (twenty-seven doors) | the eighteen of v0.6.0 stay identical, the new verb adds its doors | `0,0,0,0,4,4,0,2,0,2,2,3,3,3,0,0,0,0` for the old doors, then `2,3,0,0,0,0,3,3` for the new ones (no text, dead host, empty groups, two grounded fixture asks, the no-DB door, the two-word dead host, the scratch-gated 429) and the fresh-window write |
| GUI launch audit (the shipped page) | boot stays four verbs, nothing crawls | `status, watch, stats, status` and NO `live-search` line — the panel moves only on its own button; DB `0` items, `0` samples, empty `sync_state` |
| GUI live probe (temporary, reverted) | the panel's ask reaches the SAME ipc door | a 5th line `gui: ipc verb live-search (1 words)`; probe then reverted and rebuilt, and the clean four-verb boot re-audited |
| the user's own DB | never written by any of this | audited read-only; hash `7442814b…c3d3b8` and mtime `2026-11-19 21:42:54` identical before and after every audit |
