<!--- #!#tep:(cli.design){description="CLI design notes for command structure, syntax, and UX"} -->
<!--- #!#tep:(cli.design)->(tep){description="defines user-facing interface for"} -->
# CLI Design

## Design goals

The CLI should be:
- simple
- scriptable
- local-first
- readable
- predictable

The goal is to manage a small local graph of:
- entities
- anchors
- anchor-entity associations
- entity links

## Git-like similarities

The similarity to git is mostly ergonomic:
- one binary
- subcommands
- local workspace awareness
- nearest-ancestor workspace discovery

`tep` is not a version-control system.

<!--- #!#tep:[cli.workspace](cli.design,workspace) -->
## Workspace behavior

### Initialize a workspace
```bash
tep init
```

This creates:
- `.tep/`
- `.tep/tep.db`
- `.tepignore`

`init` also ensures the DB schema is current.

### Resolve the active workspace
For DB-requiring commands, `tep` starts from the current working directory and walks upward until it finds the nearest ancestor workspace.

Important:
- the current cwd matters
- the binary location does not
- commands from nested directories inside a workspace should work
- commands outside any workspace should fail clearly

### Schema versioning and migration
`tep` tracks schema version in SQLite via:

```sql
PRAGMA user_version
```

Current behavior:
- `tep init` creates or upgrades the DB schema
- normal DB-opening commands also auto-migrate legacy workspaces
- users should not need a separate migrate command for routine upgrades

## Current command areas

### Workspace
```bash
tep init
tep version
tep -V
tep --version
tep health [path]
```

### Entities
```bash
tep entity create <name> [--ref <value>] [--description <value>]
tep entity ensure <name> [--ref <value>]
tep entity auto <pathspec...>
tep entity show <name-or-id>
tep entity context <name-or-id> [--files-only] [--link-depth <n>]
tep entity edit <name-or-id> [--name <value>] [--ref <value>] [--description <value>]
tep entity link <from> <to> --relation <text>
tep entity unlink <from> <to>
tep entity list
```

Shorthand:
```bash
tep e ...
```

### Anchors
```bash
tep anchor auto <pathspec...>
tep anchor show <name>
tep anchor list
```

Shorthand:
```bash
tep a ...
```

<!--- #!#tep:[cli.syntax](cli.design,anchor.parser,entity.declaration) -->
## Marker syntax

### Anchor tag
```txt
<!--- #!#tep:[anchor_name](entity1,entity2) --> #tepignore
```

- name is required: lowercase, `[a-z0-9._]`, not purely numeric
- entity ref list is required (at least one)
- the name is the durable identity; numeric IDs are internal

### Entity declaration tag
```txt
<!--- #!#tep:(student) --> #tepignore
```

Bracket type decides the role:
- `[...]` = anchor
- `(...)` = entity declaration

## JSON output

All commands support a global `--json` flag:

```bash
tep --json entity list
tep --json entity show <name>
tep --json entity context <name> [--link-depth <n>]
tep --json anchor show <name>
tep --json anchor list
tep --json health
```

JSON output uses a stable DTO schema (see `src/dto.rs`). Intended for tooling and editor plugins.

## Output style

Default output should stay compact.

Entity header:
```txt
<id> (<name>)
```

Anchor block:
```txt
<anchor_id>
<file> (<line>:<shift>) [<offset>]
```

Location metadata is useful, but not identity.
The durable anchor identity is the anchor ID.

`entity context` is intentionally more retrieval-oriented.
It may include:
- `ref`
- `description`
- anchor snippets
- file shortlist
- linked entities with explicit edge notation

Example edge line:
```txt
edge: (1->2)[1] student has subjects
```

## Internal implementation notes

Recent internal cleanup points:
- shared path logic lives in `src/utils/path.rs`
- shared time helper lives in `src/utils/time.rs`
- shared output rendering helpers live in `src/output/render.rs`
- service constructors can now be workspace-root aware for more reliable tests
- command-layer boilerplate is reduced through shared support helpers

## Non-goals for current scope

Not part of the current implemented CLI surface:
- `status`
- `doctor`
- `scan`
- `resolve`
- `graph`
- `context get`
- `attach` / `detach` top-level commands (relations managed through tag syntax)
- `anchor edit` (rename command removed)
- a required explicit migrate command for routine schema upgrades

Those can be revisited later, but the docs should not present them as current commands.
