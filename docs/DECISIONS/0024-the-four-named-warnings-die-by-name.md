# ADR 0024 — the four named build warnings die BY NAME, not one of them muted

**Status:** decided (v0.11.1, 2026-11-22)
**Refs:** ADR 0011's rule that made this set EXPECTED (its `-D warnings` bullet names the
four dead-code warnings as intended, so the set was always a plan, never a surprise), ADR
0014's one-code-path rule, ADR 0021's dead-code standard for a stored field with no reader,
`docs/DEVELOPMENT.md`'s warning census, `docs/DATABASE.md`'s "two doors into the schema helper".

## The set, verbatim off the last green build (measured 2026-11-22)

`cargo build` at v0.11.0 answered exactly FOUR named warnings, and `docs/DEVELOPMENT.md`
plus `docs/DEVLOG.md` recorded the same set on EVERY build since v0.8.0 - rustc's own words
and own locations, quoted here so the record cannot be read as memory:

| # | rustc's own heading | its own location and words |
| - | ------------------- | ------------------------ |
| 1 | `type 'PendingSql' is more private than the item 'pending_plan'` | `src/stats.rs:199` — "function `pending_plan` is reachable at visibility `pub(crate)`", note: "type `PendingSql` is only usable at visibility `pub(self)`", `src/stats.rs:191` — `#[warn(private_interfaces)]` on by default |
| 2 | `multiple fields are never read` | `src/models.rs:142:9` — fields in `pub struct Tooltip`: `id`, `kind`, `name`, `quality`, `version`, `build`, `icon_url`, `links`, `attribution` (read: `html`, `fields`); note: derived `Debug`/`Clone` are "intentionally ignored during dead code analysis" |
| 3 | `function 'open_db' is never used` | `src/db.rs:490:8` |
| 4 | `function 'catalogue' is never used` | `src/sync.rs:909:8` |

The `hard linking … incremental cache` line that rode SOME builds of this tree was measured
again here: it is the FS artifact of the clear-the-incremental-cache mechanic (every build
here starts by removing `target/debug/incremental`), never a fifth named warning - this
era's builds answer it ZERO times, which is what makes the artifact's absence a measurement
and not a silence.

## The question behind each warning, and the fate

The user's ask was "fix them, and if they are not fixable, mute them". Each of the FOUR was
fixable, so the ask's second branch never had to be taken — the muting options are still
recorded below each fate, because a mute that was available and not chosen is a decision,
not a detail. What made fixing honest in all four cases is the same shape: rustc's warning
names a NAME the tree no longer needs, and the tree can say so where the survivor lives.

### 1 — `pending_plan`'s visibility: the CLAIM was the lie, so the claim dies

`pending_plan` was `pub` inside a module that is not itself `pub`, which rustc reads as
`pub(crate)`; its return type `PendingSql` is `pub(self)`. The warning is rustc saying the
signature cannot be honored. The witnesses: both `use crate::stats` lists (`cli.rs`, `sync.rs`)
leave the name out, and its FOUR callers are the four queue doors in this very file
(`icon_pending_ids`, `pending_ids`, `tooltip_pending_ids`, `price_pending_ids`).

* **FIX: the function stops claiming `pub`** (module-private). The narrowest true signature,
  and the queues are the only thing that ever asked for it. The `pending_plan` doc comment
  now says the fate and why the other direction was rejected, so the reason survives WITH the
  function that has it.
* Rejected, `pub(crate) struct PendingSql` instead: it widens the module's surface with a
  queue-plan type nothing outside this file consumes — a plan nobody can name is not a door.
* Rejected, `#[allow(private_interfaces)]` on the fn: a mute here keeps a signature the
  module cannot honor and teaches the next reader to distrust visibility claims tree-wide.

### 2 — `Tooltip`'s nine unread fields: the mirror narrows to the keys a reader reads

`Tooltip` is parsed at TWO sites (`sync.rs`'s tooltip-copy stage and its price-refresh stage)
and read at TWO fields: `html` (the copy stage writes it — the `forever/item` body carries no
description at all, the measured key list in `docs/API.md`) and `fields` (the refresh stage
reads `fields.price.min` through `TooltipFields`, ADR 0017). It derives `Deserialize, Clone,
Debug` and **no `Serialize`**, so no answer ever serialized a `Tooltip` back; and there is no
construction site — serde builds both values from the wire. The other nine names are rustc's
dead-code verdict on a mirror that typed the whole measured key list.

* **FIX: the struct keeps `html` + `fields`, and the other eight keys the probe read stay ON
  THE WIRE.** This is the rule the file already carries TWICE — `TooltipFields` types ONLY
  `price` "because `price` is the only key with a reader", and the recipe shapes (ADR 0021)
  type ONLY what has a reader, with the untyped keys recorded in the ADR and left on the wire.
  Applying it at the top level kills the warning AND keeps ADR 0010's promise (a key upstream
  adds cannot break a build) — serde ignores a key it has no field for, which is the SAME
  promise the unread fields were keeping by being typed-and-ignored.
