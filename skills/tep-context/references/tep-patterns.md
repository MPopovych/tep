# tep patterns

## Goal

Use `tep` to identify the smallest grounded context bundle before reading many files, and maintain graph quality so resets stay trustworthy.

---

## Retrieval patterns

### Entity context first
```bash
tep entity context <name-or-id>
tep entity context <name-or-id> --link-depth 2
tep entity context <name-or-id> --files-only
```

Interpretation:
- `ref` = canonical starting file
- anchors = grounded code/doc locations
- linked entities = follow-up concepts if needed

### Entity show
```bash
tep entity show <name-or-id>
```

Use for compact graph shape.

### Anchor show
```bash
tep anchor show <name>
```

Use when an anchor name is already known.

### Discovery
```bash
tep entity list
tep anchor list
```

Use when the right concept name is unclear.

---

## Maintenance patterns

### Primary sync
```bash
tep auto <pathspec...>
```

Use as the default maintenance command.

### Audit
```bash
tep health
tep health ./docs
```

Use to detect:
- duplicate anchors
- missing anchors
- orphaned entities
- orphaned anchors
- metadata warnings
- unreadable file warnings

### Rebuild
```bash
tep reset --yes
```

Use when validating that the graph can be reconstructed from files.
A healthy repo should tolerate this regularly.

---

## Example hygiene patterns

### Visible example, ignored
Use in docs that teach syntax.

```txt
#!#tep:[example.anchor](example.entity) #tepignore
#!#tep:(example.entity) #tepignore
```

### Hidden real tag in markdown
Use when the doc itself should be part of the graph.

```markdown
<!--- #!#tep:(real.concept){description="..."} -->
<!--- #!#tep:[real.anchor](real.concept) -->
```

### Ignore a whole test/example tail
Use in source files and docs with many fixture snippets.

```rust
// #tepignoreafter
#[cfg(test)]
mod tests { ... }
```

---

## Entity design patterns

### Good canonical entity
```txt
#!#tep:(entity.service){description="Service for entity auto-sync, entity reads, and link-aware context assembly"}
```

### Good relation
```txt
#!#tep:(entity.service)->(repo.entity){description="uses for entity persistence"}
```

### Good anchor
```txt
#!#tep:[workspace.reset](workspace,entity.service,anchor.sync){description="Reset entry point"}
```

---

## Maintenance heuristics for LLMs

Use `tep` well when:
- entity list mostly shows real repo concepts
- docs expose examples visibly but keep them ignored
- markdown hides real tags if rendering matters
- reset succeeds without graph pollution
- warnings are informative, not catastrophic

Investigate when:
- generic entities appear (`example`, `student`, `entity1`)
- examples from specs/docs leak into the graph
- test modules create graph noise
- unreadable files abort scans instead of warning
- duplicate anchor examples stop full rebuilds

---

## Recommended maintenance loop

1. `tep entity list`
2. identify junk/example entities
3. fix docs/examples with `#tepignore` or `#tepignoreafter`
4. keep meaningful real tags hidden in markdown comments when needed
5. `tep reset --yes`
6. `tep health`
7. repeat until graph is mostly project concepts
