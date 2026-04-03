# Entity Commands Spec

This document captures the current entity-facing behavior in `tep`.

## Entity data model

Current entity shape:
- `entity_id` integer
- `name` unique
- `ref` nullable
- `description` nullable
- `created_at`
- `updated_at`

## Identity and lookup rules

Entity commands may resolve entities by:
- integer id
- unique name

## Naming rules

Entity names should:
- be unique
- use lowercase `[a-z0-9._]`
- support dot-style names such as `student.permissions`
- not be purely numeric

## Workspace requirement

All entity commands except help/version behavior require a `tep` workspace.

Current behavior:
- `tep init` creates or upgrades the workspace in the current directory
- DB-requiring commands resolve the nearest ancestor workspace from cwd
- opening a legacy workspace may auto-migrate the DB schema
- commands outside any workspace fail clearly and suggest `tep init`

## Current command set

### Auto
```bash
tep auto ./file.md
tep auto ./docs
tep auto .
tep entity auto ./src
```

Behavior:
- scans targeted files for entity declaration tags and relation tags
- entity declaration syntax:
  ```txt
  #!#tep:(student) #tepignore
  #!#tep:(student){ref="./docs/student.md", description="A learner"} #tepignore
  ```
- relation syntax:
  ```txt
  #!#tep:(student)->(subject){description="has subject"} #tepignore
  ```
- ensures declared entities exist
- fills `ref` from metadata when present
- otherwise fills missing `ref` from the declaring file path
- fills `description` from metadata when present
- syncs directional entity links from relation tags
- unknown metadata fields do not fail sync; they surface as warnings
- duplicate/conflicting metadata resolves last-write-wins and surfaces as warnings
- if a line contains `#tepignore`, tags on that line are ignored
- if a file contains `#tepignoreafter`, tags after the first occurrence are ignored

### Show
```bash
tep entity show "student"
tep entity show 42
```

Behavior:
- accepts either unique name or entity id
- prints compact entity data
- includes related anchors
- includes `description` when present
- includes outgoing and incoming links when present

### Context
```bash
tep entity context "student"
tep entity context "student" --files-only
tep entity context "student" --link-depth 2
```

Behavior:
- returns a retrieval-oriented view of the entity
- includes `ref`
- includes related anchors and snippets by default
- includes linked entities
- preserves link direction in rendered edge notation
- `--files-only` skips anchor snippets
- `--link-depth` bounds traversal depth

### List
```bash
tep entity list
```

Behavior:
- prints entities in a compact CLI-friendly way
- auto-migrates old workspace schemas before querying when needed

## Output expectation

Compact entity format:
```txt
<id> (<name>)
```

When anchors are included, each anchor uses the shared compact anchor format.

For context output, linked entities are rendered with explicit edge notation.

## Storage model

Current storage model:
- SQLite integer primary key for `entity_id`
- unique constraint on `name`
- nullable `description` on entities
- directional entity links with one link per ordered pair
- schema version tracked via `PRAGMA user_version`

## Removed command surface

The following older manual commands are no longer the current model:
- `tep entity create`
- `tep entity ensure`
- `tep entity edit`
- `tep entity link`
- `tep entity unlink`

The current model is file-driven and reconstructed through `tep auto`.