* Rejected, `#[allow(dead_code)]` on the struct (or nine per-field `#[allow(...)]`): keeps
  nine names nothing reads alive as documentation. The documentation duty those names served
  — "this door answers these ten keys, measured" — already lives where probes are recorded,
  in `docs/API.md`, which carries the whole list (measured 2026-11-20/21: `attribution, build,
  fields, html, icon_url, id, kind, links, name, quality, version`). ADR 0021 called a
  stored-without-reader field dead code in the door's shape; muting here would contradict
  that standard instead of riding it.
* Rejected, keeping `id` while dropping the other eight, to preserve serde's required-key
  check on `id`: a required key nobody reads is a wire-shape guard this tool never needed —
  the request URL carries the id we asked for, and NOTHING compares a tooltip's own `id`
  against it (the compiler proves no reader exists). The measured cost of the fix is recorded
  in the `Tooltip` comment instead of hidden: an answer that OMITS `id` used to fail to parse
  and now parses. Measured against what the tree holds, that difference moves no answer — the
  fixture's tooltip bodies, the recorded live bodies and the user's own copies carry every key,
  and a body that is not an object with an `html` string still fails to parse and still aborts
  its stage with "tooltip of item N is not the JSON we expect", ADR 0004's message rule intact.

### 3 — `open_db`: the wrapper's contract stopped being true, and the door it opened already lives

`open_db` bundles `db_path` with a body character-for-character identical to `open_db_at`'s
(`exists` → `check_schema` for a file already on disk; `execute_batch(SCHEMA_SQL)` plus the
`schema_version` `meta` stamp for a fresh one). Its comment claimed "every entry point goes
through this", which stopped being true the moment the entry points moved: `cli.rs`'s `open_at`
returns `open_db_at(target.path)` and `cmd_status` calls `open_db_at` directly, and the GUI
reaches the same door through `cli_ok`'s argv (ADR 0014's one-code-path, so no second opener
was ever needed for the webview).

* **FIX: the wrapper dies**, with `open_db_at`'s own comment rewritten in the same breath —
  it named the removed function, and a doc that names a dead name is the drift this era must
  not leave behind. `docs/DATABASE.md`'s matching line moves to name the two doors that
  exist: `db_path` (who picks the file, the four-way rule) and `open_db_at` (who opens and
  stamps it).
* Rejected, `#[allow(dead_code)]` on `open_db`: it keeps a SECOND copy of the schema rules
  (fresh-file write, refusal, `meta` stamp) that no door reaches — the day the schema moves,
  that copy is the drifting truth `docs/DATABASE.md` exists to prevent.
* Rejected, making the entry points call it again: that would move live opening behavior
  (path resolution, the `DbTarget` the CLI owns) for a witness the audits already proved lives
  in `open_db_at`. Nothing about opening moved.

### 4 — `catalogue`: one door had two names, and the second name lost its last caller

`catalogue` is `return coverage(conn)` — the sync module's re-export of the aggregate, born at
v0.8.0 with the promise "used by `status` and the UI footer". `cli.rs` reads the real door
(`use crate::stats::{coverage, …, Coverage}`; `cmd_status` calls `coverage(&conn)`), so the
re-export lost its caller and rustc named it never used.

* **FIX: the wrapper dies AND the two import names it alone used die with it.** This half is
  the era's trap: `sync.rs`'s `use crate::stats` list named `coverage` and `Coverage` ONLY for
  this function, so removing the function and leaving either name would have traded this dead-code
  warning for an unused-import one — a warning that was never in the named set, which is exactly
  the swap ADR 0011's "the set is expected" rule refuses. The removed comment's claim moved ONTO
  the survivor (`stats.rs::coverage`'s comment now names its three readers and says the wrapper
  died), so the door's readers stay recorded where the door lives.
* Rejected, keeping the re-export alive on purpose (making `cli.rs` call `sync::catalogue`):
  two names for one door is ADR 0014's drift shape — the aggregate would have a second spelling
  that can rot while the first one answers.
* Rejected, `#[allow(dead_code)]` on `catalogue`: same mute-vs-fix grounds as above, and it
  would have left the promise "used by `status`" attached to a function no door calls.

## What the four fixes do NOT move — the honest no-delta claims

