<!--- #!#tep:(tep){description="Local-first CLI for text entity pointers over code and docs"} -->
<!--- #!#tep:(sqlite.graph){description="SQLite-backed local graph storage for entities, anchors, and links"} -->
<!--- #!#tep:(ai.context){description="Retrieval-oriented use of tep for AI context assembly"} -->
<!--- #!#tep:(tep)->(sqlite.graph){description="stores graph state in"} -->
<!--- #!#tep:(tep)->(ai.context){description="supports"} -->

# tep

> ⚠️ **Fully agent-coded.** This project was built entirely by an AI coding agent (Makki) with human direction. The code, tests, docs, and this README were written by the agent.

---

## What is tep?

`tep` is a local-first CLI for **text entity pointers** — a lightweight way to connect concepts to locations in your codebase and docs.

It maintains a local SQLite graph of **entities** (things you care about), **anchors** (tagged locations in files), and **links** (relationships between entities).

---

## Why tep?

Codebases grow. Concepts scatter across files. Documentation drifts from implementation.

**tep solves this by giving concepts a durable identity.**

Instead of grepping for "student" and hoping you find the right places, you:

1. **Tag locations** — drop anchor markers in code, docs, configs
2. **Name concepts** — declare entities like `auth_flow`, `pricing_model` in files
3. **Connect them** — attach entities to anchors and declare entity links in files
4. **Query the graph** — ask "where does `auth_flow` appear?" or "what's related to `pricing_model`?"

**Use cases:**

- **Onboarding** — new devs can trace concepts across the codebase
- **Refactoring** — see everywhere a concept touches before changing it
- **Documentation** — keep docs linked to the actual code locations
- **AI context** — feed precise, relevant code slices to LLMs instead of dumping entire files
- **Architecture** — map how logical concepts relate to each other

The graph lives in your repo (`.tep/`), works offline, and stays under your control.

---

## Current capabilities

- Initialize and auto-migrate local workspaces
- Auto-declare, list, and query entities with names, descriptions, and refs
- Enforce entity name normalization (lowercase, `[a-z0-9._]`)
- Named anchor tags — place `#!#tep:[name](entity)` in any file; `anchor auto` registers and syncs them
- Auto-declare entities from declaration markers `#!#tep:(entity_name)`
- Directional entity-to-entity links reconstructed from file tags
- Assemble retrieval-oriented context bundles for entities (`entity context`)
- Bounded link traversal (`--link-depth`)
- Audit anchor health with `tep health`
- Reset and re-index the workspace with `tep reset`
- JSON output on any command via `--json`
- Respect `.tepignore` for test fixtures and examples

---

## Workspace model

A `tep` workspace is created with:

```bash
tep init
```

That creates:
- `.tep/`
- `.tep/tep.db`
- `.tepignore`

For commands that require the database, `tep` resolves the **nearest ancestor workspace** from the current working directory.

That means:
- inside a workspace tree, commands work from nested directories
- outside any workspace, DB-requiring commands fail and tell you to run `tep init`
- the location of the `tep` binary does not define the workspace
- the current terminal cwd does

## Schema and migration

`tep` stores its graph in SQLite and tracks schema version with:

```sql
PRAGMA user_version
```

Current behavior:
- `tep init` creates or upgrades the workspace DB
- normal DB-opening commands also auto-migrate older workspaces
- older DBs are upgraded in place when new columns/tables are required

## Core syntax

### Anchor tags

Anchor tags use square brackets:

```txt
<!--- #!#tep:[anchor_name](entity1,entity2) --> #tepignore
```

Rules:
- name is required — lowercase, `[a-z0-9._]`, not purely numeric
- entity ref list is required — at least one entity
- the name is the durable anchor identity
- numeric IDs are internal only (shown in `anchor list` / `anchor show`)
- `anchor auto` registers and syncs; it does **not** rewrite the tag

### Entity declaration tags

Entity declaration tags use parentheses:

```txt
<!--- #!#tep:(entity_name) --> #tepignore
```

Meaning:
- parentheses identify an entity declaration marker
- `tep auto` ensures the entity exists
- if the entity has no `ref`, the declaring file is stored as its `ref`

