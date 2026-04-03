# Changelog

## TEP-2 — Richer tag syntax and auto-driven graph reconstruction

### Added
- Unified tag family with shared `#!#tep:` prefix
- Entity declaration tags:
  - `#!#tep:(entity) #tepignore`
  - `#!#tep:(entity) #tepignore{ref="...", description="..."}`
- Relation tags:
  - `#!#tep:(a)->(b) #tepignore`
  - `#!#tep:(a)->(b) #tepignore{description="..."}`
- Anchor tags:
  - `#!#tep:[anchor](a,b)` <!-- #tepignore -->
  - `#!#tep:[anchor](a,b){description="..."}` <!-- #tepignore -->
- Top-level `tep auto <path...>` command to run entity + anchor sync together
- Anchor description persistence in the database
- Health/reporting warnings for:
  - unknown metadata fields
  - duplicate metadata keys
  - last-write metadata overwrites

### Changed
- `tep auto` is now the primary indexing workflow
- Entity relations and metadata are reconstructed from file tags instead of manual CLI mutation
- Schema upgraded to v4 to store anchor descriptions
- Source examples, docs, specs, and internal tag comments updated to TEP-2 syntax

### Removed
- Manual entity mutation commands:
  - `tep entity create`
  - `tep entity ensure`
  - `tep entity edit`
  - `tep entity link`
  - `tep entity unlink`

### Notes
- Unknown metadata fields do not fail parsing or sync
- Duplicate/conflicting metadata resolves as last-write-wins and surfaces via warnings
- `tep reset --yes` can now rebuild much more of the graph directly from files

---

## TEP-1 — DTO layer and JSON output

### Added
- DTO layer between services and output rendering
- Global `--json` support across commands
- Structured JSON output for entity, anchor, health, and workspace flows

### Changed
- Command rendering now goes through DTO transformation before text/JSON output
- Output formatting is more consistent between human-readable and machine-readable modes

### Why it mattered
- Enabled tool/plugin integrations
- Prepared the codebase for richer automation and editor integration
- Made later TEP-2 work cleaner by separating domain behavior from presentation
