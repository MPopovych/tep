# tep-workflow

Use this skill when working on the `tep` repository.

Repository path:

```txt
/Users/agent/Desktop/projects/tep
```

This skill is for:
- implementing `tep` features
- refactoring `tep`
- maintaining the repo’s anchor/entity graph
- debugging health/indexing/migration behavior
- keeping docs/specs aligned with actual behavior

## Project model

`tep` is a local-first graph over:
- entities
- anchors
- anchor-entity relations
- directional entity links

The repo itself treats **docs + selected source code** as canonical graph material.

## Canonical graph policy

### Included by intent
- root docs
- selected `spec/` files
- selected `doc/` files
- selected `src/` implementation files with hidden markers

### Excluded / controlled
- `tests/`
- playgrounds
- noisy fixture/example-heavy files via `.tepignore`
- source-file tails after `#tepignoreafter`
- one-off fake marker lines via `#tepignore`

Do **not** revert to ignoring all of `src/` unless explicitly asked.
The intended direction is to link code and docs.

## Health workflow

Before and after graph-affecting changes, prefer this sequence:

```bash
cd /Users/agent/Desktop/projects/tep
tep health
tep anchor auto .
tep entity auto src
tep health
```

If health is noisy, inspect whether the cause is:
- intentional fixtures/examples
- missing ignore markers
- stale metadata
- duplicate materialized anchors

Prefer tightening canonical scope over polluting the graph.

## Ignore controls

### `#tepignore`
Use for a single noisy line:
- regex/test/example literals
- fake marker strings embedded in docs or source

### `#tepignoreafter`
Use for the rest of a file tail:
- `#[cfg(test)]` modules
- large fixture sections
- intentionally broken example tails

Rule of thumb:
- few noisy lines → `#tepignore`
- whole noisy tail → `#tepignoreafter`

## Source anchoring guidance

When adding hidden markers to code comments, prefer stable semantic nodes such as:
- `module.anchor`
- `module.entity`
- `service.anchor.health`
- `service.anchor.sync`
- `service.entity.context`
- `service.entity.auto`
- `repo.anchor`
- `repo.entity`
- `path.normalization`

Avoid over-anchoring tiny helpers unless they matter to docs or navigation.

## Backup rule

Before large indexing changes, back up:

```bash
.tep/tep.db
```

Example:

```bash
mkdir -p .tep/backups
cp .tep/tep.db .tep/backups/tep.db.backup-$(date +%Y%m%d-%H%M%S)
```

## Docs/spec rule

When behavior changes, update the relevant docs in the repo:
- `README.md`
- `CLI_DESIGN.md`
- `DATA_MODEL.md`
- `spec/*`
- this skill

Keep docs practical and current.

## Refactor rule

Prefer:
- extracting small helpers/utils
- strengthening unit tests near new seams
- avoiding speculative abstractions

Good recent patterns in this repo:
- shared path utils
- shared time utils
- shared output rendering helpers
- smaller service helpers
- lighter command support helpers

## Validation

Before finishing:
- run targeted tests for touched areas
- run full `cargo test`
- run repo `tep health`
- if graph-affecting, run `tep anchor auto .` / `tep entity auto src` as appropriate

## Repo note

This skill replaces scattered local notes as the canonical workflow guidance.
If local tooling needs a skill path, prefer linking/symlinking back to this repo copy so there is only one source of truth.
