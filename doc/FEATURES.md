# Features

This document tracks current `tep` feature areas.

## Core feature groups

### 1. File-driven entity management
Users can:
- declare entities in files
- attach descriptions and refs through metadata
- read entities by name or id
- list entities
- inspect retrieval-oriented entity context

Minimal entity data stays simple:
- unique name
- optional ref
- optional description

### 2. Anchor synchronization and tracking
Users can:
- place named anchor tags in files: `#!#tep:[name](entity1,entity2)` #tepignore
- attach optional anchor metadata
- register and sync anchors through `tep auto`
- inspect known anchors
- understand where anchors currently appear

Important notes:
- location metadata is useful but not durable truth
- anchor identity is the name in the tag
- the `( ... )` suffix is the entity reference list and is required

### 3. Anchor-entity relations
Users can:
- bind anchors to entities through the entity ref list in the tag
- support multiple entity references in a single anchor
- inspect which entities belong to an anchor
- inspect which anchors belong to an entity

This remains a many-to-many relation layer.

### 4. Entity links
Users can:
- define directional relations between entities via file tags
- assign relation descriptions
- inspect incoming and outgoing links through entity views

### 5. Retrieval and graph traversal
Users can:
- resolve an entity
- inspect an anchor
- follow related entities
- retrieve nearby graph context
- bound traversal depth

### 6. Diagnostics and maintenance
Users can:
- validate workspace state
- detect stale anchors
- identify duplicate or suspicious anchor situations
- inspect graph health
- rebuild the graph from files with `tep reset --yes`

## Current priorities

The current useful slice includes:
- `tep auto`
- entity show/context/list
- anchor show/list
- health
- reset
- file-driven metadata and relation reconstruction

## Features explicitly not required early

Not needed in the current iteration:
- AST-aware parsing
- automatic semantic relation inference
- remote sync
- collaboration workflows
- graph UI