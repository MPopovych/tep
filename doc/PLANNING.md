# Planning

## VS Code Plugin — Interface Plan

### Plugin features
- **Entity browser** — sidebar listing all entities with name, ref, description
- **Hover on anchor tag** — show linked entities, descriptions, refs
- **Hover on entity ref** — show entity info inline
- **Go to definition** — open the entity's `ref` file

### Plugin architecture
- Thin Node.js/TypeScript extension — shells out to `tep` CLI, parses JSON, renders in VS Code UI
- No Rust in the plugin itself

### Commands the plugin calls

| Command | When |
|---|---|
| `tep workspace info --json` | Extension activate — confirm workspace, get root |
| `tep entity list --json` | Populate entity browser |
| `tep entity show <name> --json` | Hover on entity ref, go-to-definition |
| `tep anchor show <name> --json` | Hover on anchor tag |
| `tep anchor at <file> <line> --json` | Cursor position → nearest anchor (NEW) |

### Two new commands needed (TEP-3)

**`tep anchor at <file> <line>`** — returns anchor at/near a file position. Drives hover via cursor coordinates.

**`tep workspace info`** — returns root path + basic stats. Needed by plugin on startup to locate workspace root and resolve relative paths.

### JSON shapes

```jsonc
// entity list
[{ "id": 1, "name": "auth.flow", "ref": "./src/auth.rs", "description": "..." }]

// entity show
{
  "entity": { "id": 1, "name": "auth.flow", "ref": "...", "description": "..." },
  "anchors": [{ "id": 7, "name": "auth.flow", "file": "...", "line": 42, "shift": 0, "offset": 1200 }],
  "links": [{ "direction": "->", "entity": { "id": 2, "name": "token" }, "relation": "produces token", "depth": 1 }]
}

// anchor show
{
  "anchor": { "id": 7, "name": "auth.flow", "file": "...", "line": 42, "shift": 0, "offset": 1200 },
  "entities": [{ "id": 1, "name": "auth.flow", "ref": "...", "description": "..." }]
}

// anchor at <file> <line>  (new)
{ "anchor": { ... } | null, "entities": [...] }

// workspace info  (new)
{ "root": "/abs/path", "entities": 15, "anchors": 22, "schema_version": 4 }
```

---



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
