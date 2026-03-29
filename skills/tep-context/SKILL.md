---
name: tep-context
description: Use the local `tep` CLI as a context-routing and graph-maintenance layer only when working in a repository or document set that already has a `tep` workspace or when the user explicitly asks to add or maintain `tep` coverage. Trigger for tasks that involve `tep entity context`, `tep entity show`, `tep anchor show`, `tep entity auto`, `tep anchor auto`, doc seeding with `tep`, or handling `#tepignore` example lines. Do not use for generic repo exploration in projects that are not using `tep`.
---

Use `tep` to reduce blind repo reading and to keep the graph useful over time. Prefer the smallest grounded retrieval pass first, and update graph coverage intentionally when the task calls for it.

## Workflow

1. Confirm a `tep` workspace exists for the repo tree.
2. Decide whether the task is mainly:
   - retrieval
   - maintenance
   - or both
3. Start with the smallest useful `tep` command.
4. Read or update only the files that look relevant.
5. Fall back to normal repo exploration only when `tep` coverage is missing or weak.

## Retrieval-first command order

### 1. Entity-centered retrieval
Use first when the task mentions a concept, component, feature, workflow, schema, or doc subject.

```bash
tep entity context <name-or-id>
tep entity show <name-or-id>
```

Prefer `entity context` when you want:
- primary `ref`
- related anchors
- snippets
- related file shortlist

Use `entity show` when you only need the graph shape quickly.

### 2. Anchor-centered retrieval
Use when you already have an anchor id.

```bash
tep anchor show <anchor-id>
```

## Maintenance commands

Use these only when you are intentionally updating graph coverage.

```bash
tep entity auto <pathspec...>
tep anchor auto <pathspec...>
```

## Practical rules

- Treat `ref` as the primary reading suggestion when present.
- Treat anchor ids as durable identity.
- Treat `line`, `shift`, and `offset` as metadata only.
- Respect `#tepignore` example lines when editing docs.
- Do not assume `.gitignore` affects `tep`; only `.tep_ignore` does.
- Prefer reading the smallest set of files surfaced by `tep` before doing broad repo scans.
- Prefer small, intentional anchor coverage over dense anchor spam.

## Good retrieval usage patterns

### If the user asks to change a feature
1. infer likely entity name
2. run `tep entity context <entity>`
3. inspect `ref`
4. read surfaced files/snippets
5. then change code/docs

### If the user asks what something means
1. run `tep entity context <entity>` or `tep entity show <entity>`
2. summarize from the surfaced docs and snippets
3. only broaden search if the graph is thin

## Good maintenance usage patterns

### If you are seeding or improving docs
- add a small number of intentional anchors
- anchor section boundaries or key paragraphs
- avoid anchoring every line
- keep example-only tag lines marked with `#tepignore`

### If you are adding entity declarations
- use `tep entity auto <pathspec...>`
- let declarations fill `ref` when missing
- avoid overwriting meaningful existing refs casually

### If you are syncing anchors
- use `tep anchor auto <pathspec...>`
- remember that `[...]` means anchor tags
- remember that `(...)` means entity declarations

## Limitations

- `tep` coverage may be partial.
- `entity context` snippets are bounded local windows, not semantic summaries.
- If no relevant entity exists, use normal repo exploration and consider suggesting future `tep` coverage.

## Reference

Read `references/tep-patterns.md` when you need concrete command patterns, retrieval interpretation guidance, doc-seeding guidance, or maintenance reminders.
