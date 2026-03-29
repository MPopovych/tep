# Entity Links Spec

This document captures the planned first implementation of directional entity-to-entity links.

## Goal

Entity links should let `tep` describe semantic relationships between entities directly.

Example intent:
- `Student -> Subject`
- relation: `student has subjects assigned to him each semester`

The link should be:
- directional
- human-readable
- free-text
- simple

## Data model

Proposed table:
- `from_entity_id`
- `to_entity_id`
- `relation TEXT NOT NULL`
- `created_at`
- `updated_at`

### Direction
n`from_entity_id -> to_entity_id` is directional.

That means:
- `Student -> Subject` is not the same as `Subject -> Student`

## Relation field

The `relation` field should:
- be free text
- allow long descriptions
- not be artificially limited to a small enum or label list

This makes it useful for both humans and agents.

## Simplicity rule for first version

For the first version, keep exactly one link per ordered pair:
- unique `(from_entity_id, to_entity_id)`

That means:
- one directional edge
- one current relation string

If multiple relations between the same pair become necessary later, the schema can evolve then.

## Command design

### Create or upsert link
```bash
tep entity link Student Subject --relation "student has subjects assigned to him each semester"
```

Behavior:
- resolve both entities by id or name
- create the directional link if missing
- update the relation text if the link already exists

### Remove link
```bash
tep entity unlink Student Subject
```

Behavior:
- remove the directional link from source to target
- do not remove the reverse edge unless explicitly requested separately

## Output direction

### Link creation
Example:
```txt
linked
from: Student
to: Subject
relation: student has subjects assigned to him each semester
```

### Link removal
Example:
```txt
unlinked
from: Student
to: Subject
```

## Interaction with entity show

A later enhancement should let `tep entity show <target>` include:
- entity description if present
- outgoing links
- maybe incoming links later

But the first slice can ship link commands before richer show output if needed.

## Non-goals for first version

Not included yet:
- link priorities
- typed enums for relation names
- multiple relations per ordered pair
- link traversal depth controls
- incoming/outgoing filtering flags

Those can be added later if real usage justifies them.