## Ignore controls

### `#tepignore`
Ignores only the current line.

Use it for:
- one-off example lines
- regex/test strings that look like markers
- isolated fake literals in source or docs

### `#tepignoreafter`
Ignores the rest of the file after the first occurrence.

Use it for:
- test modules
- large fixture tails
- intentionally broken example sections

Practical rule:
- prefer `#tepignore` when a few lines are noisy
- use `#tepignoreafter` when an entire tail section is fixture/test territory

## Current command surface

### Global flags
```bash
tep --json <command>   # output as JSON (works with any command)
```

### Workspace
```bash
tep init
tep reset [--yes]
tep version / -V / --version
tep health [path]
```

### Entities
```bash
tep auto <pathspec...>
tep entity auto <pathspec...>
tep entity show <name-or-id>
tep entity context <name-or-id> [--files-only] [--link-depth <n>]
tep entity list
tep e ...  (shorthand)
```

### Anchors
```bash
tep anchor auto <pathspec...>
tep anchor show <name>
tep anchor list
tep a ...  (shorthand)
```

## Entity context examples

### Get all locations and linked concepts for an entity
```bash
tep entity context anchor.parser
tep --json entity context anchor.parser   # machine-readable output
```

Output:
```
2 (anchor.parser)
ref: ./src/anchor.rs

anchor:1 anchor.parser ./src/anchor.rs (1:3) [3]
  // #!#tep:[anchor.parser](anchor.parser) #tepignore
  use crate::utils::parse::{line_contains_marker, parse_scan_limit};
  ...

links:
-> 5 (anchor.sync) [./src/service/anchor_service.rs]  anchor parser feeds anchor sync
```

### Files-only view (no snippets)
```bash
tep entity context anchor.parser --files-only
```

### Follow linked entities two hops out
```bash
tep entity context anchor.parser --link-depth 2
```

### Show compact graph shape
```bash
tep entity show anchor.parser
```

## Health and reset

`tep health` performs a read-only anchor health audit:
```bash
tep health
tep health ./src
```

`tep reset` wipes and re-indexes the workspace:
```bash
tep reset          # prompts for confirmation
tep reset --yes    # skip prompt
```

## Output style

All commands support `--json` for machine-readable output. Place it before the subcommand:
```bash
tep --json entity list
tep --json anchor show anchor.parser
tep --json entity context anchor.sync --link-depth 2
tep --json health
```

Default output is intentionally compact.

Entity header:
```txt
<id> (<name>)
ref: <path>
description: <text>
```

Anchor line:
```txt
anchor:<id> <name> <file> (<line>:<shift>) [<offset>]
```

Link line:
```txt
-> <id> (<name>) [<ref>]  <relation>
<- <id> (<name>) [<ref>]  <relation>  [depth:<n>]
```

## Notes

- entity IDs are integer-only
- entity names are unique, normalized to lowercase, charset `[a-z0-9._]`
- entity names cannot be purely numeric
- entity metadata includes `ref` and `description`
- entity links are directional in storage
- anchor names are required, unique, same charset as entity names
- anchor names cannot be purely numeric
- anchor-entity relations are managed through the tag's entity ref list
- `anchor show <name>` accepts name or numeric id
- `.tepignore` is respected; `.gitignore` is not
- `line`, `shift`, and `offset` are refreshable metadata only; anchor identity is the name in the tag

## Repo self-check

The `tep` repo uses `tep` to track its own entities and anchors.
`tep health` in the repo root reports clean for all tracked source and doc files.

## Documentation map

### Product / overview
- [Concept](./CONCEPT.md)
- [Data Model](./DATA_MODEL.md)
- [CLI Design](./CLI_DESIGN.md)
- [Use Cases](./USE_CASES.md)
- [Roadmap](./ROADMAP.md)
- [Open Questions](./OPEN_QUESTIONS.md)
- [Development Notes](./DEV_NOTES.md)

### Specs
- [Spec Index](./spec/README.md)

### Internal docs
- [Internal Doc Index](./doc/README.md)
