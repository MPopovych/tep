# V1 Overview

This document captures the current intended shape of the first meaningful `tep` implementation.

## Core model

V1 is based on four primary stored concepts:
- entities
- anchors
- anchor-entity relations
- links

## Important identity rules

### Entity identity
Entities should have:
- an integer `entity_id`
- a unique `name`
- an optional `ref`

Commands should be able to resolve entities by either:
- integer id
- unique name

### Anchor identity
Materialized anchor tags contain the anchor ID.

Planned examples:
```txt
[#!#1#tep:123763636473](student)
[#!#1#tep:123763636474](student,basic-user)
```

Rules:
- the value after `tep:` is the anchor ID
- the `:` is the anchor ID slot
- the anchor ID is the durable anchor identity
- the optional `( ... )` suffix is an entity reference instruction list, not the durable anchor identity

### Location metadata
File path, `line`, `shift`, and `offset` are useful metadata.
But they are not durable truth.

Planned meaning:
- `line` = untrusted / last-known line index derived from newline boundaries
- `shift` = untrusted / last-known amount of symbols or bytes before the tag within the line
- `offset` = untrusted / last-known file offset of the tag

Files move and change over time.
The durable identity is the anchor ID.

## Relationship model

### Entities ↔ anchors
Many-to-many.

That means:
- one entity may point to many anchors
- one anchor may point to many entities

This relation should have its own dedicated schema/table.

### Entities ↔ entities
Links are:
- directional
- typed
- prioritized

## Priority rule

Priority is user-defined.

Convention:
- `1` is best / highest priority
- larger numbers are lower priority
- the maximum cap is user- or project-defined

Primary usage:
- sorting retrieval
- ordering traversal
- limiting how much related graph is fetched

## Entity command direction

Current intended commands:
- `tep entity create <name> [--ref <value>]`
- `tep entity ensure <name> [--ref <value>]`
- `tep entity show <name-or-id>`
- `tep entity edit <name-or-id> [--name <value>] [--ref <value>]`
- `tep entity list`

Possible shorthand alias:
- `tep e ...`

## Anchor command direction

Current intended command shape:
- `tep anchor auto <pathspec...>`
- `tep anchor show <anchor-id>`

Planned behavior:
- `anchor auto` processes targeted files or directories
- materializes incomplete anchors
- refreshes existing anchor metadata
- binds anchors to entities when an entity reference instruction is present
- removes stale anchor state for dropped anchors in the targeted files
- `anchor show` returns the anchor plus related entities

## Output style

The default human-readable output should stay concise.

Anchor description format:
```txt
<id> (<optional_name>)
<file> (<line>:<shift>) [<offset>]
```

This compact anchor format should be reused in both:
- `tep anchor show`
- `tep entity show`

## V1 implementation priorities

The first useful implementation slice should likely focus on:
- workspace init
- entity create
- entity ensure
- entity show
- entity edit
- entity list
- anchor auto
- anchor show

## V1 non-goals

Not required for early versions:
- AST-aware parsing
- semantic inference
- editor plugin support
- remote sync
- cloud-first behavior
