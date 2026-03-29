# Entity Context Spec

This document describes a minimal agent-friendly retrieval command for `tep`.

## Goal

The goal is to make `tep` useful not only for storing a graph, but also for assembling the next useful context bundle for a human or agent.

The command should answer:

> Given this entity, what should I read next?

## Proposed command

```bash
tep entity context <name-or-id>
```

Possible shorthand later:
```bash
tep e context <name-or-id>
```

## Main use case

An agent often does not want the full graph.
It wants the smallest useful working set.

For example:
- the entity itself
- the primary `ref`
- the top related anchors
- a small snippet around each anchor
- a deduplicated shortlist of files worth reading next

That is more useful for context assembly than raw graph metadata alone.

## Minimal v1 behavior

For a target entity, the command should return:

1. entity header
2. entity `ref` if present
3. related anchors in compact form
4. a small text snippet around each anchor when possible
5. a deduplicated file list at the end

## Files-only mode

A lightweight retrieval mode should also be available:

```bash
tep entity context <name-or-id> --files-only
```

In this mode, the command should return only:
- entity header
- `ref` if present
- deduplicated related file list

This is useful when the caller wants a file shortlist without being tied to any specific document format or snippet strategy.

## Proposed text output

Example shape:

```txt
1 (tep)
ref: ./README.md

anchor 19
./README.md (20:0) [472]
snippet:
## Workspace model
A `tep` workspace is created with:

anchor 10
./CLI_DESIGN.md (30:0) [458]
snippet:
## Workspace behavior
### Initialize a workspace

files:
- ./README.md
- ./CLI_DESIGN.md
```

Files-only example:

```txt
1 (tep)
ref: ./README.md
files:
- ./README.md
- ./CLI_DESIGN.md
```

## Output rules

### Entity header
Reuse current compact entity format:

```txt
<id> (<name>)
```

### Ref line
Only show when present:

```txt
ref: <value>
```

### Anchor block
Reuse current compact anchor block:

```txt
anchor <anchor_id>
<file> (<line>:<shift>) [<offset>]
```

### Snippet block
If snippet extraction succeeds:

```txt
snippet:
<text>
```

If snippet extraction fails, omit the snippet block rather than printing placeholder noise.

### Files block
At the end, print deduplicated file paths in display order:

```txt
files:
- <path>
- <path>
```

## Snippet behavior

The first version should stay simple.

Suggested approach:
- read the anchor file
- use the last-known offset if available
- extract a small surrounding byte window around the anchor
- clamp safely to file boundaries
- trim to valid UTF-8 character boundaries
- then snap outward to nearby line boundaries for cleaner output
- keep snippet length small and predictable

A snippet does not need to be perfect.
It needs to be useful enough for context routing.

## Ordering

Suggested order:
1. `ref` first
2. anchors sorted by anchor id ascending, or by file then anchor id
3. files deduplicated in first-seen order

A later version may sort by priority or stronger ranking rules.

## Limits

Useful future flags:

```bash
tep entity context <target> --limit 5
tep entity context <target> --snippet-bytes 240
tep entity context <target> --files-only
tep entity context <target> --json
```

## Why this matters for agents

Current `entity show` is good for graph inspection.
It is less good for choosing what to read next.

`entity context` improves agent work because it:
- surfaces the primary `ref`
- surfaces actual local text, not just anchor coordinates
- gives a short file shortlist
- reduces manual file hopping
- makes prompt/context assembly more repeatable

`--files-only` is useful when:
- the caller only wants the next files to read
- the caller does not want snippet formatting assumptions
- the repository mixes different content styles

## Non-goals for first version

Not required in the first slice:
- graph traversal through links
- priority-aware ranking
- deep snippet semantics
- AST-aware extraction
- token-budget optimization
- recursive context assembly

Those can come later.

## Recommended implementation slice

Smallest useful implementation:
1. add `entity context <name-or-id>`
2. resolve entity and related anchors
3. print `ref` if present
4. extract a short bounded snippet around each anchor when possible
5. print a deduplicated file list
6. add `--files-only` to skip anchors and snippets when only file routing is needed

That would already make `tep` significantly more useful for agent workflows.
