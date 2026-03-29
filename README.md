# tep

`tep` is a local-first CLI for **text entity pointers**.

It connects logical entities to anchor points in text and keeps the graph in a local SQLite workspace.

## What `tep` does today

Current implemented areas:
- initialize a local workspace
- create and manage entities
- auto-declare entities from files
- materialize anchors in files
- attach entities to anchors
- show entities with related anchors
- show anchors with related entities

## Workspace model

[#!#1#tep:19](workspace,workspace.discovery)


A `tep` workspace is created with:

```bash
tep init
```

That creates:
- `.tep/`
- `.tep/tep.db`
- `.tep_ignore`

For commands that require the database, `tep` resolves the **nearest ancestor workspace** from the current working directory.

That means:
- inside a workspace tree, commands work from nested directories
- outside any workspace, DB-requiring commands fail and tell you to run `tep init`
- the location of the `tep` binary does not define the workspace
- the current terminal cwd does

## Core syntax

### Anchor tags

[#!#1#tep:20](anchor,anchor.tag)

Anchor tags use square brackets.

Incomplete anchor:
```txt
[#!#tep:](student) #tepignore
```

Materialized anchor:
```txt
[#!#1#tep:123](student) #tepignore
```

Meaning:
- square brackets identify an anchor tag
- the value after `tep:` is the durable anchor ID once materialized
- the optional `( ... )` suffix is an entity reference instruction list

### Entity declaration tags

[#!#1#tep:21](entity,entity.declaration)

Entity declaration tags use parentheses.

Incomplete declaration:
```txt
(#!#tep:Student) #tepignore
```

Materialized declaration:
```txt
(#!#1#tep:Student) #tepignore
```

Meaning:
- parentheses identify an entity declaration marker
- `Student` is the entity name
- `tep entity auto` ensures the entity exists
- if the entity has no `ref`, the declaring file path is stored in `ref`
- a backing anchor relation is created for that declaration location

## Current command surface

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

### Manual relations
```bash
tep attach <entity-id-or-name> <anchor-id>
tep detach <entity-id-or-name> <anchor-id>
```

## Output style

Default output is intentionally compact.

Entity:
```txt
<id> (<name>)
```

Anchor:
```txt
<anchor_id>
<file> (<line>:<shift>) [<offset>]
```

Location fields are metadata, not identity.
Anchor identity is the anchor ID.

## Notes

- entity IDs are integer-only
- entity names are unique and cannot be purely numeric
- entity metadata uses `ref`
- `.tep_ignore` is respected
- `.gitignore` is not
- `line`, `shift`, and `offset` are refreshable metadata only
- `shift` and `offset` are byte-oriented in practice

## Documentation map

- [Concept](./CONCEPT.md)
- [Data Model](./DATA_MODEL.md)
- [CLI Design](./CLI_DESIGN.md)
- [Use Cases](./USE_CASES.md)
- [Roadmap](./ROADMAP.md)
- [Open Questions](./OPEN_QUESTIONS.md)
- [Development Notes](./DEV_NOTES.md)
- [Spec Index](./spec/README.md)
