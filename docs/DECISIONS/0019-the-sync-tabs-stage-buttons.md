# ADR 0019 — the Sync tab's stage buttons: a click stays exactly the argv it always was

**Status:** adopted (v0.9.8)
**Related:** `ui/index.html`, `ui/app.js` (the whole change is webview-side), [ADR 0014](./0014-the-icon-doors.md)
(the click == cargo-line rule this rides), [ADR 0018](./0018-the-panel-frame-and-tabs.md)
(the Sync tab these buttons live in), [ADR 0017](./0017-the-live-min-buyout.md) (the price stage +
its once-per-record-EVER rule), [ADR 0002](./0002-two-stage-sync.md)/[ADR 0004](./0004-headless-cli-mode.md)
(chunks, resume, exit codes), [ADR 0006](./0006-checked-missing-third-state.md) (every stage's third
state), `src/cli.rs` (stage dispatch — UNTOUCHED: the verbs already existed)

## The observation that forced it

The v0.9.6 and v0.9.7 records both scoped the panel's crawl buttons to the FIVE (list · pull-all ·
details:watch · details:hits · Cancel) and said, honestly, "the icons/tooltip/price stages stay
CLI-side, said so, not slipped in". The user then asked for exactly those buttons (and then for the
live-sort knob): the panel should be able to run the stages it already prints notes about. The
engine needs NOTHING new — `sync --stage icons|tooltip|price` are already the CLI's own verbs since
v0.6.0/v0.8.0/v0.9.6; only the panel lacked doors to them.

## The decision

**Landed:** three buttons in the Sync tab's action row — `Icons: copy the files` · `Descriptions:
copy` · `Price: live refresh` — all wired to the SAME `runSync(stage)` loop the panel already had:
same chunk size (25), same cancel-at-chunk-end plumbing, same `sync_state` resume, same "no progress
this chunk → stop" guard, and same `cli_ok(["sync","--stage",X,"--budget",chunk])` argv — ADR 0014's
rule restated: a click and a `cargo run --` line are ONE code path down to the report JSON; nothing
in this release touches Rust at all (`git diff` names `ui/index.html` + `ui/app.js` + banner).

**The notes moved the SAME breath so none can lie beside the buttons:** the card's static note
("CLI-side stage" → "the Sync tab's Price button or the CLI stage", and the API-touch claim gets its
honest card-scope), the card's dynamic sentence (names the door that exists), the live-row comment
(the v0.9.6 "the panel has no button for this stage" claim is replaced with what is true SINCE this
release — the gui.rs facts stay: the Rust shell still builds no card and reads no column; the button
is webview-side).

**The completion note grows one honest clause:** when the stage's report carries `missing > 0`,
`, N answered the stage's own third state (checked once, never asked again)` — deliberately generic
("the stage's OWN third state"): detail = gone upstream, icons/tooltip = file/copy 404, price = the
AH offers nothing right now (ADR 0017's measured third state). Reports without the field keep their
note BYTE-IDENTICAL — no `missing 0` noise invented for stages that do not count.

**Rejected / deferred, recorded where they stand:**
* **Chaining price into Pull-all-info** — REJECTED: the chain's contract is "the catalogue is
  complete" (list→detail→icons→tooltip), the price refresh re-asks a LIVE pair under the once-per-
  record-EVER rule (ADR 0017) — chaining it would make a second pull-all ask zero and say nothing
  new; the button is its own activity, and the record says so where the chain's note lives.
* **Per-button budget dials** — nothing asked for one: the loop's chunk IS the budget knob and
  Cancel is the halt door, already the panel's shape.
* **Disabling buttons while another crawl runs** — the existing `sync_busy` guard already answers
  "a crawl is already running — cancel it first" (measured behavior of the OTHER buttons, same door).

## What stays true under the new buttons — and how it is known

| rule | how it survived |
| ---- | ------------- |
| ADR 0014 click==cargo line | the buttons call `runSync`, which sends the argv shape unchanged since v0.2.0; the CLI's 33-step CI block keeps running the SAME stage verbs the buttons reach (block UNCHANGED this release: still 33 step-lines + plant — the post-tag replay re-proves the wire) |
| ADR 0004 exit codes | every button's verb answers through the SAME cli_ok envelope: kind/exit 2/3/4 land as notes (the usage/DB/network doors cannot come from these argvs, but the envelope path is the same one the map audits) |
| ADR 0017 once-EVER pending | a second Price click asks ZERO: pending rule = copy EXISTS + never written + never checked-empty — measured CLI-side at v0.9.6 on the very same verb |
| ADR 0006 third states | stamps ride the writes (icon/tooltip `*_missing_at`, price `price_now_missing_at`) — the UI reads them back through the SAME card doors, unchanged |
| the live min buyout row | shows the stage's ASK-TIME observation (button or CLI); the note's and comment's claims were rewritten to match — the row NEVER pretends what filled it |
| offline rule | buttons send requests when clicked (crawl buttons ALWAYS did: list/detail/watch) — the CARD still answers offline, its note says so |

## Measured (pre-tag; the webview itself is unlaunchable here — recorded, never claimed)

* `node --check` parses app.js (PARSE witness); grep witnesses: 3 button ids in index.html, 3
  wirings in app.js, the third-state clause x1 (its text quoted above);
* `git diff --stat` for the release names ONLY `ui/index.html`, `ui/app.js`, `Cargo.toml`, `Cargo.lock`
  (banner commit) — ZERO Rust lines: every door, stamp rule and exit code this release could touch is
  byte-unchanged BY CONSTRUCTION, and the post-tag replays (mirror replay on fresh ports, wire byte
  pairs vs the v0.9.7 binary, the exit maps re-run) measure that, not argue it — landing FORWARD in
  the verification fold;
* builds: exit 0, FOUR named warnings the whole way (the FS's hard-link cache line counted a fifth
  LINE where a build skipped the incremental-clear — the mechanic's pattern is recorded since v0.9.7).

## Chain

`058d21b` feat(ui) (buttons + the four honest note moves + the generic clause) → `d60bc84` chore (banner 0.9.8, lock rides) → the record commit carries the annotated tag `v0.9.8`. v0.9.7's verification-fold push (`0236479`, head `0236479…`) started **run 44973516964** and answered, as measured: **success, 5/5 in 10 min 58 s** (01:53:08Z -> 02:04:06Z — quality 2 m 09 s · test 5 m 19 s · linux 6 m 28 s · macos 5 m 31 s · windows 10 m 55 s), exactly ONE run id for that head, no CANCELLED sibling, no rerun (matched by id/head_sha; the run's own .attempt reads null — the ladder's habit). It was IN FLIGHT while this chain built (the observed rule keeps this release's pushes waiting; local building did not); its verdict folds HERE, FORWARD, this record. v0.9.8's own chain+tag push `0236479..9aaa3f6` (head `9aaa3f6…` — the TAGGED tree) ran **run 44975077893**: **success, 5/5 in 10 min 45 s** (02:13:34Z -> 02:24:19Z — quality 2 m 03 s · test 5 m 03 s · linux 5 m 25 s · macos 4 m 54 s · windows 10 m 42 s), exactly ONE run id for that head, no CANCELLED sibling, no rerun. The verification fold waited (LOCAL while that run flew — the observed rule), GREW this verdict by amend and pushed: only the fold's OWN push lands FORWARD in the next record, never chased. Tags stay put (`v0.9.7` → `dfb8826`, `v0.9.8` → `9aaa3f6`).
