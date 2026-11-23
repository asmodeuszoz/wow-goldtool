# ADR 0026 — a clippy ERROR is not advice: it rides the run, so it dies BY NAME, and the `-D warnings` gate stays OFF

**Status:** decided (v0.12.1, 2026-11-23)
**Refs:** ADR 0024 (fix-not-mute, every rejected mute recorded — this era keeps that rule for a
DIFFERENT gate), ADR 0011 (CI states only measured truths; the lint step's `informational` claim),
ADR 0025's consequence bullet (the defect this closes), the era's chain+tag run
**45095439290 measured FAILURE** (GitHub's own words), `docs/DEVELOPMENT.md`'s warning census.

## The two gates, kept apart (measured 2026-11-23)

A tree here answers FOUR kinds of build-time line, and the record names which gate each rides:

| gate | what it answers on the v0.12.0 tree | what it gates |
| ---- | --------------------------------- | ------------- |
| `cargo build` (rustc) | ZERO named warnings — the FOUR-named set died at v0.11.1 ([ADR 0024](./0024-the-four-named-warnings-die-by-name.md)) and stayed ZERO at v0.12.0 | nothing (the tree ships) |
| the lint step's `cargo clippy --all-targets` | **ONE ERROR** plus **811 advice lines** — `error: this loop never actually loops` at `src/db.rs:3029:5`, `clippy::never_loop`, `#[deny(...)]` ON by default | the step's rc, therefore the JOB, therefore the RUN: measured, GitHub's own verdict moved **success → FAILURE** (4/5 jobs, 8 min 9 s) |
| clippy's 811 advice lines | `question_mark`, `needless_return`, `len_zero`, `clone_on_copy`, `single_match`, `manual_unwrap_or`, `assign_ops`… (the house style is hand-written; 228 of them duplicates on the bin, 583 on its test twin) | NOTHING — ADR 0011's rule keeps `-D warnings` OFF, and no gate is added |
| `cargo fmt --check` | answers a diff (the house style) | NOTHING (ADR 0011: no fmt gate) |

The ERROR is the ONLY one of these that ever moved a run's verdict, and it moved it because the
step that runs it is named "informational" — the NAME was true OF the tree at v0.11.1 (measured
0 errors / 817 advice on GitHub's own builds), and it became a LIVE LIE at v0.12.0 exactly because
an ERROR rode it. A step whose rc gates the job may not be called informational.

## The decision, and its rejected alternatives

**Fix the ERROR BY NAME; move NOTHING else.** `recipe_card` wrapped its ONE-row read in a
`for` whose body always returns, so the iterator could never take a second turn — a shape that
contradicts the FACT the door's own doc comment already states (`spell_id` is the `recipe`
table's PRIMARY KEY, so at most ONE row answers). The repair is clippy's OWN suggestion
`if let Some(one_row) = mapped.next()`, which is the shape the established single-row readers in
this tree already ride (`history_of`, `watch_list`, `deleted_at_of` take a value, not a walk), so
the fix does not invent a style: `fix(db) 56e0356`, era diff **ONE file, 7 insertions / 2
deletions**, ZERO fixture / ui / ci / Cargo lines.

Rejected, each with its measure:

* **Muting it** (an `#[allow(clippy::never_loop)]` on the function): ADR 0024's rule
  kept for the new gate — an `allow` keeps the LIE (a walk that cannot walk) inside the door and
  makes the ERROR invisible to the next era, which is exactly the drift this file exists to
  prevent. The ask's "mute them" branch stayed unneeded, as at v0.11.1: this ERROR is fixable,
  and clippy itself states the fix.
* **Adding a gate** (`-D warnings` / `-Werror`, or making the lint step deliberate): measured, the
  SAME command answers **811 advice lines**, so a gate would fail on clippy's OWN style advice
  (ADR 0011 measured ~600 at v0.8.0; the count rides `--all-targets` and duplicates, not our
  dead code). The gate stays OFF; the ERROR dies because it is a fact, not because a gate is
  tightened. Zero of the 811 advice lines are silenced, so the advice census stays visible where
  it lives.
* **Leaving the step's rc informational and letting the run stay red**: CI's verdict is the ladder
  this project reads before every push (ADR 0011's ordering rule). A red run whose only failing
  step is "informational" makes the NEXT era's gate-keeping a question of trusting a NAME rather
  than a measure. The name moves WITH the truth instead: after the fix the step measures 0 errors
  / 811 advice again, so its existing name is true AGAIN and the workflow needed NO move — measured
  rather than wished.
* **Rewriting the door as a Vec-collect-then-return** (the shape the multi-row doors ride): correct
  but noisier for a read that states ONE row; the `if let` shape is what clippy itself suggested and
  what the other single-row readers already ride, so the collect would have added a Vec and a
  second branch for no honesty gain (ADR 0014's least-shape rule).

## Consequences, kept honest

* The named rustc tally stays **ZERO** across this era (this was never a dead-code warning: rustc
  calls NOTHING here), so the era is honest about its own tally riding unchanged while a DIFFERENT
  gate's count moved — 1 ERROR → 0 errors, 811 advice → the SAME 811 advice.
* Behavior moves **ZERO**: measured on fresh ports and fresh DBs, the pinned v0.12.0 binary
  (`ace683a`, whose OWN build is green in 1 m 23 s, banner `0.12.0` — the FEATURE era moved it, this
  PATCH era leaves it PUT) vs the fixed binary answer **22/22 BYTE-IDENTICAL** on the fixture DB —
  the NINE card doors included, a STRONGER pair than earlier patch eras compared, which only had
  the pre-existing doors to compare — and **13/13 md5-equal** through BOTH binaries over the user's
  OWN 19 258-row file (read-only, LOCAL doors, ZERO requests). The mirror replay stays **51/51 rc 0**
  with the same stage tallies (48 step-lines unchanged, plant one).
* `docs/DEVELOPMENT.md`'s census moves where the lie was (the clippy ERROR line, plus the note that
  the ERROR and the named set are DIFFERENT gates); [ADR 0025](./0025-the-recipe-card-window.md)'s
  consequence bullet gains its CLOSED pointer; the workflow's step name and header comment are
  untouched because they are true again at v0.12.1.
* **THE FOLD, MEASURED (2026-11-23, post-tag, fresh ports/DBs — this era's own ladder row):** the era's
  chain+tag run — head = the TAGGED tree `d2c26de` — measured **45101284057 SUCCESS**, 5/5 jobs green in
  **8 min 13 s** (02:37:33Z → 02:45:46Z, run #44, attempt 1, ONE run id, no CANCELLED sibling): quality
  **7/7** with "Lint (informational — clippy's own advice; our named set died at v0.11.1, see
  docs/DEVELOPMENT.md)" **completed/success** — GREEN for the FIRST time since v0.11.1 — and `test` **8/8**
  with the CLI smoke step GREEN on GitHub's own build of the tagged tree; linux 8/8, macos 7/8, windows
  7/8 (the Ubuntu-libs step `skipped`, as designed). The era's claim is now measured rather than argued:
  the verdict moved **FAILURE → SUCCESS with ZERO workflow lines moved** — same step text, same gate
  (still off), the ONLY delta is the tree's shape. Its sibling, measured in the same breath: the v0.12.0
  fold push's run **45098365666 = FAILURE** (4/5 in 11 min 20 s, head `89cab34`), SAME failing step, exactly
  what the tree implied BEFORE it was read. Post-tag witnesses on FRESH ports: mirror replay **51/51 rc 0,
  ZERO nonzero** (fresh 9341; the block re-PARSED at 48 step-lines + plant one, the era moved NO step; the
  four card steps at 7 333 / 6 673 / 4 107 / 643 B; the fixture's own GET tallies 51 with 17 recipe-file
  GETs, **ZERO added** — that is the locality proof); exit map at the **SAME 31-door totals `0`×19 ·
  `2`×5 · `4`×7** (fresh 9373, FOUR rc-`4` doors in the SAME words, `recipe 132 --fixture <DEAD>` rc 0 with
  rc `3` UNREACHABLE from this door); and the PINNED v0.12.0 binary (`ace683a`, banner `0.12.0` PUT) vs the
  pushed tree **16/16 md5-equal, ZERO DELTAs** over the user's OWN file (card doors 25 213 / 7 004 / 9 430
  / 3 596 / 3 453 B, `recipe 27835` BARE 3 705 B, rc `4` on `638` 109 B · `8800111` 113 B · `999999` 112 B),
  whose own tallies carry the FACT this fix rides, read ON the file rather than cited: **1 240 recipe rows
  AND 1 240 DISTINCT `spell_id`s** — at most ONE row can answer the card read. Money over those very answers:
  **0 pairing defects**, COUNTS 6/2/2/1/1 with NO money beside them (ADR 0015's count half). The fold's own
  push starts the NEXT run, measured in the NEXT record (ADR 0011's ladder), never chased here.
* The next defect class this makes explicit: **a red CI run is read BEFORE it is explained** — the
  ladder row says which step moved, in that step's own name, and only then which gate it rides. Measured,
  the class CLOSED at this era: the same step that rode the FAILURE at v0.12.0 answers GREEN at v0.12.1,
  so the NAME is true again and the verdict moved on the tree's own account, not on a workflow's.
