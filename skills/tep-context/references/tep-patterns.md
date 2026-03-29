# tep patterns

## Goal

Use `tep` to identify the smallest relevant context bundle before reading many files.

## Common commands

### Entity context
```bash
tep entity context <name-or-id>
```

Useful when you need:
- primary `ref`
- related anchors
- snippets
- deduplicated files

Typical interpretation:
- `ref` = best starting doc/file
- anchor list = additional grounded touchpoints
- files list = shortlist to read next

### Entity show
```bash
tep entity show <name-or-id>
```

Useful when you need a compact graph view without snippets.

### Anchor show
```bash
tep anchor show <anchor-id>
```

Useful when:
- a task references an anchor id
- you want to see which entities are attached to one location

## Suggested reading strategy

### Start narrow
If the user gives a likely entity name:
1. run `tep entity context <entity>`
2. read `ref` first
3. read only returned files that look relevant

### Fall back carefully
If the entity is missing or weakly covered:
1. try a nearby entity name
2. try `tep entity list`
3. then fall back to normal repo exploration

## Signals to trust

### Strong signals
- entity `ref`
- repeated anchors across core docs
- matching entities across docs and code anchors

### Weak signals
- sparse single-anchor entities
- stale-seeming snippets without reinforcing refs/files

## Editing guidance

### If updating graph coverage
Use:
```bash
tep entity auto <pathspec...>
tep anchor auto <pathspec...>
```

### If editing docs with examples
Keep example-only lines marked with:
```txt
#tepignore
```

This prevents incomplete tag examples from being auto-discovered.

## Workspace behavior reminder

`tep` resolves the nearest ancestor workspace from the current cwd.
That means:
- run it from inside the project tree
- do not assume the binary location defines the workspace

## When this skill is most useful
- repo triage in a `tep`-annotated project
- doc-first implementation work
- context assembly for agent coding tasks
- understanding architecture/doc relationships with minimal repo scanning
