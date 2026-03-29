# Entity Catalog

This document tracks the main conceptual entities inside the `tep` product itself.

This is not about markers or syntax details.
It is about the objects the system works with.

## Core entities

### Entity
A logical thing tracked by the graph.

Current intended shape:
- numeric or otherwise generated `entity_id`
- unique `name`
- optional `ref`

Possible examples inside tep itself:
- `scan`
- `workspace`
- `retrieval.query`
- `relation.type`
- `module.database`

Important:
- `name` should be unique
- `ref` is optional metadata for a primary reference path or URL

### Anchor
A physical tagged occurrence in project material.

Current meaning:
- may start incomplete in the file
- becomes materialized with its own durable anchor ID
- is associated with current file path and location metadata

Important:
- anchor identity is durable once materialized
- `line`, `shift`, and `offset` are untrusted or last-known metadata only
- repeated incomplete entity reference instructions in different positions become separate anchors

### Anchor-entity relation
The many-to-many association layer between anchors and entities.

This is essential because:
- one entity may appear in several places
- one place may be relevant to several entities
- anchor and entity records should stay independent
- entity association is driven by optional entity reference instructions in anchor syntax
- a single anchor may refer to multiple entities via comma-separated references

### Link
A directional connection between entities.

Typical fields:
- from_entity_id
- to_entity_id
- relation_type
- priority

## Product-facing conceptual entities

These are not necessarily database rows yet, but they are useful for reasoning about the project.

### Workspace
A local project area managed by `tep`.

### Anchor sync
A process that materializes incomplete anchors, refreshes known anchor state, and updates anchor-entity relations in targeted files.

### Query
A retrieval request such as:
- resolve entity
- show anchor
- traverse graph
- gather context

### Relation type
A user-defined or project-defined semantic edge label.

Examples:
- uses
- documented-by
- depends-on
- mentions

## Open modeling question

Some concepts may remain implementation concerns rather than becoming first-class modeled entities.

Examples:
- output format
- traversal policy
- scan rules

This should be decided pragmatically, not ideologically.
