# TheWoWDB API — notes we rely on

Source of truth: <https://thewowdb.com/developers/> (checked **2026-11-19** and again **2026-11-21** — exactly FOUR doors both times: list, record, search, tooltip). No API key.

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
  each with its own `has_more` + `more` URL. Measured 2026-11-20 (ADR 0009's transcript):
  the answer's top keys are `attribution`, `groups`, `q`, `version` (`"forever"`), `groups`
  holds EXACTLY the six kinds, each answers `{has_more, more, results}`, and a hit is NOT a
  full record — the one item hit measured for `q=Worn%20Shortsword` (1288 b) carries `id`,
  `item_level`, `kind`, `links{api,html,tooltip}`, `name`, `quality`, `required_level`,
  `version`. Inside the ANSWER's own `more` URLs a space rides as `+` (upstream's shape):
  `…/api/v1/forever/item/?page=1&per_page=50&q=Worn+Shortsword`. `?q=Ring` (7271 b) answers
  `has_more true` with five results each on item/quest/spell and 4/2/0 on npc/zone/sound.
* The door asks with the query percent-encoded (`%20` for a space — the shape the measured
  ask used and the fixture's GET log later confirmed), and the answer echoes `q` DECODED.
* The burst headers ride every answer: `x-ratelimit-limit: 120`, `x-ratelimit-remaining: 119`
  on a measured live ask, CORS-exposed per
  `access-control-expose-headers: ETag, Retry-After, X-RateLimit-Limit, X-RateLimit-Remaining, X-WGF-Build`.
  The answer also carries an `ETag` (measured `W/"76db61a78b8398bc089262d2"`), but v0.7.0's
  door stores nothing and revalidates nothing: one human ask is one request (ADR 0009), so
  a `429` here answers with `Retry-After` and takes the door to exit `3`, it is not retried
  in place like a crawl stage would.
* Measured 2026-11-20: `?q=socket` and `?q=Socketed` answer **empty result groups** — the
  match is over NAMES only, so the live search can never find a record by tooltip text
  ("socketed") even though such items exist in the catalogue. Local SQLite search has the
  same limit (it indexes names), and no part of the app claims otherwise.
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
* Answer-rate measured 2026-11-20 against a completed 193-page walk: a 75-id spread asked
  of the stored ids answered **23 records and 52 × 404**; a contiguous block answered
  18 of 46. Most listed ids simply have no `forever` detail record — which is exactly why
  the checked state costs one request per id per crawl (a full detail pass that re-asked
  them every run could never finish), and why the ≈ 19.3 k-request figure below is an
  upper bound: the honest queue is "listed and never looked at", not "listed".

## Icon files (the asset hosts behind `icon_url`)

Measured 2026-11-20 against the live catalogue — this is where the v0.6.0 icon copies
come from, and these two shapes are the ONLY two seen on answered records:

* Every answered `forever/item` record carries an ABSOLUTE `icon_url` on one of:
  `https://render.worldofwarcraft.com/<region>/icons/56/<name>.jpg`
  (`/us/`, `/forever-eu/`, `forever-us` variants) or `https://thewowdb.com/assets/icons/<name>.jpg`.
  Measured bytes: `.../icons/56/inv_mace_03.jpg` → **200 `image/jpeg` 1945 b**;
  `thewowdb.com/assets/icons/inv_axe_11.jpg` → **200 `image/jpeg` 1936 b** (56..64 px squares).
* An answered record CAN carry an **empty** `icon_url` (`"icon_url": ""`) — measured
  live 2026-11-20 on 4 of the 29 detailed rows of the scratch catalogue (Peace Tea,
  Craftsman's Writ: Mechanical Repair Kit, Scroll of Imbue Iceknife, Lodestone). It is
  not a broken URL and not a `404`: upstream answered "no icon here", so the icon stage
  must NOT treat such records as pending work (`SQL_PENDING_ICON` requires a NON-EMPTY
  url), and no third-state stamp is written for them either.
* An absent icon FILE answers differently per host, both measured 2026-11-20:
  `thewowdb.com/assets/icons/<bogus>.jpg` → **404 `text/html`** (the site's own 40 147-b
  page, whose body is prose and never an icon), and
  `render.worldofwarcraft.com/us/icons/56/<bogus>.jpg` → **403 `application/xml`** (243 b,
  the CDN's S3-style `AccessDenied` answer). The crawler reads the STATUS for icon bytes and
  discards the body; the icon stage's third state (`icon_missing_at`) is written for the
  `404`, because that answer's own words say "not found here" — the `403` says `AccessDenied`,
  which states no such thing about an icon, so the stage treats it as a stage failure
  (exit `3`, resumable, nothing marked, the icon stays pending): a fact the upstream never
  stated stays unstated (docs/DATABASE.md, ADR 0008). A bogus path on either host was only
  EVER asked by these probes — every real `icon_url` measured so far answered `200`.
* The bare root-relative form (`/icon/...` appended to a host root) answers **404**
  (measured: `thewowdb.com/icon/<bogus>.png` → 404 `text/html`): NO live record uses that
  shape. The fixture does — against the base the process already points at
  (`WGT_FIXTURE`), which is the only server that can serve fixture bytes offline.
  `wowdb.rs::icon_target` resolves exactly the two live shapes (absolute `http…` as
  measured) plus that base-relative form for the fixture, and REFUSES to guess any other
  (the stage then fetches nothing, marks nothing: a fact the upstream never stated stays
  unstated).
* The icon hosts are NOT the rate-limited API endpoints, but politeness is per host:
  the stage shares the same 2 req/s schedule and honours `Retry-After` if answered
  (ADR 0008). Full-copy cost: ≈ one request per answered record (≈ 2.7 h at 2/s —
  staged, resumable, cancellable, like the detail stage).
* No icon ETag is stored: v0.6.0 copies each icon exactly ONCE (the pending rule
  excludes copied icons, see `docs/DATABASE.md`), so there is no revalidation path
  whose promise we would need to keep.

## Tooltip copy (the door the description lives behind)

Measured 2026-11-20 against the live catalogue — this is where v0.8.0's description copies
come from, and it is why the crawl grew a FOURTH door at all ([ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md)):

* The `forever/item` record carries **no description of any kind**. Measured top-level keys of
  the detail payloads the user's own walk stored (`raw_json` of id `25`, read back off their
  file): `attribution, icon_url, id, item_level, item_subtype, item_type, links, name, price,
  quality, required_level, unique, vendor_sell, weapon` — no `html`, no `fields`, no prose field.
  Inventing a description column for the detail door would have invented a field upstream never
  answers.
* The text lives behind `GET /api/v1/tooltip/<version>/item/<id>`, measured:
  `https://thewowdb.com/api/v1/tooltip/forever/item/25` → **200 `application/json` 3060 b** with
  top keys `attribution, build, fields, html, icon_url, id, kind, links, name, quality,
  version`, ETag `W/"2b111cbcd3bbb4d7078f6d05"`,
  `cache-control: public, max-age=300, s-maxage=900, stale-while-revalidate=3600` (tooltips are
  the ~5-min answers the caching section already names), `x-ratelimit-limit: 120`,
  `x-ratelimit-remaining: 62` mid-burst, `x-wgf-build: wow_classic_beta-1.60.1.70532`.
  `links.api` is the door itself; `links.html`
  (`https://thewowdb.com/wow-forever/item/worn-shortsword-25/`) is the record's own upstream
  page — the attribution target the terms require.
* Inside the copy (measured again for ADR 0012, 2026-11-21): every tag is a `<div class="twdb-*">`
  or `<span class="twdb-*">` (plus `<img … class/src/width/height/alt/loading/decoding>`), and
  BOTH probed bodies — the plain item 25 and item 728, picked off the user's own DB as a record
  whose `raw_json` carries `flavor` (`"Teaches you how to cook Westfall Stew."` shows in its
  `twdb-quote`, 1281 b of html) — answer **ZERO** `&entity;` matches. The copy hotlinks pixels
  from `render.worldofwarcraft.com` AND `thewowdb.com/assets`: which is exactly why the card
  renders the derived `tooltip_text` twin (docs/DATABASE.md, ADR 0012) and prints raw markup
  as escaped text, never as markup.
* `html` (1404 b for id 25) is upstream's OWN card markup — `<div class="twdb-card twdb-k-item">`,
  its own `<img class="twdb-icon" src="https://render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg">`,
  its own wording (`Item Level 2`, `Sell Price: 7c`). The tool stores it as a value (`tooltip_html`)
  and the card prints it as **escaped text**: pasting an upstream body into the page would let
  upstream write the card.
* An id the door has not answers **404 with upstream's own JSON body**, not an HTML page:

  ```json
  {"error":"not_found","message":"No item 999999999999 in forever.","status":404}
  ```

  measured 79 b, `cache-control: public, max-age=300`, no ETag offered, `x-ratelimit-remaining: 119`.
  The body is upstream's own words, not ours; the tooltip stage reads the STATUS and gives that
  record its own checked third state (`tooltip_missing_at`), exactly as the detail door's 404 did
  in v0.5.3 — one request per record per crawl, and the crawl keeps walking.
* The door revalidates, so the stage stores its own `tooltip_etag`: measured `ETag` +
  `max-age=300`, a targeted re-ask with `If-None-Match` costs a `304` and writes nothing — the
  same reuse discipline the detail door already promises. The fixture answers the door the same
  way, with its own records and a small stand-in `html` in upstream's class names (we never copy
  upstream's prose), and answers `404` with the JSON shape above for ids it has no tooltip for.
* `fields` (measured 26 keys: `armor, binding, build, classes, effects, faction_prices, flavor,
  icon_url, id, item_level, item_set, item_subtype, item_type, kind, name, price, quality,
  required_level, slot, socket_bonus, sockets, stats, unique, vendor_sell, version, version_label,
  weapon`) mirrors the detail record's own column set. The copy stage (v0.8.0) read ONLY `html`;
  FROM v0.9.6 (ADR 0017) the tool types the object it NEEDS — `Tooltip.fields` is the mirror
  `TooltipFields` with ONLY `price` typed, and serde still ignores every key with no reader, so
  "a key upstream adds cannot break a build" keeps its exact teeth. Nothing here claims a second
  source of truth for a column the detail row already owns.
* `fields.price` — measured LIVE 2026-11-21 (the door the REFRESH stage now re-asks, v0.9.6): the
  block carries `available, market, min, realms, region, scope, updated_at`. Id 25 asked TWICE in
  one day answered `{available 5→7, market 1200, min 149, realms 3, region "us", scope "WoW
  Forever Normal Alliance", updated_at 16:25:03Z → 22:25:10Z}` — the SAME door, its OWN words
  moving: this door answers NOW-ness, and upstream's OWN developers page says tooltips cache ~5
  min "because they carry a live Auction House price". The block has a THIRD state: id 200
  answered `fields.price` as literal **NULL** while its own copy still says "Lowest buyouts" —
  the door looked, the AH offers NOTHING right now. Measured apart, not asserted: ids 202/237/763
  answer `min 5000/3500/4425` while their STORED detail copies carry NO price at all, so the live
  min (`price_min_now`, ADR 0016's home) moves apart from the aging `price_min` ON THE SAME
  record. The detail door carries the same block at ITS own cache rhythm (measured `detail/237`
  @ `22:25:08`) — the refresh stage rides the TOOLTIP door, the one the crawl already owns.
* Cost: **≈ one request per record whose detail answered** (≈ 19 258 tooltip requests ≈ 2.7 h
  at the measured 2/s), staged, resumable, cancellable like the detail and icon stages. The
  pending rule requires a detail stamp, because asking this door for an id upstream lacks costs
  a request AND would stamp a state about a fact upstream never stated.

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
  requests ≈ 2.7 h, which is why it is staged, resumable, cancellable and opt-in. From v0.6.0
  the icon pass is the same order (≈ one request per answered record), and from v0.8.0 so is the
  tooltip-copy pass (≈ 19 258 requests ≈ 2.7 h) — a full "Pull all info" is therefore four legs,
  ≈ 58 k requests, and every one of them resumes where `sync_state` says it stopped.

## Terms (we comply)

* Free for any project, including commercial.
* **A visible link to TheWoWDB (or the record's `links.html`) wherever the data is shown**
  → rendered in the UI footer and in every item detail card.
* Do not resell the raw data as a dataset. We only ship per-item caches inside the app.
* Breaking API changes ship under `/api/v2/` — we pin `/api/v1/` explicitly.
