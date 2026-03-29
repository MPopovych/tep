# tep skill notes

This file captures practical project-specific guidance for working on `tep`.

## Canonical graph policy

The `tep` repo now treats both docs and selected source code as canonical graph material.

Current approach:
- docs are indexed
- selected implementation code in `src/` is indexed
- noisy fixture/test tails inside source files are cut off with `#tepignoreafter`
- isolated noisy lines can use `#tepignore`
- `.tep_ignore` excludes non-canonical material such as `tests/`, playgrounds, and selected example-heavy files

## Health workflow

Before or after graph-affecting work, run:

```bash
tep health
```

If needed, repair with:

```bash
tep anchor auto .
tep entity auto src
```

Typical healthy repo result should show no moved/missing/duplicate/unknown anchors.

## Source indexing guidance

When adding hidden markers to code:
- prefer stable module/service/repository-level entities
- keep names readable and layered
- avoid anchoring noisy fixture strings
- use `#tepignore` for one-off fake marker lines
- use `#tepignoreafter` for full `#[cfg(test)]` tails or large fixture sections

Good examples:
- `module.anchor`
- `service.anchor.health`
- `service.entity.context`
- `repo.anchor.path-normalization`
- `path.normalization`

## Safety notes

- back up `.tep/tep.db` before large indexing changes
- repo-wide `anchor auto .` is a repair tool, not just a scanner
- keep the canonical graph small and meaningful
- if source indexing starts polluting health, tighten ignore rules instead of disabling code graphing entirely

## Refactor notes

Recent cleanup already extracted:
- path utilities
- time utilities
- shared output rendering helpers
- smaller service helpers
- lighter command-layer boilerplate

Before adding new abstractions, prefer finishing the same style of cleanup consistently across neighboring modules.
