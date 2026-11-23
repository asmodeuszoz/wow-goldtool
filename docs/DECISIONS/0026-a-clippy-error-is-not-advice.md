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
* The next defect class this makes explicit: **a red CI run is read BEFORE it is explained** — the
  ladder row says which step moved, in that step's own name, and only then which gate it rides.
