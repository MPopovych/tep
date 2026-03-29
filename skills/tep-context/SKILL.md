---
name: tep-context
description: Use the local `tep` CLI as a context-routing layer when working in a repository that has a `tep` workspace. Trigger when reviewing a codebase or docs that are already tagged with `tep`, when deciding what files to read next, when an entity or anchor name appears in a user task, or when an agent should assemble a grounded context bundle from `tep entity show`, `tep entity context`, or `tep anchor show` before making changes.
---

Use `tep` to reduce blind repo reading. Prefer a small, grounded retrieval pass before loading many files.

## Workflow

1. Confirm a `tep` workspace exists for the repo tree.
2. Start from the most obvious entity name when possible.
3. Run the smallest useful retrieval command first.
4. Read only the returned files/snippets that look relevant.
5. Fall back to normal repo exploration only when `tep` coverage is missing or weak.

## Preferred command order

### 1. Entity-centered retrieval
Use first when the user task mentions a concept, component, feature, workflow, or doc subject.

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

### 3. Discovery and sync
Use only when you are intentionally updating the graph.

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

## Good agent usage patterns

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

### If you edit docs with tag examples
- keep example-only lines marked with `#tepignore`
- avoid materializing example tags unintentionally

## Limitations

- `tep` coverage may be partial.
- `entity context` snippets are bounded local windows, not semantic summaries.
- If no relevant entity exists, use normal repo exploration and consider suggesting future `tep` coverage.

## Reference

Read `references/tep-patterns.md` when you need concrete command patterns, expected outputs, or guidance on how to interpret `tep` results during repo work.
