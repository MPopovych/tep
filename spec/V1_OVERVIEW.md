# V1 Overview

This document captures the current shape of the `tep` implementation.

## Core model

Four primary stored concepts:
- entities
- anchors
- anchor-entity relations
- entity links

## Identity rules

### Entity identity
- integer `entity_id` (internal)
- unique `name` (the user-facing identity)
- optional `ref`
- optional `description`

Resolved by either integer id or name.

### Anchor identity
The anchor name in the tag is the durable anchor identity.

Example:
```txt
#!#tep:[student_processor](student) #tepignore
```

The name `student_processor` is the identity. Numeric IDs are internal and shown in list/show output.

Anchors without entity refs are ignored. Anchors without names are ignored.

### Location metadata
`line`, `shift`, and `offset` are metadata, not identity. They are refreshed on each `anchor auto` run.

## Relationship model

### Entities ↔ anchors
Many-to-many. One entity may have many anchors; one anchor may reference many entities.

Relations are managed via the entity ref list in the tag and synced by `anchor auto`.

### Entities ↔ entities
Links are:
- directional in storage
- free-text described
- one link per ordered pair

## Workspace behavior

- `tep init` creates a workspace in the current directory
- DB-requiring commands resolve the nearest ancestor workspace from cwd
- commands outside any workspace fail clearly

Schema version tracked via `PRAGMA user_version`. Opening a workspace auto-migrates older schemas.

## Command surface

### Workspace
```bash
tep init
tep version / -V / --version
tep health [path]
```

### Entities
```bash
tep entity create <name> [--ref <value>] [--description <value>]
tep entity ensure <name> [--ref <value>]
tep auto <pathspec...>
tep entity show <name-or-id>
tep entity context <name-or-id> [--files-only] [--link-depth <n>]
tep entity edit <name-or-id> [--name <value>] [--ref <value>] [--description <value>]
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

## Output style

Entity:
```txt
<id> (<name>)
```

Anchor:
```txt
<anchor_id> (<name>)
<file> (<line>:<shift>) [<offset>]
```

`entity context` is retrieval-oriented: includes ref, description, anchor snippets, files, and linked entities with edge notation.

## Non-goals

- AST-aware parsing
- semantic inference
- editor plugin support
- remote sync
- cloud-first behavior
- attach/detach CLI commands (relations managed through tag syntax only)
