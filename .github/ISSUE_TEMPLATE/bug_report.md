---
name: Bug report
about: The app, the sync or the data layer does not behave as documented
title: "bug: "
labels: bug, triage
---

<!--
Please read docs/DEVELOPMENT.md (module map + house rules) and skim docs/DEVLOG.md
for known caveats before filing. Most "why is it built this way" questions are
already answered in docs/DECISIONS/.
-->

### What happened

What you expected versus what you got, in plain words.

### How to reproduce

Prefer the headless CLI — it reproduces most data-layer bugs without a display server
(see docs/DEVELOPMENT.md). Paste the exact command and its output.

```
cargo run -- status
cargo run -- sync --stage list
```

### Environment

- app version (`cargo run -- status`, or `tauri.conf.json > version`):
- OS / desktop:
- `rustc` + `cargo` version:
- display server (X11 / Wayland) if the problem is about the UI:

### Relevant logs

GUI: stderr around the failure. CLI: the JSON error object. Sync: the `sync_state`
row plus the DB path printed by `cargo run -- status`.

### Pre-flight

- [ ] I read docs/DEVELOPMENT.md (module map + house rules)
- [ ] I checked docs/DEVLOG.md for a known caveat
- [ ] I redacted anything sensitive
