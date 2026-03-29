# Entity Commands Spec

This document captures the current intended behavior of entity-related commands.

## Entity data model

Current entity shape:
- `entity_id` integer
- `name` unique
- `ref` nullable
- `created_at`
- `updated_at`

## Identity and lookup rules

- entity IDs are integer-only
- entity commands may resolve entities by either:
  - integer id
  - unique name
- hexadecimal input is not part of the current behavior

## Naming rules

Entity names should:
- be unique
- be human-readable
- support dot-style names such as `student.permissions`
- not be purely numeric

## Workspace requirement

All entity commands except help/version behavior require a `tep` workspace.

Current behavior:
- `tep init` creates the workspace in the current directory
- DB-requiring commands resolve the nearest ancestor workspace from cwd
- commands outside any workspace fail clearly and suggest `tep init`

## Command set

### Create
```bash
tep entity create "student"
tep entity create "student" --ref "./docs/student.md"
```

Behavior:
- create a new entity
- fail if the name already exists
- print the created entity

### Ensure
```bash
tep entity ensure "student"
tep entity ensure "student" --ref "./docs/student.md"
```

Behavior:
- if entity exists, return it
- if entity does not exist, create it
- always print the resulting entity

### Auto
```bash
tep entity auto ./file.md
tep entity auto ./docs
tep entity auto .
```

Behavior:
- scan targeted files for entity declaration tags
- declaration syntax:
  ```txt
  (#!#tep:Student)
  ```
- ensure the declared entity exists
- if the entity has no `ref`, fill it with the declaring file path
- create a backing anchor for the declaration location
- attach the entity to that anchor
- rewrite the declaration to versioned form:
  ```txt
  (#!#1#tep:Student)
  ```
- do not overwrite an existing non-null `ref`
- already-versioned declarations reuse the existing related anchor in the same file
- `line`, `shift`, and `offset` are metadata only

### Show
```bash
tep entity show "student"
tep entity show 42
```

Behavior:
- accept either unique name or entity id
- print compact entity data
- include related anchors

### Edit
```bash
tep entity edit "student" --ref "./docs/student.md"
tep entity edit 42 --name "student.profile" --ref "./docs/profile.md"
```

Behavior:
- accept either unique name or entity id
- update only provided fields
- print the updated entity

### List
```bash
tep entity list
```

Behavior:
- print entities in a compact CLI-friendly way

## Output expectation

Compact entity format:
```txt
<id> (<name>)
```

When anchors are included, each anchor uses the shared compact anchor format:
```txt
<anchor_id>
<file> (<line>:<shift>) [<offset>]
```

## Storage direction

Current storage model:
- SQLite integer primary key for `entity_id`
- unique constraint on `name`
