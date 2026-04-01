# Core Modules

This document tracks the likely internal modules of the `tep` implementation.

The exact file layout may evolve, but these are the main responsibilities.

## 1. CLI module

Responsibility:
- parse command-line arguments
- define subcommands
- validate user inputs at the interface level

Current Rust direction:
- `clap`
- `src/cli.rs`

## 2. Command modules

Responsibility:
- dispatch subcommands
- translate CLI requests into application actions
- avoid owning text formatting or persistence rules
- avoid turning `main.rs` into a large match block

Current Rust direction:
- `src/commands/mod.rs`
- `src/commands/init.rs`
- `src/commands/entity.rs`
- `src/commands/anchor.rs`

## 3. Service layer

Responsibility:
- own application workflows
- coordinate repositories, validation, and command-facing behavior
- keep command modules thin
- keep repositories focused on persistence

Current Rust direction:
- `src/service/`
- `src/service/entity_service.rs`
- `src/service/workspace_service.rs`
- `src/service/anchor_service.rs`

## 4. Filter layer

Responsibility:
- own reusable path filtering logic
- load and apply `.tepignore`
- expose filtering decisions to traversal-heavy modules
- remain separate from individual services such as `anchor_service`

Current Rust direction:
- `src/filter/`
- `src/filter/tep_ignore_filter.rs`

Implementation direction:
- backed by the Rust `ignore` crate
- configured for `.tepignore`
- Git-specific ignore sources disabled
- reusable across future file-walking features

## 5. Database module

Responsibility:
- open SQLite connections
- apply schema
- expose persistence helpers
- support future migrations

Current Rust direction:
- `rusqlite`
- `src/db.rs`

## 6. Entity domain module

Responsibility:
- define entity types
- parse entity lookups
- validate entity naming rules

Current Rust direction:
- `src/entity.rs`

## 7. Anchor domain module

Responsibility:
- define incomplete/materialized anchor concepts
- parse anchor syntax structures
- support anchor materialization workflows
- compute last-known location metadata

Current Rust direction:
- `src/anchor.rs`

## 8. Repository layer

Responsibility:
- isolate persistence logic from services and command handlers
- make database behavior testable directly
- keep SQL and row mapping in one place

Current Rust direction:
- `src/repository/`
- `src/repository/entity_repository.rs`
- `src/repository/anchor_repository.rs`
- `src/repository/anchor_entity_repository.rs`

## 9. Output layer

Responsibility:
- format command results for terminal output
- keep presentation logic separate from command orchestration
- prepare a clean path for future JSON or alternate output modes

Current Rust direction:
- `src/output/`
- `src/output/entity_output.rs`
- `src/output/workspace_output.rs`
- `src/output/anchor_output.rs`

## 10. Link module

Responsibility:
- create directional links
- list links
- remove links
- sort and filter by priority

## 11. Retrieval module

Responsibility:
- resolve entities and anchors
- traverse graph relationships
- gather nearby context
- support output shaping for human and machine use

## 12. Diagnostics module

Responsibility:
- status checks
- validation helpers
- stale-anchor detection
- suspicious-state detection

## Structural note

A good early implementation should keep modules shallow and obvious.

Avoid over-separating too early.
It is fine if several of these responsibilities start close together and split later when the code justifies it.
