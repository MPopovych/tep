# Data Model

## Core model

The data model should stay intentionally small.

The main objects are:
- **entities**
- **anchors**
- **anchor_entity relations**
- **links**

SQLite is a good fit for storing them.

## 1. Entity

An entity is a logical thing.

Examples:
- `checkout-endpoint`
- `checkout-request-dto`
- `checkout-service`
- `character:mara`
- `feature:billing-retries`
- `student`
- `student.permissions`

### Suggested fields
- `entity_id` integer
- `name` unique
- `ref` nullable
- `created_at`
- `updated_at`

## Entity naming

Entity names should be unique.

This keeps commands such as:
- `create`
- `ensure`
- `read`
- `edit`

clear and deterministic.

Examples:
- `student`
- `student.permissions`
- `feature.billing`
- `doc.architecture.context-model`

## Entity reference field

An entity may optionally have a single `ref` field.

This is lightweight metadata for a primary reference location.

Examples:
- local document path
- relative markdown path
- URL
- canonical note path

Important:
- `ref` is optional metadata
- `ref` is not a replacement for anchors
- an entity may have both a `ref` and many anchors

## 2. Anchor

An anchor is a physical tagged occurrence in text with its own durable anchor ID once materialized.

Planned examples:
```txt
[#!#tep:](student)
[#!#tep:](student,basic-user)
[#!#1#tep:123763636473](student)
[#!#1#tep:123763636474](student,basic-user)
[#!#1#tep:123763636475]
```

Important:
- incomplete anchors may exist before materialization
- the `:` is the anchor ID slot
- materialized anchors contain the durable `anchor_id`
- the optional `( ... )` suffix is an entity reference instruction list, not anchor identity

### Suggested anchor schema fields
- `anchor_id` integer
- `version` integer
- `file_path`
- `line` nullable
- `shift` nullable
- `offset` nullable
- `created_at`
- `updated_at`

## Anchor location metadata

Anchor location fields are useful but untrusted.
They represent current or last-known location metadata.

Planned meaning:
- `line` = untrusted / last-known index of newline boundaries for the tag location
- `shift` = untrusted / last-known amount of symbols before the tag within the line
- `offset` = untrusted / last-known file offset of the tag

These fields are helpful for navigation and diagnostics, but they are not durable anchor identity.

## 3. Anchor-entity relation

This is the many-to-many relation layer between anchors and entities.

That means:
- one entity can connect to many anchors
- one anchor can connect to many entities

This should remain a separate schema/table instead of being folded into either side.

Entity association should come from the optional entity reference instruction list when present and resolvable.

### Suggested relation fields
- `anchor_id`
- `entity_id`
- `created_at`

Possible later additions:
- `relation_source`
- `is_primary`
- `notes`

But those are not required for the core model.

## 4. Link

A link is a directional relationship between two entities.

Examples:
- endpoint -> uses -> dto
- endpoint -> calls -> service
- feature -> documented-by -> spec
- chapter -> mentions -> character

### Suggested fields
- `from_entity_id`
- `to_entity_id`
- `relation_type`
- `priority`
- `created_at`
- `updated_at`

## Priority model

Priority is user-defined.

Important convention:
- **1 is best / highest priority**
- larger numbers are lower priority
- the maximum cap is defined by the user or project

Priority is mainly useful for:
- sorting retrieval results,
- deciding which links to fetch first,
- limiting traversal breadth in graph queries.

So the tool should not hardcode a strict upper bound unless a user or config does.

## Example interpretation
- `1` = most important / fetch first
- `2` = highly relevant
- `5` = useful but less important
- `20` = weak or low-priority relation in a broader graph

## Retrieval behavior

A query can use priority to:
- sort linked entities,
- stop after the top N results,
- prefer stronger links during graph traversal.

This is especially useful for AI-context assembly, where breadth must often be limited.

## Suggested storage approach

### SQLite tables
- `entities`
- `anchors`
- `anchor_entities`
- `links`

Possible later additions:
- `scan_state`
- `entity_aliases`
- `tags`
- `workspace_config`

## Anchor synchronization model

A practical first version of anchor handling can be:
- scan targeted files,
- detect incomplete and materialized anchors,
- create missing anchors,
- rewrite incomplete anchors into materialized form,
- record current file path and location metadata,
- upsert matching anchor rows,
- refresh anchor-entity relations from entity reference instructions when applicable.

## Important design choice

The source of truth for durable anchor identity is the anchor ID embedded in the materialized file tag.

That means v1 does not need to solve:
- AST-aware symbol matching,
- stable line tracking under refactors,
- automatic semantic inference.

Those can come later if needed.
