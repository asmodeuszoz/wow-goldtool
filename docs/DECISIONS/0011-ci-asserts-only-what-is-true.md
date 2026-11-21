# ADR 0011 — CI asserts only what is true: real action names, real gates, and named lies

**Status:** adopted (v0.8.1)
**Related:** `.github/workflows/ci.yml` (jobs `build` / `quality` / `test`), `actions/checkout@v7`,
`dtolnay/rust-toolchain@stable`, the 21-step smoke block (unchanged), `docs/DEVELOPMENT.md` (the
CI bullet), [ADR 0001](./0001-flat-cargo-layout-no-cli.md), [ADR 0004](./0004-headless-cli-mode.md),
[ADR 0005](./0005-gui-ipc-commands.md)

## The measurement that forced the decision

The repo was published (public `wow-goldtool`, `main` + 11 annotated tags) and GitHub started its
own run. It went **red in 7 s**, and the annotations are the whole answer — every one of them names
an action that does not exist:

```
build (x86_64-pc-windows-msvc, windows-2025)   Unable to resolve action actions/rust-action,
                                               repository not found. Unable to resolve action
                                               ubuntu-setup-action/install-ubuntu-system-dependencies,
                                               repository not found. Unable to resolve action
                                               windows-setup-action/install-windows-system-dependencies,
                                               repository not found.
quality / test / build (linux) / build (macos)  same, at step 1
```

Measured against GitHub itself, not inferred:

```
gh api orgs/ubuntu-setup-action     -> HTTP 404 Not Found
gh api orgs/windows-setup-action    -> HTTP 404 Not Found
gh api orgs/rustup-provided         -> HTTP 404 Not Found
```

Two more placeholder lies were measured in the same file before touching it:

```
$ cargo lint --help
error: no such command: `lint`            (cargo 1.94.0, 85eff7c80 2026-01-15)

$ cargo fmt --check
Diff in src/cli.rs:9: ...                  (it answers a diff on OUR files)

$ cargo clippy --all-targets
rc=0    errors 0    warnings 600
```

And the structural one: the committed workflow has **no `checkout` step at all**, so even if the
actions had resolved, `cargo build` would have been handed an empty directory. Five claims in the
file, four of them false, one of them (the smoke block) audited and true.

## Options considered

* **Leave the red run as the record.** Rejected now, kept as a fallback: the user asked for CI
  fixed, and a red CI on a public "documented milestone project" contradicts the release
  discipline it advertises.
* **Install the missing actions by guessing similar names.** Rejected on the project's own rule —
  inventing a shape is not our call to make. The two orgs above answer 404; the replacement names
  had to come from GitHub's marketplace/docs (`actions/checkout@v7`, `dtolnay/rust-toolchain@stable`)
  and be resolvable.
* **Keep a third-party "system dependencies" action for gtk/webkit2gtk.** Rejected: no such action
  resolves, and `apt-get` is the dependency-free truth on an Ubuntu image, so the workflow stops
  owing its correctness to someone else's unpublished org.
* **Keep `cargo fmt --check` as a gate.** Rejected by the measurement: it answers a diff on our own
  hand-written style (long `use` lists, explicit `return`). Making the tree rustfmt-clean is a
  deliberate, large, cross-file change — a milestone of its own, not a CI patch — and a gate that
  is red for a style reason teaches nothing.
* **Keep a `-D warnings` lint gate.** Rejected by our own plan: `docs/DEVELOPMENT.md` NAMES the
  build's dead-code set (`catalogue`, `open_db`, `pending_plan`'s private `PendingSql`, `Tooltip`'s
  unread fields) as intended. `-D warnings` would make the workflow contradict that document, i.e.
  lie. So clippy runs informational, at its measured 0 errors / 600 warnings.
* **Cut the matrix to Linux.** Rejected for this patch (kept as an option the user may still choose):
  the three-target matrix is the documented promise that the flat layout builds everywhere, and
  narrowing it is a different decision than fixing the names. If a target turns out to be
  unbuildable on GitHub's images, that becomes its own honest change rather than a silent narrowing.
