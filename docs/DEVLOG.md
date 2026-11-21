# Development log

Dated, reverse-chronological notes on what was tried, measured and decided while building
WoW Goldtool. The changelog records *what* shipped; this file records *why*.

---

## 2026-11-21 · v0.9.5 — the price homes: FOUR columns own FOUR questions, carved now, filled later

### What forced it — the user's own test

The user tested the app and said: "the prices shown right now are not the vendor prices —
there should be a vendor sell value and a vendor buy value (if the item can be bought), and
down the line two more: current auction-house min buyout and the historic normal the live
trend answers. How we'll get to those values can be ignored for now, but the columns should
be allocated now." The answer until today really does carry ONE price (plus the vendor sell):
`price_min` is the STORED copy's cheapest AUCTION-house buy — an aging payload's number, not
a live one, and not a vendor's. The honest shape is FOUR homes, and the user's sentence
settled the scope: carve NOW, fetch LATER.

One thing the test exposed that the record must keep: `price_min` answering "the gold column"
in docs/DATABASE.md was always the stored copy's number — users read `2g 5s 27c` on the
money cell and assumed the AH OFFERS that now. It never promised now-ness. The hop names the
live question as its OWN column instead of quietly overloading the old one.

### The decision, and what was rejected

The schema hop 5 → 6 adds `vendor_buy`, `price_min_now`, `price_trend_normal` — names that
are OURS: no upstream door has EVER answered these words (docs/API.md's measured keys hold
nothing near them), so writing a producer now would INVENT an upstream shape (rejected twice:
the user deferred fetching, AND the house rule forbids guessing at payloads). Computing live
values at answer time was rejected too — unlike money (ADR 0015: a pure rendering of copper
that already exists), "what the AH offered at 14:05" is an OBSERVATION that AGES: it cannot
be re-derived from what is on disk, so its home IS the DB. Overloading `price_min` for the
live value was rejected: one column answering two questions over time is two truths in one
word — `price_min` stays what the payload said, `price_min_now` is what the AH says NOW.
Carving nothing and documenting a wish was rejected by the user's own words. Chosen:
additive ALTERs, `check_schema` chains v1..v5 up to 6 — the staircase v0.9.0's comment
promised since its own hop. NULL in the trio means **"no producer has ever written it"** —
deliberately NOT yet the fetched/not-there triple the price columns split: a checked-missing
state for a door that does not exist is ADR 0011's rejected plan; the third state arrives
WITH the first producer.

### Chain

`cb80fcc` feat(db) (`SCHEMA_VERSION` 6, trio in `SCHEMA_SQL` before `raw_json`, `SQL_MIGRATE_TO_V6`,
arms 1–5 in `check_schema`) → `5fa6b23` ci (money paragraph gains its tense note, schema claims
truthful, 31 steps stay 31) → `e5acc83` chore (banner 0.9.4 → 0.9.5, Cargo.lock rides) →
this record carries the tag.

### Verification, pre-tag — the hop, ZERO requests

