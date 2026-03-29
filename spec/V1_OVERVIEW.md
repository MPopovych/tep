# V1 Overview

This document captures the current intended shape of the first meaningful `tep` implementation.

## Core model

V1 is based on four primary stored concepts:
- entities
- anchors
- anchor-entity relations
- entity links

## Important identity rules

### Entity identity
Entities have:
- an integer `entity_id`
- a unique `name`
- an optional `ref`
- an optional `description`

Commands can resolve entities by either:
- integer id
- unique name

### Anchor identity
Materialized anchor tags contain the anchor ID.

Examples:
```txt
[#!#1#tep:123](student)
[#!#1#tep:124](student,basic-user)
```

Rules:
- the value after `tep:` is the anchor ID
- the `:` is the anchor ID slot
- the anchor ID is the durable anchor identity
- the optional `( ... )` suffix is an entity reference instruction list, not identity

### Location metadata
File path, `line`, `shift`, and `offset` are useful metadata.
They are not durable truth.

Meaning:
- `line` = last-known line index
- `shift` = last-known byte shift within the line
- `offset` = last-known byte offset in the file

Files move and change over time.
The durable identity is the anchor ID.

## Relationship model

### Entities ↔ anchors
Many-to-many.

That means:
- one entity may point to many anchors
- one anchor may point to many entities

### Entities ↔ entities
Links are:
- directional in storage
- free-text described
- simple in the first version

Current simplification:
- one link per ordered pair
- no link priorities

## Workspace schema behavior

Current behavior:
- workspace DBs track schema version via `PRAGMA user_version`
- opening a workspace may auto-migrate legacy schemas
- `tep init` ensures the DB is at the current schema version

## Entity command direction

Current commands:
- `tep entity create <name> [--ref <value>] [--description <value>]`
- `tep entity ensure <name> [--ref <value>]`
- `tep entity auto <pathspec...>`
- `tep entity show <name-or-id>`
- `tep entity context <name-or-id> [--files-only] [--link-depth <n>]`
- `tep entity edit <name-or-id> [--name <value>] [--ref <value>] [--description <value>]`
- `tep entity link <from> <to> --relation <text>`
- `tep entity unlink <from> <to>`
- `tep entity list`

Shorthand:
- `tep e ...`

## Anchor command direction

Current commands:
- `tep anchor auto <pathspec...>`
- `tep anchor show <anchor-id>`

Shorthand:
- `tep a ...`

Current behavior:
- `anchor auto` processes targeted files or directories
- materializes incomplete anchors
- refreshes existing anchor metadata
- binds anchors to entities when an entity reference instruction is present
- removes stale anchor state for dropped anchors in the targeted files
- `anchor show` returns the anchor plus related entities

## Workspace behavior

Current behavior:
- `tep init` creates a workspace in the current directory
- DB-requiring commands resolve the nearest ancestor workspace from cwd
- commands outside any workspace fail clearly

## Output style

Default human-readable output stays concise.

Entity format:
```txt
<id> (<name>)
```

Anchor format:
```txt
<anchor_id>
<file> (<line>:<shift>) [<offset>]
```

`entity context` is more retrieval-oriented and includes linked entities by default.
Direction is preserved in edge notation rather than as separate query modes.

## V1 implementation priorities

The first useful implementation slice focuses on:
- workspace init
- automatic workspace DB migration
- schema version tracking
- entity create
- entity ensure
- entity auto
- entity show
- entity context
- entity edit
- entity list
- anchor auto
- anchor show
- attach
- detach
- directional entity links

## V1 non-goals

Not required for early versions:
- AST-aware parsing
- semantic inference
- link priorities
- editor plugin support
- remote sync
- cloud-first behavior
