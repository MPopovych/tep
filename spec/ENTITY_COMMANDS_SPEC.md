# Entity Commands Spec

This document captures the current intended behavior of entity-related commands.

## Entity data model

Early entity shape:
- `entity_id` integer
- `name` unique
- `ref` nullable
- `created_at`
- `updated_at`

## Identity and lookup rules

- entity IDs are currently integer-only
- entity commands may resolve entities by either:
  - integer id
  - unique name
- hexadecimal input is not part of the current behavior
- hexadecimal support may be added later without changing the underlying integer storage model

## Naming rules

Entity names should:
- be unique
- be human-readable
- support dot-style names such as `student.permissions`

This allows practical namespacing without enforcing a heavy schema.

## Command set

### Create
```bash
tep entity create "student"
tep entity create "student" --ref "./docs/student.md"
```

Behavior:
- create a new entity
- fail if the name already exists
- print the created entity, including its generated id

### Ensure
```bash
tep entity ensure "student"
tep entity ensure "student" --ref "./docs/student.md"
```

Behavior:
- if entity exists, return it
- if entity does not exist, create it
- always print the resulting entity

This is especially useful for automation and agents.

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
- anchor reuse for already-versioned declarations should rely on durable identity and existing relations, not on `line`, `shift`, or `offset`
- `line`, `shift`, and `offset` are refreshable metadata only

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
- allow editing multiple fields in one command
- update only provided fields
- print the updated entity

### List
```bash
tep entity list
```

Behavior:
- print entities in a CLI-friendly way
- intended to feel somewhat similar to `git log` in usefulness, not in exact format
- should be readable by both humans and agents

Likely later additions:
- filters
- ordering controls
- pagination or navigation
- JSON output

## Output expectation

Compact entity format:
```txt
<id> (<name>)
```

When anchors are included, each anchor should use the shared compact anchor format:
```txt
<anchor_id>
<file> (<line>:<shift>) [<offset>]
```

## Storage direction

Current recommended storage model:
- SQLite integer primary key for `entity_id`
- unique index or unique constraint on `name`

This keeps the database simple and efficient while leaving room for richer CLI representations later if needed.
