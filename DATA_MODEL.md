# Data Model

## 1. Entity

[#!#tep:](entity)

Current shape:
- `entity_id` (integer)
- `name` (unique)
- `ref` (nullable)
- `description` (nullable)
- `created_at`
- `updated_at`

Notes:
- `entity_id` is currently integer-only
- `name` should be unique and human-readable
- `ref` points to a primary file or reference location when useful
- `description` holds a direct free-text summary of the entity

## 2. Anchor

[#!#tep:](anchor,anchor.tag)

Current shape:
- `anchor_id` (integer)
- `version`
- `file_path`
- `line`
- `shift`
- `offset`
- `created_at`
- `updated_at`

Notes:
- the durable identity is the `anchor_id`
- `line`, `shift`, and `offset` are metadata, not identity
- `shift` and `offset` are byte-oriented in practice

## 3. Anchor-entity relation

[#!#tep:](anchor_entity_relation,entity,anchor)

This is a many-to-many relation.

Meaning:
- one entity may have many anchors
- one anchor may point to many entities

This relation should stay separate from both:
- the entity table
- the anchor table

## 4. Entity link

[#!#tep:](link,entity)

Current shape:
- `from_entity_id`
- `to_entity_id`
- `relation`
- `created_at`
- `updated_at`

Notes:
- links are directional in storage
- `Student -> Subject` is not the same as `Subject -> Student`
- `relation` is free-text and may be long-form
- current implementation keeps one link per ordered pair
- link priorities are intentionally out of scope for now

## 5. Workspace schema metadata

`tep` tracks DB schema version using SQLite:

```sql
PRAGMA user_version
```

Notes:
- schema version is part of workspace state
- opening a workspace may trigger in-place migration
- legacy DBs can be upgraded automatically when missing newer fields such as `entities.description`
