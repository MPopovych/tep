# Planning

## Workflow

- All work happens on feature branches: `TEP-1`, `TEP-2`, etc.
- Changes land via pull requests into `main`.
- Releases are tagged: `v1.0.0`, `v1.0.1`, etc.

---

## v1.0.1 Milestones

### TEP-1 — JSON output

Services currently return domain structs directly to formatters. The goal is to introduce DTO objects as an intermediate layer, so output can be rendered as either the current human-readable format or structured JSON.

- Services return DTO structs (not raw domain objects)
- Formatters accept DTOs and produce either text or JSON
- JSON output enabled via a global `--json` flag or per-command
- Useful for tooling, plugins, and piping into other tools

### TEP-2 — Richer tag syntax

Goal: `tep auto .` should be able to reconstruct the full graph from scratch — entities, anchors, relations, links, and metadata — without any manual `tep entity link` or `tep entity edit` calls.

New tag types to design and implement:

- **Entity relation tags** — declare links between entities directly in files
- **Entity metadata tags** — declare `ref`, `description`, or other metadata inline
- **Anchor metadata tags** — additional anchor-level metadata (e.g. description, weight, custom fields)

After this milestone, a full `tep reset --yes` should restore the complete graph without any manual steps.

### TEP-3 — VS Code plugin preparation

Groundwork to make a VS Code extension viable:

- JSON output (TEP-1) is a prerequisite
- Define the plugin's expected interface (hover, go-to, search)
- Likely: a language server or simple JSON-RPC wrapper around `tep` CLI
- Evaluate whether a Rust-native LSP approach or a thin Node.js extension shell makes more sense

---

## Branch naming

| Branch | Milestone |
|--------|-----------|
| `TEP-1` | JSON output / DTO layer |
| `TEP-2` | Richer tag syntax / full `tep auto` |
| `TEP-3` | VS Code plugin preparation |
| `TEP-N` | Next task in sequence |
