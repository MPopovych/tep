# Features

This document tracks feature areas for `tep`.

## Core feature groups

### 1. Entity management
Allow users to:
- create entities
- ensure entities exist
- read entities by name or id
- edit entities
- list entities

Minimal entity data should stay simple:
- unique name
- optional ref

### 2. Anchor synchronization and tracking
Allow users to:
- place incomplete anchors in files
- materialize anchors through `tep anchor <pathspec...>`
- inspect known anchors
- understand where anchors currently appear
- refresh file-local anchor state

Important note:
- location metadata is useful
- location metadata is not durable truth
- anchor identity comes from the materialized in-file anchor ID
- the optional `( ... )` suffix acts as an entity reference instruction list

### 3. Anchor-entity relations
Allow users to:
- bind anchors to entities through optional entity reference instructions
- support multiple entity references in a single anchor via comma separation
- inspect which entities belong to an anchor
- inspect which anchors belong to an entity

This is a many-to-many model and should remain a dedicated relation layer.

### 4. Entity links
Allow users to:
- define directional relations between entities
- assign relation types
- assign priorities
- list incoming and outgoing links

### 5. Retrieval and graph traversal
Allow users to:
- resolve an entity
- inspect an anchor
- follow related entities
- retrieve nearby graph context
- sort and limit traversal using priority

### 6. Diagnostics and maintenance
Allow users to:
- validate workspace state
- detect stale anchors
- identify duplicate or suspicious anchor situations
- inspect database health

## Early priorities

The first useful feature slice probably includes:
- entity create/ensure/read/edit/list
- init
- later: anchor materialization via `tep anchor`
- anchor inspection
- relation handling
- resolve

## Features explicitly not required early

Not needed in the first iteration:
- AST-aware parsing
- automatic semantic relation inference
- remote sync
- collaboration workflows
- VS Code integration
- graph UI
