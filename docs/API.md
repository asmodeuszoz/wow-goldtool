# TheWoWDB API — notes we rely on

Source of truth: <https://thewowdb.com/developers/> (checked **2026-11-19** and again **2026-11-21** — exactly FOUR doors both times: list, record, search, tooltip; and re-checked **2026-11-22** when the recipe doors landed — still FOUR, and the five probed professions paths answer `404`, see "What the API does NOT answer" below). No API key.

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

## What the API does NOT answer (measured **2026-11-22**, the same day the recipe doors landed)

The user asked for "every job with every recipe … icons, descriptions, items needed" and
asked whether an API gives this for WoW Classic "forever". Measured, the answer is SPLIT, and
the split is recorded here because it shapes [ADR 0021](./DECISIONS/0021-the-recipe-doors.md):

* **There is no professions door.** `/api/v1/forever/professions`, `/profession`, `/jobs`,
  `/skills`, `/tradeskills` answer **`404`** today (five paths probed; the developers page
  still lists exactly FOUR doors — list, record, search, tooltip — and names no fifth).
* **Recipe CARDS exist upstream but carry no job, no skill, no materials.** The mirror's own
  copy has **716** rows with `item_type "Recipe"`, **699** of 716 tooltip copies say
  "Teaches you how to…" (flavor prose), and **0** copies anywhere contain "Requires Materials".
  The `?q=Recipe` search door answers NAMES only (259 ids — it undercounts the 716 type-`Recipe`
  rows): the names-only trap again, and names alone cannot fill a tab.
* **Blizzard's own Game Data API answers `404` today.** `api.blizzard.com/data/wow/recipe/2424`
  with the exact namespace a 2025 bug report names (`static-11.1.5_60179-us`) answers `404`;
  the `11.2.0`-shaped guesses answer `404` too. Recorded as a dead end, not a door — inventing
  its shape would invent upstream, so the tool ships ZERO requests against it.

## The recipe set — the SECOND source (measured **2026-11-22**)

