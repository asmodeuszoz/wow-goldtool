<!--
Keep the title in Conventional Commits form: feat(scope): …, fix(db): …, docs(devlog): …
The squash-merge title becomes the commit message, so the title is the last line of
history for this change. Put the *why* in the body.
-->

## What

One paragraph: what this change does, and the milestone it belongs to
(`v0.2.0` data layer, `v0.3.0` sync hardening, `v0.4.0` UI shell, `v0.5.0` polish).

## Why

The problem being solved, and why this shape was chosen over the alternatives considered.
If it changes a decision, add/replace an ADR in `docs/DECISIONS/` instead of arguing in
the thread.

## Pre-flight

- [ ] Branch is `main`, history stays linear (squash merge, no merge commits)
- [ ] Commit title(s) follow Conventional Commits, body explains the *why*
- [ ] Version bump (if any) lands in **both** `Cargo.toml` and `tauri.conf.json`
- [ ] `CHANGELOG.md` entry written for the released version
- [ ] `docs/DEVLOG.md` dated entry written for anything measured or discovered
- [ ] `docs/DEVELOPMENT.md` module map still matches the tree
- [ ] Docs updated: `docs/API.md` for API behaviour, `docs/DATABASE.md` for schema/queries
- [ ] `cargo build` passes on every commit of the change (bisectable history)
- [ ] `cargo test` passes; new behaviour covered by a test that does not hit the live API
- [ ] Visible attribution link to TheWoWDB still rendered in the UI (`ui/index.html`)

## How to test

```bash
# exact commands a reviewer should run, plus the expected result
cargo run -- status
cargo run -- search "Worn Shortsword"
```

## Notes / risk

- Rate-limit impact: requests added, ETag reuse, worst-case wall clock.
- Anything a later reader must know: follow in `docs/DEVLOG.md`.
