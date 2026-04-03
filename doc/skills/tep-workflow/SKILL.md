# tep-workflow

Use this skill when working on the `tep` repository.

Repository path:

```txt
/Users/agent/Desktop/projects/tep
```

This skill is for:
- implementing `tep` features
- refactoring `tep`
- maintaining the repo’s graph
- debugging health/indexing/migration behavior
- keeping docs/specs aligned with actual behavior

## Project model

`tep` is a local-first graph over:
- entities
- anchors
- anchor-entity relations
- directional entity links

The repo itself treats selected docs + selected source files as canonical graph material.

## Canonical graph policy

### Included by intent
- root docs that carry real product meaning
- selected `doc/` files
- selected `src/` implementation files with real hidden markers

### Excluded / controlled
- `tests/`
- `skills/`
- example-heavy spec files unless intentionally curated
- source-file tails after `#tepignoreafter`
- one-off fake marker lines via `#tepignore`

## Preferred maintenance workflow

Before and after graph-affecting changes:

```bash
cd /Users/agent/Desktop/projects/tep
tep entity list
tep reset --yes
tep health
```

If the graph is noisy, fix the source of pollution rather than accepting junk entities.

## Ignore controls

### `#tepignore`
Use for a single noisy line:
- regex/test/example literals
- visible syntax examples in docs
- fake marker strings embedded in source/docs

### `#tepignoreafter`
Use for the rest of a file tail:
- `#[cfg(test)]` modules
- large fixture sections
- intentionally noisy parser examples

Rule of thumb:
- few noisy lines → `#tepignore`
- whole noisy tail → `#tepignoreafter`

## Markdown rule

For markdown files:
- real tags: hide in HTML comments
- sample tags: keep visible and ignored

Examples:

Real hidden tag:
```markdown
<!--- #!#tep:(real.entity){description="..."} -->
```

Visible ignored sample:
```txt
#!#tep:(example.entity) #tepignore
```

## Source anchoring guidance

When adding hidden markers to code comments, prefer stable semantic nodes such as:
- `repo.entity`
- `repo.anchor`
- `entity.service`
- `entity.context`
- `entity.links`
- `anchor.sync`
- `anchor.health`
- `workspace`
- `path.normalization`
- `workspace.scanner`

Avoid over-anchoring tiny helpers unless they materially improve retrieval.

## Docs/spec rule

When behavior changes, update the relevant docs in the repo:
- `README.md`
- `CLI_DESIGN.md`
- `DATA_MODEL.md`
- `doc/*`
- `spec/*` that are still current
- relevant skills

Do not leave old command specs pretending to be current behavior.

## Validation

Before finishing:
- run targeted tests for touched areas
- run full `cargo test`
- run `cargo clippy -- -D warnings`
- run `tep reset --yes`
- run `tep health`

## Repo note

This skill should stay aligned with the repo’s actual workflow and graph-hygiene practices.