The only measured place that states, per recipe, the profession, the skill number and the
exact reagents is a plain-JSON dataset, not an API: [ReagentRoute](https://github.com/Nyte-fyre/ReagentRoute)
(`github.com/Nyte-fyre/ReagentRoute`, fetched at
`raw.githubusercontent.com/Nyte-fyre/ReagentRoute/main/data/<file>.json`), **MIT** (text measured
at the repo's `addon/LICENSE`).

```
GET https://raw.githubusercontent.com/Nyte-fyre/ReagentRoute/main/data/{alchemy,blacksmithing,cooking,enchanting,engineering,first_aid,leatherworking,tailoring}_recipes.json
```

* **Eight classic files** — the classic set has NO Jewelcrafting file (Jewelcrafting rides the
  set's `*_tbc_recipes.json` pair, the TBC-Anniversary era, a DIFFERENT era this DB does not
  keep). Sizes measured the same day, fetched whole: 241 981 · 638 864 · 82 407 · 348 098 ·
  296 531 · 26 372 · 518 352 · 690 879 B = **2 843 484 B**, one request each, `200` + a real
  ETag on every one (conditional GETs work, so the stage's ETags ride `meta`).
* **1 240 entries**, `recipe_count` stated per file matching the array every time
  (113 · 250 · 81 · 152 · 161 · 15 · 232 · 236). Top level
  `{profession, recipe_count, recipes[]}`, `profession` per file (measured words:
  `Alchemy · Blacksmithing · Cooking · Enchanting · Engineering · First Aid · Leatherworking ·
  Tailoring`), entries carrying **16 keys on ALL 1 240** (`spell_id · craft_spell_id ·
  spell_name · crafted_item_id · crafted_item_name · required_skill_value · skill_tier ·
  trivial_low · trivial_high · reagents[{item_id,item_name,count}] · learned_from ·
  acquisition · source · confidence · auto_granted · vendor_sell_price`) and `acquisition_note`
  on **745** of them. `source` values measured `cmangos-classic-db` / `cmangos-tbc-db+wago-tools-db2`.
* **The id spaces MEET the DB** (this is why the join works): the classic files speak classic
  ids (`spell_id 2339` "Minor Mana Potion" ↔ `crafted_item_id 2455`). Join measured the SAME
  day against the user's own file (19 258 rows): **1 105 distinct crafted ids**, **1 095 have an
  item row**, **10 have none**; **133 entries carry NO crafted id at all** — all 133 are
  `Enchanting` (the craft puts an ENCHANT on a slot, not an item in a bag); **416 distinct
  reagent ids**, **415 have an item row**, **1 has none**, and **392** of the joined reagents
  already answer a `price_min`. The stage wrote **1 240 rows** for those 8 files end to end
  against the real host (rc 0, 8 requests) — the tool's own first real-set witness.
* **`learned_from` speaks the SCROLL, and it states it BY VALUE** (measured 2026-11-22 on the
  user's OWN fetched 1 240 rows, read-only, ZERO requests — the day v0.12.0 gave the column its
  FIRST reader, [ADR 0025](./DECISIONS/0025-the-recipe-card-window.md)). The object's keys are
  exactly THREE: `item_id · item_name · type` — no more, no fewer. The `type` census over all
  1 240 entries sums to the whole set, so NO entry leaves learning unstated in this shape:
  **804** state `schematic_item` and name a scroll id + the set's OWN name for it, **413** state
  the `trainer` word ALONE (no `item_id` key at all), **23** the `automatic` word ALONE. Of the
  **804** stated ids **747** have a visible item row in this DB and **477** of those carry the
  scroll's OWN tooltip copy — so a scroll cell exists for 804 entries, shows the scroll's own
  in-game words for 477 of them, and the other 57 stated ids answer "the DB never saw this id
  alive" beside the set's name. Rejected the SAME pass as links, with their measures: `spell_id`
  used as an item id (651 lands, only **20** of them `Recipe`-typed rows) and the name link
  `"Recipe: " + crafted_item_name` (reaches **123** of 1 240, and of those 123 ZERO land on the
  entry's own `crafted_item_id` — all 123 attach elsewhere; the DB has 716 `Recipe`-typed rows,
  ALL 716 with a copy, but only **122** of them are named `Recipe:*`). The set's own words stay
  the set's: the DB's OWN names differ (the detail stage overwrites, deprecated prefixes), which
  is exactly why a name join writes lies. Upstream's `vendor_sell_price` / `confidence` /
  `auto_granted` keys are still NOT read by this tool (serde ignores them; `vendor_sell_price` is
  the SET's claim about an id whose OWN `vendor_sell` column already answers — ADR 0016's grounds
  that only the API's OWN copy may fill a column, kept): recorded here so the census stays visible
  where upstream facts live, and the `type` key's spelling stays in `docs/DATABASE.md`, where the
  read lives (its key is a reserved word, so the door reads it by KEY, never by field name).
* **The set's `*_forever_recipes.json` files are NOT this set**: they carry
  `game_version "forever-beta"` in a DIFFERENT id space (`spell_id 1283400`,
  `crafted_item_id 279961`) that does not join these rows. Rejected, recorded.
* **The other candidate carrier was rejected on measured grounds:** Napalmsteak's
  WoW-Classic-Item-Caches answers `404` at its `LICENSE` path (a carrier with no license is
  not a carrier), and its `Forever Cache.json` keys are `sourceKeys/sourceLabels/sourceTypes`
  — ACQUISITION words, not reagents.
* **Attribution stays REQUIRED** (MIT): the citation rides the UI footer, the Recipes tab's
  own note and `docs/API.md` here — the same discipline TheWoWDB's terms already force.
* **The dataset is never vendored into the repo** (2.6 MB of somebody else's set would be a
  second, drifting copy the day the source moves): the stage FETCHES it, `meta` carries the
  file ETags, and the counts above are what the docs record — CI asserts only what it RUNS
  (ADR 0011's rule, kept in ADR 0021).

## Terms (we comply)

* Free for any project, including commercial.
* **A visible link to TheWoWDB (or the record's `links.html`) wherever the data is shown**
  → rendered in the UI footer and in every item detail card.
* Do not resell the raw data as a dataset. We only ship per-item caches inside the app.
* Breaking API changes ship under `/api/v2/` — we pin `/api/v1/` explicitly.
