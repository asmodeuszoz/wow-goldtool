---
name: Feature request
about: A new command, UI affordance, query or data-layer capability
title: "feat: "
labels: enhancement, triage
---

<!--
Check docs/DECISIONS/ first so the request does not contradict an accepted ADR, and
docs/DEVLOG.md for follow-ups already parked (packaging, retail catalogue, tooltips).
-->

### What you want

One paragraph, then the concrete behaviour you expect to see.

### Why it matters

What you are trying to do with the catalogue (prices, filters, sorts, watchlist). The
product question this app always tries to answer is: *what does this item cost, and is it
worth hunting for?*

### Where it would live

Name the module from the docs/DEVELOPMENT.md module map (`wowdb.rs`, `sync.rs`,
`db.rs`, `query.rs`, `stats.rs`, `commands.rs`, `ui/app.js`) and say why that module is
the right home.

### Cost and constraints

If the idea touches the API, state the request cost against the documented rate model
(120 request burst, refilled at 2/s — docs/API.md) and whether ETag reuse would help.

### Pre-flight

- [ ] I read docs/DECISIONS/ and this does not contradict an accepted ADR
- [ ] I checked docs/DEVLOG.md for parked follow-ups
