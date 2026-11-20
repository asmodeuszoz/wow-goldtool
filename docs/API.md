# TheWoWDB API — notes we rely on

Source of truth: <https://thewowdb.com/developers/> (checked **2026-11-19**). No API key.

## Base shape

```
GET https://thewowdb.com/api/v1/<version>/<kind>/?page=1&per_page=50&q=
GET https://thewowdb.com/api/v1/<version>/<kind>/<id>
GET https://thewowdb.com/api/v1/<version>/search/?q=
GET https://thewowdb.com/api/v1/tooltip/<version>/<kind>/<id>
```

* `version` = `forever` (WoW Forever, beta client data) or `retail`.
  **We pin `forever`** — it is what the classic-style client the tool targets actually
  uses, and it is listed first by the API.
* `kind` = `item`, `spell`, `quest`, `npc`, `zone`, `sound`. We use `item` (and `search`
  across all kinds for the live-search panel).

## Paging

* `per_page`: `1..100` (default 50) — anything above 100 is clamped by the server to 100.
* `page`: `1..200`, i.e. at most 20 000 records reachable by listing.
* Large lists do **not** send a `total`; they answer `has_more` + `links.next`.
  Small lists also send `total`.
* Measured for `forever/item`: last page carrying data is **193** (`has_more:false`,
  57 records) ⇒ **≈ 19 257 items**. Recorded in `docs/DEVLOG.md` on 2026-11-19.
* Fallback rule we implement: stop when `has_more` is false **or** a page comes back
  empty, so we never issue page `201+` requests.

## Search

* `q` is a case-insensitive name match, max 64 chars.
* `/search/?q=` returns grouped hits (`item`, `spell`, `quest`, `npc`, `zone`, `sound`),
  each with its own `has_more` + `more` URL.
* We use it for the "live search" panel; local search is done in SQLite instead (see
  `docs/DATABASE.md`), because the API cannot sort by price.

## Caching & conditional GETs

* Responses are public, cacheable ~1 h (item tooltips ~5 min, they carry live AH prices).
* Every response has an `ETag`; send `If-None-Match: W/"<etag>"` and an unchanged answer
  is **`304` with no body**. We store the ETag per item (`list_etag`, `detail_etag`) and
  revalidate instead of re-parsing on repeat syncs. From v0.5.0 the detail stage sends the
  stored ETag as well, so re-asking for the same ids (`--ids 25,40` twice, or a watch list
  re-sync) answers `304` and writes nothing; the pending stage still sends none, because a
  pending item has no `detail_etag` to send yet.
* A `304` still costs one request against the burst — revalidation saves bandwidth and
  DB churn, not rate budget. Because a `304` carries no body, the list crawl also caches
  each page's `has_more` and `count` in the local `meta` table so it can decide whether
  to walk on without parsing anything (v0.3.0, keys documented in docs/DATABASE.md).

## A record upstream dropped (`404`)

* The `?page=` walk still lists ids whose `forever` detail record no longer exists. The
  API answers `404` **with a body**, and the body is the upstream's own words, not ours:

  ```json
  {"error":"not_found","message":"No item 5828 in forever.","status":404}
  ```

  Measured against the live catalogue (`build: wow_classic_beta-1.60.1.70532`): ids `41`
  (`OLDRecruit's Belt`), `42` (`OLDSquire's Belt`) and `46` (`OLDFootpad's Belt`) answer
  that way while the walk lists them; the same name shapes are visible among the ids the
  user's own walk stored.
* Our policy (`wowdb.rs::classify` → `HttpKind::Missing`, [ADR 0006](./DECISIONS/0006-checked-missing-third-state.md)):
  - the detail crawl marks the id (`detail_missing_at`) and **keeps crawling**: a removed
    record is a fact about the catalogue, not a failed request, so it does not abort the
    run and does not eat the network exit code;
  - the mark means later crawls do not re-ask it — one `404` per removed record **per crawl**,
    which is what a 19 k-record walk needs;
  - `404` is never retried in place: retrying a `429` waits on a `Retry-After`, and a `404`
    offers nothing to wait on;
  - a targeted ask (`item <id>`, `sync --ids`, `sync --watch`) may re-ask once per human
    request and answers the exit-code contract's `4` (`not-found`);
  - a **list page** that answers `404` is still a failure: pages exist as long as the walk
    is told to walk, so that shape stays `HttpKind::Failed` and exits `3`.

## Rate limits

* **Per IP:** a burst of 120 requests, refilled at **2 requests/second**. Beyond that the
  API answers **`429`** with `Retry-After` and reports `X-RateLimit-Remaining`.
* Our policy (see `wowdb.rs::RateBudget`):
  - keep 30 requests of headroom under the 120 burst (never plan on the whole burst),
  - after the headroom is spent, wait ~`500 ms` per request (≈ 2/s) before sending,
  - on `429`: sleep for `Retry-After` (default 2 s if the header is missing), refill the
    budget to a low value and continue — never abort a sync because of a 429,
  - always send a descriptive `User-Agent` (`wow-goldtool <version> (<repo>)`) so a
    misbehaving client can be contacted,
  - never parallelise more than 2 workers against one host.
* Cost of a full sync: 193 list requests ≈ 2 minutes. Detail sync of all ≈ 19.3 k
  requests ≈ 2.7 h, which is why it is staged, resumable, cancellable and opt-in.

## Terms (we comply)

* Free for any project, including commercial.
* **A visible link to TheWoWDB (or the record's `links.html`) wherever the data is shown**
  → rendered in the UI footer and in every item detail card.
* Do not resell the raw data as a dataset. We only ship per-item caches inside the app.
* Breaking API changes ship under `/api/v2/` — we pin `/api/v1/` explicitly.
