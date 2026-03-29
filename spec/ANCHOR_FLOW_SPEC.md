# Anchor Flow Spec

This document captures the current anchor workflow for `tep`.

## Core idea

Users place incomplete anchors into files, then ask `tep` to materialize and synchronize them.

Current command shape:

```bash
tep anchor auto <pathspec...>
```

Examples:
```bash
tep anchor auto ./file.md
tep anchor auto .
tep anchor auto ./docs ./src
```

Shorthand:
```bash
tep a auto <pathspec...>
```

## Workspace requirement

Anchor commands require a `tep` workspace.

Current behavior:
- `tep init` creates the workspace in the current directory
- DB-requiring commands resolve the nearest ancestor workspace from cwd
- commands outside any workspace fail clearly and suggest `tep init`

## Incomplete anchor syntax

Examples:

```txt
[#!#tep:](student)
[#!#tep:](student,basic-user)
[#!#tep:]
```

Rules:
- square brackets identify anchor tags
- the `( ... )` part is optional
- the `:` is the anchor ID slot
- in incomplete form, the ID slot is empty
- multiple entity references are comma-separated inside `( ... )`

## Materialized anchor syntax

Examples:

```txt
[#!#1#tep:123](student)
[#!#1#tep:124](student,basic-user)
[#!#1#tep:125]
```

Meaning:
- `1` = anchor format version
- the value after `tep:` is the anchor ID
- `( ... )` remains an optional entity reference instruction list

## Behavior of `tep anchor auto <pathspec...>`

For targeted files, the command:

1. scans files for incomplete and existing anchors
2. creates new anchor records for incomplete anchors
3. rewrites incomplete anchors into materialized anchors
4. refreshes file-path and location metadata for existing anchors
5. synchronizes anchor-entity relations when `( ... )` is present
6. detects dropped anchors in those files
7. removes stale anchor state that no longer exists in the file

## Entity reference instruction behavior

The payload in `( ... )` is optional.

When present, it is treated as an entity reference instruction list.

Behavior:
- entries are comma-separated
- each entry may be an entity ID or entity name
- numeric values resolve by ID
- non-numeric values resolve by name and may be ensured

Important:
- the entity instruction list is not anchor identity
- anchor identity is the anchor ID

## Entity declarations are separate

`tep anchor auto` does **not** process entity declaration tags.
Those belong to `tep entity auto`.

Entity declarations use parentheses, for example:

```txt
(#!#tep:Student)
```

## Anchor show

Command:
```bash
tep anchor show <anchor-id>
```

Shorthand:
```bash
tep a show <anchor-id>
```

Behavior:
- print the anchor in compact form
- print related entities beneath it

Shared anchor format:
```txt
<anchor_id>
<file> (<line>:<shift>) [<offset>]
```

## Repeated incomplete anchors

If the same entity reference instruction appears multiple times, those are still separate anchors.

Example:
```txt
[#!#tep:](student)
...
[#!#tep:](student)
```

After materialization:
```txt
[#!#1#tep:101](student)
...
[#!#1#tep:102](student)
```

Each physical occurrence gets its own anchor ID.

## Path selection behavior

Supported input forms:
- direct file path
- directory path
- current directory (`.`)

Selection respects `.tep_ignore`.
It does not use `.gitignore`.

## Storage model

Current storage keeps these concerns separate:

### Anchors table
Stores:
- anchor identity
- version
- current file path
- current location metadata (`line`, `shift`, `offset`)
- timestamps

### Anchor-entity relation table
Stores:
- associations between anchors and entities

This keeps the many-to-many model explicit.

## Notes on metadata

`line`, `shift`, and `offset` are refreshable metadata.
They are useful for display, but not durable identity.
The durable identity is the anchor ID.
