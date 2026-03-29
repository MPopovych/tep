# Roadmap

## Phase 1: minimal useful CLI
- initialize a local `tep` workspace
- create entities
- auto-declare entities from files
- place and materialize anchor tags in files
- store anchors in SQLite using the in-file anchor ID as identity
- attach entities to anchors through a many-to-many association
- retrieve direct entity and anchor context

## Phase 2: agent-friendly retrieval
- add `tep entity context <name-or-id>`
- show primary `ref` first when present
- include anchor snippets for nearby text
- emit a deduplicated related file list
- include linked entities by default
- support bounded traversal with `--link-depth`
- add cleaner JSON output for agents and tooling

## Phase 3: better query controls and diagnostics
- link/result limits
- relation filters
- stale-anchor detection
- duplicate-anchor warnings
- validation and repair commands
- schema/migration diagnostics if needed beyond auto-upgrade
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
