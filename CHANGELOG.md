# Changelog

All notable changes to **WoW Goldtool** are documented here, following
[SemVer](https://semver.org/) and [Conventional Commits](https://www.conventionalcommits.org/).
`Added` / `Changed` / `Fixed` / `Removed` follow the [Keep a Changelog](https://keepachangelog.com/)
sections. The full narrative of *why* each change was made lives in
[docs/DEVLOG.md](./docs/DEVLOG.md); each entry below points at the matching log entry and git tag.

## v0.2.0 — data layer + working sync · 2026-11-19 · tag `v0.2.0`

Added

- `src/` headless core, one buildable unit: `errors.rs`, `models.rs`, `wowdb.rs`,
  `db.rs`, `query.rs`, `stats.rs`, `sync.rs`, `cli.rs` plus `main.rs` argv dispatch —
  the six CLI verbs of [ADR 0004](./docs/DECISIONS/0004-headless-cli-mode.md).
- SQLite storage ([docs/DATABASE.md](./docs/DATABASE.md)): wide `item` row plus
  `meta`/`watch`/`sync_state`, list + detail upserts, `--db PATH` overriding `WGT_DB`.
- HTTP client with pacing by construction: 90-request headroom under the 120-request
  burst, 2 req/s schedule, `429` honours `Retry-After` and never aborts.
- Both sync stages verified end-to-end against a fixture server, incl. retry-in-place
  for a `429`'d item and an honest `complete`/`partial` report derived from the pending
  count.
- CI smoke test is now live (was commented in the scaffold tag): fixture server
  `tests/fixture_api.py` + `sync --stage list` + `sync --stage detail` + `stats`.
- `Cargo.toml` version 0.2.0 with the pinned dependency line unchanged. `tauri.conf.json`
  joins the tree at v0.4.0 (GUI shell), so its version bump lands there — see the DEVLOG
  note on the amended house rule.

## v0.1.0 — scaffold · 2026-11-19 · tag `v0.1.0`

Added

- Repository layout: `src/` (Rust), `ui/` (frontend assets), `docs/`, `.github/`.
- Project documentation: [DEVELOPMENT](./docs/DEVELOPMENT.md) (architecture, roadmap, house
  rules), [API](./docs/API.md) (TheWoWDB notes: paging, ETag revalidation, rate limits),
  [DATABASE](./docs/DATABASE.md) (SQLite schema + query recipes), [DECISIONS](./docs/DECISIONS/)
  (ADR-style records).
- `README.md` with the roadmap and the required data attribution, `CHANGELOG.md`,
  `LICENSE` (MIT + data note), `.gitignore` for Rust/Tauri/IDE artefacts.
- GitHub-style repository metadata: issue + PR templates, CI workflow (`build`, `test`,
  `stats` on a seeded fixture DB).
- Default `hello world` binary kept untouched (base crate compiles as-is).