* **Rewrite the smoke block while at it.** Rejected: those 21 lines are the sequence the local CI-mirror
  replay proves green (31 fixture GETs, 4 × 429 retried in place, 9 × 304, 3 × 404, ZERO asks for
  the manual id). Rewriting them would throw away an audited artifact; the diff of the step list
  before/after answers EMPTY.

## Decision

**One file, four truths.** `.github/workflows/ci.yml` becomes:

* every `uses:` names a resolvable action (`actions/checkout@v7` on all three jobs — the step the
  old file lacked — and `dtolnay/rust-toolchain@stable`, with `components: clippy` on the
  informational job);
* the Linux Tauri libraries come from `apt-get` (`libgtk2.0-0 libgtk2.0-dev libwebkit2gtk-4.0-dev`)
  on the jobs that link against them;
* the Linux Tauri libraries come from `apt-get`, and the SET is measured off our own binary:
  `ldd target/debug/wow-goldtool` answers `libwebkit2gtk-4.1.so.0`, `libgtk-3.so.0`,
  `libgobject-2.0.so.0`, `libsoup-3.0.so.0`, `libpango-1.0.so.0` among its NEEDED lines, so the
  dev packages are `libgtk-3-dev libwebkit2gtk-4.1-dev` — the pair the old file named (`libgtk2.0-dev`,
  `libwebkit2gtk-4.0-dev`) is wrong on TWO majors and nothing in the repo had ever proved it;
* the gates are what this tree can assert: `cargo clippy --all-targets` informational, no fmt gate,
  and the file's own header comment says why, so the workflow documents its own limits;
* the smoke block is byte-identical to the audited one (verified by diffing the parsed step list
  against the committed file: 21 steps, empty diff).

The rule this installs: **a workflow may only claim what was measured.** When the workflow's own
comments claim gates, they must be gates that pass on this tree; when they claim actions, those
actions must resolve. And when a claim turns out false — as four did here — the fix records the
measurement, because "the template lied" is a fact worth keeping, not something to paper over.

## Audit (executed, not asserted)

