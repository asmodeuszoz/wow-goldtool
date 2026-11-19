# Changelog

All notable changes to **WoW Goldtool** are documented here, following
[SemVer](https://semver.org/) and [Conventional Commits](https://www.conventionalcommits.org/).
`Added` / `Changed` / `Fixed` / `Removed` follow the [Keep a Changelog](https://keepachangelog.com/)
sections. The full narrative of *why* each change was made lives in
[docs/DEVLOG.md](./docs/DEVLOG.md); each entry below points at the matching log entry and git tag.

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
