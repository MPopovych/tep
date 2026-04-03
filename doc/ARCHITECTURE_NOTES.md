# Architecture Notes

These are lightweight architecture notes for `tep`.

## Core architectural shape

`tep` has three layers of concern:

### 1. Physical text layer
This is the real material on disk:
- source files
- markdown docs
- notes
- manuscripts

### 2. Graph layer
This is the logical model in SQLite:
- entities
- anchors
- entity-anchor associations
- links

### 3. Query layer
This is how humans and agents interact with the graph:
- inspect
- resolve
- traverse
- collect context

## Important durability rule

The durable identity is the anchor name in source.

File path, position metadata, and materialized numeric IDs are operational details and should not be treated as permanent truth.

## Main architectural strengths

- local-first
- explicit relationships
- simple persistence model
- suitable for CLI usage
- suitable for future AI retrieval workflows

## Main architectural risks

- anchor clutter if usage patterns are poor
- graph noise if relation types become messy
- retrieval becoming too broad without good limits
- overengineering before proving value

## Practical principle

Prefer:
- explicit data
- predictable behavior
- simple implementation steps

Avoid:
- magic inference too early
- deep parsing too early
- speculative abstractions too early
