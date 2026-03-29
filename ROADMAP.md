# Roadmap

## Phase 1: minimal useful CLI
- initialize a local `tep` workspace
- create entities
- manually place versioned anchor tags in files
- scan files for anchor tags
- store anchors in SQLite using the in-file anchor ID as identity
- attach entities to anchors through a many-to-many association
- create directional typed links between entities
- support link prioritys for sorting and traversal
- retrieve direct and nearby graph context

## Phase 2: better query controls
- depth-limited graph traversal
- relation filters
- result limits
- better sorting controls
- cleaner JSON output for agents and tooling

## Phase 3: richer diagnostics
- stale-anchor detection
- duplicate-anchor warnings
- validation and repair commands
- better workspace ignore behavior

## Phase 4: scaling and ergonomics
- cross-repo support
- workspace configuration
- import/export utilities
- better diagnostics and repair commands

## Phase 5: ecosystem
- editor integrations
- visualization tools
- optional higher-level context bundling workflows
- use tep itself to help track and evolve tep

## Non-goals for early versions
- heavy code intelligence
- semantic inference by default
- cloud-first architecture
- trying to solve every knowledge-management problem at once
