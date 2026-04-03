---
name: tep-context
description: Use the local `tep` CLI as a context-routing and graph-maintenance layer only when working in a repository or document set that already has a `tep` workspace or when the user explicitly asks to add or maintain `tep` coverage. Trigger for tasks that involve `tep entity context`, `tep entity show`, `tep anchor show`, `tep auto`, graph cleanup, example-tag hygiene, or handling `#tepignore` / `#tepignoreafter` lines. Do not use for generic repo exploration in projects that are not using `tep`.
---

Use `tep` to reduce blind repo reading and keep the graph trustworthy over time.

For LLM work, optimize for:
- small grounded retrieval before broad reading
- stable canonical entities
- clean graph signal with minimal example pollution
- reproducible reset behavior

## Workflow

1. Confirm a `tep` workspace exists.
2. Decide whether the task is mainly:
   - retrieval
   - graph maintenance
   - graph hygiene
3. Start with the smallest useful `tep` command.
4. Prefer fixing graph quality over compensating with broad repo scans.

## Retrieval-first command order

### 1. Entity-centered retrieval
Use first when the task mentions a concept, workflow, component, schema, or document topic.

```bash
tep entity context <name-or-id>
tep entity show <name-or-id>
```

Prefer `entity context` when you want:
- canonical `ref`
- anchor-backed locations
- nearby related files/entities

Use `entity show` when you only need the graph shape quickly.

### 2. Anchor-centered retrieval
```bash
tep anchor show <name>
tep anchor list
```

Use when a durable anchor name is already known.

### 3. Entity list
```bash
tep entity list
```

Use to discover the right concept name before reading files.

## Current maintenance commands

```bash
tep auto <pathspec...>
tep health [path]
tep reset --yes
```

Notes:
- `tep auto` is the primary sync command
- `tep reset --yes` should be treated as a rebuild + validation pass
- `tep health` is the audit view for drift, duplicates, and graph hygiene

## TEP-2 syntax

### Real tags
Use real tags only where you genuinely want the repo graph to persist a concept.

Entity:
```txt
#!#tep:(entity.name)
#!#tep:(entity.name){ref="./path", description="..."}
```

Relation:
```txt
#!#tep:(entity.a)->(entity.b)
#!#tep:(entity.a)->(entity.b){description="..."}
```

Anchor:
```txt
#!#tep:[anchor.name](entity.a,entity.b)
#!#tep:[anchor.name](entity.a,entity.b){description="..."}
```

### Example tags
Example tags shown in docs should remain visible to readers but should not pollute the graph.

Use:
```txt
#!#tep:[example.anchor](example.entity) #tepignore
#!#tep:(example.entity) #tepignore
```

### Hidden real tags in markdown
If a markdown file needs real graph tags but you do not want them rendered, wrap them in HTML comments.

Use:
```markdown
<!--- #!#tep:(real.entity){description="..."} -->
<!--- #!#tep:[real.anchor](real.entity) -->
```

Rule:
- real tag in markdown: hidden, not ignored
- sample tag in docs: visible, ignored

## Ignore controls

### `#tepignore`
If a line contains `#tepignore`, that line is skipped by `tep` parsers.

Use for:
- visible syntax examples
- fixture-like snippets in docs
- illustrative tag forms

### `#tepignoreafter`
Everything after the first `#tepignoreafter` in a file is skipped.

Use for:
- test modules in source files
- long fixture tails
- sections full of parser examples that should not enter the graph

Typical pattern:
```rust
// #tepignoreafter
#[cfg(test)]
mod tests { ... }
```

## LLM-oriented graph design guidelines

### 1. Prefer canonical entities
Declare an entity where an LLM should start reading.

Good:
- top of the main service module
- top of the authoritative design doc
- top of the schema doc

Bad:
- every mention of the concept
- examples and throwaway fixtures
- secondary references only

### 2. Use descriptions aggressively for important entities
Descriptions help retrieval ranking and reduce ambiguity.

Good:
```txt
#!#tep:(entity.service){description="Service for entity auto-sync, entity reads, and link-aware context assembly"}
```

### 3. Use relations to encode navigational value
Add relations when they help an LLM jump between concepts intentionally.

Good examples:
- service -> repository
- workspace -> reset flow
- doc -> product concept

Avoid weak relations like:
- generic mentions
- obvious adjacency with no retrieval value

### 4. Keep anchors sparse and meaningful
Anchors should mark:
- module entry points
- key functions
- important doc sections
- schema definitions
- lifecycle boundaries

Do not anchor every function or paragraph just because you can.

### 5. Keep example pollution near zero
If a doc teaches syntax, default to ignored examples.
If a markdown file carries real graph meaning, hide the real tags in comments.

### 6. Make reset safe to run often
A healthy repo should survive:
```bash
tep reset --yes
```

without turning docs/examples/tests into junk entities.

## Graph hygiene checklist

When maintaining a repo graph, check:
- does `tep entity list` mostly show real project concepts?
- are example entities leaking from docs/specs/tests?
- do markdown docs hide real tags and show examples visibly?
- do source test modules have `#tepignoreafter` where needed?
- do duplicate anchor examples stay ignored?
- does `tep reset --yes` produce warnings instead of catastrophic failure where possible?

## Practical rules

- Prefer `tep auto` over narrower sync commands unless you are debugging one layer specifically.
- Treat `ref` as the first recommended file to read.
- Treat anchor names as durable identifiers.
- Only `.tepignore` affects scanning; `.gitignore` does not.
- If a file cannot be decoded as text, skip it and surface a warning.
- If a file has duplicate anchor examples, prefer warning/skip over aborting the entire rebuild.

## Reference

Read `references/tep-patterns.md` for compact retrieval and maintenance patterns.