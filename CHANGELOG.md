# Changelog

All notable changes to **WoW Goldtool** are documented here, following
[SemVer](https://semver.org/) and [Conventional Commits](https://www.conventionalcommits.org/).
`Added` / `Changed` / `Fixed` / `Removed` follow the [Keep a Changelog](https://keepachangelog.com/)
sections. The full narrative of *why* each change was made lives in
[docs/DEVLOG.md](./docs/DEVLOG.md); each entry below points at the matching log entry and git tag.

## v0.9.5 — the price homes: FOUR columns own FOUR questions, carved now, filled later · 2026-11-21 · tag `v0.9.5`

Added

- **Three reserved columns** (ADR 0016): `vendor_buy` (what a vendor BUYS — only when the
  item can be bought at all), `price_min_now` (the CURRENT auction-house min buyout) and
  `price_trend_normal` (the historic "normal" the live trend answers) join the schema beside
  the `vendor_sell` / `price_min` / `price_market` homes that already exist. **Schema 5 → 6**,
  additive ALTERs, and `check_schema` now chains EVERY stale shape (v1..v5) up to 6 — the
  staircase v0.9.0's comment promised since its own hop.
- **The reservation moves ZERO bytes**: no producer, no reader, no door, no UI change this
  release — measured with BOTH binaries (the v0.9.4 tree at `4875222`, built in a scratch
  worktree, and this one) over two copies of ONE fixture DB: 12 of 13 doors BYTE-IDENTICAL
  at the same byte counts, and `status` identical once its own `db.path` echo is pinned
  (the paths cannot match: the OLD binary refuses the file the NEW one migrated).
- **The migration is real and idempotent**: the user's own file went 5 → 6 the moment a
  door opened it — **19 258 rows / 4 231 vendor_sell / 4 285 history samples intact**, trio
  NULL on all of them, and a SECOND open re-migrated NOTHING (second `status`
  byte-identical).

Honest about what did NOT move

- **NULL in the trio means "NO PRODUCER HAS EVER WRITTEN IT"** — deliberately NOT yet the
  "fetched / fetched-and-not-there" split the price columns carry; the third state arrives
  WITH the first producer, never before it (ADR 0011's rule: no plan for a door that does
  not exist). HOW the values get fetched is later work — the user said so, and no upstream
  door has EVER answered these words, so the column NAMES are ours and docs/API.md gains
  nothing (never invent upstream shapes).
- `price_min` (the stored copy's cheapest AH buy, an aging payload's number) and `price_min_now`
  (what the AH says NOW) are TWO columns on purpose: one column answering two questions over
  time is two truths in one word.
- **money (ADR 0015) waits**: `money_of` takes any `Option<i64>` copper, so the words will
  reach these columns the day producers write them — with ZERO measured rows there is ZERO
  money to answer now, and money of NULL was rejected by ADR 0004's rule again.
- **CI gains no step** (31 stays 31 + one plant line, verified by step-line count after the
  edit): the smoke file is now BORN at 6, and the ALTER chain does NOT run in CI — CI has
  no schema-5 file — so the migration gets measured on scratch copies in the post-tag audit
  (ADR 0011's honest split).
- **the stale direction keeps its honest refusal**: the pinned v0.8.1 binary answers exit
  4 — `database on disk uses schema 6, this build wants 4` — over a migrated file, untouched.

Refs: ADR 0016, ADR 0015's money formatter (it will reach these columns later), ADR 0007's
sampling rule, ADR 0011's CI rule; schema is `6` — the FIRST hop since v0.9.0's own.

## v0.9.4 — prices read like the GAME writes them (`2g 50s`), copper stays the number · 2026-11-21 · tag `v0.9.4`

Added

- **Money words beside every copper number** (ADR 0015): one formatter in `src/db.rs`
  (`money_text`, `money_of`, `money_of_avg`) writes the game's OWN wording — 100 copper is 1
  silver, 100 silver is 1 gold, and the unit is proven from the **DB's own tooltip copies**,
  because a copied tooltip already states its sell price in money words (`money_text(vendor_sell)`
  equals the row's own copy for **4 230 of 4 231** rows). A zero denomination is DROPPED exactly
  as the copies write it (`25000` ↔ `2g 50s`, `20000` ↔ `2g`, `20042` ↔ `2g 42c`), and the average
  **floors** to whole copper (`AVG(price_min) 54 391.705896` → `5g 43s 91c`).
- **The money siblings ride the answers, not the DB**: `Hit` grew `price_min_money`/`price_market_money`,
  `Coverage` grew `min/max/avg_price_money`, `QualityStat` and `Extreme` grew their mean/row money,
  `PriceSample` grew the sample pair (filled by `history_of` from the copper the read carries), and
  the card answers `price_min_money`/`price_market_money`/`vendor_sell_money` as top-level sibling
  fragments beside `item` — never inside it, the same ground `icon_b64` has: `ItemDetail` and `Price`
  mirror the wire (docs/API.md), and money is OURS, upstream never answered that word. Money reads the
  MERGED detail (ADR 0010's merge-last rule), so a money word cannot disagree with the number beside it.
- **The UI shows money in the cell and the exact copper in the hover**: one `money_cell` renders
  the words Rust wrote, so the webview does no copper maths and cannot drift. Money reaches the hits
  table, both price lists, the card's price rows and its trend bars' hovers, the status grid's price
  range and average, the by-quality bar labels and the stats note's spread; `ui/index.html` names money
  in the three price theads and in the notes a card shows before its first click, and `ui/app.css` gives
  `.moneycell` `tabular-nums` so a money column's digits align.

Honest about what did NOT change

- **the DB grew nothing**: money is derived at answer time, so there is no `*_money` column, no
  schema hop (`schema_version` stays `5` on the mirror's and the user's DB after a replay that ran
  every money door), nothing to migrate and **no refresh door** — `refresh-twins` over the mirror still
  answers 4 copies / 0 rewritten / 4 matching / ZERO requests. The sorts and the scripts keep reading
  INTEGER copper (docs/DATABASE.md's numeric contract survives unchanged), and `SQL_COVERAGE`/`SQL_QUALITY`/
  `SQL_CHEAP`/`SQL_RICH`/`SQL_PRICE_SAMPLE`/`SQL_HISTORY_OF`/`HIT_COLUMNS` are untouched, so the read slots
  stay `0..14`.
- **`price_available` and `price_realms` never become money**: they are auction COUNTS (measured, row 25
  answers `available 12` for 12 offers of 15 000 copper; `realms` spans `1..12`), and money riding a count
  would be a lie — the card's `available` row stays a plain count.
- **an unpriced row keeps answering `null` beside `null`** and the UI cell still says `no price`, never
  `0c` (ADR 0004's rule against made-up zeros): measured, 0 rows in the user's DB sit at copper 0 in any
  of the four copper columns, so `0c` is a RULE and not a measurement.
- **money adds no CI step and no request**: the smoke block stays 31 `cargo run --` steps + one plant line,
  every step keeps its own exit code, and the fixture's GET tallies cannot move (ADR 0011's rule again).
- **money needs no flag**, unlike ADR 0014's icons: money costs about 56 bytes per priced row on the mirror
  and the longest money string measured in the user's whole DB is 13 chars (`1334g 99s 49c` for `13 349 949`).

Refs: ADR 0015, ADR 0010's merge-last rule, ADR 0004's exit-code contract, ADR 0011's CI rule,
ADR 0014's sibling-fragment precedent; schema stays `5` — money writes nothing.

## v0.9.3 — the lists draw the DB's own icon: `--icons` on the list doors · 2026-11-21 · tag `v0.9.3`

Added

- **`--icons`** (ADR 0014), a no-value flag on the two list doors: `search TEXT … --icons`
  and `stats --icons` put each row's icon next to it, from the **DB's own bytes** — the
  doors read `icon_blob`/`icon_at`/`icon_missing_at` and call the ONE `b64_text` encoder,
  which moved into `src/db.rs` beside the bytes it encodes so the card and the lists
  cannot encode the same copy two ways. **ZERO requests leave either door**, so ADR 0004's
  network door (`3`) is unreachable here: these doors answer `0`, `2` (usage) and `4`
  (missing DB) only, and the mirror replay's GET tally not moving is the proof.
- **The icon trio on the row shapes**: `Hit` and `Extreme` grew `icon_b64` / `icon_at` /
  `icon_missing_at`. The keys are always present; the base64 payload rides only when asked
  (measured: about 3.1 K chars per icon — avg 2 327 bytes per copy — so an unbounded hit
  list would be huge and `--limit` bounds it). Rows are COLLECTED before the per-row
  icon reads run, so a list can never straddle a write of the table it came from.
- **The UI draws an icon column** on the search hits and on both price lists
  (cheapest/priciest), asks both with `--icons` at its own bounds (40 hits, 6 extremes),
  and shows a placeholder glyph (`◇`) where a row has no bytes — with the WHICH state of
  the row's own stamps in the hover (`icon_missing_at` set → checked as gone upstream;
  neither stamp → never fetched). ADR 0008's rule again: upstream's `icon_url` rides the
  row shape but is never an image src.

Honest about what did NOT need to change

- **the item card needed no code**: it has rendered an actual `<img>` (a data URI, never a
  hotlink) since v0.6.0. The lists were the gap, the card was not;
- **the watch list gained no panel**: `ui/index.html` has no watch table (measured — the only
  watch surface is the `Watch these hits` button and the sync scope), so the watched rows
  gain icons where they appear, in the hits table, and no panel was invented;
- **the live-search table keeps no icon column**: that door reads and writes NO DB (ADR 0009),
  so upstream's names have no local copy to show.

Grounded by measurement, on the user's own DB

- 4 030 rows hold icon bytes, 1 was checked as gone upstream, 15 228 were never asked; all
  4 602 detailed rows carry an `icon_url` but only 4 030 carry bytes — so hotlinking the
  address would show an image the DB never claims;
- the encoder did not change across the move: `item 25 --cached` answered `icon_b64` of 2 596
  chars before and after, same sha256 prefix, and the emitted base64 decodes back to the DB's
  own 1 945 bytes.

Refs: ADR 0014, ADR 0008's icon stage, ADR 0004's exit-code contract, ADR 0011's CI rule;
schema stays `5` — the doors write nothing.

## v0.9.2 — old copies get their twin re-derived, with ZERO requests · 2026-11-21 · tag `v0.9.2`

Added

- **`refresh-twins`** (ADR 0013): one ask re-derives every twin in the DB from the copy it
  already holds. The copy is the source of truth, so **ZERO requests leave the door** —
  ADR 0004's network door (`3`) is unreachable here and the doors are `0` and `4` only.
  It writes only rows whose twin differs or is missing, rides tombstoned and manual rows
  (ADR 0012's rule is that the twin matches what the DB SAYS), collects every row before
  writing any (a sqlite row walk over the table it rewrites must not interleave), and is
  idempotent: a second press answers `0 rewritten` and still exits `0`.
- **The UI button `Reword the copies`** calls the SAME verb through the same one-command
  path (ADR 0005), and prints the door's own numbers.
- **CI smoke grew the door** (24 → 29 cargo steps + one plant line): first press answers
  0 rewritten / 4 unchanged, the v0.9.0 glued shape is planted with python3's own sqlite3
  (the same interpreter the fixture server needs — no new dependency), the phrase search
  answers `0`, refresh answers `1 rewritten`, the same search answers `1`, a third press
  answers `0`.

Honest size of the gain, measured on the user's 4 602 copies

- readable card row on every copy, and `twin == derivation(copy)` for the whole DB —
  ADR 0012's gap and v0.9.1's residue both close here, without a re-fetch;
- the description door is a substring door, so only phrases spanning an ELEMENT boundary
  gain findability: `damage speed` +358 rows, `hand one` +90, `sword 1` +14, `1 main` +3;
  `main hand` stays 75 before AND after and `sell price: 7` stays 291.

Changed

- `docs/DATABASE.md`'s honest-gap paragraph now names the door that closes it; the version
  banner moves 0.9.0 → 0.9.2 (the v0.9.1 patch chain left it put). The schema does NOT
  move: the refresh writes columns schema `5` already has.

Refs: ADR 0013, ADR 0012, ADR 0010's merge rule, ADR 0004's exit-code contract,
[docs/DEVLOG.md](./docs/DEVLOG.md), tag `v0.9.2`.

## v0.9.1 — the scope door answers and the twin keeps its spaces: v0.9.0's measured defects fixed forward · 2026-11-21 · tag `v0.9.1`

Fixed

- **`search TEXT --where …` hung instead of answering** (found by measurement, not by theory):
  both post-tag replays of v0.9.0 froze **2 h 01 m in state R** at the same door, and a scratch
  `timeout 5` repro answers **rc 124** — the `--where` block's bare `continue` on VALID values
  skipped the parser's shared value-flag advance (`index = index + 2`). It now falls through
  like every other value flag (`--sort`/`--quality`/…); measured after: the `description`/`all`/
  `name` doors answer rc **0** in well under a second and a bogus scope still answers the usage
  door **2** with the same message.
- **The twin glued words at element boundaries** (`Worn ShortswordItem Level 2Main HandOne-Handed
  Sword1 - 3 DamageSpeed 1.9…` across the user file's first 4 602 copies): live tooltip html is
  MINIFIED — adjacent spans, no whitespace between them — and the v0.9.0 derivation DROPPED spans.
  The fixture could not show this: its stand-in card carries spaces AT the tag boundaries, so
  fixture-shaped data cannot disprove claims about the live shape (only a live pass can). The rule
  now: every `<…>` span leaves ONE space, then collapse whitespace runs, trim.
- Honest residue, stated not swept: the 4 602 glued twins already on the user's file stay in their
  v0.9.0 shape until a copy's CONTENT changes (the pending rule never re-asks stamped copies, and
  a `304` writes nothing per ADR 0007) — a refresh door is a named plan for a later milestone, not
  invented code.

Changed

- ADR 0012's derivation bullet and audit row carry the correction (with the hung-replay and
  cancelled-run facts); `docs/DATABASE.md` and `docs/DEVELOPMENT.md` state the corrected span →
  space rule; the DEVLOG gained the v0.9.0 post-tag verification section and this entry.

Refs: ADR 0012, ADR 0004's exit-code contract, [docs/DEVLOG.md](./docs/DEVLOG.md), tag `v0.9.1`.

## v0.9.0 — the copies become READABLE and FINDABLE: every copied tooltip gets a readable twin · 2026-11-21 · tag `v0.9.0`

Added

- **The readable twin** (ADR 0012): schema `5` — `tooltip_text` / `tooltip_lc` derived from the
  copy **in the SAME write** on EVERY write path: the tooltip stage derives from the MERGED copy
  (ADR 0010's overrides merge LAST, so the twin always matches what the DB SAYS), `set-item ID
  tooltip_html=…` derives beside the stated value, `add-item` births it with the stated fields —
  the `name` → `name_lc` pair shape the name columns have ridden since v0.1.x. The derivation is
  exactly the measured three moves (drop `<…>` spans, collapse whitespace runs, trim): TWO live
  tooltip bodies read whole — the plain id 25 (1404 b of `html`) and id 728, picked off the USER's
  own DB because its `raw_json` carries flavor (`"Teaches you how to cook Westfall Stew."` shows in
  its `twdb-quote`) — hold div/span/img tags with class/src/width/height/alt/loading/decoding
  attributes and **ZERO entities**, so NO decoder is invented (an entity would show verbatim; the
  words still search). Why it had to exist: the copy's `<img>`s hotlink pixels from
  `render.worldofwarcraft.com` and `thewowdb.com/assets` — pasting the copy into a card would
  break the icon stage's rule (nothing hotlinks, the DB gets the bytes) — and grepping markup for
  words finds tags, not meaning.
- **The local search door grew a scope**: `search TEXT --where name|description|all` (the LOCAL
  door — upstream's OWN was measured NAMES-only at v0.7.0, docs/API.md): default `name` keeps
  every existing search exactly what it was, `description` reads `tooltip_lc` behind `IS NOT NULL`
  (never-fetched and checked-gone records skip honestly), `all` is name OR description; a bogus
  scope answers the usage door (`2`); tombstoned rows stay invisible in EVERY scope
  (`visible_clause` stays first); the door stays DB-only — ZERO requests.
- **The UI grew both sides**: the search panel's where select (names only | descriptions |
  anywhere — ONE argv pair, the same door the CLI answers) and the card's `description (readable
  copy)` row that RENDERS the twin as words, while the raw upstream markup row stays printed-as-
  TEXT (v0.8.0's escape rule, unchanged: an upstream body cannot write our card).
- **CI's smoke block grew 21 → 24 steps**: the `add-item` step now ALSO states a description (its
  twin is born beside it), and the block ends with `search sword --where description`,
  `search sword --where all` and the manual row's DB-only card (`item 900000000 --cached` —
  `MANUAL_ID_BASE` on a fresh DB). The fixture grounds the counts with its OWN code: its detail
  server assigns `body["name"] = "Fixture Detail <id>"` to every detailed row, so the LIST door's
  row named "Worn Shortsword" gets its name OVERWRITTEN and NO final name carries "sword" while
  the copied tooltip of 25 does — both search steps answer 2 (row 25 + the manual mug), a name
  scope would answer 0: findable exactly where names cannot.
- **`check_schema` now chains EVERY stale shape up to 5** (v1→v2→v3→v4→v5, all additive): the
  user's own 19 258-row catalogue rides the chain the moment a 0.9.x build opens it.

Changed

- `SQL_SET_TOOLTIP` grew the two derived columns (value count `4 → 6`, `bind_sql`'s marker count
  moves with it); the card now carries BOTH tooltip shapes (`tooltip_html` printed as text,
  `tooltip_text` rendered); `docs/DATABASE.md` grew the readable-twin section with the honest
  gap — a row migrated from schema 4 that ALREADY had a copy has no twin until its next tooltip
  write (SQLite's `ALTER` cannot run Rust's reader; duplicating it across languages is exactly
  the drift that file exists to forbid), and `docs/API.md` records the tooltip copy's measured
  shape (ZERO entities in two live bodies, the two hotlink hosts) as ADR 0012's ground.

Added

- **The repo's own icon assets**: `icons/icon.png` (512 px) and `icons/icon.ico` (seven
  entries, 16–256 px) — our own art, drawn by hand in PIL (a dark navy card, a gold coin, an
  "W" made of four strokes, never a font), verified real by `file` (`PNG image data 512 x 512`,
  `MS Windows icon resource - 7 icons`). They are product work, not a CI side effect:
  `tauri.conf.json`'s `bundle.icon` ALREADY named `icons/icon.png` for a file that never
  existed, and the windows-2025 build answered `` `...icons/icon.ico` not found; required for
  generating a Windows Resource file during tauri-build`` (run #3, measured).

Changed

- **The workflow is rewritten at the measured truths** ([ADR 0011](./DECISIONS/0011-ci-asserts-only-what-is-true.md),
  commits `c29e45c` → `5c888b9` → `f29a2b2` → `59b8379`): every `uses:` names a resolvable
  action (`actions/checkout@v7` — the step the old file lacked — and `dtolnay/rust-toolchain@stable`),
  the Linux dev libraries ride the sonames our OWN binary answers (`ldd target/debug/wow-goldtool` →
  `libwebkit2gtk-4.1.so.0 libgtk-3.so.0 libgobject-2.0.so.0 libsoup-3.0.so.0 libpango-1.0.so.0`,
  i.e. apt `libgtk-3-dev libwebkit2gtk-4.1-dev`, NOT the template's gtk-2 + webkit-4.0 pair),
  `cargo clippy --all-targets` stays INFORMATIONAL (measured: rc 0, 0 errors, 600 warnings —
  this build's four dead-code warnings ARE the plan `docs/DEVELOPMENT.md` names, so `-D warnings`
  would lie about it) and there is deliberately **no `cargo fmt --check` gate** (measured: it
  answers a diff on our hand-written style; a rustfmt-clean tree would be its own milestone).
  The workflow's header comment states both limits, so the file documents itself.
- **The scratch DB rides a real path and the macOS row rides the real arch** — the second
  measurement round (run #3) caught three more lies: `${{ runner.tmpDir }}` is NOT a context
  property, it evaluated to the EMPTY string so the smoke sequence died at its FIRST step with
  `cannot open /fixture.sqlite` → exit 4 (the local mirror could not see this because it hardcodes
  its own DB path — only a runner evaluates the env map); and macos-latest IS arm64 — the same
  run's toolchain step answers `host: aarch64-apple-darwin` — so `x86_64-apple-darwin` std was
  never installed and the build died with `can't find crate for std` (exit 101). Fix:
  `WGT_DB: fixture.sqlite` (relative, the checkout root, `*.sqlite` gitignored) and the matrix
  row `aarch64-apple-darwin`. Three promises still ride — linux, windows, macos.

Removed

- The three action names that never resolved (`actions/rust-action`,
  `ubuntu-setup-action/install-ubuntu-system-dependencies`,
  `windows-setup-action/install-windows-system-dependencies` — `gh api orgs/…` answers HTTP 404
  for the two orgs), and the two gate claims that were never true: a `cargo fmt` step (rewrites,
  never gates) and a `cargo lint` step (`error: no such command: lint` in cargo 1.94.0).

Nothing about the RELEASE moved: v0.8.1 is the discipline of the repository's own
infrastructure (ADR 0011), no `src/` line changed, the 21-step smoke block survives
byte-identical, and `cargo build` answers rc 0 with exactly the four documented warnings.
The banner rides 0.8.1 (`chore` `fe691c9`), so the USER_AGENT line built from
`CARGO_PKG_VERSION` now says `wow-goldtool 0.8.1` on every upstream ask.
**GitHub run #4 (`59b8379`): 5/5 jobs green in 9 min 13 s** — quality 2 min 04 s ·
build (macos) 2 min 27 s · test 4 min 19 s · build (linux) 5 min 46 s · build (windows) 9 min 12 s.
The whole story — four runs, every error quoted — is in [docs/DEVLOG.md](./docs/DEVLOG.md).

## v0.8.0 — every forever item's description copy lands, and the DB takes edits by hand · 2026-11-20 · tag `v0.8.0`

Added

- The **description copy as a FOURTH sync stage** ([ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md)):
  `sync --stage tooltip [--budget N] [--ids A,B] [--watch]` reads `/api/v1/tooltip/forever/item/<id>`
  and stores upstream's own `html` in `tooltip_html` + `tooltip_at` + `tooltip_etag`. It exists at all
  because the `forever/item` body carries NO description of any kind (measured top-level keys read
  back off the user's own `raw_json`: `attribution, icon_url, id, item_level, item_subtype, item_type,
  links, name, price, quality, required_level, unique, vendor_sell, weapon` — no prose field, no
  `html`, no `fields`). The door has the same three states as every other door — copied, checked-gone
  (`tooltip_missing_at`, from the measured `404` whose body is upstream's own JSON
  `{"error":"not_found","message":"No item 999999999999 in forever.","status":404}`), never looked at
  — revalidates with its own ETag (measured `W/"2b111cbcd3bbb4d7078f6d05"`, `cache-control public,
  max-age=300`), honours `Retry-After` and retries a `429` IN PLACE, so a rate limit never aborts a
  stage (ADR 0002/ADR 0004's contract). Cost ≈ 19 258 requests ≈ 2.7 h, staged/resumable/cancellable
  like the other three. ADR 0007's price sample stays on the DETAIL write only: a tooltip re-ask is
  not a second observation of the auction house in this build.
- The **manual layer**, one rule: overrides merge LAST. `manual_overrides` is one JSON ARRAY per row
  of `{field, value, at}` entries, applied AFTER the payload binds on every write path (`upsert_summary`
  from the list stage, `upsert_detail`, `write_tooltip`) AND on both read paths (`cached_card`, `cmd_item`)
  — so a field the user stated survives the next crawl while the columns the crawl owns keep refreshing,
  and a card can never show a value the DB does not have. Four CLI verbs: `set-item ID FIELD=VALUE`
  (exactly one whitelist field of one row, stamps `edited_at`, and rides the derived search columns
  `name_lc`/`quality_rank` in the SAME write so a rename is findable on the same breath), `add-item FIELD=VALUE […]`
  (the DB picks the id in the reserved block at `MANUAL_ID_BASE = 900000000` — measured max upstream
  id on the user's own file: `288 340` — stamps `manual_at`, and the row wears its fields as overrides
  FROM BIRTH), `delete-item ID` = the tombstone `deleted_at`, `undelete-item ID` clears it. Whitelist:
  `name, quality, item_level, required_level, item_type, item_subtype, flavor, vendor_sell, tooltip_html`
  — the auction columns stay out on purpose (an override "fixing" a live price is a permanent lie for a
  gold tool), so do `id`, the icon columns and the JSON-shaped ones.
- A UI write panel — **Say it / Make one / Hide it / Bring it back** — whose buttons send exactly those
  four verbs, and a card that prints the tooltip copy as **escaped TEXT** (it is upstream's own markup:
  rendering it would let upstream write the card), marks each field the user stated from the card's own
  override array rather than from what the panel typed, and refuses a tombstoned row.
- The status grid tells both new door families (`descriptions copied`, `checked: no description
  upstream`, `fields the user said`, `items the user made`) and "Pull all info" chains the fourth leg.
- CI grew the smoke sequence to **twenty-one steps**: the fourth stage budgeted (budget 8) and drained,
  then `set-item 25 vendor_sell=606`, `add-item "name=CI Test Mug" quality=Rare`, `delete-item 60`
  and `undelete-item 60` on the same DB.
- The fixture grew the tooltip door GROUNDED BY THE PROBE before those steps landed (unlike v0.7.0,
  where the CI steps landed before the fixture): its own `html` in upstream's class names with its
  OWN words (the probe measured upstream's 1404 b; the fixture never copies upstream's prose), its own
  ETag revalidation, and upstream's own JSON `404` body for ids it has no tooltip for — so one ANSWERED
  record (`12809`) has a real twin for the tooltip stage's third state. Its `429` gate names the tooltip
  stage, since that stage retries in place.
- `stats`/`status` coverage grew five counts — `tooltips`, `tooltip_missing`, `edited`, `manual`,
  `deleted` — and the price columns moved from read slots 7-9 to 12-14; the expected-count guard in
  the audits is what catches that drift, and the CLI, the UI and the audits read ONE answer.
- A row the user **MADE** answers its card from the DB's OWN columns (`SQL_COLUMNS_CARD`, 14 columns)
  on BOTH card doors, and the live door short-circuits BEFORE building a client — ZERO requests, because
  upstream cannot know the id. Fields only an upstream record can state answer `null` rather than being
  invented (the wire's labels, its `links` and `attribution`, the auction and weapon columns, the
  JSON-shaped ones).

Changed

- Schema `3 → 4`: four tooltip columns and four manual columns, all additive; `SQL_MIGRATE_TO_V4` is
  eight `ALTER`s and the chain now runs v1→v2→v3→v4, so the user's own 19 258-row catalogue comes up
  in one call. A file NEWER than this build is still refused, and that consequence was measured: a
  pinned v0.7.0 binary refuses the user's migrated file with `database on disk uses schema 4, this
  build wants 3` — which is why the real pull-all had to be re-pinned at a v0.8.0 snapshot.
- All three pending queues refuse a tombstone and a manual row (`SQL_PENDING`, `SQL_PENDING_ICON`,
  `SQL_PENDING_TOOLTIP` carry `deleted_at IS NULL` and `manual_at IS NULL`): a crawl never asks for a
  record upstream cannot know, and never re-stamps a row the user buried. An explicit `--ids` ask still
  reaches both — targeting is a human decision, a queue is not.
- `query.rs` grew `visible_clause()`, pushed FIRST in `where_parts`, so `search`, its `count_plan` and
  the card agree BY CONSTRUCTION that the user's rows are invisible (measured: a bare search's total
  went 6 → 5 across `delete-item 60`, and back to 6 across `undelete-item 60`).
- The version banner rides `0.8.0` (`Cargo.toml` plus the pinned `Cargo.lock`, the v0.7.0 precedent),
  and the usage line names all four stages and all four manual doors.
- The build's dead-code set grew by exactly ONE: rustc's "multiple fields are never read" on `Tooltip`
  — the measured key list rides the model so a key upstream adds cannot break a build, and the door
  reads only `html` (ADR 0009's rule: keep the measured mirror, read what the door needs). `catalogue`,
  `open_db` and `pending_plan`'s private `PendingSql` stay the documented set.

Fixed

- The fresh DB's `CREATE TABLE item` lost a comma INSIDE the Rust string — `"… tooltip_missing_at INTEGER"`
  glued onto `"icon_blob BLOB"` — so SQLite read `INTEGERicon_blob` as a type name: the `CREATE` **succeeded**
  and BOTH columns came out absent, so a fresh file had no icon bytes and no tooltip third state and every
  reader naming them would have died. The chain migration masked it (its `ALTER`s name the columns), the
  fresh write did not. Caught by reading `.schema item` back off a fresh DB. Same family as the earlier
  notes: a schema string that BUILDS is not a schema that WORKS.

## v0.7.0 — the live-search door: one human ask, one request, a look not a sync · 2026-11-20 · tag `v0.7.0`

Added

- CLI verb `live-search TEXT` ([ADR 0009](./DECISIONS/0009-live-search-door.md)): the WHOLE
  verb tail joined with spaces is the query (`live-search Worn Shortsword` asks one name),
  ONE request to the grouped `/search/` door, payload `{status, retry_after, remaining, answer}`
  with `answer` a serde pas-through of `SearchResponse` (`groups` stays a `Value`, upstream's
  `+` inside its own `more` URLs rides verbatim). Names-only means an all-empty answer is a
  success (exit `0`), and the door writes NOTHING: no row, no `meta` key, no sample, no mark.
- A live-search panel in the UI: one field, one `Ask once` button, one request per click, a
  `group · id · name` table with a `view upstream` link per hit (attribution reaches the
  ANSWER, not only the footer), and a note printing `remaining` (honestly `null` when the
  host sends no burst header), the names-only fact, and whether any group said `has_more`.
- The fixture grew the grouped `/search/` answer from the measured transcript (six groups each
  `{has_more, more, results}`, names-only substring matching, the `+` form inside its own
  `more` URLs), and its `429` gate now never names the `search` stage — the door answers a
  `429` with exit `3` (like the `item` door) rather than retrying in place, so a gated search
  stage would turn the committed CI step red.
- CI grew the smoke sequence to fifteen steps: three live-search asks — one word, the two-word
  ask riding as `%20` on the wire (the fixture's GET log is the witness), and the names-only
  trap whose all-empty answer still passes at exit `0`.

Changed

- The build's dead-code set shrank as promised: `search_url`, `SearchResponse` and
  `ApiResponse.remaining` — compiled-but-unused through v0.6.0 — got their consumer, and `cargo
  build` now warns exactly `catalogue`, `open_db` and `pending_plan`'s private `PendingSql`
  (plus the incremental-cache hardlink notice on this filesystem).
- The version banner rides `0.7.0` (`Cargo.toml` and the `User-Agent` line naming the source),
  and the usage line says both search doors now: `search TEXT […]` (local, filtered) beside
  `live-search TEXT` (upstream's own answer).
- Nothing schema-wise: schema stays `3`. The door touches no DB — audited with an impossible
  `--db` path answering exit `0` with the answer while its directory is never created.

## v0.6.0 — price history + local icons: the offline catalogue starts paying · 2026-11-20 · tag `v0.6.0`

Added

- A second table, `price_history`, and one price sample per **fresh** detail write
  ([ADR 0007](./DECISIONS/0007-price-sampling.md)): `upsert_detail` appends a row
  after a fresh (200) write — and only when the payload carried prices, so a record
  not on the AH never becomes a blank chart point — while a `304` revalidation, a
  list write, a check-mark or a re-insert write **nothing**. The ledger is what the
  card draws (one bar per stored sample, newest-N window) and what `stats`, `status`
  and the sync panel print, so the sampling rule is auditable, not decorative.
  SQLite's FK clause turned out not to be the guard: measured 2026-11-20, an `INSERT`
  referencing a missing `item` id succeeds, so the order inside `upsert_detail` is
  the guard and `seq` is auto-assigned so an append can never collide with an older
  observation.
- The icon stage `sync --stage icons` ([ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md)):
  every answered record's OWN icon file is fetched once and copied into the DB as a
  `BLOB` — three states like the detail record's: copied + `icon_at`, checked-as-gone
  + `icon_missing_at` (one `404` on the icon FILE of an existing record, never asked
  again), and waiting. The pending rule requires a NON-EMPTY `icon_url`: live, 4 of
  30 detailed records answer `"icon_url": ""` (Peace Tea, Craftsman's Writ, Scroll
  of Imbue Iceknife, Lodestone), and under a NULL-only rule those would keep pending
  forever. The item card shows the local icon as a `data:` URI — nothing is hotlinked,
  which is the offline point — and CI's fixture now answers `/icon/<name>.jpg` with
  REAL `image/jpeg` bytes (1945 b and 1936 b, fetched once from the grounded asset
  host and embedded so the suite stays offline) plus one record whose icon FILE is
  gone upstream, the third state the stage must check once.
- Schema 3: three `ALTER`s append the icon columns, a `CREATE` adds `price_history`
  and its index, and the chain migrates **both** stale shapes in place — a file
  saying `schema_version = 1` (the user's own 19 258-row catalogue) gets v0.5.3's
  `detail_missing_at` first, then the v0.6.0 batch; a v2 file gets just the batch.
  A file **newer** than this build is still refused.

Changed

- `SCHEMA_VERSION` `2 → 3`; both fresh-DB stamps write the build's own version, so
  a fresh v3 file never refuses itself.
- `sync_state` gained the third stage (`kind = "icons"`); `status` prints three
  crawl rows, the icon pair (`copied`, `checked`) and the ledger count, and the
  sync panel's grid + "Pull all info" follow the chain list → detail → icons.
- The item card renders `stats` as name/value pairs (the one previously-unshaped
  field the live API has ever filled — measured) and gives the never-measured
  fields (`sockets`, `socket_bonus`, `classes`, `effects`) an honest raw row only
  when they carry something; the icon row and the trend bars join.

Fixed

- The icon queue's empty-url exclusion above (a NULL-only rule would never drain
  on the measured empty answers); and `ORDER BY` stays plain `id` here, because
  unlike the detail queue every icon candidate already carries a detail stamp, so
  a re-ask-first term would be a no-op — the docs now say what the code does.

[ADR 0007](./DECISIONS/0007-price-sampling.md) · [ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md) · [docs/DEVLOG.md](./DEVLOG.md)

## v0.5.3 — patch: a record the upstream dropped is a state · 2026-11-20 · tag `v0.5.3`

Fixed

- A detail record the upstream no longer has (`404`) aborted the whole crawl
  (`item 41 answered 404` → exit `3` on the live catalogue). The list walk lists
  those ids — `OLD…`, `Deprecated …`, `… (TEST)` names sit in the user's 19 k rows
  — so the one-click "Pull all info" from v0.5.2 could never finish. `classify` now
  answers `HttpKind::Missing`, the detail crawl marks the id
  (`detail_missing_at`, guarded by a row-exists check so a mark cannot invent a row)
  and counts it in the report (`missing`), then keeps walking; the stage-2 upsert
  clears that stamp, so "gone" and "fetched" cannot both be true.
- Pending is now "never fetched **and** never checked-missing" (`SQL_PENDING`), so a
  removed record costs one request per crawl instead of one per run. The arithmetic
  holds on the migrated scratch DB: `3000 = 10 fetched + 3 checked-gone + 2987
  pending`, and the next run did not re-ask the three marked ids.
- `SCHEMA_VERSION` moved `1 → 2` **with** a migration: a file saying `schema_version
  = 1` gets `ALTER TABLE item ADD COLUMN detail_missing_at INTEGER` in place and its
  `meta` stamp moves in the same call, because refusing it would throw away a 19 258-row
  catalogue. A file **newer** than this build is still refused. Both fresh-DB stamps now
  write the build's own version, so a fresh v2 file never refuses itself.

Added

- Coverage counts the third state (`missing` in `status`/`stats`), and the status card
  prints it (`checked as gone upstream`). The pull-all notes say what a completion
  means with removed records in it: fetched, gone, and what remains live.

Changed

- `item <id>` on a removed record answers the exit-code contract's `4` (`not-found`)
  and re-stamps the mark: a human-aimed ask may re-ask once per request, a crawl may
  not. A **list page** answering `404` stays a failure (exit `3`).
- The fixture now lists one id (`127555`) that has **no** detail record and answers
  upstream's own error body, so CI exercises the third state: `item 127555` → exit
  `4`, and the crawl after it asks nothing. The exit-code audit gained that probe:
  `2 · 2 · 3 · 4 · 4 · 4 · 0`.

See [ADR 0006](./docs/DECISIONS/0006-checked-missing-third-state.md) and
[docs/DEVLOG.md](./docs/DEVLOG.md).

## v0.5.2 — patch: one click finishes the catalogue · 2026-11-20 · tag `v0.5.2`

Fixed

- Pending is now "never fetched", not "no price" (`SQL_PENDING` in `src/stats.rs`).
  The old `OR price_min IS NULL` clause kept a fetched item that is simply **not
  on the auction house** pending forever: the detail stage walked it on every run,
  so the stage never terminated on the real catalogue and no "pull everything"
  flow could honestly promise nothing left to pull. `docs/DATABASE.md` keeps those
  two states apart on purpose; the pending rule now uses the same distinction.
- A sync verb launched from the GUI never reached the API at all: the marker
  audit stopped after the DB-open point with an empty fixture access log, while
  status polls kept completing around it. reqwest's blocking client builds a
  **private tokio runtime on an internal thread** and parks the calling thread
  waiting for that runtime's startup handshake; the verb body sat on a worker of
  tauri's tokio-backed async manager, so the handshake never landed and the verb
  stalled at `build_client` with zero requests issued. `cli_call` keeps the async
  wrapper from v0.5.1 (the page still answers immediately) and the verb body now
  hops to a fresh `std` thread — the runtime-free home the headless CLI has always
  used in the CI mirrors.

Added

- "Pull all info" replaces the "Sync pending details" button: one flow runs the
  list walk to its end, then the pending-detail queue to its end, so a single
  click retires all pending work. `runSync` reports an outcome
  (`complete`/`cancelled`/`stalled`/`failed`/`already-running`) so a cancelled
  chain stops at the stage in flight instead of jumping ahead — the next click
  resumes the list from `sync_state` and the pending queue recomputes what details
  remain. A stage that is complete with zero requests is reported as complete,
  not as "made no progress". The completion note is explicit about what "nothing
  left to pull" honestly means: every listed item has a detail record, a NULL
  price on a fetched item means not on the AH, and prices stay live, so later
  syncs only revalidate (every page answers `304` from the stored ETags).

## v0.5.1 — patch: the crawl no longer freezes the webview · 2026-11-19 · tag `v0.5.1`

Fixed

- A running sync chunk no longer parks the whole app. A bare-`fn` command
  gets tauri-macros' blocking wrapper — its last acts are
  `kind.block(result, resolver)` then `return true` (`wrapper.rs:425`/`:428`)
  — so one chunk (2/s pacing, plus a `Retry-After` sleep on a 429: minutes in
  the worst case) ran synchronously **inside** the webview's IPC handler,
  which is the page's message loop: hover stopped highlighting, clicks queued
  behind the crawl for minutes. `cli_call` is now an `async` command
  (`respond_async_serialized`, tauri's tokio-backed async manager — tokio is
  a required dependency of tauri 2.x, so no feature switch was needed): same
  argv, same dispatch, same envelope, only the wrapper kind flips.
- Mid-chunk the page now says so: a ticker re-reads the DB once a second
  (coverage grows per page in list, per item in detail) and prints the
  chunk's elapsed time in the sync note; the cursor line honestly stays at
  the last chunk end, because progress truth lives in `sync_state` (ADR 0002).

Added

- The launch audit gained a witness: during a crawl, `gui: ipc verb sync`
  alternates with one-per-second `gui: ipc verb status` polls — that
  interleaving is what proves the webview runs mid-chunk
  (see [docs/DEVLOG.md](./DEVLOG.md)).

## v0.5.0 — detail-sync targets, price analytics, item card · 2026-11-19 · tag `v0.5.0`

Added

- Targeted detail sync: `sync --stage detail --ids 25,40,60` and `sync --stage detail --watch`
  visit exactly those records instead of the pending queue, which is what turns the watch list
  from something that prints into something that acts. The UI drives it with two buttons
  ("Details: watch list", "Details: shown hits") that slice the ids 20 per verb call, so
  Cancel between slices stays meaningful and one slice stays far below the 120-request burst.
- `item ID --cached`: a DB-only card read — no request at all. A record that was never
  fetched is `not-found` (the first use of that error kind, which retires its dead-code
  warning) and the UI turns the word into an offer instead of an empty card.
- UI cards: an item card that answers offline first (`Read the DB` vs `Re-check online`,
  printing when the record was fetched), a price-analytics card that renders
  `stats --limit 6` as per-quality bars in the search-chip palette plus cheapest/priciest
  extremes, and watch toggles inside the hit rows (the id travels in `data-id`, one
  delegated handler serves the whole table) with a `Watch these hits` batch action.
- Detail fetches send the stored `detail_etag` when one exists, so re-asking for the same
  ids answers `304` and rewrites nothing — the audit reads `written 0, unchanged 2`.
- The CI smoke block grew by six steps (watch add → targeted fetch → the same ids again
  answering 304s → watch-list target → DB-only card read), all eleven green in the local
  mirror before the commit.

Fixed

- Three single-column lookups in `db.rs` inferred a bare `String` getter for a nullable
  column, so a NULL came back as rusqlite's `Invalid column type Null` **DB error** (exit 4)
  instead of `None`. The pending stage never read those columns before fetching; the
  targeted stage reads `detail_etag_of` for every id, so the latent wrongness surfaced on the
  first audit run. The getters now ask for the `Option` type they always claimed to return.
- Targeted runs now answer their own question: `remaining` counts the target list ("left in
  the list", not "left pending") and `complete` means the target set was fully visited.
- `boot()` reads status, the watch list and the analytics at startup, so no card paints
  empty; the second status read is the launch audit's witness — four `gui: ipc verb …`
  lines prove the analytics render ran to the end rather than throwing after its await.

## v0.4.0 — Tauri shell + webview UI · 2026-11-19 · tag `v0.4.0`

Added

- The GUI shell lands: `tauri.conf.json` (window declared in `app.windows`, `withGlobalTauri`,
  `frontendDist: "ui"`, `bundle.icon`, no `version` key — see below), `src/gui.rs` (the
  `Builder` + `generate_context!` shell whose `run_gui()` blocks until the window closes) and
  `src/commands.rs` (the app's **one** command, `cli_call(argv)`, returning the envelope
  `{"ok":true,"result":…}` / `{"ok":false,"error":{kind,exit,message,retry_after}}`).
- `ui/index.html`, `ui/app.css`, `ui/app.js`: a status card (DB path, coverage, crawl state),
  a search card (text, quality, sort order, `priced only`) rendering the `Hit` fields with
  quality chips and a `view` link per record, and a sync card whose Cancel button works at
  chunk granularity. The attribution footer is static HTML — no code path can hide the link
  TheWoWDB's developer terms require.
- `src/cli.rs` grew `dispatch(Invocation) -> AppResult<String>`: the CLI door (`run`) and the
  GUI door (`cli_call`) send verbs through the same function, so ADR 0004's single code path
  reaches into the UI (see [ADR 0005](./docs/DECISIONS/0005-gui-ipc-commands.md)).
- Chunked sync from the UI: `sync --stage … --budget 10` (list) / `--budget 25` (detail) in a
  row until `report.complete`; Cancel raises a JS flag read at the next chunk boundary, and
  every chunk boundary is a `sync_state` write, so stopping is always resume-safe (ADR 0002).
- `build.rs` calling `tauri_build::build()` plus `tauri-build = "2.7.6"` as a **build**-dependency:
  this is what writes the global-API script list into `OUT_DIR`, i.e. what makes
  `window.__TAURI__.core.invoke` exist. It is cargo's own build script, not the Tauri CLI —
  [ADR 0001](./docs/DECISIONS/0001-flat-cargo-layout-no-cli.md) carries an amendment.
- `icons/icon.png` is generated by `tools/gen_icon.py` (codegen reads `bundle.icon` at compile
  time, so the file must exist before the first GUI build); `.gitignore` gains `/gen/` for the
  JSON schemas `tauri-build` writes next to `Cargo.toml`.
- `gui:` log lines on stderr (`gui: ipc verb status (0 words)` …) are the app's audit channel —
  nothing in `tauri`/`wry` forwards the webview console on Linux.
- `Cargo.toml` version `0.4.0`. `tauri.conf.json` deliberately omits `version`:
  `tauri-codegen` falls back to `env!("CARGO_PKG_VERSION")`, so `Cargo.toml` is the single
  version source and the "bump both files" house rule was amended, not broken twice.

Fixed

- The `cli.rs` refactor re-audited green rather than assumed so: exit-code audit `2 · 2 · 3 · 4 · 4 · 0`,
  the CI smoke mirror all `exit=0`, and the v0.3.0 contract (`resumed_from: 2`, `unchanged: 2`,
  `item 25` answered `304 reused true` from stored `raw_json`) unchanged.

## v0.3.0 — sync hardening: reuse, resume, cancel · 2026-11-19 · tag `v0.3.0`

Added

- Conditional GETs everywhere the crawler revisits: each list page keeps its ETag plus
  `has_more`/`count` continuation hints in `meta` (a `304` has no body — the crawl walks
  on using the hints), and the `sync_state` page column is an absolute cursor.
- Stop→resume: a list run that stopped by budget or cancel resumes at the page after the
  one `sync_state` says it last fetched; a finished run re-crawls from page 1, where the
  stored ETags make that a walk of `304`s. Honest reports: `resumed_from`, `unchanged`.
- Live `item` lookups revalidate instead of re-download: stored `detail_etag` goes out as
  `If-None-Match`, a `304` is answered from `raw_json` on disk, a fresh `200` is written
  through the same upsert the sync stage uses. Output gained `"reused"`.
- Cancel plumbing: both crawl loops check a `CancelFlag` before every request; the CLI
  hands out a fresh one (Ctrl-C stays resume-safe), the GUI raises it via AppState at
  v0.4.0. A cancelled run records status `cancelled` and exits 0.
- The fixture server now speaks conditional GETs (stable per-endpoint ETags, real `304`
  no-body), and the CI smoke sequence exercises the whole v0.3.0 contract: budget stop
  → resume → full re-sync (all `304`) → detail crawl → `item` answered from the DB.

## v0.2.0 — data layer + working sync · 2026-11-19 · tag `v0.2.0`

Added

- `src/` headless core, one buildable unit: `errors.rs`, `models.rs`, `wowdb.rs`,
  `db.rs`, `query.rs`, `stats.rs`, `sync.rs`, `cli.rs` plus `main.rs` argv dispatch —
  the six CLI verbs of [ADR 0004](./docs/DECISIONS/0004-headless-cli-mode.md).
- SQLite storage ([docs/DATABASE.md](./docs/DATABASE.md)): wide `item` row plus
  `meta`/`watch`/`sync_state`, list + detail upserts, `--db PATH` overriding `WGT_DB`.
- HTTP client with pacing by construction: 90-request headroom under the 120-request
  burst, 2 req/s schedule, `429` honours `Retry-After` and never aborts.
- Both sync stages verified end-to-end against a fixture server, incl. retry-in-place
  for a `429`'d item and an honest `complete`/`partial` report derived from the pending
  count.
- CI smoke test is now live (was commented in the scaffold tag): fixture server
  `tests/fixture_api.py` + `sync --stage list` + `sync --stage detail` + `stats`.
- `Cargo.toml` version 0.2.0 with the pinned dependency line unchanged. `tauri.conf.json`
  joins the tree at v0.4.0 (GUI shell), so its version bump lands there — see the DEVLOG
  note on the amended house rule.

## v0.1.0 — scaffold · 2026-11-19 · tag `v0.1.0`

Added

- Repository layout: `src/` (Rust), `ui/` (frontend assets), `docs/`, `.github/`.
- Project documentation: [DEVELOPMENT](./docs/DEVELOPMENT.md) (architecture, roadmap, house
  rules), [API](./docs/API.md) (TheWoWDB notes: paging, ETag revalidation, rate limits),
  [DATABASE](./docs/DATABASE.md) (SQLite schema + query recipes), [DECISIONS](./docs/DECISIONS/)
  (ADR-style records).
- `README.md` with the roadmap and the required data attribution, `CHANGELOG.md`,
  `LICENSE` (MIT + data note), `.gitignore` for Rust/Tauri/IDE artefacts.
- GitHub-style repository metadata: issue + PR templates, CI workflow (`build`, `test`,
  `stats` on a seeded fixture DB).
- Default `hello world` binary kept untouched (base crate compiles as-is).
