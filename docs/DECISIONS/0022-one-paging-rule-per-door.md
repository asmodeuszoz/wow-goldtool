# ADR 0022 — the paging doors ride ONE composition rule

**Status:** decided (v0.10.1, 2026-11-22)
**Refs:** ADR 0004's exit-code contract, ADR 0014's one-code-path rule, ADR 0021's
post-tag defect record (the recipe door) and the v0.10.0 exit map (the search door)

## The question two doors were answering

Both doors that page — the local search door (`search TEXT --limit/--offset`,
born v0.9.0) and the local recipe door (`recipes --limit/--offset`, born v0.10.0)
— compose their paging clause themselves. The v0.10.0 exit map measured the class
TODAY, on both doors, with the same shape:

| door (no `--limit`) | answer today, BEFORE the rule |
| ----------------- | ------------------------------------ |
| `search sword --offset 3` | rc **4** carrying SQLite's raw refusal: `near "OFFSET": syntax error … ORDER BY … OFFSET ?` |
| `recipes --offset 10` | rc **4**, same clause, same refusal |
| `search paging --offset 4` on a 12-row scratch store (AFTER the rule) | rc **0**, 8 rows (rows 5..12) — the REST, exactly what a bare SELECT's tail answers |
| `recipes --offset 10` over the fixture's 12 rows (AFTER) | rc **0**, 2 rows of `matching 12`; `--offset 12` answers an **EMPTY set rc 0** (the door's no-row-past-the-end shape, never a 4) |
| `--limit 4 --offset 10` (the pair that already worked) | answers the SAME rows before and after — only the broken composition moves |

SQLite's own rule, measured the same day on the DB this tool links (sqlite 3.54.1):
`LIMIT ? OFFSET ?` with a **negative** limit answers "every row after the OFFSET"
(`LIMIT -1 OFFSET 3` on a 10-row scratch table → rows 4..10, IDENTICAL to the bare
SELECT's tail). A bare `OFFSET ?` is refused outright.

## The decision (three parts, one per truth)

* **One composition, two doors (ADR 0014).** `query.rs::paging_clause(limit, offset)`
  is the ONLY place a paging clause is written; `search_plan` and `recipes_rows` call
  it. A door that grows its own copy is a second, drifting truth the day the rule
  moves — exactly the shape ADR 0014 refuses.
* **Negative limit = SQLite's own "no bound", not a magic constant we invented.**
  "no `--limit` given" means "the rest of the rows"; the rule keeps that meaning
  UNCHANGED by riding SQLite's documented negative-limit semantics, measured above.
  No sentinel row-count query is added (no second SELECT to ask the DB how big it is),
  and no arbitrary cap is invented that would silently hide rows.
* **The raw clause never belongs in a door's answer (ADR 0004's message rule).**
  The rc family was already right (`4` = DB family), but the message carried the
  whole SELECT. Fixing the composition kills the class; NO redaction parser is
  invented for a class that can no longer occur (measuring-first: inventing a
  decoder for text we can no longer produce is invention). Where SQLite still
  refuses something, its own words ride — they are the engine's fact, never ours.

## What the fix does NOT do

* No schema move, no column, no new dependency: clause composition only.
* No request on either side of the witness — BOTH doors are LOCAL (ADR 0012's local-door
  rule: the search door reads the DB's own rows, the recipe door joins BY VALUE).
* The pair `--limit L --offset O` keeps answering L rows from O, byte-identical before
  and after — the fix rides ONLY the broken composition, not the working pair.
* `stats`/the pending queues keep their single `LIMIT ?` shape untouched (they never
  page past an OFFSET — measured: no OFFSET clause in their SQL).

## CI row (measured green, tag pinned)

**45044360477** (the chain+tag push `c94f6b0..09c8d77`, head = the TAGGED tree): **success, 5/5
in 9 min 45 s** (2026-11-22T16:31:58Z → 16:41:43Z — quality 2 m 27 s · linux 6 m 38 s · test 7 m
18 s · macos 7 m 20 s · windows 9 m 42 s), exactly ONE run id for that head, no CANCELLED sibling,
no rerun. The tag stays put: `v0.10.1` → `09c8d77`; the earlier era's tag stays put too (`v0.10.0`
→ `e4d3447`). The verification fold's own push starts the NEXT run, measured in the NEXT record —
ADR 0011's ordering rule, the ladder never chases a tag.

## Post-tag (measured FORWARD, folded in the DEVLOG's v0.10.1 entry)

The mirror replayed the GROWN block on a fresh fixture process + a DB born 8: **46/46 commands
rc 0, ZERO NONZERO**. The exit map re-ran GROWN (fresh fixture 9617, fresh seed DB, fresh EMPTY
DB, DEAD 9997, the scratch 404-file server 9558): **87 doors `0`×58 · `2`×17 · `3`×6 · `4`×6**,
with the SIX paging doors answering rc 0 — the class this ADR kills cannot recur. Over the USER's
own 19 258-row file (opens + LOCAL doors, ZERO requests): `search sword --offset 3` answers the
REST — **220 hits of `total 223`** rc 0 — `recipes --offset 1238` 2 rows of `matching 1 240`,
`--offset 1240` an EMPTY set rc 0, `--profession Enchanting --offset 150` 2 of `matching 152`; and
the pinned v0.10.0 binary answers **8 read-only doors BYTE-IDENTICAL** on the same file (md5-equal,
1 098 B … 81 428 B) — the fix moves nothing that does not page past an OFFSET without a LIMIT.

## Chain and witnesses

`9f4da68` fix(query) (the rule + the search door rides it) → `2a41ce3` fix(db) (the
recipe door rides the same helper) → `9e009b8` ci (41 → **43 step-lines**, the two door
steps `recipes --offset 10` + `search sword --offset 4` are green BECAUSE the rule holds
— they answered rc 4 before the fix, ZERO requests added) → this record carries the tag.
The mirror was measured BEFORE the ci commit off the very steps that landed (fresh fixture,
fresh DB born 8): **46/46 commands rc 0, ZERO NONZERO** (`TOTAL_STEPS=46` = 43 step-lines
+ plant + fixture start + sleep).

The PINNED pair (the v0.10.0 binary, a worktree at the tagged tree `e4d3447`, vs the new
binary, ONE shared fresh fixture process, TWO fresh DBs): **22 DB-only doors BYTE-IDENTICAL**,
the two paging doors show exactly the rc delta **4 → 0** (`recipes --offset 10` and `search
sword --offset 4`), and BOTH DBs answer the same tallies (schema 8, 5 items, 4 priced, 12
recipe rows — no schema delta to hide behind the fix). The working paging pair is byte-stable
under the SAME doors, md5-equal both sides: `recipes --limit 4 --offset 10` **1 629 B** both,
`search sword --limit 4 --offset 4` **182 B** both, `search sword --limit 4` **185 B** both.