| probe | answer |
| ----- | ------ |
| fresh DB born at 6 | `.schema item`: trio BEFORE `raw_json` in CREATE; `schema_version|6` |
| scratch schema-5 copy, first door | `status` rc=0 → meta 6; rows 6 / 4 priced / 4 vendor_sell intact, copper unchanged (25 keeps its stated `606`), trio NULL on all; SECOND `status` BYTE-IDENTICAL, nothing re-migrated |
| the user's own file, first door | 5 → 6 in place: **19 258 rows / 4 231 vendor_sell / 4 285 samples intact**, trio NULL on ALL (`COUNT ... IS NOT NULL` = 0), money doors re-answer (`item 14138`: `vendor_sell 20042 → 2g 42c`, no money key inside `item`) |
| door pairs, BOTH binaries (v0.9.4 tree `4875222`, scratch worktree; this one) over two copies of ONE fixture DB | **12 of 13 BYTE-IDENTICAL** at equal byte counts: search 1 941 / 763 / 755 / 3 363, stats 1 533 / 1 850 / 17 462, cards 4 336 / 1 103 / 3 868 / 4 022, `refresh-twins` 223; `status` differs ONLY in its `db.path` echo — pinned, IDENTICAL |
| pinned v0.8.1 binary on a migrated file | exit **4**, `database on disk uses schema 6, this build wants 4`, file untouched after |
| build through the chain | exit 0, the FOUR named dead-code warnings the whole way — `SQL_MIGRATE_TO_V6` had its consumer in the SAME commit (no dead window, unlike v0.9.4's `money_of_avg`) |

This release's own push verdicts land in the post-tag section below, joining the ladder
(`v0.9.4` → `4875222`, `v0.9.3` → `78426d1`, `v0.9.2` → `46ee524`, `v0.9.1` → `5631168`,
`v0.9.0` → `174460f`).

### Verification, post-tag — fresh fixture ports 9190 / 9191 / 9192..9199 + 9200

* **Mirror replay from the committed block** (the sed-generated driver, fresh port 9190 —
  verified free by a python socket bind — fresh scratch DB): BEFORE running, the driver's step
  list was compared to the committed block after stripping the `cargo run --` prefix —
  **EQUAL, 31 steps, zero diffs** (an early comparator pass printed noise from its own prefix
  slicing; the honest filter was the one used). Result **31/31 steps rc 0, plant line rc=0**,
  fixture tally **31 GETs = 200×15 / 304×9 / 404×3 / 429×4** — identical to the v0.9.4 and
  v0.9.3 replays: the hop asks nothing. DB tail `6|4|4|1|1|0`, icons `3|1`, twins `4|4|4|0`,
  made row **ZERO** asks, committed fixture untouched — and schema reads **`schema_version|6`**
  now: the fresh file is BORN at 6 (the replay driver's own summary header still says "must
  stay 5" — that header line is the v0.9.4 claim the sed driver carried along; the ANSWER the
  replay prints is 6, and the comment in the committed ci.yml names the truth). Trio
  `COUNT(...) IS NOT NULL` = **0** on the mirror: reserved homes, NULL everywhere.
* **Pre-hop replay**: the whole 31-step block run with the **v0.9.4-era binary** (tree `4875222`,
  scratch worktree) on fresh port 9191 — 31/31 green, same GET tallies — then the v0.9.5 build
  READ that schema-5 file: 9 doors over two copies, **8 BYTE-IDENTICAL** at equal byte counts
  (`status` aside, whose 1-byte delta is its own `db.path` echo — with that string pinned the
  JSON is IDENTICAL), `item 25 --cached` answers on the MIGRATED file exactly as on the old one
  (`4 336` bytes, money pair `vendor_sell 606 → 6s 6c` rides), stamps equal row-by-row, trio NULL
  on all — **no refresh door**: the reservation reaches old files on the next read.
* **The full exit-code audit re-run** on fresh ports 9192..9199 + 9200 and fresh DBs: **90 doors**,
  the v0.9.4 map's **90/90 rc pairs IDENTICAL, zero changed** — money doors 87/88 still answer
  usage `2` (still NO flag for money or the trio), 89/90 still `0`; totals `0` ×59 · `2` ×14 ·
  `3` ×6 · `4` ×11 — byte-for-byte the v0.9.4 distribution. Fixture A **29 GETs = 200×16 /
  304×3 / 404×6 / 429×4**, B **2**, C **4 = 200×2 / 404×1 / 429×1** — identical to the v0.9.4
  audit: the trio sent no ask anywhere.
* **One driver glitch found and fixed, honestly recorded**: the audit's FIRST pass let door 65
  open a scratch DB the pinning was NOT supposed to see — `migr095b.sqlite` sat there from an
  earlier pre-tag measurement, already migrated to 6 — so door 65 printed rc **4** (the pinned
  binary's HONEST refusal of a schema-6 file, product behaving perfectly) while its "FRESH
  file" precondition was quietly broken. The DRIVER lied, not the product; `rm -f` for the pinned
  scratch DB (and its WAL sidecars) restored the precondition, and the re-run measured door
  65 rc=0 (pinned crawl WRITES schema 4: 3 requests — 2 pages + one 429 — matching C's 4-GET
  tally with the liveness probe), door 66 rc=0 with the CURRENT build migrating **4 → 5 → 6**
  in one open (5 items intact, meta `schema_version|6`), and door 67 rc=4 honest refusal on the
  migrated file (`database on disk uses schema 6, this build wants 4`), file untouched. The
  stale-chain hop chain is proven from the PINNED era all the way to 6.
* **CI run 44960088009** (chain+tag push `1bc4e50..23ceb4e`, `head_sha 23ceb4e…` — the TAGGED
  tree): **attempt 1 CANCELLED** (created 23:05:59Z, ended 23:12:07Z) — GitHub's branch
  management superseded it the moment this verification record's push landed a new head on
  `main`: a run-management fact, not a build failure, not the app's. Resubmitted through the
  rerun endpoint: **attempt 2 SUCCESS, 5/5 jobs green in 6 min 12 s** (jobs 23:14:47Z →
  23:20:59Z — quality 1 m 57 s · test 4 m 40 s · linux 5 m 23 s · macos 6 m 10 s · windows
  5 m 59 s). Same head, re-measured: the TAGGED tree is green.
* **CI run 44960596846** (verification record's push `23ceb4e..62a7501`, head `62a7501…`):
  **attempt 1 CANCELLED** (jobs started 23:12:09Z, ended 23:12:09Z → 23:14:3x–4x — only
  quality finished green) — superseded by the rerun submission above, landing on the SAME
  branch while this run was still in flight; the observed rule stated where it bit: a rerun
  or a fresh push on the branch cancels that branch's in-flight run, which is why NOTHING
  was pushed while these runs were underway and the two rerun requests were the only
  submissions. **Attempt 2 (rerun): SUCCESS, 5/5 in 9 min 35 s** (23:22:03Z → 23:31:38Z —
  quality 1 m 56 s · test 4 m 47 s · linux 5 m 14 s · macos 5 m 48 s · windows 9 m 35 s), on
  the verification tree.
* **The v0.9.4 ladder is now COMPLETE**: its verdict-fold push run **44957300299** (head
  `1bc4e50…`) landed **success, 5/5 in 10 min 50 s** (22:32:48Z → 22:43:38Z), beside chain+tag
  **44954802500** (10 min 36 s) and verification push **44956129051** (10 min 44 s) — every
  recorded push through this release is green, either straight or after an honest resubmission
  of the SAME head.
* **This CI-record commit** starts its own run when pushed; its verdict lands FORWARD the
  same way — never chased with extra commits — and the tags stay put (`v0.9.5` → `23ceb4e`,
  `v0.9.4` → `4875222`, both re-verified after this push).


## 2026-11-21 · v0.9.4 — the money words: prices read like the GAME writes them, and copper stays the number

### What forced it — the unit was an open question, and the DB's OWN copies settle it

Until today the app printed raw integers next to a column comment that says `-- copper`: the
DB holds `20628`, the game writes `2g 6s 28c`. Nothing in this repo states the base, so the
base got MEASURED before any code changed — and the DB already carries the answer, because a
copied tooltip states its sell price in money words. One read of the user's own file:

```
rows                                                19 258
rows with a vendor_sell INTEGER                      4 231
rows whose copy states a "Sell Price:" line           4 231   the SAME rows
money_text(vendor_sell) EQUALS the copy's own words   4 230   of 4 231
rows that DISAGREE with their own copy                    1   → item 25: column 500 ("5s")
                                                            beside its copy "Sell Price: 7c"
rows at copper 0 in ANY copper column                     0   MIN over each column is 1
```

The copies also state the zero rule, so the shape is upstream's, not our invention:

```
  810  vendor_sell  45 267  copy "Sell Price: 4g 52s 67c"   all three parts
 2528  vendor_sell  10 367  copy "Sell Price: 1g 3s 67c"    100 c = 1 s, 100 s = 1 g
 4499  vendor_sell  25 000  copy "Sell Price: 2g 50s"       zero copper DROPPED
12720  vendor_sell  20 000  copy "Sell Price: 2g"           zero silver AND copper
  733  vendor_sell     100  copy "Sell Price: 1s"
   36  vendor_sell       7  copy "Sell Price: 7c"
14138  vendor_sell  20 042  copy "Sell Price: 2g 42c"       the MIDDLE zero: silver dropped
                                                            between gold and copper
```

Money classes over `vendor_sell`: 1 413 rows are copper-only, 810 whole silver with zero
copper, 219 carry all three parts, 73 whole gold, and exactly **1** is middle-zero — while
`price_min` has 19 middle-zero rows and `price_market` 17, so the rule is per value, not an
accident of one column. Spans and the money each column reaches:

```
price_min      n 4 291   1 .. 14 006 500   →  "1c" .. "1400g 65s"  longest 12 chars "799g 99s 98c"
price_market   n 4 291   1 .. 5 000 001 000 → "1c" .. "500000g 1s" longest 13 chars "1334g 99s 49c"
vendor_sell    n 4 231   1 .. 1 000 000    →  "1c" .. "100g"       longest 10 chars "8g 42s 91c"
AVG(price_min) = 54 391.705896 copper      →  floored "5g 43s 91c", rounded would print "5g 43s 92c"
```

### The decision, and what was rejected

Storing money in the DB was rejected before it was considered seriously: money is a rendering
of a number that already exists, and a `*_money` column would split one truth in two, break
the numeric contract docs/DATABASE.md names (the price sorts read INTEGER copper, and so do
scripts), and would drag a refresh door — which is exactly the shape v0.9.2's `refresh-twins`
had to be because a twin is STORED. Formatting money in JavaScript was rejected for the house
rule ADR 0014 restates with `b64_text`: the CLI and the GUI would then hold two formatters of
the same words and could drift. Always-visible three parts (`2g 0s 0c`) was rejected BY THE
COPIES: upstream drops a zero denomination, so printing `0s` puts words in its copy. Money
only in the UI was rejected because ADR 0004 made the CLI and the GUI one code path down to
the same JSON; a hidden money in the CLI would mean the two doors answer different shapes.
So: ONE Rust formatter, `money_text` in `src/db.rs` beside the price accessors, riding as a
**sibling fragment beside every copper number**, derived at answer time, with `money_of`
keeping an absent price absent (ADR 0004's made-up-zeros rule again: `null` beside `null`,
never `0c`) and `money_of_avg` FLOORING (money has no fraction of a copper, and rounding up
would print `5g 43s 92c` for a mean that never reached it). Money sits OUTSIDE the `item`
object on the card because `ItemDetail` and `Price` mirror the wire (docs/API.md): a field
inside them would claim upstream answered a word it never answered — the same ground ADR 0014
gave `icon_b64`. And money reads the MERGED detail, ADR 0010's merge-last rule applied to
money, which the fixture proves is not a harmless choice: row 25's `vendor_sell` was STATED by
a user at `606`, and the card prints `6s 6c` next to it, not the payload's own number.

Money needs NO flag, and that is the honest contrast with ADR 0014: an icon costs ≈ 3.1 K chars
once base64'd, money costs about 56 bytes per priced row on the mirror and the longest money
string in the user's whole DB is 13 chars. So money rides every answer, always.

### Honest non-gains and print glitches, recorded not swept

* **the money disagreement is upstream's and stays upstream's**: item 25 is the ONE row of 4 231
  whose column (`500`) disagrees with its own copy (`Sell Price: 7c`). The committed fixture
  carries the same shape (`"vendor_sell": 500` beside a copy saying `7c`), and after the CI's own
  `set-item 25 vendor_sell=606` plus the block's plant line, ALL THREE mirror rows that state
  Sell Price words disagree with their own column (`606 → 6s 6c` and `500 → 5s` beside `7c`).
  Money is money OF THE COLUMN, never a copy of a tooltip's words; nothing in this release
  reconciles the two, and no commit tries to — recording the disagreement is the honest answer.
* **the chain printed FIVE dead-code warnings for a window, then FOUR**: `money_of_avg` had no
  consumer between feat(db) and feat(stats), so the builds in that window said 5; feat(stats)
  landed its consumers and the set came back to the four names docs/DEVELOPMENT.md keeps. The
  feat(db) body says so; nothing swept the note.
* **the feat(stats) body printed `status 1 078 -> 1 125`; the measured pair is 1 035 → 1 125.**
  The commit bodies cannot move and the tag never moves, so the correction lives HERE, as the
  measurement, and every audit below re-reads 1 035 → 1 125.
* two fixes landed BEFORE any commit, so nothing shipped with them: the formatter's `parts.join(" ".to_string())`
  answered E0308 (Rust wants a `&str` separator), and one edit to `cli.rs`'s import line duplicated
  five names (E0252). Both were fixed and re-built before the commits that carry them.
* the money rows read `no price` on a row the user MADE (its `column_card` has no price to money) —
  honest, and it is the same wording ADR 0014's glyph carries: the card also says the row was made.

### Chain

`2cfb977` feat(db) (the formatter + `PriceSample`'s money pair, `history_of` fills it) →
`edbdfc5` feat(query) (`Hit`'s money pair, `HIT_COLUMNS` grows NOTHING) → `680e3e1` feat(stats)
(`Coverage`/`QualityStat`/`Extreme` money, `money_of_avg` consumed) → `21b316b` feat(cli)
(`card_money_parts` at all four card sites) → `8ae2939` feat(ui) (`money_cell`, the money rows,
trend hovers, the grid, the bars, the note, `.moneycell`, the three theads) → `f87e98f` ci (money
rides the block, NO new step, 31 stays 31) → `261b2d9` chore (banner 0.9.3 → 0.9.4, Cargo.lock rides,
schema stays `5`) → this record carries the tag.

### Verification, pre-tag — the money doors, ZERO requests

No network was touched by any money probe: every door below is `--cached` or DB-only, so the
mirror's fixture never saw an ask. Measured against the mirror DB and the user's own DB:

| door | answer |
| ---- | ------ |
| `item 3850 --cached` (user's DB) | `price_min 10075` beside `1g 75c` (the middle-zero class), `price_market 12500` beside `1g 25s`, `vendor_sell 7304` beside `73s 4c`; `available 8`, `realms 9` — no money sibling |
| `item 14138 --cached` | `vendor_sell 20042` beside `2g 42c` — and its OWN copy says `Sell Price: 2g 42c`, so formatter and copy agree with no code comparing them |
| `item 25 --cached` (mirror) | `20628 → 2g 6s 28c`, `17628 → 1g 76s 28c`, stated `606 → 6s 6c`, its ONE history sample `20628 → 2g 6s 28c` |
| `item 900000000 --cached` (the row the user made) | `price null` and money `null` beside `null` — the card answers from the DB's own columns |
| `search --limit 6` (mirror) | four priced rows `20628 → 2g 6s 28c` / `17628 → 1g 76s 28c`; the two unpriced rows (900000000, 127555) answer `null` beside `null`, NOT `0c` |
| `stats --limit 3` (mirror) | coverage `min/max/avg 20628 / 20628 / 20628.0` beside three `2g 6s 28c`; the Rare facet with ZERO priced items keeps `avg_price null` beside `avg_price_money null` |
| the UI cells (node runs the REAL `ui/app.js` over the REAL door JSON) | hits cells print `<span class="moneycell" title="20,628 copper exact">2g 6s 28c</span>`, unpriced rows print `no price`; the card prints `min price 2g 6s 28c` / `market price 1g 76s 28c` / `vendor sell 6s 6c` and keeps `available 12` un-moneied; a trend bar's hover reads `2g 6s 28c · 20,628 copper` |
| `schema_version` after money | `5` on BOTH the mirror's DB and the user's file; `refresh-twins` answers 4 copies / 0 rewritten / 4 matching / ZERO requests — money needed no door |

The v0.9.3 window's own verdict, recorded here so the record holds the whole ladder: the v0.9.3
record's push `e2db5a9..1b56f45` started GitHub run **44946961792: success, 5/5 jobs in 10 min 15 s**
(20:34:06Z → 20:44:21Z — quality 2 m 11 s · test 5 m 02 s · linux 5 m 27 s · windows 10 m 13 s ·
macos 6 m 36 s), and the tags stayed put (`v0.9.3` → `78426d1`, `v0.9.2` → `46ee524`, `v0.9.1` →
`5631168`, `v0.9.0` → `174460f`). This release's own push verdicts land in the post-tag section below.

### Verification, post-tag — fresh fixture ports 9177 / 9178..9185 + 9186 / 9187

* **Mirror replay from the committed block** (the sed-generated driver, fresh port
  9177 — verified free by a python socket bind — fresh scratch DB): BEFORE running, the
  driver's step list was compared to the committed smoke block — the SAME 31 steps, the
  only diffs being the fixture-port substitution. Result **31/31 steps rc 0, plant line
  rc=0**; fixture tally **31 GETs = 200×15 / 304×9 / 404×3 / 429×4**, byte-identical to
  the v0.9.3 replay — money sent no ask, so the tally could not move. DB tail `6|4|4|1|1|0`
  (rows|detailed|tooltips|tooltip_missing|manual|deleted), icons `3|1`, twins `4|4|4|0`,
  schema `schema_version|5`, the made row 900000000 saw **ZERO** asks, the icon encoder still
  answers **1** distinct shape. This replay's OWN copper differs from the v0.9.3 replay's
  (the fixture re-randomizes prices each run): `20527 → 2g 5s 27c`, market `17527 → 1g 75s 27c`
  — every money number in this section is read off the replay that produced it.
* **Money audit over the replay's 31 step JSONs**: **53 money keys** — `price_min_money` 28,
  `price_market_money` 12, `avg_price_money` 6, `min_price_money` 2, `max_price_money` 2,
  `vendor_sell_money` 3 — with **zero drift**: each money equals money of the very copper in
  the SAME answer (the float `avg 20527.0` answers `2g 5s 27c` — the floor holds), and **zero
  money keys inside any `item` object**. Honest probe glitch, recorded: the audit driver's FIRST
  pass stripped 5 characters (`money`) instead of 6 (`_money`) when looking up the money's
  sibling, which printed every sibling as "absent"; fixed and re-measured — the product never
  changed, the driver did.
* **Money against the fixture's OWN copies**: **3 of 3** rows that state Sell Price words
  disagree with money OF THE COLUMN (25: stated `606 → 6s 6c`; 40 and 60: `500 → 5s` beside
  `7c`), and 12809 carries 500 while its copy states no Sell Price line. Same shape the user's
  DB shows on exactly 1 of 4 231 rows — money reads the column and the record names the
  disagreement, nothing reconciles a column against a copy.
* **The full exit-code audit re-run post-tag** on fresh ports 9178..9185 + 9186 and fresh DBs:
  **90 doors**, and the v0.9.3 map's **86 pairs BYTE-FOR-BYTE identical**, zero changed. Money
  adds only four doors: an invented flag answers usage — `search sword --where description
  --moneyx` → **2**, `stats --money` → **2** (money has NO flag, ADR 0015) — while the money-
  bearing fixture door `search sword --where description --fixture` → **0** and `status` →
  **0**. New totals `0` ×59 · `2` ×14 · `3` ×6 · `4` ×11; the pinned migration trio 65/66/67
  stays `0/0/4`. Fixture A tallied **29 GETs = 200×16 / 304×3 / 404×6 / 429×4**, fixture B
  **2**, fixture C (the pinned window) **4 = 200×2 / 404×1 / 429×1** — money doors sent
  nothing, the tallies moved exactly as far as the doors ABOVE money did, nowhere further.
* **Money over a DB that never saw money**: the whole 31-step block replayed with the PRE-money
  v0.9.3 binary (built in a detached v0.9.3 worktree, since removed) on fresh port 9187 — 31/31
  green, same tally 200×15 / 304×9 / 404×3 / 429×4 — then the money binary read that DB:
  `schema_version` reads **5 before and after** (money wrote nothing to it), `search --limit 6`
  answers `16702 → 1g 67s 2c` / `13702 → 1g 37s 2c`, coverage `16702 / 16702 / 16702.0` beside
  three `1g 67s 2c`, the card prints its stated `vendor_sell 606 → 6s 6c` and the sample rides
  money, and the DB still has **zero** `*_money` columns. This is the opposite of the twin: the
  twin needed v0.9.2's `refresh-twins` because it is STORED; money answers on the next read of
  a file the OLD binary wrote, so there is nothing to refresh.
* **Money over the pinned-era DB** (the schema-4→5 migration hop, 5 rows / 0 detailed / 0
  priced): every money answer is `null` beside `null`, ZERO requests — money invents no zero
  on a migrated file, ADR 0004's rule survives the hop.
* **CI**: the chain+tag push `1b56f45..4875222` started GitHub run **44954802500: success,
  5/5 jobs in 10 min 36 s** (22:04:26Z → 22:15:02Z — quality 2 m 15 s · test 4 m 23 s · linux
  4 m 33 s · macos 7 m 16 s · windows 10 m 32 s), and it ran on the TAGGED tree
  (`head_sha 4875222…` — exactly what the tag holds). The verification record's own push
  `4875222..90cb963` started **run 44956129051: success, 5/5 jobs in 10 min 44 s**
  (22:19:17Z → 22:30:01Z — quality 2 m 16 s · test 4 m 40 s · linux 5 m 33 s · macos 6 m 19 s ·
  windows 10 m 41 s) on head `90cb963…`, the verification tree. This verdict commit starts its
  own run the same way — the ladder's rule keeps it honest: verdicts land FORWARD, the tags
  never move (`v0.9.4` → `4875222`, joining `v0.9.3` → `78426d1`, `v0.9.2` → `46ee524`, `v0.9.1` →
  `5631168`, `v0.9.0` → `174460f`).


### What forced it — one read of the user's own DB

Measured before any code changed, and it is the whole decision:

```
rows with icon_blob                       4 030  of 19 258 rows
checked as gone upstream (icon_missing_at)     1
never asked (no icon_at, no blob)         15 228
detailed rows                             4 602  ALL carry an icon_url, 4 030 carry bytes
blob bytes    min 1 385 · avg 2 327 · max 3 268   total 9 380 048  → ≈ 3.1 K chars each
```

(a) `icon_url` on 4 602 rows but bytes on 4 030: hotlinking the address would show an image
the DB never claims to hold — ADR 0008 already says the opposite of that ("nothing is
hotlinked, the DB gets the bytes"), and the hosts were measured serving `image/jpeg`. (b)
3.1 K chars per icon: the UI asks 40 hits (≈ 124 K chars) and 6 extremes (≈ 19 K), while a
CLI `search` with NO `--limit` would carry every visible row — tens of MB. So icons ride
when ASKED, and the ask is bounded by the caller.

### The decision, and what was rejected

Always-inline was rejected by that size measurement; a separate `icon ID…` verb was rejected
because the hits are already collected and the icon columns sit on the same rows in the same
DB — a second door would split one truth in two. A flag with no stamps was rejected because
an empty cell would read as "this item has no icon in the game", a lie for 15 228 rows. So:
`--icons`, a no-value flag beside `--watch`/`--cached`, and BOTH row shapes (`Hit`, `Extreme`)
grew the trio `icon_b64`/`icon_at`/`icon_missing_at` — keys always present, payload only when
asked, stamps always. The encoder moved out of `cli.rs` into `db.rs`, beside the bytes it
encodes, and `icon_b64_of` became the ONE helper the card and the lists call: the house rule
is one truth, one encoder, and the move was measured byte-identical (2 596 chars before and
after, sha256 prefix `c67a63484dc91767` both times, and the emitted base64 decodes back to the
DB's own 1 945 bytes). ZERO requests leave either door, so ADR 0004's `3` is unreachable there.

### Three honest non-gains, recorded not swept

The card needed NO code — it has drawn an `<img>` data URI since v0.6.0; the lists were the
gap. The watch list has no panel at all (measured in `ui/index.html`), so nothing was invented
to hang an icon on: watched rows gain icons where they appear, in the hits table. The live-search
table keeps no icon column, because that door reads and writes NO DB (ADR 0009) and upstream
names have no local copy. And 15 228 of 19 258 rows still answer the glyph — honest, because
those rows genuinely hold no bytes.

### Chain

`770b5bf` refactor(db) (the encoder moves) → `4de3b0d` feat(query) (the `Hit` trio) →
`d03e2f2` feat(stats) (the `Extreme` trio) → `618fae3` feat(cli) (`--icons`) →
`7db9502` feat(ui) (the icon columns + the glyph) → `41cc1f0` ci (smoke 29 → 31 cargo steps) →
`7fcd90c` chore (banner 0.9.2 → 0.9.3, schema stays `5`, Cargo.lock rides) → this record carries the tag.

### Verification, post-tag — fresh ports 9165 / 9166..9174 / 9176 (2026-11-21), nothing moved the tag

The tag stays exactly where it was: `v0.9.3` → `78426d1` (the record commit that carries it),
`v0.9.2` → `46ee524`, `v0.9.1` → `5631168`, `v0.9.0` → `174460f`. 9144..9164 were burned by
earlier windows, so every fixture below sits on a port that was verified free first (a python
socket bind) - 9165, 9166..9173, 9174 and 9176.

| probe (post-tag, v0.9.3) | measured answer |
| ------------------------ | --------------- |
| mirror replay on **9165**, driver generated FROM the committed ci.yml smoke block (31 cargo steps + the plant line) | **31/31 rc 0** (`MIRROR: 31 steps, 0 non-zero`), `plant rc=0`; the icon doors land LAST in the block: `search sword --where description --icons` → row 25 **2 596 chars + icon_at**, the manual row **none of the three** · `stats --icons` → both lists filled and row 60 answers **no bytes with `icon_missing_at` set** · `stats` without the flag → the trio present, payload null. Fixture tallies **GETs 31 = 200×15 / 304×9 / 404×3 / 429×4**, byte-identical to the v0.9.2 replay, so the icon doors asked it NOTHING. Mirror DB tail: rows 6 · detailed 4 · copies 4 · checked-gone 1 · manual 1 · **icons 3 copied + 1 gone** · twins 4\|4\|4\|0 · schema **5** (`900000000` asks ZERO, committed fixture untouched) |
| the ONE encoder across the card AND the doors (same replay) | every `icon_b64` any step emitted collapses to **one shape**: 2 596 chars → 1 945 bytes → **EQUALS the mirror DB's own blob** (`distinct shapes: 1`) - the card's row and the list doors cannot encode one copy two ways |
| exit-code audit on **9166..9173 + 9174** | **86 doors**: 01–77 keep the identical rc map (usage `2` ×11, network `3` ×6, not-found `4` ×11, rest `0`) incl. the pinned migration trio 65/66/67 = 0/0/4; the icon doors 78–86 answer **0/0/0/0/2/0/4/0/0** - `search sword --iconsx` the usage `2`, a missing DB the honest `4` - and with a fixture URL both doors still answer 0 with **A GETs 29 before = 29 after**: the flag asks nothing |
| the three icon states as three answers (door 80, `search --icons --limit 6`) | 12809/40/25 → **2 596 chars + `icon_at`** · 60 → **no bytes, `icon_missing_at` 1795292160** · 900000000/127555 → **neither stamp** ("no copy and never asked for one") - so the glyph's two states are distinguishable, not guessed |
| a DB whose icon stage never ran (door 83, a scratch copy with the blobs blanked) | states `{(None,None,None)}` plus the gone row's `{(None,None,stamp)}` - exactly what the glyph means on the user's 15 228 never-asked rows |
| **the icon doors over a DB BORN under the pinned v0.8.1 snapshot** (fresh port 9176, the pinned binary rebuilt from `a160ffa`) | the pinned era crawls list+detail+**icons** and leaves `schema_version 4` with 3 icon blobs and NO v5 columns; the current build opens it (`3\|1\|5` after the v4→v5 hop) and **both icon doors read those pinned-era bytes**: 3 rows × 2 596 chars + `icon_at`, row 60 gone-stamped, 127555 neither; every emission **decodes EQUAL** to the DB's own 1 945-byte blob (3/3, one shape); the fixture saw **15 GETs, all from the pinned crawl - ZERO from the doors** |
| GitHub | run **44944988663: success, 5/5 jobs in 10 min 42 s** (20:11Z → 20:22Z) for the chain push `ff1d174..78426d1` + the tag push - so this tag HAS a green run on its own tree (the v0.9.0 tag never did); this record's own push run gets its verdict recorded as soon as it lands |

The push of THIS record (`78426d1..e2db5a9`) started run **44946023051: success, 5/5 jobs in
9 min 44 s** (20:23Z → 20:33Z) — green on the record's own tree too, joining the chain push
`44944988663` (5/5 in 10 min 42 s) on the tagged tree. The tags stayed put throughout
(`v0.9.3` → `78426d1`, `v0.9.2` → `46ee524`, `v0.9.1` → `5631168`, `v0.9.0` → `174460f`).

Two honest print glitches in MY OWN drivers, recorded not swept: the audit's decode line fell
into its own "no bytes for row 25" fallback (my `sqlite3.connect` read of the audit DB returned
nothing while the door's answer clearly carried 2 596 chars - the clean decode proofs are the
mirror replay's `distinct shapes: 1 / EQUALS the DB's own blob: True` and the gap probe's 3/3
EQUAL against the pinned blob); and the gap probe's regex-based emission counter printed 0 while
the same answers parsed as objects show 3 emissions of 2 596 chars each. Both were re-measured
with the parsed JSON (the numbers above are the re-measurements), and neither touches the product.

## 2026-11-21 · v0.9.2 — the refresh door: old copies get their twin re-derived, ZERO requests

### What forced it — one measurement on the user's own file

ADR 0012 made the twin DERIVED and v0.9.1 corrected the derivation, but the pending rules
refuse a stamped copy (ADR 0007) and a `304` writes nothing — so the corrected rule only
ever reaches NEW copies. Measured the day this door was written, on the one file that
finished a crawl: **4 602 copies, ZERO matching the corrected rule, and the stored twin
equal to the recomputed v0.9.0 GLUED shape for all 4 602** (`Worn ShortswordItem Level
2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9(1.1 damage per second)Sell Price: 7c…`).
ADR 0012's "honest gap" (a migrated row with a copy has no twin until a write touches it)
has the SAME cause, and on this file the gap is every copy.

### The decision, and what was rejected

Re-asking upstream would spend 4 602 requests to re-fetch bytes the DB already holds — and
the pending rules would not even schedule it. Re-deriving at read time would put two
answers on one copy and let the column drift, against ADR 0012's invariant. Folding it
into the next sync pass is measured dead (the queues answer ZERO pending for stamped copies).
So: an explicit door the user presses — `refresh-twins`, no arguments, the whole DB, the
SAME `readable_tooltip_text` every write path calls (no second derivation to disagree),
only differing/missing rows written, rows collected before any write, tombstones and manual
rows riding, idempotent, and exit `0` with a report for the nothing-to-do door (`4` would
mean "the DB does not have it"). ZERO requests leave it, so ADR 0004's `3` is unreachable.

### The honest size of the gain

The card's words row becomes readable on every copy and the invariant `twin == derivation(copy)`
holds for the whole DB — those are the real wins. The description door is a SUBSTRING door,
so a phrase inside one span was always findable: measured on the user's 4 602 copies,
`damage speed` +358, `hand one` +90, `sword 1` +14, `1 main` +3, while `main hand` is 75
before AND after and `sell price: 7` is 291 both ways. The search gain is the phrases that
span an element boundary, not a general rewriting of what search can do.

Chain: `a5da052` feat(db) → `4cd95b3` feat(cli) → `d1703e4` feat(ui) → `cb8eee9` ci (24 → 29
cargo steps + one plant line) → `bb9b2c2` chore (banner 0.9.0 → 0.9.2, schema stays 5) →
this record carries the tag.

### Verification, post-tag — fresh ports 9161/9153..9160+9162/9163, and the door run over the user's own file (2026-11-21)

Nothing here moved the tag (`v0.9.2` stays on the record commit that carries it; `v0.9.1` →
`5631168`, `v0.9.0` → `174460f`). Every number was measured after the tag on FRESH ports —
9144..9152 and 9145..9160 were burned by the v0.9.1 window and no replay reuses them.

| probe (post-tag, v0.9.2) | measured answer |
| ------------------------ | --------------- |
| mirror replay, fresh port **9161**, driver generated from the committed ci.yml smoke block | **29/29 cargo steps rc 0** (`MIRROR: 29 steps, 0 non-zero`) + the plant line (`plant rc=0`); the refresh doors land IN ORDER: 0 rewritten / 4 unchanged → `search "damage speed" --where description` **total 0** → refresh **1 rewritten** → same door **total 1** → third press 0 rewritten; fixture tallies **GETs 31 = 200×15 / 304×9 / 404×3 / 429×4**, byte-identical to the v0.9.1 replay, so the refresh asked the fixture NOTHING; DB tail rows 6 · detailed 4 · copies **4** · checked-gone 1 · manual 1 · twins **4\|4\|4\|0** · schema **5** · `900000000` asks **ZERO** · committed fixture untouched |
| exit-code audit, fresh ports 9153..9160 + 9162 | **77 doors**: 01–67 rc map identical to the v0.9.1 run (usage 2 ×10, network 3 ×6, DB/not-found 4 ×10, rest 0), the migration trio still 0/0/4; the refresh doors 68–77 answer 0/0/0/0/0/0/0/**4**/**2**/0 — a missing DB is the honest 4 and `refresh-twins extra` the usage 2 — and **A GETs 29 before / 29 after** the refresh doors: the door asks nothing |
| ADR 0012's honest gap, measured the way it really happens (fresh port 9163, the pinned v0.8.1 snapshot rebuilt from `a160ffa`) | the pinned era crawls list+detail+tooltip and leaves `schema_version 4` with **3 copies and NO twin columns at all** (`.schema item` counts 0 of them); the current build opens that file (the `v4→v5` hop rides, rc 0) and reports `3\|0`; **refresh-twins answers `filled 3`** → `3\|3`; the card then reads the twin with ZERO requests (`item 25 --cached` → `Fixture Detail 25 Item Level 1 Main Hand One-Handed Sword 1 - 3 Damage Speed 1.9 Sell Price: 7c`); press 2 answers 0 rewritten; the fixture's 15 GETs all belong to the pinned crawl |
| **the door over the user's own file** — the residue v0.9.1 named, closed | BEFORE: 19 258 rows, 4 602 copies, the FIRST twin `Worn ShortswordItem Level 2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9(1.1 damage per second)Sell Price: 7cLowest buyoutsAlliance…`; the phrase doors answer `damage speed` 0 · `hand one` 0 · `sword 1` 0 · `1 main` 0 · `main hand` 75 · `sell price: 7` 291. **Press 1: rc 0 in 114 s, `copies 4602, rewritten 4602, filled 0, requests 0`** — the FIRST twin now reads `Worn Shortsword Item Level 2 Main Hand One-Handed Sword 1 - 3 Damage Speed 1.9 (1.1 damage per second) Sell Price: 7c Lowest buyouts Alliance …`; the same phrase doors answer **358 · 90 · 14 · 3 · 75 · 291** — the predicted gains exactly, and `main hand`/`sell price: 7` unchanged exactly as the substring rule says. **Press 2: rc 0, 0 rewritten / 4 602 unchanged.** File tail after: 19 258 rows · 4 602 copies · 4 602 twins · 4 602 `tooltip_lc` · schema **5** · icons 4 030 + 1 checked-gone · 4 284 price samples · manual 0 · deleted 0 · html-without-twin **0** — the file is intact and the ONLY thing that changed is the twin's wording |
| GitHub | the chain push `51a8d24..46ee524` + the tag push = run **44939561521: success, 5/5 jobs in 9 min 17 s** on the tagged tree, so this tag HAS a green run (the v0.9.0 tag never did); this record's own push run gets its verdict recorded as soon as it lands |

The push of THIS record (`46ee524..e746c4a`) started run **44940485574: success, 5/5 jobs in
10 min 36 s** (19:21Z → 19:32Z) — green on the record's own tree as well, and the tag stayed
exactly where it was (`v0.9.2` → `46ee524`, `v0.9.1` → `5631168`, `v0.9.0` → `174460f`).

Two honest print glitches in MY drivers, not in the product: `mirror092.sh`'s rows query named
a column this DB does not have (`detail_at`) and `refresh-user.sh`'s tail query named
`price_samples` — both tallies were re-measured with the real column names above, and the
drivers keep the glitch in their text so the record says which lines were re-measured.

## 2026-11-21 · v0.9.1 — the v0.9.0 defects die here: the scope door answers, the twin keeps its spaces

### What forced it — two measurements, no theory

1. **A hang**: both v0.9.0 replays (ports 9124 / 9125..9133) froze **2 h 01 m** on
   `search sword --where description`, the child in **state R** with no output; a scratch `timeout 5`
   repro answers **rc 124**. Read off `parse_invocation`: the value-flag path advances at the loop
   TAIL (`index = index + 2`); the v0.9.0 `--where` block answered VALID values with a bare
   `continue` — skipping the advance, re-reading the same pair forever. The house pattern (assign
   `out.scope`, fall through to the shared advance like `--sort`/`--quality`) lands; the bogus
   scope still answers the usage door `2`.
2. **Glued twins**: the user file's first live tooltip pass (4 602 copies) shows live tooltip html
   is minified — spans adjacent, no whitespace between them — and the v0.9.0 derivation DROPPED
   spans: `Worn ShortswordItem Level 2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9…`. The
   fixture's stand-in card carries spaces AT the tag boundaries, which is exactly why this survived
   every fixture-shaped audit. Measured lesson written down: **fixture-shaped data cannot disprove
   claims about the live shape; only the live pass can.** The rule now: every `<…>` span leaves
   ONE space, collapse whitespace runs, trim.

`ee8c06d` `fix(cli)` + `5a36d19` `fix(db)`, this record carries the tag. The v0.9.0 tag stays put
(tag `v0.9.0` = `174460f`, its verification record `05d073d`); the user file's 4 602 glued twins
stay until a copy's content changes (pending rule + 304-writes-nothing) — a refresh door is a
named plan.

### Verification, post-tag — fresh ports, the tag stays put (2026-11-21)

Nothing in this section moved the tag (`v0.9.1` = the record commit that carries it, `v0.9.0`
still `174460f`). Every number below was measured after the tag, on FRESH fixture ports — the
ports the earlier windows burned (9124..9143) stay BUSY/defunct and no replay reuses them.

| probe (post-tag) | measured answer |
| ---------------- | --------------- |
| mirror replay, fresh port 9144 | **24/24 steps rc 0** (`MIRROR: 24 steps, 0 non-zero`) — the three v0.9.0-added steps land: `search sword --where description` → rc 0, `"total": 2`, `--where all` → rc 0, `"total": 2`, and `item 900000000 --cached` answers the made row from the DB (`"cached": true, "reused": false`, twin `a plain CI sword mug`); fixture tallies **GETs 31 = 200×15 / 304×9 / 404×3 / 429×4**, stages detail 18 / icon 5 / tooltip 5 / search 3; rows 6, twins **4\|4\|4\|0** (`html-without-twin` = 0), schema stamp **5**, `900000000` asks **ZERO** |
| exit-code audit, fresh ports 9145..9152 | **67 doors**: the v0.8.1 map is byte-identical (05/06 = 4, 08 = 2, 10/11 = 2, 12–14 = 3, 19 = 2, 20/25/30 = 3, 29 = 2, 32–34 + 36 = 2, 35 = 4, 48–51 + 54 = 4), the v0.9.0 doors hold (58/59/60 = 0/0/0 with hits **1/1/0**, bogus scope 61 = **2**, twin doors 62/63/64 = 0/0/0 with the stated `<b>Probe Sword stated</b>` → twin `Probe Sword stated` in the SAME write, counts **3\|3\|3**), and `900000000` asks stayed **ZERO** across the stages |
| the schema chain, REAL now (fresh port 9143 + a scratch DB, the user's file never opened) | 65 rc **0**: the pinned v0.8.1 snapshot rebuilt from `a160ffa` BIRTHS its own shape → `schema_version\|4`, 5 rows; 66 rc **0**: the current build opens that schema-4 file and the `v4→v5` hop rides → `schema_version\|5`; 67 rc **4**: the pinned snapshot on the bumped file answers `database on disk uses schema 5, this build wants 4; delete the file to rebuild it` and leaves the file untouched (meta still 5, rows 5\|0) — the pre-tag run had to SKIP these doors (its scratch copy had been wiped by the harness restart); this one answers them |
| GitHub runs | the fix-chain push `174460f..5a36d19` = run **44934053776: success, 5/5 jobs in 8 min 11 s**; the record + tag push = run **44934961058: success, 5/5 jobs in 7 min 17 s** on the tagged tree — so this tag HAS a green run, which the v0.9.0 tag never did; the hung run **44863357406** stays `completed/cancelled` at 18:07Z, exactly as recorded under v0.9.0 |
| the user's own file opened by the v0.9.1 build | rc 0, `19 258 rows · 4 602 detailed · 14 656 checked-removed · tooltips 4 602 · twins 4 602 · twin_lc 4 602 · schema 5` — rows intact through the additive migration, and the 4 602 twins keep their glued v0.9.0 shape (the pending rule never re-asks a stamped copy, and a 304 writes nothing): the refresh door is still a named plan, not code |
| the committed fixture | `git status --porcelain tests/fixture_api.py` empty — the drivers generate their own scratch copies, `tests/fixture_api.py` untouched |

The push of THIS record (`5631168..b6d19d1`) started run **44935854879: success, 5/5 jobs in
10 min 9 s** (18:30Z → 18:40Z) — green on the record's own tree as well, and the tag stayed
exactly where it was (`v0.9.1` → `5631168`, `v0.9.0` → `174460f`).

## 2026-11-21 · v0.9.0 — the copies are READABLE and FINDABLE: the readable twin of every copied tooltip

### What forced the milestone

v0.8.0 stored each record's description copy as upstream's OWN tooltip `html`, and its own docs
state the two honest gaps that came with it: the card prints that markup as escaped TEXT (and
`search` would grep TAGS), and the local search door reads NAMES only — so the DB HOLDS
descriptions it can never FIND, and the card shows markup where the words belong. The user chose
BOTH together: one milestone ("the copies are readable and findable"), one ADR (0012), the code
chain split into `feat` commits.

### The measurements — every number probed, none invented

| probe (2026-11-21) | measured answer |
| ------------------ | --------------- |
| `GET /api/v1/tooltip/forever/item/25` | 200 json 3060 b, `html` 1404 b, `ETag W/"2b111cbcd3bbb4d7078f6d05"`, `cache-control public, max-age=300, s-maxage=900, stale-while-revalidate=3600` |
| tags inside that html | `<div class="twdb-…">`, `<span class="twdb-…">`, `<img class="twdb-icon" src=… width height alt loading decoding>` |
| `<img>` hotlinks INSIDE the copy | `render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg` + `thewowdb.com/assets/pvp/faction/alliance.jpg` — TWO hosts, pixels: the card must NEVER render upstream markup (the icon stage's rule, aimed at the same pixels, says the DB gets the bytes) |
| entities in TWO bodies | **ZERO** `&[A-Za-z#0-9]+;` matches in both — 25 (plain stats prose) AND 728, picked off the USER's DB as a row whose `raw_json` carries flavor (728/1132/1357 answer it; 728's `twdb-quote` shows `"Teaches you how to cook Westfall Stew."`, 1281 b of html) → the derivation is EXACTLY drop-spans/collapse/trim, and NO entity decoder exists — inventing one is not our call to make; if upstream ever ships entities they show VERBATIM and the words still search |
| fixture detail server CODE (`tests/fixture_api.py`) | `body["name"] = "Fixture Detail {}".format(tail)` for EVERY detailed row → the list door's row named "Worn Shortsword" (id 25) is OVERWRITTEN before the smoke's search steps run; the only row keeping its list name is the checked-404 127555 → `search sword --where description` AND `--where all` BOTH answer 2 (row 25 + the manual row's stated "a plain CI sword mug"); the name scope answers 0 |
| `next_manual_id` on a fresh DB | `MANUAL_ID_BASE` = 900000000 → the smoke's mug IS 900000000; and `cmd_item` answers manual-row cards from the DB's columns BEFORE the `--cached` branch → `item 900000000 --cached` sends ZERO requests, exits `0`, and prints the twin its fields earned at birth |
| the user's own file, decision date | schema `4`, 19 258 rows, **ZERO** tooltip copies (`tooltip_at` count 0) → the honest twin gap (a migrated row that ALREADY had a copy shows no twin until its next tooltip write — SQLite `ALTER` cannot run Rust, and duplicating the reader across languages is the drift DATABASE.md exists to forbid) only bites DBs that copied at v0.8.x; for this file EVERY copy will be born with a twin, because its tooltip pass runs AFTER this release |

### Own fixes, owned mid-chain (nothing swept under the rug)

- a python-style `.format(` leaked onto a parser line — `cargo build` answered, `format!` landed;
- the derivation arm missed a `;`, and the tooltip write double-wrapped an `Option` into
  `optional_text` — E0308 + parse errors, three small honest fixes;
- the `--where all` two-needle loop moved `needle` and rebound an immutable `seen` —
  E0382/E0384 → `needle.clone()` + `let mut seen`;
- the twin reader's regex function-clone failed twice (anchor drift) → exact-source insert
  after `tooltip_html_of`'s block instead;
- the smoke comment's FIRST draft claimed "all answers 3", grounded only on the LIST payload —
  measured against the fixture's OWN server BEFORE the tag (fa4311f rewrote it): a comment is
  trusted text for the next reader, so the lie died pre-tag, not post-record;
- the rustc ICE + "hard linking … failed" pair on this FS again: the recorded workaround
  (`rm -rf target/debug/incremental`) applied first, then `cargo build` rc 0 with the four
  documented warnings.

### The crawl sequencing — this release does NOT touch the running crawl

The pinned v0.8.0 snapshot crawl keeps working the user's file (measured mid-milestone: the
detail stage ≈ 63.5 % done — 12 235 of 19 258 rows PROCESSED, i.e. 3 580 answered + 8 655
checked-404). A schema-5 build stamps that file the MOMENT it
opens it, and the pinned snapshot would then REFUSE it (exit 4 — ADR 0004's honest refusal, and
the file would still be intact). House rule for this milestone: while the pinned crawl runs,
NO 0.9.x binary opens the user's DB; every audit of this release works on scratch DBs. When the
crawl finishes, the tooltip pass rides a v0.9.0 snapshot — and every copy it ever makes is born
with a twin, so the honest gap never shows on this file.

### What shipped

`89ff23d` schema 5 + the v4→v5 hop chained from EVERY stale shape → `cafe3af` `readable_tooltip_text`
+ `SQL_SET_TOOLTIP` grows both columns (value count 4→6) derived from the MERGED text, same write
→ `5c1341b` the manual doors birth the twin (`set-item tooltip_html=`, `add-item`) → `96f9f7a`
`Filter.scope` + `text_clauses`/`text_needles` (tombstone clause stays first) → `e3f7f8f` CLI
`--where` + usage line + `Invocation.scope` default `name` → `dd1599d` `tooltip_text_of` reader +
card fragment (BOTH shapes on the card) → `48b16a8` UI where select + rendered twin row →
`0e0f0ac` CI smoke 21→24 → `dbb699a` banner 0.9.0 → `fa4311f` the smoke comment's measured truth
→ docs record + ADR 0012 (this commit carries the tag).

### Verification, post-tag — what the replays ACTUALLY did (2026-11-21, kept not papered over)

The two replays this tag's own entry promised ("the post-tag mirror replay… recorded in the
verification record") ran — and did NOT land green. The truth, every row measured:

| probe (2026-11-21) | measured answer |
| ------------------ | --------------- |
| mirror replay (port 9124), steps 1–16 | all rc 0 — the v0.8.1-shaped steps still pass (the `add-item` now states a description) |
| step 17 / exit-audit door 58: `search sword --where description` | child process **R (running), 2 h 01 m, no output** — BOTH replays froze on this exact door; mirror stopped at 16/24, the audit at 57/66 |
| reproduction on a scratch DB | `timeout 5` → **rc 124** killed at 5 s — the door never lands: not slowness, a loop |
| the cause, read off `parse_invocation` | the v0.9.0 `--where` block used a bare `continue` on VALID values; the parser's value-flag path advances at the loop TAIL (`index = index + 2`) — the `continue` skipped it, re-reading the same pair forever |
| GH run for the tag-push tree (run 44863357406) | created 03:53Z, sat queued overnight, **cancelled — never ran**; a rerun was fired at 11:04 EST and reached the SAME smoke door: **in_progress 2 h 03 m, cancelled by hand at 18:07Z** (measured) — the tree at this tag never had a green run |
| the user file's crawl this window | the file read **schema 5 at ≤10:25** (a 0.9.x build opened it EARLIER than the sequencing rule assumed), and the pinned 0.8.0 snapshot then refused it (exit-4 door) and died with the harness restart. The crawl RESUMED on pinned v0.9.0 snapshots (pending rule = resume point): detail COMPLETE (990 fetched + 4 663 checked-404 → 4 602 answered / 14 656 checked-removed of 19 258), icons COMPLETE over 4 tries (a 403 icon door retried in place; 1 393 copied; DB: 4 030 icons + 1 checked-gone), tooltip COMPLETE in ONE try: **4 602 requests → 4 602 copies in 2 256 s (~2 rec/s)** — and `tooltip_text`/`tooltip_lc` both count **4 602**: every copy on this file was born with a twin; rows 19 258 intact through the additive migration |
| the FIRST twin on the user file | `Worn ShortswordItem Level 2Main HandOne-Handed Sword1 - 3 DamageSpeed 1.9(1.1 damage per second)...` — **words glued**: live tooltip html is MINIFIED (no whitespace between spans) and the v0.9.0 derivation DROPPED spans instead of leaving a space. The fixture never showed this — its stand-in card carries spaces AT the tag boundaries — so fixture-shaped data cannot disprove claims about the live shape: only the live pass can |

Both defects are fixed FORWARD, and this tag stays put: `ee8c06d` `fix(cli)` (the shared
advance) and `5a36d19` `fix(db)` (span→space) open the v0.9.1 chain; its green replays
(fresh ports 9134 / 9135..9142: mirror **24/24**, GETs 31 = 429×4/304×9/404×3, twins
4|4|4|0, hits 2/2/0, ZERO asks for the manual id; audit **64 doors**, the v0.8.1 map byte
identical + `0,0,0,2` scope + `0,0,0` twin doors) and its CI runs are recorded there. The
user file's 4 602 GLUED twins stay in their v0.9.0 shape until a copy's content changes
(the pending rule will not re-ask stamped copies, and a 304 writes nothing by ADR 0007) —
a refresh door is a named plan, not invented code.

## 2026-11-20 · v0.8.1 — CI says only what it can prove: the public project is green on three targets

### What forced the milestone

The repo got published — public `wow-goldtool`, `main` at `ebee66b` plus 11 annotated
tags, all verified on the remote (`git ls-remote` answers 24 refs, every tag object
`^{}` resolves to its commit). Publishing a "documented milestone project" carries one
promise it had NEVER made good: its CI. GitHub started its own run and it went red —
and the release discipline forbids leaving it red with a lie on top, so v0.8.1 is the
milestone that makes the workflow assert only what was measured. No `src/` line moves;
the discipline of the repo's OWN infrastructure earns its own ADR (0011).

### The measurement rounds — four runs, every error quoted

GitHub's runs are the witness. The local CI mirror replays the smoke block green (it
did since v0.8.0, 21/21), which proves WHAT THE SMOKE BLOCK DOES — it cannot prove the
WORKFLOW IS TRUE: it hardcodes its own DB path, it runs on one Linux box, and it never
evaluates env expressions or resolves action names. Only GitHub can say those things,
so every round of this milestone is "ask the runner, read the annotation, fix exactly
that line".

| run | commit | measured result |
| --- | ------ | --------------- |
| #1 | `ebee66b` | **failure after 7 s**, 5 annotations, ALL of them `Unable to resolve action`: `actions/rust-action`, `ubuntu-setup-action/install-ubuntu-system-dependencies`, `windows-setup-action/install-windows-system-dependencies` — `gh api orgs/…` answers **HTTP 404** for the two orgs; and the committed file has **no `checkout` step at all**, so even a resolvable action would have handed `cargo build` an empty directory |
| — | local | the same file's two gates measured on this tree: `cargo lint --help` answers `error: no such command: lint` (cargo 1.94.0) — the command does not exist; `cargo fmt --check` answers a **diff** on our own files (hand-written house style: long `use` lists, explicit `return`); `cargo clippy --all-targets` rc 0, **0 errors / 600 warnings** |
| #2 | `c29e45c` | **cancelled** — the workflow's own `concurrency` block cancelled it when #3 landed (fix round 1: `actions/checkout@v7` + `dtolnay/rust-toolchain@stable`, apt for the dev libs, clippy informational, header comment states the two limits) |
| #3 | `5c888b9` (fix round 2: apt set = `libgtk-3-dev libwebkit2gtk-4.1-dev`, the majors measured off `ldd target/debug/wow-goldtool` — the template pair was wrong on TWO majors: gtk 2 vs gtk 3, webkit-4.0 vs 4.1) | **failure**: build (linux) + quality **success**, three NEW lines answered: test died at the FIRST smoke step `cannot open /fixture.sqlite: unable to open database file` → rc 4, because `runner.tmpDir` is NOT a context property and evaluated to the EMPTY string; build (macos) rc 101 `error[E0463]: can't find crate for std` + `the x86_64-apple-darwin target may not be installed` while the SAME run's toolchain step answers `host: aarch64-apple-darwin` — macos-latest IS arm64; build (windows) rc 1 `` `D:\a\wow-goldtool\wow-goldtool\icons/icon.ico` not found; required for generating a Windows Resource file during tauri-build`` — the repo's `icons/` was EMPTY, and `tauri.conf.json`'s `bundle.icon` already named `icons/icon.png` for a file that never existed |
| #4 | `59b8379` (fix round 3: icon assets landed as `f29a2b2`, then `WGT_DB: fixture.sqlite` + matrix row `aarch64-apple-darwin` as `59b8379`) | **SUCCESS, 5/5 jobs in 9 min 13 s**: quality 2 m 04 s · build (macos, aarch64) 2 m 27 s · test 4 m 19 s · build (linux) 5 m 46 s · build (windows) 9 m 12 s — the whole sequence green on the public account |

### Decisions (one line each, full text in ADR 0011)

* Only resolvable action names — the two orgs answer 404, inventing similar names is
  not our call to make; `actions/checkout` and `dtolnay/rust-toolchain` are the
  documented, marketplace-real ones, and the checkout step is ALSO the fix for the
  structural half of run #1 (a cargo build with no checked-out tree).
* apt, not third-party setup actions: the package SET is measured off our own binary's
  NEEDED lines, so the workflow stops owing correctness to an unpublished org.
* clippy informational, no fmt gate: the gates would contradict measured truth (fmt
  answers a diff; the four dead-code warnings ARE the plan this very file names).
* No matrix narrowing: three targets still ride, but each now names the triple its
  image actually builds — macos-latest is arm64, measured.
* The icon assets are product work: `bundle.icon` already lied once (it names a file
  that never existed); the honest fix is to HAVE them — drawn by hand in PIL, our own
  card/coin/W, `file` confirms PNG 512x512 and a real 7-entry ICO.
* The smoke block stays verbatim through every round: its 21 `cargo run --` lines diff
  EMPTY against the audited sequence each time (asserted before each commit).

### The audit (executed, not asserted)

| probe | expectation | observed |
| ----- | ----------- | -------- |
| the old workflow's claims | resolvable + true, or the run is red | run #1 red in 7 s, quoted above |
| `cargo lint` / `cargo fmt --check` / `cargo clippy` on this tree | the gates that survive | lint DOES NOT EXIST / fmt answers a diff / clippy 0 errors + 600 warnings |
| `ldd target/debug/wow-goldtool` | which dev packages Linux needs | `libwebkit2gtk-4.1.so.0 libgtk-3.so.0 libgobject-2.0.so.0 libsoup-3.0.so.0 libpango-1.0.so.0` → apt `libgtk-3-dev libwebkit2gtk-4.1-dev` |
| the icon assets | real files, not ignored, our own | `file`: PNG 512x512 + ICO 7 entries; drawn by hand; `git check-ignore` says no |
| every ci.yml rewrite | smoke block preserved | parsed `cargo run --` list diffs EMPTY, 21 steps; YAML: build 5 / quality 4 / test 5 |
| `cargo build` after the banner bump (`fe691c9`) | everything still compiles | rc 0, 8.94 s incremental, exactly the four documented warnings, 0 errors |
| GitHub's verdict | green | **run #4: 5/5 jobs, 9 m 13 s** — and run #2 cancelled by the concurrency block, honestly |

The post-tag replays (CI mirror on fresh port 9114, the 57-door exit audit on 9115..9122
+ 9123, fresh DBs) follow in the verification record under this tag, same as at v0.8.0.

### What this milestone is NOT

It is not a code release: no door, no schema, no UI element moved. It IS the release
banner (Cargo.toml `0.8.1` at `fe691c9`) because the promise it fixes is the repo's own,
and the USER_AGENT line built from `CARGO_PKG_VERSION` now says `wow-goldtool 0.8.1` —
every upstream ask announces which build of the discipline it rode.

### Verification at the tag (executed AFTER the tag, on ports nothing had touched)

GitHub's own witness at the tagged tree and both local replays re-run against it:

| probe at `v0.8.1` | expectation | observed |
| ----------------- | ----------- | -------- |
| run #5 (`a160ffa`, the commit the tag rides) | green on the tagged tree | **SUCCESS 5/5 in 8 min 01 s**: quality 2 m 18 s · test 4 m 23 s · build (linux) 5 m 37 s · build (macos, aarch64) 6 m 30 s · build (windows) 8 m 01 s — five measured runs total, the last two green |
| `cargo build` at the tag (after the documented `rm -rf target/debug/incremental` workaround for the hardlink-free FS) | still compiles | rc 0, **0 errors, exactly the four documented warnings** |
| 21-step mirror replay GENERATED from the committed ci.yml (fixture port 9114, fresh DB) | the smoke sequence still green | **21/21 rc=0** · **31 GETs** (list 5 · detail 13 · icons 5 · tooltip 5 · search 3) · `4 × 429` retried IN PLACE · `9 × 304` · `3 × 404` · **ZERO asks for the manual id 900000000** · DB ends `6|4|3|1|1|0|4` · four `sync_state` rows `complete` |
| 57-door exit audit (ports 9115..9122 + 9123, fresh DBs) | the contract unchanged | **57/57 door/rc pairs identical** to the v0.8.0 record — usage `2`, network `3`, not-found `4` on the same doors; manual-row card 940 b DB-only; fresh write `requests 2, written 1, rate_limited 1`; keeps your `vendor_sell 606` while the crawl-owned columns refresh; ledger 4 → 5 |
| the harness hiccup, honestly owned | the mirror is GENERATED, so it cannot drift | my regeneration for v0.8.1 first pasted the ci.yml's literal `8765` into the step lines (fixture bound at 9114, steps knocked a dead port: 14 non-zero). The fix was IN THE HARNESS — steps re-pasted with `$PORT` — and the second replay is the one recorded. The repo's ci.yml stayed byte-identical throughout |
| only figures that moved between audits | prices, by design | the fixture gives each process its own RUN offset (ADR 0007's sampling rule) — a fresh fixture process IS a fresh observation window, so price fields cannot and must not reproduce |

## 2026-11-20 · v0.8.0 — the DB holds every forever item's description copy, and takes edits by hand

### What forced the milestone

The user's milestone asks for something the last seven releases never promised: a DB that holds
**every forever item with its vendor price, its icon and its description copy**, and that can be
**edited by hand**. Two of the three were already reachable — the vendor price rides the detail
record, the icon bytes live in the DB since v0.6.0 — and the third is not behind the door the
crawl already reads. That is not a nicety to design around: it is a fact about the wire, and it
decides the shape of the release. And the manual half is forced by the crawl's own shape: the list
stage UPSERTs every id upstream still lists, so a row a user HARD-deleted would be resurrected by
the next list pass. Both halves were measured before anything was written
([ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md) carries the full record).

### The grounding, measured 2026-11-20 (live probes, then read back off the user's own file)

```
# the record the crawl ALREADY reads (raw_json of id 25, read back off the user's own file):
attribution, icon_url, id, item_level, item_subtype, item_type, links, name,
price, quality, required_level, unique, vendor_sell, weapon
    -> NO description, NO html, NO `fields`: a description column on the detail door
       would have invented a field upstream never answers

# the door the description lives behind (one live request, UA wow-goldtool 0.8.0):
GET https://thewowdb.com/api/v1/tooltip/forever/item/25
  -> 200 application/json 3060 b
  etag W/"2b111cbcd3bbb4d7078f6d05"
  cache-control: public, max-age=300, s-maxage=900, stale-while-revalidate=3600
  x-ratelimit-limit: 120   x-ratelimit-remaining: 62
  x-wgf-build: wow_classic_beta-1.60.1.70532
  top keys: attribution, build, fields, html, icon_url, id, kind, links, name, quality, version
  `html` = 1404 b of upstream's OWN card markup: <div class="twdb-card twdb-k-item">… its own
           <img class="twdb-icon" src="https://render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg">,
           its own wording ("Item Level 2", "Sell Price: 7c")
  `fields` = 26 keys mirroring the detail record's own column set (armor, binding, build, classes,
             effects, faction_prices, flavor, … sockets, socket_bonus, stats, unique, vendor_sell,
             version, version_label, weapon)
  links.api = the door itself, links.html = https://thewowdb.com/wow-forever/item/worn-shortsword-25/

# an id the door has not (one live request):
GET https://thewowdb.com/api/v1/tooltip/forever/item/999999999999
  -> 404 application/json 79 b, no ETag offered, x-ratelimit-remaining: 119
  body: {"error":"not_found","message":"No item 999999999999 in forever.","status":404}
        (upstream's OWN words — the stage reads the STATUS and gives that record its own
         checked third state, exactly as v0.5.3 gave a dropped detail record)

# the user's own file, read-only: 19 258 rows, MAX(id) below the manual block = 288 340
```

Three measured facts decide the shape, none of them invented: the description is not on the record
the crawl reads, so a FOURTH door is a fact (≈ 19 258 tooltip requests ≈ 2.7 h at the measured 2/s
— the same order as the detail and icon stages, so it belongs to the crawl, budgeted/resumable/
cancellable, and not to a live ask); the door revalidates (`ETag` + `max-age=300`), so the stage
stores its own `tooltip_etag` and a targeted re-ask costs a `304` and writes nothing; and its `404`
answers upstream's own JSON, so the third state gets ADR 0006's treatment. The reserved manual id
block at `900 000 000` is provably clear of every id the walk can bring back (measured max upstream
id on the user's file: `288 340`), and a tombstone — not a hard delete — is the only shape that
survives the next list pass.

### The decision

[ADR 0010](./DECISIONS/0010-manual-layer-and-tooltip-copy.md) puts the description copy in as **stage
4 of a pull** with the plumbing every stage already has (its own pending rule, its own `sync_state`
row, the shared `RateBudget`, cancel at chunk granularity, `429` retried IN PLACE so a rate limit
never aborts a stage) and puts the user's edits in as **one rule: overrides merge LAST**. The
rejected alternatives are recorded there in full — inline-in-stage-2 (the reuse path would leave
the `2 919` records the user already fetched permanently description-less, a stage whose output
depends on which build fetched the record), copying the whole tooltip body (two sources of truth
for one record), rendering upstream's markup (that lets upstream write the card), a hard delete
(resurrected by the next UPSERT), a user-picked id (it can collide with a FUTURE forever record),
a separate `manual` table (a second place "the user already said" could hide), editing the auction
columns (a permanent lie about CURRENT prices, for a gold tool), and making a manual row crawlable
(upstream has no such id: the queue would ask a record that cannot exist and stamp "gone" on the
user's own row).

### Schema 4, and the migration that had to chain

`SCHEMA_VERSION 3 → 4`, eight additive columns (`tooltip_html`, `tooltip_etag`, `tooltip_at`,
`tooltip_missing_at` + `manual_at`, `manual_overrides`, `edited_at`, `deleted_at`), and `SQL_MIGRATE_TO_V4`
is eight `ALTER`s so the chain is now v1→v2→v3→v4 — the user's own 19 258-row catalogue comes up
in ONE call, and ALTER appends, so a migrated file's column order differs from a fresh file's and
every reader names columns explicitly.

The direction that cannot be guessed at was MEASURED at this release: the moment a v0.8.0 build
first opened the user's file it moved 3→4, and the pinned v0.7.0 crawl that was still walking it
died with `database on disk uses schema 4, this build wants 3` — which is why the real pull-all had
to be re-pinned on a v0.8.0 snapshot mid-release. A file newer than this build is still refused;
that is the honest answer, not a guess.

### What the audits caught in the code

* **a fresh DB was writing the wrong shape.** `SCHEMA_SQL` lost a comma INSIDE the Rust string —
  `"… detail_missing_at INTEGER, tooltip_missing_at INTEGER"` glued onto `"icon_blob BLOB"` — so
  SQLite read `INTEGERicon_blob` as a type name: the `CREATE` **succeeded**, and BOTH columns came
  out ABSENT. The chain migration masked it (its `ALTER`s name the columns), the fresh write did
  not: a fresh file had no icon bytes and no tooltip third state, and every reader naming them
  would have died. The witness is `sqlite3 -readonly .schema item` read back off a fresh DB —
  `tooltip_missing_at` present at slot 44, `icon_blob BLOB` at 45 — and `fix(db)` landed the comma.
  The family is now named three times in this file: a schema string that BUILDS is not a schema that
  WORKS.
* **the coverage answer's read slots moved.** Five new counts pushed the price columns from slots
  7-9 to 12-14; rusqlite reads rows by position, so an un-guarded read would have averaged the
  wrong numbers into `stats`. The expected-count guard in the audits is what catches that drift,
  and the CLI, the UI and the audits read ONE answer.
* **the card was asked for a row upstream cannot know.** `item 900000000 --cached` answered the v0.7.0
  `not-found` — "fetch it with `sync --stage detail`" — and every pending queue then REFUSED that
  exact ask. The promise was a lie in one direction, so the row's own columns answer it instead
  (`column_card`/`SQL_COLUMNS_CARD`, 14 columns), and the live door short-circuits BEFORE building a
  client: ZERO requests, witnessed as ZERO asks for 900000000 in the fixture log before and after
  all three queue stages ran.
* **the fresh schema's comma fix had to land before CI grew the steps.** Chronology this release got
  right, and v0.7.0 got wrong: the fixture grounds the tooltip door (`test(fixture)` 413e472) and
  the schema fix (`fix(db)` 13cd1e6) land BEFORE the six new CI steps (`chore(ci)` b1df47a), so the
  mirror stayed green at the commit that grew the sequence.

### What the release grew

Eighteen commits between `944b510` and the tag: `feat(db)` schema 4 + chained migration → the manual
layer (overrides merge last, CRUD writers, tombstone) → the derived search columns in the edit door
→ `feat(cli)` the four CRUD verbs and the usage line → `feat(stats)` the pending queues refuse
tombstones and manual rows → `feat(query)` `visible_clause()` as the FIRST WHERE clause → `feat(stats)`
coverage grows five counts → `feat(wowdb,models,db)` the tooltip door's URL, its `Tooltip` mirror and
its writers → `feat(stats)` the tooltip queue → `feat(sync,cli)` `sync_tooltips` as stage 4 →
`test(fixture)` the door's measured shape → `fix(db)` the fresh schema's comma → `chore(ci)` twenty-one
mirror steps → `feat(db,cli)` the card carries the copy/marks/refuses tombstones → the manual row's
card answers from its own columns → `feat(ui)` the status grid + the fourth pull-all leg → `feat(ui)`
the write panel + the card's tooltip/marks/tombstone note → `chore` the version banner rides 0.8.0
(both `Cargo.toml` and the pinned `Cargo.lock`, the v0.7.0 precedent).

### The numbers, not adjectives (every audit re-run against the shipped tree)

| audit | expectation | observed |
| ----- | ----------- | -------- |
| CI mirror (the committed twenty-one steps, fixture process, fresh DB, budget 8) | green with the fourth stage and the four manual doors inside | **21/21 steps exit 0**; the fourth stage budgeted `requests 5, items 3, written 3, missing 1, rate_limited 1, complete true` — asked ids `25, 40, 60, 12809`, and `12809` is the checked third state — then drained `requests 0, complete true`; the three manual answers are their own: `set-item 25 vendor_sell=606` → `{"id": 25, "field": "vendor_sell", "value": "606", "edited_at": 1795218573}`, `add-item "name=CI Test Mug" quality=Rare` → `{"id": 900000000, "manual_at": …, "fields": ["name","quality"]}`, `delete-item 60` → `{"deleted": [60], "at": …}`, `undelete-item 60` → `{"restored": [60]}`; `stats` after the mirror `items 6, named 6, priced 4, detailed 4, missing 1, icons 3, icon_missing 1, tooltips 3, tooltip_missing 1, edited 2, manual 1, deleted 0, history 4`, min/max/avg `18136`; the DB ends `6\|4\|3\|1\|1\|0\|4` (rows\|detailed\|tooltips\|tooltip_missing\|manual\|tombstones\|samples); `sync_state` FOUR rows `complete` (`list page 2`, `detail 0/2/2`, `icons 0/4/4`, `tooltip 0/0/0`); fixture log **31 GETs** — detail 13, list 5, icons 5, tooltip 5, search 3 — with `4 × 429`, `9 × 304`, `3 × 404`, and **ZERO** asks for 900000000 |
| exit-code audit (**57 doors**) | the old doors byte-for-byte identical, the new families land where ADR 0004 says | the eighteen of v0.6.0 `0,0,0,0,4,4,0,2,0,2,2,3,3,3,0,0,0,0`; the seven of v0.7.0 `2,3,0,0,0,0,3`; the tooltip stage `0,0,0,2,3` (drained queue, targeted `--ids 12809` on the gone tooltip → **0** `requests 1, items 0, missing 1, complete false` and the DB says `tooltip_missing_at` set / `tooltip_html` NULL, a re-ask `--ids 25` → **0** `requests 1, items 1, written 0, unchanged 1` = the door's own ETag revalidation, `--stage tooltipx` → **2** `sync wants --stage list, detail, icons or tooltip, got tooltipx`, dead host → **3** naming the door `…/api/v1/tooltip/forever/item/25`); the manual layer `0,2,2,2,2,4,2,0,0` with the messages `an editable INTEGER field wants a plain number` / `nose is not editable (…)` / `set-item states exactly one FIELD=VALUE per ask` / `no item 999999999 in the DB (add-item creates one)` / `add-item wants at least a name (name=...)`, and BOTH card doors of the made row → **0**, 940 b, `status null`, `detail_synced_at null`, name `Probe Mug`, quality `Rare`; three queue doors after the create → `0,0,0` with `requests 0` each and the fixture's ask count for 900000000 still **0**; the edit doors `0,0,0,0` — `set-item 40 name=ProbeRenamed` then `search ProbeRenamed` → **0** `total 1`, and the DB says `name` "ProbeRenamed" with `name_lc` "proberenamed" in ONE write; the tombstone five `0,4,4,4,4` (bare search total **6 → 5** across `delete-item 60`, double-delete → **4** `item 60 is already tombstoned`, `item 60 --cached` → **4**, `item 60` live → **4** while the fixture's ask count for `forever/item/60` stays **2** — refused BEFORE any request goes out — and `set-item 60 …` → **4** `… (undelete-item first)`); the restore doors `0,0,4,0` (`{"restored": [60]}`, again → **4** `item 60 is not tombstoned`, search total back to **6**, `stats` `deleted 1` → `deleted 0`); and one fresh-write door against a SECOND fixture process → **0** `requests 2, written 1, rate_limited 1`: the row KEEPS the user's `vendor_sell` 606 while the crawl-owned `price_min` moves 15528 → 19913, `tooltip_html` stays in the row, and `price_history` grows 4 → 5 with exactly one new sample (`seq 5, item 25, 19913`) — ADR 0007's rule holds under the merge. No DB-only door reached **3**: the manual verbs send no request, so they cannot be rate limited |
| GUI launch audit (scratch DB seeded by a fixture crawl: 5 rows, 4 samples) | the shipped page boots FOUR verbs; the new panel fires nothing itself | run 1: exactly `status, watch, stats, status` and **ZERO** requests to the fixture; temp probe (the four write verbs plus the made row's LIVE card, rebuilt into the page then reverted): the ipc log grows exactly `add-item (2 words), set-item (2 words), delete-item (1 words), undelete-item (1 words), item (1 words)` and the fixture saw **0** requests during that run — the manual row's card asked nothing — the DB ends 6 rows, manual 1, edited 2, deleted 0, samples 4, the made row `id 900000000, manual_at …, name "GUI Probe Mug", quality "Rare"`; run 3 re-audits the clean page after the revert (four verbs again, `git status --porcelain ui/` clean) |
| the schema chain on a real file | a fresh DB and a migrated one answer the SAME columns | a fresh DB's `status` writes schema `4`, and `.schema item` names `tooltip_missing_at` (slot 44) and `icon_blob BLOB` (45); a genuine schema-3 file migrated by this build lands at `schema_version` 4 with `icon_blob` present and its rows intact — the comma bug above is what the fresh half of that row caught |
| `cargo build` at the tag | warnings are the plan, not surprises | **0 errors**, FOUR warnings: `catalogue`, `open_db`, `pending_plan`'s private `PendingSql` and `Tooltip`'s unread fields (the measured key list rides the model, only `html` is read) — the set grew by exactly one, ADR 0009's rule; plus the hardlink notice when the DB-path doc recompiles on this filesystem |
| syntax of the three embedded UI files | they parse before the build embeds them | `node --check ui/app.js` parses (v22.23.5); `ui/index.html` tag-balanced (7 sections, 17 divs, 4 selects, 16 buttons); `ui/app.css` braces 80/80 |
| post-tag verification (fresh ports 9111 and 9102..9109 plus the second window 9099, fresh DBs) | the recorded numbers reproduce, or explain themselves | mirror **21/21 exit 0** with the identical counts (31 GETs, `4 × 429`, `9 × 304`, `3 × 404`, **0** asks for 900000000, DB `6\|4\|3\|1\|1\|0\|4`, four `sync_state` rows `complete`); exit audit **57 doors** with the SAME code line, card 940 b on both doors, fresh write `requests 2, written 1, rate_limited 1` keeping 606 and growing the ledger 4 → 5. Only the price figures moved — min/max/avg 18136 → 20814 and 15 528 → 17 923 — because the fixture gives every process its own `RUN` offset (ADR 0007's rule: a fresh process is a fresh observation window), so a re-run cannot reproduce a price and is not meant to |
| the user's own DB | never written by an audit; the authorized crawl walks it | read-only (`sqlite3 -readonly`) at the tag: schema `4`, `19 258` rows, max upstream id `288 340`, `tooltip_at` set on **0**, `detail_synced_at` set on `2 919` — the tooltip pass cannot start before the tag, because the pinned crawl snapshot has no fourth stage; the authorized real pull-all (detail → icons) kept walking on a pinned v0.8.0 snapshot, and the tooltip pass runs as a THIRD crawl pass with the released binary after the tag |

The audits themselves never touched the user's file: every run pointed at a scratch DB through
`WGT_DB` and the file was read hashed / `sqlite3 -readonly`. The ONE authorized write to it is the
real pull-all the user asked for, and the schema migration 3→4 that any v0.8.0 build does on first
open (measured consequence above: the pinned v0.7.0 crawl died refusing it).

---

## 2026-11-20 · v0.7.0 — the live-search door: one human ask, one request

### What forced the milestone

v0.6.0 shipped three names compiled but never used — `search_url`, `SearchResponse`,
`ApiResponse.remaining`: the client already built the URL, the mirror already shaped the
grouped answer (`groups` as a `Value`), the response struct already carried the burst
counter — and none had a consumer, because the door docs/API.md had promised ("we use it
for the live search panel") was never opened. Six milestones of dead code are a promise the
next release makes to itself; opening the door is this milestone. WHAT it does is decided
only by measurement — inventing a shape for a door nobody had opened is not our call to make.

### The grounding, measured 2026-11-20 (live, one request each)

Three live probes (UA `wow-goldtool …; data source: thewowdb.com`), verbatim answers:

```
GET /api/v1/forever/search/?q=Worn%20Shortsword -> 200 application/json 1288 b
    etag W/"76db61a78b8398bc089262d2"
    x-ratelimit-limit: 120  x-ratelimit-remaining: 119
    access-control-expose-headers: ETag, Retry-After, X-RateLimit-Limit, X-RateLimit-Remaining, X-WGF-Build
    top keys: attribution, groups, q, version ("forever"); q echoes DECODED "Worn Shortsword"
    groups exactly item/npc/quest/sound/spell/zone, each { has_more, more, results }
    the one item hit's keys: id, item_level, kind, links{api,html,tooltip}, name,
    quality, required_level, version
    every group's `more` is upstream's own paged list endpoint, e.g.
    https://thewowdb.com/api/v1/forever/item/?page=1&per_page=50&q=Worn+Shortsword
    — a space rides as `+` INSIDE the answer's own URLs, not %20

GET /api/v1/forever/search/?q=Ring              -> 200 application/json 7271 b
    item/quest/spell: has_more TRUE, five results each
    npc 4 / zone 2 / sound 0: has_more false

GET /api/v1/forever/search/?q=socket            -> 200, every group EMPTY
    (the names-only trap: socketed items exist in the catalogue, the door matches NAMES,
     so the word answers nothing — docs/API.md measures it)
```

`SearchResponse` already matched this shape EXACTLY — the mirror needed nothing. Three
measured facts decide the door: an empty answer is a SUCCESS (the names-only trap is a
word upstream never said, not a failure); every group's `has_more` + `more` means chasing
it would ask the paged per-group endpoint, one request per group per page — a human ask
becomes a crawl; and the answer is upstream's own JSON printed as it came (serde pas-through),
so the `+` rides verbatim while OUR request URL uses `%20` because the measured ask asked
with `%20` and answered 200 (a raw space is not a URL character).

### The decision

[ADR 0009](./DECISIONS/0009-live-search-door.md) keeps the door an ASK, not a sync: ONE
request like `item`, exit codes riding ADR 0004's contract (`2` no text, `3` network or
`429`-with-`Retry-After`, `0` even for all-empty groups, `4` only if the door itself answers
`404` for the shape), NO DB write and NO DB read. The load-bearing rejection is the write-from-
an-ask one: ADR 0007 made every fresh write an OBSERVATION in `price_history`, and an answer
that is not a record's own detail body would put ledger rows that never came from a detail
write. Chasing `more` was rejected on the measured `?q=Ring` answer (three-plus extra requests
and more per page: that is a crawl, not an ask; a later patch could take `--page` honestly);
retry-in-place was rejected because that dance belongs to a stage that owns a queue and a
budget, so the door mirrors the `item` door — and the fixture's `search` stage is UNGATED for
exactly this reason: the committed CI step must stay green.

### What the door grew

* `src/cli.rs`: verb `live-search` (usage line 30 between `item ID [--cached]` and the watch
  verbs, dispatch one line); the WHOLE verb tail joined with spaces is the query (`join_tail`,
  so `live-search Worn Shortsword` asks one name, not two words); one `get_json` with an empty
  stored ETag — the door revalidates nothing; payload `{status, retry_after, remaining, answer}`
  with `answer` the serde pas-through.
* `src/wowdb.rs`: `encode_query` — letters/digits/`-._` ride through, a space is `%20`,
  everything else its UTF-8 bytes as `%XX`: the shape the measured ask proves.
* `tests/fixture_api.py`: the grouped answer from the measured transcript — six groups each
  `{has_more, more, results}` by fixture-name substring matching (names only: `socket` answers
  empty), `+` inside its own `more` URLs, `version "forever"`, `etag W/"fixture-search-etag"` —
  AND the `429` gate no longer names the `search` stage.
* `ui/index.html` + `ui/app.js`: a live-search card, one field, one `Ask once` button, a
  `group · id · name` mini table, a `view upstream` link per hit at each hit's own `links.html`,
  and a note printing `remaining` (honestly `null` when the host sends no burst header — the
  fixture does not on this stage), the names-only fact, and whether any group said `has_more`.
  The wiring grew twice and got deduped: a second `btn-live` listener would have made one
  click two asks, and one request per click is the whole promise of the panel.
* CI grew the smoke sequence to fifteen steps: the door's one-word ask, the two-word ask
  (riding `%20`, the fixture's own GET log is the witness), and the names-only trap
  (`tooltip` answers all-empty groups — the step passes at exit `0`).
* The chain order deserves honesty: `chore(ci)` landed the door's steps BEFORE the fixture
  grounds them — between those two commits the v0.2.0 gate answered the door's first request
  `429` (the gate named every stage) and the stub shape was the v0.2.0 placeholder, so that
  intermediate sequence was red. Every commit in this series only guarantees a build, never
  a green CI run mid-series; the mirror replay below runs the committed fifteen steps against
  the committed fixture, both landed.
* `Cargo.toml` and the `User-Agent` banner moved 0.6.0 → 0.7.0 (the UA names both the version
  and the source, per the developer terms).

### The numbers, not adjectives (every audit re-run against the shipped tree)

| audit | expectation | observed |
| ----- | ----------- | -------- |
| CI mirror (fixture process A, fresh DB, budget 8: the committed fifteen steps + the v0.5.3 gone-id door + a drained icon queue) | green with the door inside; the door touched no DB state | fifteen steps exit `0`; the icon stage's numbers unchanged `requests 5, items 4, written 3, missing 1, rate_limited 1, complete true`; `stats` after CI `items 5, named 5, detailed 4, missing 1, icons 3, icon_missing 1, history 4` — the door wrote no sample; extra `item 127555` answers exit `4` and re-stamps once, the crawl after it `requests 0, complete true`; drained icons `requests 0`; fixture A log **28 GETs, 3 of them the door** (`sword`, `Worn%20Shortsword` — the two-word ask rides `%20` on the wire, the answer echoes `q` decoded — and `tooltip` with its all-empty 200), `3 × 429`, `4 × 404`; second process (fresh window): `detail --ids 25` `requests 2, written 1, rate_limited 1` grows the ledger 4 → **5** (`seq 5, item 25`), its log 3 GETs `1 × 429` |
| exit-code audit (twenty-seven doors) | the eighteen of v0.6.0 stay identical, the new verb adds its doors | old eighteen `0,0,0,0,4,4,0,2,0,2,2,3,3,3,0,0,0,0`, byte-for-byte v0.6.0's audit; new: `live-search` alone → **2** (`live-search wants text, for example: live-search Worn Shortsword`), dead-host ask → **3** `error sending request for url (http://127.0.0.1:9974/api/v1/forever/search/?q=ring)` — the URL in the message proves the verb's own URL building, two-word on a dead host → **3** with the percent-shape `…q=Worn%20Shortsword` in the message; all-empty groups → **0**, two grounded fixture asks → **0** `0` (the answer's `more` rides the `+` form `…q=Worn+Shortsword`), the no-DB door `live-search sword --db …/exit70-nodb-dir/none.sqlite` → **0** with the answer while `exit70-nodb-dir` is NEVER created — opening that path would have answered `4`; and the retry branch: a SCRATCH copy gates `search` once (the committed fixture untouched, `git status` clean after) → **3** `rate limited by the API: the API asked us to back off`, the scratch log one `429` — witnessed WITHOUT touching the committed one; the fresh-window write door → `0` grew the ledger |
| this build's own live asks | upstream's answer as it came, exit 0 | `live-search "Worn Shortsword"`: `status 200, remaining 119`, item group **1 hit** `has_more false`; `live-search Ring`: **5 hits** `has_more true` with `more` = `https://thewowdb.com/api/v1/forever/item/?page=1&per_page=50&q=Ring` — the `+` rides verbatim; `live-search zzzqqqnothere` → **exit 0**, all six groups `results: []` |
| GUI launch audit (the shipped page) | boot stays four verbs, nothing crawls | exactly `status, watch, stats, status` and NO `live-search` line — the panel moves only on its own button; scratch DB `0` items, `0` samples, empty `sync_state`, schema `3` |
| GUI live probe (temporary, rebuilt into the page, reverted right after) | the panel's ask reaches the SAME ipc guard | a 5th line `gui: ipc verb live-search (1 words)` + one fixture GET `/search/?q=sword … 200`; probe then reverted with `git checkout -- ui/app.js` (`grep probe` answers `0`), rebuilt clean, and the four-verb boot above re-audited against the reverted tree |
| `cargo build` at the tag | warnings are the plan, not surprises | exactly THREE dead-code warnings — `catalogue`, `open_db`, `pending_plan`'s private `PendingSql` — plus the hardlink notice: the set SHRANK from v0.6.0's six, the door consumed the three names it was owed |
| the user's own DB | never written by any of this | audited read-only before and after every run: hash `7442814b…c3d3b8`, mtime `2026-11-19 21:42:54`, `19 258` rows — identical throughout |

The user's DB (`HOME/.local/share/wow-goldtool/wow-goldtool.sqlite`) was never written by
any of this: every run above pointed at a scratch file through `WGT_DB`, and the file
itself was only ever read hashed / `sqlite3 -readonly`.

---

## 2026-11-20 · v0.6.0 — price history, local icons and the card that shows both

### What forced the milestone

Two questions the schema on disk cannot answer, both asked by the app's own purpose:

* **A trend.** The `item` row keeps ONE price per item (`price_min` …), overwritten by the
  next write. "What does this cost?" answers; "is it getting cheaper?" has no answer in the
  file, and nothing upstream does either: a `forever/item` payload carries the CURRENT
  auction-house state, and a record that did not move answers `304` with no body at all —
  a revalidation says "nothing moved", not "here is a price". Any growth of the ledger had
  to come from **our** fresh observations, which makes the sampling rule a cost question
  about the one request per record the detail stage already pays.
* **Icons offline.** v0.5.x's card deliberately never hotlinked `icon_url` (the README and
  the card notes say why). So the bytes have to come from the DB, and where they live and
  how they arrive is a decision, not a detail.

### The grounding, measured 2026-11-20

The live catalogue answers exactly two `icon_url` shapes on answered records, and both
serve JPEG bytes; the same records also answer an EMPTY `icon_url` sometimes:

```
https://render.worldofwarcraft.com/<region>/icons/56/<name>.jpg   (/us/, /forever-eu/, /forever-us/)
https://thewowdb.com/assets/icons/<name>.jpg
.../us/icons/56/inv_mace_03.jpg      -> 200 image/jpeg 1945 b
.../forever-eu/icons/56/133749.jpg  -> 200 image/jpeg 2265 b
thewowdb.com/assets/icons/inv_axe_11.jpg -> 200 image/jpeg 1936 b
"icon_url": ""                       -> 4 of the 29 detailed rows of the scratch catalogue
                                        (Peace Tea, Craftsman's Writ, Scroll of Imbue
                                         Iceknife, Lodestone)
```

An ABSENT icon path answers differently per host — measured, and it decided the third
state's rule: `thewowdb.com/assets/icons/<bogus>.jpg` → **404 `text/html`** (the site's own
"Page Not Found" body: an answer about the file), while
`render.worldofwarcraft.com/us/icons/56/<bogus>.jpg` → **403 `application/xml`**
(243 b of S3-style `AccessDenied`: an answer about a CDN, not about an icon). `icon_missing_at`
is written for the `404` and the `403` stays a stage failure (exit `3`, resumable, nothing
marked, icon pending) — a fact the upstream never stated stays unstated. The card's `stats`
pairs ride the same discipline: across the scratch catalogue's 29 detailed rows, `stats`
carried values on 2 (`Cold Basilisk Eye`: `[{"name":"Frost Resistance","value":9}]`,
`Tough Scorpid Bracers`), and `sockets`/`socket_bonus`/`classes`/`effects` were empty on
**all** 29 — so `stats` renders as pairs and the others get a raw row only when they carry
something. No shape was invented anywhere.

### The two decisions

[ADR 0007](./DECISIONS/0007-price-sampling.md) put the ledger in a second table and sampled
**once per fresh detail write**, prices present: a `304`, a list write, a check-mark, a
re-insert write nothing, so the mirror's `written 0, unchanged 2` steps must leave the count
untouched and a second crawl session must grow it exactly by its fresh writes. Five
alternatives rejected there: sampling `304`s (the ledger becomes a function of click count),
a dedicated sampling pass (doubles the request cost and spends the budget on records the
write visit could not refresh), a "numbers moved" rule (hides repeated observations and lies
about flat markets), a timer binary (ADR 0004: the headless CLI is the integration path; no
second binary has ever shipped), and stuffing samples into the `item` row (the point of
`docs/DATABASE.md` is that the UI reads SQLite). The FK probe measured the guard's shape:
SQLite 3.54.1 **accepts and does not enforce** an FK clause on `INSERT` — an
`INSERT INTO price_history` with `item_id = 999` and no such item row succeeded — so no
clause ships, and the guard is the Rust order inside `upsert_detail`.

[ADR 0008](./DECISIONS/0008-icon-storage-third-stage.md) made the icon a **third crawl
stage** (`sync --stage icons`) with the detail stage's whole plumbing — own pending queue,
budget, cancel, resume, third state — copying each answered record's own icon file into an
`icon_blob BLOB` exactly once. The load-bearing rejection: fetching icons INLINE inside the
detail stage would double a chunk's requests AND leave every record fetched before this
build permanently icon-less, because those details now answer `304` and a `304` re-fetches
nothing. A file sidecar next to the DB got the same treatment: the DB is the one artifact
the user manages (the README names its path), and a second place for bytes is a second
place for "gone upstream" to hide. No icon ETag is stored anywhere, because the pending
rule excludes copied icons and the copy path is asked once.

### Schema 3, and the migration that chains v1→v2→v3

`SCHEMA_VERSION 2 → 3`: three `ALTER`s append `icon_blob`/`icon_at`/`icon_missing_at`, a
`CREATE` adds `price_history` plus its index, and the stamp moves in one call. The file on
the user's disk says `schema_version = 1`, so `check_schema` now **chains**: v1 first gets
v0.5.3's `detail_missing_at`, then both v1 and v2 get the v0.6.0 batch. Refusing either
would have thrown away a 19 258-row catalogue on the excuse of a schema bump; a file
**newer** than this build is still refused (the audit's schema-9 file answers exit `4` with
`database on disk uses schema 9, this build wants 3`). A migrated file's column ORDER
differs from a fresh file's (`ALTER` appends), and every reader in the code names columns
explicitly, so nothing reads positions.

### What the audits caught in the code

Two fixes landed because the audits read what the code actually did, not what the notes
claimed:

* `SCHEMA_SQL`'s v3 rewrite had dropped `CREATE INDEX item__name_lc` — a migrated file
  keeps its copy (the user's file has `item__detail item__name_lc item__price item__rank`
  and the migration re-issues only `price_history__item`), but a FRESH v3 file would have
  had none, and local search drives `name_lc LIKE` on every query the UI issues. Restored
  in the fresh `CREATE` list, deliberately NOT re-issued by the migration (a migrated file
  already has that index, and a duplicate `CREATE INDEX` is a hard parse error).
* `raw_json_of` inferred a plain `String` from the row cell, so a NULL `raw_json` answered
  as rusqlite's type error instead of "no stored copy". The v0.6.0 mirror was the first run
  to exercise `item <checked-gone id> --cached`, and it printed `Invalid column type Null
  at index: 0` — the same exit `4` as not-found by the contract's own sharing, but the wrong
  answer. Bound `Option<String>` like every stamp reader in the file, so the honest door
  prints `item 127555 has no stored copy: fetch it with sync --stage detail --ids 127555`.
* The icon queue's pending rule gained the NON-EMPTY url terms from the start: under a
  NULL-only rule the four measured empty-`icon_url` records would have kept pending forever
  (the stage fetches nothing for them, marks nothing), which is v0.5.2's price trap again.

### The fixture grew the same grounding

`tests/fixture_api.py` now embeds the two REAL JPEGs the grounding above measured (the
`inv_mace_03.jpg` and `inv_axe_11.jpg` bytes, fetched once and committed base64'd — CI
never asks a host for pixels), answers `/icon/<name>.jpg` on its own server, and makes one
record's `icon_url` name a FILE it answers `404` — the icon stage's third state inside CI.
Detail prices carry a per-process `RUN` offset (the ETag salt moves with it): one fixture
process is one observation window, so a fresh write in a SECOND process grows the ledger by
exactly one sample while a re-ask inside one process still writes none — ADR 0007's rule
becomes a measurable CI claim instead of a promise.

### CI grew the third stage

The committed smoke sequence gains the icons step (twelve steps now) before the card and
stats reads, and the header comment says what each new step proves.

### The numbers, not adjectives (every audit re-run against the shipped tree)

| audit | expectation | observed |
| ----- | ----------- | -------- |
| CI mirror (fixture process A, fresh DB, budget 8, twelve steps) | copy lands, gone file checked once, ledger counts fresh writes | icons `requests 5, items 4, written 3, missing 1, rate_limited 1, complete true`, message `icon crawl copied 3 icons; 1 answered 404 for the icon file and are checked as no-icon-upstream`; samples seq 1..4; the three reuse steps each `written 0, unchanged 2` and the count stayed 4 |
| fixture process B (new observation window) | one fresh write, one new sample | `detail ids 25`: `requests 2, written 1, rate_limited 1` → ledger 4 → **5**; the card answers `history` with the old 20271 and the new 15311; a targeted `--ids 25` icon visit re-copied (a human ask, two requests, pending queue unaffected) |
| icons run on a drained queue | asks nothing | `requests 0, complete true`, `icons stage: 0 icons copied (0 requests)` |
| DB witnesses after the twelve | three states visible in rows | 5 items, 4 detailed, 1 checked-gone; `icon_at` on ids 25/40/12809 with `1945 b` blobs (magic `0xff 0xd8 0xff … 0xff 0xd9`, the grounded JPEG's own ends), `icon_missing_at` on id 60; both queues pending **0**; `sync_state` three rows `complete` |
| exit-code contract (18 doors) | the new doors land on the codes | `usage 2 · 2 · 2` (`sync --stage detailx` quotes `detailx`; `item` without id; unknown verb), `3` on a dead host for detail/list/icons (the icons line names `http://127.0.0.1:9923/icon/inv_mace_03.jpg`, which also proves `icon_target`'s base-relative resolution), `4` twice (`no item 127555 upstream`, and the cached door's honest text), `0` for status/search/card/idle stages; a bare `sync` runs the list stage (v0.2.0 default, unchanged by this release) |
| migration, user's own v1 file (copied; hashed read-only before/after) | the chain lands, the file on disk untouched | v1 → `schema_version 3`, `19 258` rows, `7` detailed, `price_history` 0, `sync_state` list row `stopped … page 170` intact, index set gains only `price_history__item`; second `status` re-reads 3 without re-migrating; hash `7442814b…` and mtime `2026-11-19 21:42:54` identical before/after |
| migration of a v2 scratch catalogue | same legs, one step | 19 257 rows, 29 detailed, 57 checked-gone, `price_history` 0, `sync_state` intact |
| refusal door kept closed | a newer file still refuses | schema-9 scratch file → exit `4`, `database on disk uses schema 9, this build wants 3` |
| FK probe (SQLite 3.54.1) | the guard is Rust's order, not a clause | `item_id INTEGER REFERENCES item` accepted, `INSERT … item_id = 999` with no such row **succeeded** |
| GUI launch audit, shipped binary (probe removed) | nothing crawls by itself | exactly four verbs `status/watch/stats/status`, zero fixture hits, fresh DB gains schema 3 and nothing else |
| GUI pull-all launch audit (boot probe, fresh fixture + DB) | one click chains three stages | three `sync` verbs with ticker `status` polls interleaved; fixture log 14 requests, **3 × 429** (one per stage) and **2 × 404** (the removed record + the gone icon file); DB 3 icon copies, 1 checked no-icon, ledger 4, pending 0 both queues, `sync_state` three rows `complete`; probe removed and the tree rebuilt before the docs commit |
| `cargo build` at the tag | warnings are the plan, not surprises | exactly the six dead-code warnings (`search_url`, `SearchResponse`, `catalogue`, `open_db`, `ApiResponse.remaining`, `PendingSql`'s privacy) plus the incremental-cache hardlink notice on this filesystem |

The user's DB (`HOME/.local/share/wow-goldtool/wow-goldtool.sqlite`, 19 258 rows) was
never written by any of this: every run above pointed at a scratch file through `WGT_DB`,
and the file itself was read hashed/`sqlite3 -readonly` only.

---

## 2026-11-20 · v0.5.3 — patch: a record the upstream dropped is a state, and the crawl survives it

### The measurement, not a hypothesis

The v0.5.2 build run against the **live** API on a scratch DB whose list walk had
already stored 3000 ids:

```
$ cargo run -- sync --stage detail --budget 3
fetching item details: this is the slow stage, about two records per second
item 41 answered 404
$ echo $?
3
```

The detail stage read every non-200/304/429 status as `HttpKind::Failed`, and a failed
record is a network error: the run aborts, exits `3`, and leaves the rest of the queue
untouched. The ids that answer that way are not exotic — the live catalogue reports
`build: wow_classic_beta-1.60.1.70532`, and `41` is `OLDRecruit's Belt`, `42`
`OLDSquire's Belt`, `46` `OLDFootpad's Belt`; the same `OLD…` / `Deprecated …` /
`… (TEST)` name shapes sit inside the user's 19 258 stored rows. The upstream answers
with a body, and the body is its own words:

```json
{"error":"not_found","message":"No item 5828 in forever.","status":404}
```

So the "Pull all info" flow shipped in v0.5.2 — one click that is supposed to retire
every pending record — **cannot finish on the real catalogue**: it dies at the first
removed record. The fixture never showed this because it answers every id. Fixing the
crawler has to land **before** v0.6.0's price history and icon copies, which are the
things that read what a finished crawl produced.

### The decision: a third state, not a fourth retry

[ADR 0006](./DECISIONS/0006-checked-missing-third-state.md) is the record; the shape is
one column and one rule. `classify` grows `HttpKind::Missing`, the crawl marks the id
(`SQL_MARK_MISSING`, guarded by `SQL_ROW_OF_ID`) and counts it in the report (`missing`),
then walks on. `SQL_PENDING` becomes `detail_synced_at IS NULL AND detail_missing_at IS
NULL`, so a removed record costs **one request per crawl** rather than one per run, and
"nothing pending" stays literally true. Four alternatives were rejected in the ADR:
staying fatal (the real DB would never finish), skipping silently (every later run re-asks
the same dead ids and the crawl spends budget on records that cannot arrive), a staleness
cadence ("nothing pending" becomes a lie; nothing measured says a removed record returns),
folding the fix into v0.6.0 (ships a crawl that cannot finish next to the features that
depend on it).

The mark never invents a row: `SQL_MARK_MISSING` is an `INSERT … ON CONFLICT` in the
codebase's idiom, so the existence check in front of it is what stops an un-listed id from
becoming a name-less phantom record. And the stage-2 upsert now writes
`detail_missing_at = NULL` — a record that answered is not a missing record, so the two
stamps cannot both be set.

### The schema step that was not optional

The column is a schema change, and `check_schema`'s honest refusal would have refused the
user's file. Refusing is right for a file this build cannot understand, but here it would
mean asking a human to delete a 19 258-row catalogue because one column was added. So
`SCHEMA_VERSION` moves `1 → 2` **with** a migration: when the file says `1`, `ALTER TABLE
item ADD COLUMN detail_missing_at INTEGER` runs in place and the stamp in `meta` moves in
the same call. A file **newer** than this build is still refused, because that direction
cannot be guessed at. Both fresh-DB stamps now write `format!("{}", SCHEMA_VERSION)` —
without that, a v2 file written by this build would have refused itself on the next open.

### Where the pending arithmetic got checked

Numbers, not adjectives. After the first live run on the migrated DB: `3000` listed =
`10` fetched + `3` checked-gone + `2987` pending, and the report read `requests 6, items
3, missing 3` with **exit 0**. A second run asked four ids and got `missing: 1` — the newly
fetched ids were `47, 48, 49`, i.e. the three already-marked ids were **not** re-asked, and
the new 404 is a fourth removed record, not a re-ask of an old one. That is the difference
between "keeps crawling" and "silently wastes budget", and the arithmetic proves it.

### The UI says what the third state is

The status card gained a row (`checked as gone upstream · N answered 404, not re-fetched`)
and the pull-all note gained its clause: the intro explains that a dropped record is
checked once and counted as gone, and the completion note names how many of the listed ids
were checked as gone, beside the honest sentence about NULL prices. The numbers come from
`stats`/`status` coverage, so the card and the CLI report read one aggregate.

### Audit

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| v1 scratch DB made by the v0.5.2 build (3000 rows) migrated by this build | rows survive, stamp moves | `status` exits `0`, `3000` rows, `schema_version 2`, `detail_missing_at` column present, the old `list` cursor (`stopped`, page 30) intact |
| fresh DB written by this build | does not refuse itself | `schema_version 2`, second open exits `0` |
| live detail run, budget 6 | a 404 does not abort | `requests 6, items 3, missing 3, complete false`, **exit 0**; DB `10` detailed, `3` missing, pending `2987 = 3000 − 10 − 3` |
| second live run, budget 4 | a marked id is not re-asked | `requests 4, items 3, missing 1`; newly fetched ids `47, 48, 49`; the three marked ids absent |
| `item 41` on the live API | the exit-code contract's answer for "no record" | exit `4`, message `no item 41 upstream: the forever catalogue has no record for it`, mark re-stamped |
| fixture mirror (virgin port, thirteen steps) | the third state inside the mirror, pacing intact | detail run `requests 6` (one forced `429` retried in place), `items 4, missing 1, complete true`, message ends `; 1 answered 404 and are checked as removed`; `item 127555` → **exit 4**; the crawl that follows `requests 0, complete true`; DB `5` items, `4` detailed, `1` missing, pending `0` |
| exit-code audit, seven probes | the new case lands on `4` | `2 · 2 · 3 · 4 · 4 · 4 · 0`, and `detail_missing_at` set for `127555` |
| committed CI sequence with the new fixture | still green | 11/11 `exit=0`, DB carries `1` missing |
| GUI pull-all launch audit (virgin port, temp boot probe) | the flow completes with a removed record inside it | verbs `status/watch/stats/status` then `sync` × 2 with the ticker's `status` polls interleaved, then the probe's `item`; fixture log: crawl's single `404` for `127555` plus the probe's targeted `404`; `sync_state` detail row `complete … 1 answered 404 and are checked as removed`; DB `4` detailed, `1` missing, pending `0`; probe removed afterwards |
| warnings on a clean build | the planned dead-code set only | `PendingSql` privacy, `SearchResponse`, `search_url`, `remaining`, `open_db`, `catalogue` plus the filesystem's hardlink notice — the three new SQL constants and both new helpers are used, so nothing new goes quiet |
| build from a cleared incremental cache | green, and it is what the notice says | the filesystem under the project (`/run/media/w/...`) cannot hard link, so the incremental cache came back with a corrupt dep-graph and rustc 1.94.0 ICEd in `setup_dep_graph`; `rm -rf target/debug/incremental` then a fresh `cargo build` exits `0` with the planned warnings. The workaround is environmental, not a code change: every audit below ran on that binary |

### Deliberately not done in this tag

No re-check cadence for removed ids ([ADR 0006](./DECISIONS/0006-checked-missing-third-state.md)
rejects it); no price-history table and no icon copies (those are v0.6.0 and they read what
a finished crawl produced); no index for the new pending filter. Also still open for v0.6.0:
the live API has only answered `[]` for `sockets`, `socket_bonus`, `classes` and `effects` on
every id probed so far (the scratch walks found no level-60 epic gear in the first 3000 ids,
and the live id space has shifted since the user's crawl — `40` is `Recruit's Boots` there,
`Mossborer` in the DB), while `stats` is grounded as
`[{"name":"Intellect","value":4},{"name":"Spirit","value":3}]`. The item card will render the
grounded shape and print the raw payload for the rest, rather than invent a shape.

---

## 2026-11-20 · v0.5.2 — patch: pending's real rule, the client-build stall, the one-click catalogue

### The ask

"instead of the 'sync pending details' make it into a pull all info… basically
run the 'Re-check online' for every item that doesn't have all its information
in the DB yet… one big run and then never have to touch it again." A button that
retires pending work is only honest if pending means *work that exists*.

### Pending: the rule had to carry the truth

`SQL_PENDING` asked `price_min IS NULL OR detail_synced_at IS NULL`. The first
clause was never a pending rule: `docs/DATABASE.md` distinguishes fetched-with-
NULL-price (the item exists, it is **not on the AH**) from never-fetched on
purpose, and `price_min` is read field-by-field out of an optional object, so a
fetched record can legitimately carry a NULL price. Keeping such an item pending
means the detail stage walks it again forever — the stage never terminates on the
real catalogue, and no pull-all flow can promise "nothing pending". The user's DB
(read-only query: 19258 items, 7 with details, 0 fetched-and-unpriced at this
hour) cannot expose the trap yet, because only 7 records have ever been fetched:
the clause is latent, and it becomes loud the moment a real catalogue item answers
without AH data. It is fixed **before** the flow that depends on it ships, not
after that flow has lied. `detail_synced_at` already encodes "has a detail record",
so pending is now that one predicate.

### The GUI never reached the API

Driving a full `runPullAll()` from the boot probe made the next failure loud: the
stage verb entered Rust and then nothing happened, and the fixture access log stayed
empty for the whole session. Markers at each verb step pinned the stop point: the
verb logged the DB-open marker and never reached the request site — it stalled inside
`build_client()`. The DB was never the lock: status polls completed normally between
those marker lines, and the page stayed interactive throughout.

Mechanism read from the crates: reqwest's blocking `Client::new` does not build a value
on the spot. `ClientHandle::new` (`blocking/client.rs:1424` spawns a thread named
`reqwest-internal-sync-runtime` that builds its **own** single-threaded tokio runtime
(`runtime::Builder::new_current_thread()` at `:1428`) and `rt.block_on`s the request loop
(`:1468`); the thread that asked for the client then parks while waiting for that
thread's startup handshake (`wait::timeout(spawn_rx, None)` at `:1476`), and that wait
polls the future and `thread::park()s` the caller (`blocking/wait.rs:58`). Headless that
is exactly the CLI's shape, and every CI mirror has always worked; in the GUI the verb
body sits on a tokio worker of tauri's async manager (v0.5.1's `async fn cli_call`), and
parking there while the client's private runtime has to start is the nested-runtime case:
measured as the stall at `build_client`, zero requests issued.

Fix: the command stays `async` — v0.5.1's wrapper decision is untouched, the IPC handler
still answers immediately and the page keeps ticking — and the verb body hops to a fresh
`std` thread, the runtime-free home the headless CLI proved good. `join` occupies one
tokio worker, which the webview never borrows. A verb that panics on that thread becomes
an `AppError` instead of leaking `Result`'s `String` payload, so the envelope stays the
one shape the UI knows. The helper sits at the crate root because that is where both doors
reach it without an import cycle.

### The flow itself

`btn-detail` becomes `btn-pull` ("Pull all info") and `runPullAll()` chains the two
stages to their ends: the list walk resumes from `sync_state` and runs to completion,
then the pending queue is drained to completion. Orchestration only — ADR 0005's one-
command door, the chunk budgets, the cancel flag and the DB's progress truth are
untouched. For the chain to know a stage finished, `runSync` returns an outcome
(`complete`/`cancelled`/`stalled`/`failed`/`already-running`): Cancel stops the stage in
flight and the flow stops with it rather than jumping ahead, because the next click
resumes the list from the stored cursor and the pending queue recomputes which details
remain. The same change re-orders the chunk's two checks (`complete` before `requests
=== 0`): a stage with nothing left to fetch answers `complete: true, requests: 0`, which
is a legal completion, not a stalled crawl — the old order called it "made no progress".

The completion note refuses to overpromise. "Nothing left to pull" means every listed
item has a detail record; a NULL price on a fetched item is *not on the AH*, not missing
work; and prices are live data, so later syncs still make sense — they revalidate cheaply
(a finished walk restarts at page 1 and every page answers `304` from the stored ETags).

### Audit

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| pull-all launch probe (shipped code + boot probe, fresh fixture server + fresh DB) | both stages crawl, page alive throughout | `sync` verb → two ticker `status` polls → second `sync` verb → three more polls |
| fixture access log for that run | one forced `429` per stage, retried in place, then payloads | page 1 `429` then `200`, page 2 `200`, item 25 `429` then `200`, items 40/60/12809/127555 `200` |
| scratch DB after the pull | everything fetched, nothing pending | 5 items, 5 detailed, **0 pending**, both `sync_state` rows `complete` |
| boot audit on the shipped binary (probe removed) | nothing crawls by itself | exactly four verbs `status/watch/stats/status`, zero fixture hits |
| CI mirror | eleven steps | 11/11 `exit=0`, and both stages report `rate_limited: 1` with the wait inside the run (`seconds: 1`) |
| exit-code audit | unchanged | `2 · 2 · 3 · 4 · 4 · 0` |
| stop → resume → resync | the cursor contract stays | `--budget 1` stops (`complete: false`), the next run reads `resumed_from: 2` without refetching page 1, and a finished walk restarts at page 1 with both pages `304` (`unchanged: 2`, `written: 0`) |
| clean build | warnings only | the dead-code set already planned for v0.6.0 (`PendingSql` privacy, `SearchResponse`, `search_url`, `remaining`, `open_db`, `catalogue`) plus the filesystem's hardlink notice |

### Deliberately not done in this tag

No engine change for pull-all: it is UI orchestration over the existing stages, so ADR
0005 and the CI mirrors stay untouched. No new progress channel — the rejected `emit`/
`event.listen` note in ADR 0005 still applies, and the ticker remains the mid-chunk
feedback. No price history, no icon copies, no item-card rendering of `sockets`/`stats`/
`classes` (that stays v0.6.0).

---

## 2026-11-19 · v0.5.1 — patch: the crawl no longer owns the page

### The report

During a sync list run the whole app was frozen: hovering a button stopped
highlighting, clicking anything took several minutes. The measurements agree
on the length: one GUI chunk is 3.5 s (fixture) / ~5 s (real) at the 2/s
pacing — and a 429 with a `Retry-After` sleep inside the chunk stretches it
to minutes. The "several minutes" is that wait, charged to the page rather
than to the crawl.

### The cause, read from the crate source

`#[tauri::command]`'s wrapper kind decides it. A bare `fn` gets the macro's
blocking body, and that body's last acts are `kind.block(result, resolver)`
(`tauri-macros/src/command/wrapper.rs:425`) then `return true` (`:428`): the
verb runs synchronously **inside** the webview's IPC handler — and that
handler is the page's message loop. So hit-testing (CSS hover), click
dispatch and any later IPC message can only run when the verb returns. A
hover that stays dead for exactly one chunk is that stall.

### The fix: the command itself

`cli_call` becomes `pub async fn`. `body_async` hands the future to
`respond_async_serialized` (`wrapper.rs:~372`): tauri's tokio-backed async
manager — tokio `rt-multi-thread` is a required dependency of tauri 2.x,
not a feature switch, so the manager exists in this build — and the IPC
handler returns immediately. Same argv, same dispatch in `cli.rs`, same
envelope: only the wrapper kind flips (sync → async). The crawl still runs
synchronously inside its own future (pacing blocks one tokio worker and
none of the webview); tokio logged no complaint about the blocking calls,
stderr stayed all `gui:` lines.

Cancel granularity deliberately unchanged (chunk ends, ADR 0005): this
release is about the page, not the crawl — and "always honour `Retry-After`"
is policy, so the wait still delays Cancel; what must not delay is the page.

### Liveness made visible

The UI gains a ticker: while a chunk's await is pending, every second the
status card re-reads the DB (coverage grows per page in list, per item in
detail) and the sync note carries the chunk's elapsed time. Honest about
position on purpose: `sync_state` moves only at chunk ends (`finish()` in
src/sync.rs), and ADR 0002 keeps progress truth in the DB — so the note
prints elapsed seconds, not a mid-chunk cursor that would be a lie.

### Audit

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| boot audit after the fix | unchanged from v0.5.0 | four verbs `status/watch/stats/status` |
| probe launch: 12-id targeted chunk, timestamps added for the run (probe removed before commit) | status polls **inside** the chunk await | `sync` verb at t=…268, ticker polls t=…269…293 one per second — the Retry-After stretch with the page alive |
| tokio slow-task warnings | none | stderr still only `gui:` lines |
| exit-code audit | unchanged | `2 · 2 · 3 · 4 · 4 · 0` |
| CI mirror | unchanged | 11/11 `exit=0` |

**Amendment (v0.5.2)**: the probe row above attributes the visible stretch to a
`Retry-After` wait inside the chunk. The v0.5.2 audit shows that probe's crawl never
issued a request from the GUI at all — it stalled at `build_client`, so no wait was ever
reached from the webview. The liveness witness itself stands (the `status` polls
interleave while the verb runs, and that is what the fix is about), and the wrapper
decision it recorded is unchanged; only the cause of the stretch was attributed wrong.

Rejected for mid-crawl feedback: a JS progress channel (`emit`/`event.listen`)
— those are plugin commands and ACL-gated; the exemption recorded in ADR 0005
would stop covering them (an `app:core:event` grant would be needed), and
progress truth is already the DB.

---

## 2026-11-19 · v0.5.0 — targets, analytics, the card: what the buttons finally answer

### The question this release answers

v0.4.0's sync buttons could only crawl "everything": the list stage, and the whole pending
queue. The watch list had existed since v0.2.0 as something that only **printed**. The
release starts where the roadmap arithmetic already insisted: 19 k details cost 2.7 hours,
so the only useful question is "which details do I actually need?" — and that question
belongs to the user, not the crawler.

### A latent bug found by the first run that read those columns

`sync --ids 25,40` exited 4 on the first audit, and the message was rusqlite's:
`Invalid column type Null at index: 0, name: detail_etag`. Three single-column lookups in
`db.rs` were written `Ok(value) => Ok(Some(value))`, which makes rusqlite infer a **bare
String** for the getter, so a NULL column is an error instead of a value. The signatures
always claimed `Option<String>`; nothing had ever read a NULL through them, because the
pending stage fetched an item before it ever asked for that item's ETag. The targeted stage
reads `detail_etag_of` for every id, so the latent wrongness surfaced immediately. Asking
for the Option type makes NULL a `None` — which is what the type always promised.

### Chunking targeted ids in JS, not in the engine

The engine's `--budget` truncates from the head of the list, so a second chunk of the same
`--ids` would re-visit what the first chunk already fetched and the UI's chunk loop would
never reach the tail. The UI slices instead (20 ids per call), which also keeps Cancel
meaningful between chunks and one slice far under the 120-request burst. `--budget` still
bounds the pending crawl, where it means "items per run".

### Honest accounting, twice more

A targeted run answers its own question: `remaining` is the target list, not the pending
queue ("left in the list" ≠ "left pending"), and `complete` means the target set was fully
visited. The detail stage now sends the stored `detail_etag` when one exists, so re-asking
for the same ids is a `304` — the audit reads `requests 2, written 0, unchanged 2` on the
second pass over 25,40. docs/API.md counts both sides the same way (a `304` costs rate
budget, saves bandwidth and DB churn) and the sentence about "pending items have no ETag
yet" is now scoped to what it actually is: the pending stage.

### The card reads the DB, and offline stays the point

`item ID --cached` sends no request: the card answers from `raw_json` and prints
`detail_synced_at` so the user can tell today's prices from last month's. A record never
fetched is `not-found`, not an empty card — the UI turns that word into an offer, and the
first use of the `NotFound` error kind retired its dead-code warning. No item icons are
rendered: `icon_url` is a remote path, and hotlinking it would trade the app's whole
purpose (answering offline) for decoration. Quality colour carries identity instead; a
local icon copy is roadmap work.

### Analytics before any crawl

`stats --limit 6` is one verb, and its JSON (coverage, by-quality, extremes) renders as
bars in the same palette as the search chips, cheapest/priciest extremes, and one sentence
about the spread. The question "is this worth hunting" is answerable before the detail
stage has seen a single item.

### boot() as the audit's witness

Startup reads status, the watch list and the analytics, so no card paints empty — and the
Rust log shows four verbs (`status`, `watch`, `stats`, `status`). The fourth line is the
witness that the analytics **render** ran to the end: a JS throw after the awaited verb
would leave three lines, not four. CI still cannot open a window, so the eleven-step CLI
mirror (six new steps: `watch add`, a targeted fetch, the same ids again answering 304s, a
watch-list target, the DB-only card) is what guards this release.

### Audit table

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| `sync --ids 25,40` (first pass) | both written | `requests 2, written 2, unchanged 0, complete true` |
| `sync --ids 25,40` (second pass) | revalidation, nothing rewritten | `requests 2, written 0, unchanged 2` |
| `sync --watch` after `watch add 25 40` | target = the watch list | `visited every targeted record: 2 fetched` |
| `--ids 25,x` | usage | exit 2 |
| `item 999 --cached` | not fetched yet, not a crash | `not-found`, exit 4 |
| GUI launch | window + four verbs from `boot()` | `gui: ipc verb status/watch/stats/status` |
| CI mirror | eleven steps | all `exit=0` |
| exit-code audit | unchanged | `2 · 2 · 3 · 4 · 4 · 0` |

### Deliberately not done in this tag

No price history yet: the DB stores one price per item, so a trend line needs a second
table (roadmap). No icon download. No item-card rendering of `sockets`/`stats`/`classes`
values (they arrive as unshaped JSON, and inventing a shape is not our call to make).

---

## 2026-11-19 · v0.4.0 — the shell: `build.rs`, `window.__TAURI__`, chunked cancel

### The question that shaped the release

v0.3.0 proved the pipeline headless. The next question is the one the IDE asks first: does
`cargo run` open a window at all? Everything in this release is downstream of reading the
crate sources in the registry cache instead of tutorials — every tutorial assumes `create-tauri-app`
plus `tauri dev`, which [ADR 0001](./DECISIONS/0001-flat-cargo-layout-no-cli.md) forbids.

### The trap that cost the most time

`cargo build` failed with `error: OUT_DIR env var is not set, do you have a build script?`,
raised by `tauri-codegen/src/embedded_assets.rs:335` while expanding `generate_context!`. The
obvious fix — an empty `build.rs` — compiles, opens the window, loads the page **and leaves
`window.__TAURI__` undefined**, so every button in the UI fails silently. The reason is one
line up the pipeline: `tauri-codegen/src/context.rs:412` injects the global-API scripts only
when `tauri_utils::plugin::read_global_api_scripts(out_dir)` finds a list file that
`tauri_build::build()` writes. So the fix is not "a build.rs", it is "the build.rs that runs
`tauri-build`", which is why `tauri-build = "2.7.6"` is a build-dependency and why
`tauri.conf.json`'s `withGlobalTauri` means anything at all. A GUI feature that can fail
silently is a design smell, and it is why the audit below ends in Rust log lines.

### ACL: measured, not assumed

The gate is `tauri/src/webview/mod.rs:2080`:

```rust
if (plugin_command.is_some() || has_app_acl_manifest || !is_local)
  && request.cmd != crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND
  && invoke.acl.is_none()
```

Ours is none of the three: `cli_call` is an app command, the page is a local origin, and
`has_app_acl_manifest` is false — `acl/mod.rs:353` defines it as `acl.contains_key("__app-acl__")`
and the build's own artifact `gen/schemas/acl-manifests.json` carries 10 plugin keys with no
`__app-acl__` in it. The app command is reachable **by construction**, and the evidence is a
file the build itself produced. The trap that follows is recorded in ADR 0005: the moment an
app-side ACL manifest lands, `has_app_acl_manifest` flips and a missing grant becomes a hard
failure.

### Cancel without `AppState`, because v0.3.0 already made it possible

ADR 0002 says the progress truth is the `sync_state` row, and v0.3.0 made `sync_state.page` an
absolute cursor with resume legal for `stopped` and `cancelled`. That makes chunked sync free:
the UI issues `sync --stage <stage> --budget 10` (list) or `--budget 25` (detail) in a row until
the report says `complete`, each chunk ends with a `sync_state` write, and a JS flag read at the
next chunk boundary *is* cancel. There is no `state.rs`: nothing mirrors the cursor in Rust, so
nothing can drift. The cost is honest — cancel waits for the chunk, and a `304` still costs one
request of rate budget (the v0.3.0 note), so a chunk of list pages is cheap in bandwidth, not
in budget.

### The icon is generated, so the blob is explainable

Codegen reads `bundle.icon` at compile time (`CachedIcon`), so `cargo build` fails without the
PNG. `tools/gen_icon.py` draws it in PIL from fixed constants — navy plate, gold coin, rising
price line — supersampled 1024 → 256 with LANCZOS. Nothing is downloaded or pasted: the script
sits next to the committed PNG, so a fresh clone builds and a reviewer can see how the icon was
made.

### The version source, once more

`tauri-codegen/src/context.rs:255` computes `package_version` as `config.version` **else**
`env!("CARGO_PKG_VERSION")`. The shipped conf therefore omits `version` entirely: `Cargo.toml`
is the only version source, and the house rule that used to say "bump both files" was amended
rather than broken silently.

### Launch audit — display server: KDE Plasma on Wayland (`wayland-1`), XWayland `:1`

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| `cargo build` with the new files | still one buildable unit | four errors, each fixed at the source of the message: `OUT_DIR` (build.rs + `tauri-build`), `cannot find macro __tauri_command_name_cli_call` and `__cmd__cli_call` (→ `generate_handler![crate::commands::cli_call]`, because `handler.rs:166` rewrites the last path segment), `E0382` partial move of `inv` (→ match on `inv.fixture.clone()`) |
| `WGT_DB=/tmp/opencode/fixture.sqlite cargo run` | window opens, page loads, JS reaches Rust | `gui: wow-goldtool shell starting…` → `gui: the webview asked for tauri://localhost (window main)` → `gui: ipc verb status (0 words)` — the page-load log proves the asset map served `index.html`, the ipc line proves the awaited `core.invoke` round-tripped |
| webview console | not a channel | `console.log` added to `ui/app.js` never appeared; grepping `tauri`/`wry`/`tauri-runtime-wry` for console forwarding found nothing on Linux, so the observable audit channel is Rust's `gui:` lines and the UI states its own rendered-field count in the status card |
| CI mirror + exit codes + v0.3.0 audits after the `cli.rs` refactor | unchanged | smoke mirror all `exit=0` (it is a zsh script: run with `zsh`, `bash` mis-parses the `$=step` split), exit-code audit `2 · 2 · 3 · 4 · 4 · 0`, `resumed_from: 2`, `unchanged: 2`, `item 25 → 304 reused true` |
| `window.__TAURI__` presence | proven by the round trip above | the empty-`build.rs` trap above is exactly the state that produces no `gui: ipc …` line |

### Deliberately not done in this tag

No packaging yet (`bundle.icon` and `identifier` are in place for it). No `state.rs`. No
detail-sync target selection from the UI yet — the buttons run the whole stage with chunk
budgets, and choosing watchlist/ids is v0.5.0. The webview's first paint was not reviewed
pixel-by-pixel: KDE is Wayland, `grim`/`kscreen-shot` are absent and X11 `import` cannot see
Wayland windows, so the visual check is a `cargo run` for the user to run.

### Follow-ups parked for later

* `.deb`/`.AppImage` once packaging is worth doing — the identifier and icon groundwork is done.
* Whether a second window (`app.windows` with `templates`) is worth the config surface.
* `tests/api.rs` / `tests/pipeline.rs` unit tests are still a plan, not code.

---

## 2026-11-19 · v0.3.0 — reuse, resume, cancel: the second crawl

### The question that shaped the release

The v0.2.0 crawler only ever crawled forward from scratch. Re-opening the app would
re-download everything it already knew. The upstream had told us how to avoid that
(`ETag` + `If-None-Match` → `304` **with no body**) — so the release is built around
what a body-less response forces us to admit.

### What a 304 forces, and what the v0.2 code would have done

The old loop fell through to `parse_page(body)` for every non-429 status: a `304` would
have died as a decode error on an empty body. Every reuse path now branches **before**
parsing. Because a list page has no DB row to re-read, each page also caches its
`has_more` and `count` in `meta` (`list-page-<n>-etag/-more/-count`) — the crawl needs
exactly those two facts to decide "walk on?" and keep `total` honest across 304s.

The detail side is opposite: the record is already on disk, so `raw_json` is the stored
copy and `item 25` revalidates the row instead of echoing a fresh payload. A `304`
**without** a stored copy is a real inconsistency — reported as an honest decode error,
never papered over. And a fresh `200` from a live lookup is now written through the
same upsert the sync stage uses, so its ETag can be reused next run.

### Honest accounting, twice

- A `304` still costs one request against the burst: revalidation buys bandwidth and
  DB churn, **not** rate budget. Written into docs/API.md so nobody reads "reuse" as
  "19 k fewer requests".
- `pages` counts a run; the `sync_state` page column must be an **absolute** cursor for
  resume to mean anything. `finish()` took an explicit `cursor` parameter — the two
  numbers were quietly the same value in v0.2 only because every run started at page 1.
- The report grew `resumed_from` and leans on `unchanged`: a run of two 304 pages
  honestly reads `complete: true, items: 0, unchanged: 2`.

### Resume only for pauses, not for completions

`stopped` and `cancelled` resume at stored page + 1. `complete`/`ceiling`/`idle` restart
at page 1 — a completed crawl has nothing to resume from, and the ETag walk makes that
cheap. Unknown status strings fall to the same branch, so a hand-edited DB cannot wedge
the crawler.

### Cancel without signal juggling

No atomics, no FFI `sigaction`: a plain `CancelFlag` struct that the loops check before
every request. The GUI will own one in AppState (v0.4.0) and raise it from the UI; the
CLI hands out a fresh one because Ctrl-C already leaves the DB consistent (every write
is its own statement) and the next run resumes by construction. A cancelled run records
status `cancelled` and exits **0** — the user asked for it, nothing failed.

### Fixture audit — every claim executed, never asserted

| Run | Expectation | Observed |
| --- | ----------- | -------- |
| list `--budget 1` | stopped, 1 page, forced 429 retried | `requests 2, pages 1, rate_limited 1, complete false` |
| list (resume) | `resumed_from 2`, page 1 not refetched | `requests 1, resumed_from 2, complete true` |
| list (full re-sync) | both pages `304`, nothing written | `unchanged 2, written 0, items 0` |
| detail | 429 + retry + 5 records | `requests 6, written 5, rate_limited 1` |
| `item 25` | `304`, record served from stored `raw_json` | `status 304 reused true` |
| exit-code audit | 2 usage · 3 network · 4 DB · 0 healthy | all six cases re-ran green |

The fixture grew conditional-GET behaviour itself (stable per-endpoint ETags, real 304
responses with no body): a server that cannot answer `If-None-Match` would have made the
release untestable, so the test asset learned the protocol at the same commit.

## 2026-11-19 · v0.2.0 — data layer, sync engine, contract checks

### One buildable unit, eight modules

The data layer and the sync engine landed together because they are one compile unit:
`main.rs` declares every module, so a half-written tree does not build. Splitting further
would have broken the "every commit must build" rule, so the code ships as one
`feat(core)` commit and the docs/CI split into separate commits after it.

### rustc as the arbiter, twice

The session had a display quirk: tool output and echoed payloads mangled `::` into `.`
and angle-bracket generics into parentheses. Never patched Rust from displayed text;
byte counters in python verified file contents and rustc decided what was right. A
multi-file "apply rustc suggestions" script also overwrote files **inside `~/.cargo`**
(`rusqlite`'s and `serde_json`'s extracted sources); the fix restricted the applier to the
diagnostic's primary file, and the corrupted crates were restored byte-identical from the
registry cache tarballs.

### Two bugs the compiler cannot see

- **SQL arity drift**: the detail upsert declared 38 columns but its VALUES run had 40
  markers — rusqlite fails at *runtime*, rustc stays silent. A checker script cross-verified
  column names, VALUES markers, SET assignments and the `*_VALUE_COUNT` consts; 40 fixed to
  38.
- **WHERE-chain builder**: it emitted `a AND b AND ` with no keyword — SQLite answered
  `near 'LIKE': syntax error`. Now the builder owns its keyword: `format!("WHERE {} ",
  joined)`.

Verified along the way: bundled rusqlite 0.43 really does accept the
`CREATE INDEX name ON table (cols)` spelling that docs/DATABASE.md documents — the open
question from v0.1.0 is closed.

### The fixture caught a lie in the report

`sync --stage detail` against the fixture reported `done: 4 / total: 5` next to
`"complete": true`: a 429 hit the `continue` of the per-item loop, which dropped that item
from the run, and `complete` was hardcoded. Fixture forced this by answering the **first
request of each stage** with 429 + Retry-After. Fixed the semantics: retry the same item
in place (up to 3 retries after each Retry-After), leave it pending if that still fails,
and derive `complete` from the remaining pending count — otherwise report `partial`.
Re-run: `requests: 6, written: 5, rate_limited: 1, complete: true`, exit 0.

### Exit-code audit (ADR 0004)

| Scenario                     | Observed | Expected |
| ---------------------------- | -------- | -------- |
| unknown verb / unknown flag  | 2 / 2    | 2        |
| unreachable base URL         | 3        | 3        |
| corrupt DB via `--db`        | 4        | 4        |
| unwritable DB path           | 4        | 4        |
| healthy `status`             | 0        | 0        |

### House rule amended mid-flight: the version bump

"Version bumps happen in `Cargo.toml` **and** `tauri.conf.json` together" cannot run yet:
`tauri.conf.json` joins the tree at v0.4.0 (it is what `generate_context!` reads). The
rule now means "every version field that exists today"; `Cargo.toml` bumps to 0.2.0, and
the conf-file half of the bump arrives with the GUI shell.

## 2026-11-19 · v0.1.0 — scaffold, measurements, decisions

### Setting the stage

The project started life as a default RustRover binary (`hello world`, git history empty,
branch `master`). Before writing any application code I pinned down the two things that
decide the whole architecture: **what the data source can actually do** and **what the
toolchain can actually build**.

### Toolchain probe (before writing a line of app code)

| Check | Result |
| ----- | ------ |
| `rustc` / `cargo` | 1.94.0 (2026 snapshot) — comfortably newer than Tauri's 1.81 floor |
| Node | v22 present, but deliberately **not** used (see below) |
| `tauri` crate resolves | 2.12.7 (`cargo add tauri@2` locked 427 packages, incl. webkit2gtk/gtk stack) |
| `tauri` + webkit2gtk + gtk compile on this box | ✅ probe crate built in 61 s |
| `rusqlite` / `libsqlite3-sys` | already in the cargo cache, `bundled` feature available |
| `reqwest` 0.13 | default TLS is rustls (`aws-lc-rs`), so no OpenSSL linkage assumption needed |
| Desktop | KDE Plasma on Wayland + XWayland (`WAYLAND_DISPLAY=wayland-1`, `DISPLAY=:1`) |

The probe crate is thrown away; its only purpose was to prove the GUI layer can build
before I spend effort on it. Lesson learned early and written into the house rules:
**probe the toolchain, then design**.

### The hard constraint that shaped the layout

The app must compile straight out of RustRover. That killed the usual
`create-tauri-app` + `tauri dev` route, which needs the Tauri CLI and a Node toolchain to
generate the config context and the asset map. Reading the actual crate sources in the
cargo registry cache answered the question precisely:

* `tauri-macros` documents `generate_context!` as reading `tauri.conf.json` **relative to
  `CARGO_MANIFEST_DIR`, at compile time**;
* `tauri-codegen/src/context.rs` then reads `config.build.frontend_dist` **from the
  filesystem during compilation** and generates a `phf_map!` of assets by itself
  (`EmbeddedAssets::new(phf_map! { … })`).

So the plain `cargo build` path already embeds config + UI assets. The Tauri CLI is only
needed for hot reload and installer bundling. That made the flat layout
([ADR 0001](./DECISIONS/0001-flat-cargo-layout-no-cli.md)) strictly better for this
project than the `src-tauri/` convention.

### API measurement day (TheWoWDB, all figures below are measured, not assumed)

* `GET /api/v1/forever/item/25` → full record for *Worn Shortsword*: `price.available 11`,
  `price.market 400`, `price.min 290` copper, `vendor_sell 7`, `weapon{1-3 dmg, 1.9 s,
  dps 1.1}`, `quality "common"`, faction price buckets present but empty for that item.
* `GET /api/v1/forever/search/?q=Worn Shortsword` → grouped hits with per-kind `has_more`
  and a `more` URL.
* Paging: `per_page` is clamped server-side to **100** (asked 200, got `per_page: 100`);
  `page` runs to 200.
* Total item count is **not** reported on large lists — only `has_more` + `links.next`.
  Binary search over pages located the last page carrying data: **page 193, 57 records,
  `has_more:false`** ⇒ ≈ **19 257 items**. Ids are sparse (first 25, last 288 340), so
  id-space iteration is useless: crawl by page, not by id.
* Docs confirm caching rules (`ETag`, ~1 h public cache, tooltips 5 min, `304` on
  `If-None-Match`) and the rate model: **120 req burst per IP, refilled at 2/s**, `429` +
  `Retry-After` + `X-RateLimit-Remaining` past that.

Cost arithmetic written down before designing the sync, because it decides the design:
193 list requests ≈ 2 minutes, a full detail pass ≈ 19.3 k requests ≈ **2.7 hours**. A
single "sync everything" button is therefore unusable as designed → stage the sync, make
it resumable, cancellable and opt-in ([ADR 0002](./DECISIONS/0002-two-stage-sync.md)).

### Decisions recorded as ADRs

| ADR | Verdict in one line |
| --- | ------------------- |
| [0001 flat Cargo layout, no Tauri CLI](./DECISIONS/0001-flat-cargo-layout-no-cli.md) | RustRover must build the app with plain `cargo build`; `generate_context!` already does the embedding |
| [0002 two-stage sync](./DECISIONS/0002-two-stage-sync.md) | list crawl is cheap, per-item detail crawl is 2.7 h → stage, resume, cancel, opt-in |
| [0003 rusqlite over the DB](./DECISIONS/0003-sqlite-rusqlite-bundled.md) | one disposable file with sort orders the upstream API cannot express |
| [0004 headless CLI in the same binary](./DECISIONS/0004-headless-cli-mode.md) | the data pipeline must be testable without a display server |

### Deliberately not done yet

No application code in this tag. `src/main.rs` is still the generator's `hello world`, so
`cargo build` passes and CI would be green: the scaffold is documentation-only.

### Follow-ups parked for later

* Packaging: `.deb`/`.AppImage` generation without the CLI (or a documented
  `src-tauri`-compatible shim) — roadmap item after `v0.5.0`.
* Retail version (`/api/v1/retail/...`) as a second catalogue tab; `forever` is pinned for
  now (see [docs/API.md](./API.md)).
* Whether item tooltips (`/api/v1/tooltip/...`) should be fetched lazily on demand instead
  of reconstructed from the detail record.
