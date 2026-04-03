# Entity Links Spec

This document captures the current directional entity-link model in `tep`.

## Goal

Entity links describe semantic relationships between entities directly from file tags.

Example intent:
- `student -> subject`
- relation: `has subject`

The link is:
- directional
- human-readable
- free-text
- reconstructed from files

## Data model

Stored fields:
- `from_entity_id`
- `to_entity_id`
- `relation TEXT NOT NULL`
- `created_at`
- `updated_at`

### Direction
`from_entity_id -> to_entity_id` is directional.

That means:
- `student -> subject` is not the same as `subject -> student`

## Relation field

The `relation` field is:
- free text
- allowed to be descriptive
- not restricted to a small enum

## Current syntax

Relation tags are declared in files:

```txt
#!#tep:(student)->(subject) #tepignore
#!#tep:(student)->(subject){description="has subject"} #tepignore
```

Behavior:
- both entities are ensured if missing
- the directional link is created or updated
- later declarations overwrite the prior relation text
- overwrite cases surface as warnings

## Interaction with entity show/context

`tep entity show <target>` and `tep entity context <target>` include:
- outgoing links
- incoming links
- directional rendering
- optional traversal depth in context mode

## Non-goals

Not included:
- link priorities
- typed enums for relation names
- multiple relations per ordered pair
- separate manual link commands as the primary workflow

The current model is tag-driven rather than manual CLI mutation.