| probe | expectation | observed |
| ----- | ----------- | -------- |
| the old workflow's action names | resolvable or the run is red | run #1 (commit `ebee66b`) **failure after 7 s**, 5 annotations, all "Unable to resolve action" |
| the orgs behind two of those names | real org or 404 | `gh api orgs/…` → **404** for `ubuntu-setup-action`, `windows-setup-action`, `rustup-provided` |
| `cargo lint` on this toolchain | exists or is a lie | `error: no such command: lint` (cargo 1.94.0) |
| `cargo fmt --check` on this tree | clean or a diff | answers a **diff** (long `use` lists, explicit `return`) → cannot gate |
| `cargo clippy --all-targets` on this tree | passes without the plan's warnings being errors | **rc=0**, 0 errors, **600 warnings** (4 of them rustc's dead-code set) |
| the smoke block before vs after the rewrite | the audited sequence is preserved | parsed `cargo run --` lines: **21 steps both sides, diff EMPTY** |
| our binary's own NEEDED lines | which dev packages Ubuntu must install | `ldd target/debug/wow-goldtool` → `libwebkit2gtk-4.1.so.0`, `libgtk-3.so.0`, `libgobject-2.0.so.0`, `libsoup-3.0.so.0`, `libpango-1.0.so.0` → the apt line is `libgtk-3-dev libwebkit2gtk-4.1-dev`, three jobs, verified by count in the file |
| the new workflow's YAML | parses and keeps three jobs | `yaml.safe_load` → jobs `build` (5 steps), `quality` (4), `test` (5) |
| the push after the fix | the workflow is what GitHub resolves | commit `c29e45c` pushed; the follow-up run's result is recorded in docs/DEVLOG.md once it lands |
| run #2 (`c29e45c`) | the concurrency block cancels it when #3 lands | cancelled — measured |
| run #3 (`5c888b9`) | every remaining claim must hold | **failure** again: linux build + quality SUCCESS, but three new lines |
| run #3, test job, first smoke step | the scratch DB path must exist | `cannot open /fixture.sqlite: unable to open database file` → exit 4: `runner.tmpDir` is NOT a context property, the expression evaluated to the EMPTY string; the local mirror could not catch this because it hardcodes its own DB path — only a runner evaluates the env map |
| run #3, build (macos) | the triple must be installed on the image | `error[E0463]: can't find crate for std` + `the x86_64-apple-darwin target may not be installed` (rustc exit 101) while the SAME run's toolchain step answers `host: aarch64-apple-darwin` — macos-latest IS arm64 |
| run #3, build (windows) | the repo must have what tauri-build names | `` `D:\a\wow-goldtool\wow-goldtool\icons/icon.ico` not found; required for generating a Windows Resource file during tauri-build `` — and `icons/` was EMPTY locally too: the `bundle.icon` line in tauri.conf.json already named `icons/icon.png` for a file that never existed |
| the icon assets landed (`f29a2b2`) | real files, our own art, not ignored | `file` answers `PNG image data 512 x 512` and `MS Windows icon resource - 7 icons` (16..256 px); drawn by hand in PIL (dark navy card, gold coin, a W made of four strokes, never a font); `git check-ignore` answers not ignored |
| the ci rewrite for run #3 (`59b8379`) | only the two measured lies change | `WGT_DB: fixture.sqlite` (relative, `*.sqlite` gitignored at the checkout root) + the macOS row to `aarch64-apple-darwin`; YAML: build 5 / quality 4 / test 5, targets `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`; 21 smoke lines diff EMPTY |
| GitHub run #4 (`59b8379`) | green, all promises ride | **SUCCESS, 5/5 jobs in 9 min 13 s**: quality 2 m 04 s · build (aarch64-apple-darwin, macos-latest) 2 m 27 s · test 4 m 19 s · build (linux) 5 m 46 s · build (windows) 9 m 12 s — the FIRST green run in this repo's history, four runs measured total |
| GitHub run #5 (`a160ffa`, the tree the tag rides) | green AT the tag | **SUCCESS 5/5 in 8 min 01 s**: quality 2 m 18 s · test 4 m 23 s · build (linux) 5 m 37 s · build (macos) 6 m 30 s · build (windows) 8 m 01 s |
| post-tag replays at `v0.8.1` (fresh ports 9114 / 9115..9122 + 9123, fresh DBs) | the recorded audits still hold | `cargo build` 0 errors + exactly 4 warnings · mirror **21/21**, 31 GETs, `4 × 429 / 9 × 304 / 3 × 404`, ZERO manual-id asks, DB `6|4|3|1|1|0|4` · exit audit **57/57 pairs identical**, manual card 940 b, ledger 4 → 5 |
| the repo's published state | tags ride as annotated objects | `git ls-remote origin` → 24 refs = HEAD + `main` + **11 tags × 2**; `v0.8.0^{}` = `e5ac299`, `v0.7.0^{}` = `944b510` on both sides |

## The second measurement round (the runs GitHub itself started)

The fix round produced data, not confidence: run #3 got the actions right (linux
build + quality now ride green) and answered three more lies in one go — an empty
`runner.tmpDir` expression, a macOS image that is arm64 while the matrix promised
x86, and a Windows build demanding `icons/icon.ico` the repo never had. Each was
fixed at its own level: the workflow for the first two, the REPO for the third
(the icon assets commit `f29a2b2` — `bundle.icon` in tauri.conf.json already
pointed at `icons/icon.png`, a file that never existed, so this was real product
work and not CI noise at all). The lesson stays written down because it generalizes:
**a runner is a stricter witness than any local mirror** — the local mirror cannot
see env expressions, and only GitHub can say which images exist and what their
toolchains have installed.

## Deliberately not done in this patch

* no rustfmt reformat of the tree (its own milestone, and it would churn every file),
* no narrowing of the three-target matrix,
* no change to any `src/` line: the release behaviour, the 21-step mirror and the 57-door exit audit
  are untouched, and their post-tag re-runs (fresh ports 9111 / 9102..9109 + 9099) already stand,
* no GitHub-side settings changes beyond what `gh` did to create the repo (description, homepage,
  public).
