# Implementation Backlog

This is a lightweight implementation backlog for the early `tep` project.

## Now

### Project setup
- [x] initialize Rust project
- [x] choose base crates for CLI, SQLite, errors, and tests
- [x] prepare test scaffolding
- [x] prepare internal doc area

### Early command shell
- [x] move command handling into dedicated modules
- [x] prepare a real CLI entry structure with clap help support
- [x] add first command module for `init`

### Design consolidation
- [x] refine entity command direction
- [x] switch entity metadata field from `path` to `ref`
- [x] choose integer-only entity IDs for now
- [x] confirm entity schema direction for name/ref/id
- [x] plan anchor materialization flow around `tep anchor <pathspec...>`
- [x] separate anchor schema and anchor-entity relation schema conceptually
- [x] clarify anchor ID slot semantics and entity reference instructions
- [x] clarify anchor location metadata (`line`, `shift`, `offset`)
- [x] decide conflict behavior for duplicate, unknown, and malformed anchors
- [ ] confirm retrieval output expectations

## Next

### Minimal persistence
- [x] create `.tep/` workspace layout
- [x] create SQLite database file
- [x] apply schema on init
- [x] create `.tep_ignore`

### Minimal entity flow
- [x] create entity records
- [x] ensure entity records
- [x] read entity by id or unique name
- [x] edit entity fields
- [x] list entities
- [x] add repository-layer tests for entity persistence

### Anchor flow
- [x] introduce anchor schema
- [x] introduce anchor-entity relation schema
- [x] implement initial `tep anchor <pathspec...>` command
- [x] detect incomplete anchors
- [x] materialize anchors with version and anchor ID
- [x] support comma-separated multiple entity references at parse level
- [x] record `line`, `shift`, and `offset`
- [x] bind entity references to entities
- [x] persist anchor-entity relations
- [x] add manual relation manipulation via `attach` and `detach`
- [x] improve failure behavior for unknown materialized anchors
- [x] introduce reusable filter layer for `.tep_ignore`
- [x] detect dropped anchors in targeted files
- [x] fail on duplicate materialized anchor IDs in the same file
- [x] fail on cross-file anchor ID conflicts
- [x] add regression coverage for Unicode and encountered edge cases
- [ ] rewrite files safely under additional edge cases
- [ ] refine `.tep_ignore` semantics further if needed

## Playground / local experimentation
- [x] create `playground/` for local experiments
- [x] add `playground/` to `.gitignore`
- [x] add `playground/` to local `.tep_ignore`
- [x] validate explicit anchor materialization on playground files

## Later

### Better traversal
- [ ] depth-limited graph retrieval
- [ ] priority-aware sorting
- [ ] relation filtering
- [ ] JSON output contracts

### Diagnostics
- [ ] duplicate anchor warnings beyond current hard-fail cases
- [ ] repair guidance

### Ergonomics
- [ ] clearer help text
- [ ] better error messages
- [ ] workspace ignore behavior
- [ ] import/export ideas

## Rule for this backlog

Keep it implementation-oriented.

If something becomes detailed enough to deserve a separate design note, move it out instead of turning this file into a wall of text.
