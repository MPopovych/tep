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
- future entity links

## Git-like similarities

The similarity to git is mostly ergonomic:
- one binary
- subcommands
- local workspace awareness
- nearest-ancestor workspace discovery

`tep` is not a version-control system.

## Workspace behavior

[#!#1#tep:10](cli,workspace,workspace.discovery)


### Initialize a workspace
```bash
tep init
```

This creates:
- `.tep/`
- `.tep/tep.db`
- `.tep_ignore`

### Resolve the active workspace
For DB-requiring commands, `tep` starts from the current working directory and walks upward until it finds the nearest ancestor workspace.

Important:
- the current cwd matters
- the binary location does not
- commands from nested directories inside a workspace should work
- commands outside any workspace should fail clearly

## Current command areas

### Workspace
```bash
tep init
tep version
tep -V
tep --version
```

### Entities
```bash
tep entity create <name> [--ref <value>]
tep entity ensure <name> [--ref <value>]
tep entity auto <pathspec...>
tep entity show <name-or-id>
tep entity edit <name-or-id> [--name <value>] [--ref <value>]
tep entity list
```

Shorthand:
```bash
tep e ...
```

### Anchors
```bash
tep anchor auto <pathspec...>
tep anchor show <anchor-id>
```

Shorthand:
```bash
tep a ...
```

### Entity-anchor associations
```bash
tep attach <entity-id-or-name> <anchor-id>
tep detach <entity-id-or-name> <anchor-id>
```

## Marker syntax

[#!#1#tep:11](anchor.tag,entity.declaration,cli)


### Anchor tag
```txt
[#!#tep:](student) #tepignore
[#!#1#tep:123](student) #tepignore
```

### Entity declaration tag
```txt
(#!#tep:Student) #tepignore
(#!#1#tep:Student) #tepignore
```

Bracket type decides the role:
- `[...]` = anchor
- `(...)` = entity declaration

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

## Non-goals for current scope

Not part of the current implemented CLI surface:
- `status`
- `doctor`
- `scan`
- `anchor list`
- `link add`
- `link list`
- `link remove`
- `resolve`
- `graph`
- `context get`

Those can be revisited later, but the docs should not present them as current commands.
