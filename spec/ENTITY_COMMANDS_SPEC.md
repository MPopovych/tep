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

### Read
```bash
tep entity read "student"
tep entity read 42
```

Behavior:
- accept either unique name or entity id
- print basic entity data
- later may expand to include more connected context

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

Even in early versions, commands should print enough structured output to make automation practical.

At minimum, output should include:
- entity id
- name
- ref if present
- timestamps when relevant

## Storage direction

Current recommended storage model:
- SQLite integer primary key for `entity_id`
- unique index or unique constraint on `name`

This keeps the database simple and efficient while leaving room for richer CLI representations later if needed.