No SQL constant moved, no wire shape moved, no schema or migration moved (schema stays `8`),
no exit code moved (ADR 0004's `0`/`2`/`3`/`4` family), no money formatter moved (ADR 0015's
one writer), no UI line moved, no request rule moved (ADR 0017's pending queues still own the
same SQL), and the trio's NULL-vs-0 truth moved nowhere (ADR 0016: `vendor_buy` /
`price_trend_normal` stay unwritten). The four fixes are the ONLY Rust that moved this era, and
each one's claim is a witness claim, not a memory: removal of a zero-call-site wrapper,
narrowing of a signature whose callers are all in-module, narrowing of a mirror whose readers
are two named fields, with the compiler itself proving that no reader of the eight unread
fields exists.

## What rides put

The `-D warnings` gate stays OFF (ADR 0011's measurement: `cargo clippy` answers 0 errors and
hundreds of its OWN advice lines, so the gate would still lie about clippy's advice even with
our named set at ZERO). `ci.yml`'s header and the lint step's NAME now carry the history — FOUR
through v0.11.0, ZERO from v0.11.1 — because both claimed the FOUR as live truth, and a comment
that lies about the build is the same drift class this era is killing. `docs/DEVELOPMENT.md` keeps
its v0.8.0 census as history AND names the current ZERO. `docs/DATABASE.md` names the two doors
that exist. ADR 0010's and ADR 0011's measured tables stay UNTOUCHED — they recorded what the
build answered in THEIR era, and this era's record rides forward with them.

## Chain and witnesses

`523dfcc` fix(stats) (fate 1) → `7b3bdb0` fix(models) (fate 2) → `6333783` fix(db) (fate 3) →
`316bb0e` fix(sync) (fate 4 + its orphaned import names) → `092ab61` ci (the workflow's two warning
claims follow the set, ZERO steps moved) → this record carries the tag. The measured tally rode
every commit: FOUR named → THREE (`523dfcc`) → TWO (`7b3bdb0`) → ONE (`6333783`) → **ZERO**
(`316bb0e`), with `cargo build` at rc `0` each time and the hard-link FS line answering ZERO
times in this era's builds. ## Post-tag (measured FORWARD, folded into the DEVLOG's v0.11.1 entry)

The four fixes were claims about what the COMPILER accepts; the fold measured them where the
answers live, and every claim held:

| claim in this ADR | measured |
| ----------------- | -------- |
| no SQL constant moved | the 44-step mirror replay off the UNCHANGED block (port-stripped compare EQUAL **44/44 step-lines**) answers **47/47 rc 0, ZERO NONZERO**, with the stage presses UNCHANGED (budget-3 3/5/4 + `rate_limited 1` · resumed-3 5/7/5 complete · re-pull 8 files / 8 same-copy / 0 written) |
| no door's rc moved | the exit map re-run GROWN on fresh ports: **89 doors `0`×60 · `2`×17 · `3`×6 · `4`×6, ZERO mismatches** — the same totals as v0.11.0's map |
| no answer shape or content moved | the PINNED pair (the v0.11.0 binary at the tagged tree `59b0185` — whose OWN build is the BEFORE witness, answering the FOUR named warnings verbatim — vs the new binary, one shared fresh fixture, two fresh DBs): **25 DB-only doors, ZERO content delta**, recipes doors INCLUDED (unlike v0.11.0's pair, no live-key strip was needed — no key was added), the three stamp-carrying doors raw-EQUAL on a same-second re-measure with ZERO non-stamp field differing |
| the wire's unread keys riding the wire moves no answer | the user's OWN 19 258-row file, both binaries over the SAME file: **10 of 10 doors md5-equal** (1 098 B … 21 061 B), tallies INTACT (19 258 / 4 231 vendor_sell / 4 287 samples / 1 240 recipe rows), `price_min_now` 0 / `price_now_missing_at` 0 on this host so the REAL joined rows keep answering `live price not refreshed yet` (3/3 crafted, 3/3 reagents, Blacksmithing matching **250** REAL rows), `vendor_buy` / `price_trend_normal` **0/0** |
| the named set died for the build, not for this filesystem | the chain+tag run **45058168649** (head = the tagged tree `3f20b51`) success 5/5 in 9 m 25 s, and its OWN `cargo build` dev + release jobs printed NO named warning — while the era's local builds answer ZERO named and the `hard linking … incremental cache` notice rode SOME mirror builds ("`generated 1 warning`" = that notice alone, never a fifth named) |

What stays put after the fold: schema `8`, ZERO ui line (nothing NEW to parse, so no parse witness
was owed), ZERO fixture change, ZERO CI step (44 step-lines + plant one), tags put
(`v0.11.1` → `3f20b51`), and the ladder line owed by the previous era (45049266077, success 5/5 in
8 m 2 s) recorded in THIS era's chain rather than chased.
