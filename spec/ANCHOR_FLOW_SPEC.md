# Anchor Flow Spec

This document captures the planned anchor workflow for `tep`.

## Core idea

Users should be able to place **incomplete anchors** directly into files, then ask `tep` to materialize and synchronize them.

The user-facing command shape is planned as:

```bash
tep anchor auto <pathspec...>
```

Examples:
```bash
tep anchor auto ./file.md
tep anchor auto .
tep anchor auto ./docs ./src
```

The command should work on targeted files and directories in a pathspec-oriented way similar in spirit to `git add`.

## Incomplete anchor syntax

Planned incomplete anchor examples:

```txt
[#!#tep:](student)
[#!#tep:](student,basic-user)
```

Important:
- the `( ... )` part is optional
- an incomplete anchor may exist without any entity reference instruction
- the `:` is the anchor ID slot
- in incomplete form, the ID slot is empty
- when multiple entity references are present, they are comma-separated inside `( ... )`

Examples:
```txt
[#!#tep:](student)
[#!#tep:](student.permissions)
[#!#tep:](42)
[#!#tep:](student,basic-user)
[#!#tep:]
```

## Materialized anchor syntax

After processing, an incomplete anchor becomes a materialized anchor.

Planned examples:

```txt
[#!#1#tep:123456](student)
[#!#1#tep:123457](student,basic-user)
```

Meaning:
- `1` = anchor format version
- `123456` = anchor ID
- `:` is the anchor ID slot separator
- `( ... )` remains as an optional entity reference instruction list

If no entity reference instruction was present, a materialized anchor may look like:

```txt
[#!#1#tep:123458]
```

## Behavior of `tep anchor auto <pathspec...>`

For the targeted files, the command should:

1. scan files for incomplete and existing anchors
2. create new anchor records for incomplete anchors
3. rewrite incomplete anchors into materialized anchors
4. refresh current file-path and position metadata for existing anchors
5. if an entity reference instruction exists, try to bind the anchor to entities
6. detect dropped anchors in those files
7. remove stale file-local anchor state that no longer exists in the file

## Entity reference instruction behavior

The payload in `( ... )` is optional.

When present, it is treated as an **entity reference instruction list**.

Planned behavior:
- entries are comma-separated
- each entry may be an entity ID or entity name
- if an entry looks like an entity ID, try resolving by ID
- otherwise treat it as an entity name and likely use `ensure`

Important assumption:
- entity names are expected to be relatively stable
- the entity reference instruction list is not the durable identity of the anchor
- it is an instruction for synchronizing anchor-entity relations

## Anchor show

Planned command:
```bash
tep anchor show <anchor-id>
```

Behavior:
- print the anchor in compact form
- print related entities concisely beneath it

Shared anchor format:
```txt
<anchor_id> (<optional_name>)
<file> (<line>:<shift>) [<offset>]
```

## Repeated incomplete anchors

If the same entity reference instruction appears multiple times, those are still **separate anchors**.

Example:
```txt
[#!#tep:](student)
...
[#!#tep:](student)
```

After materialization, these should become something like:
```txt
[#!#1#tep:101](student)
...
[#!#1#tep:102](student)
```

Each physical occurrence gets its own anchor ID.

## Pathspec behavior

Planned input forms:
- direct file path
- directory path
- current directory (`.`)
- future pathspec or glob-like expansion in the style of local file-targeting commands

The command should respect `.tep_ignore` when selecting and scanning files.

## Separate schemas

The planned storage model should keep these concerns separate:

### Anchors schema
Stores the anchor itself.

Likely concerns:
- anchor identity
- version
- current file path
- current location metadata (`line`, `shift`, `offset`)
- timestamps

### Anchor-entity relation schema
Stores associations between anchors and entities.

This relation should remain separate from both:
- the anchor record
- the entity record

This keeps the many-to-many model explicit and flexible.

## Non-goals for this stage

This spec does not yet freeze:
- exact regex or parser grammar
- exact pathspec semantics
- exact stale-anchor deletion strategy
- exact rewrite safety guarantees

Those should be tightened before implementation begins.
