# CLI Design

## Design goals

The CLI should be:
- simple,
- scriptable,
- local-first,
- composable,
- readable,
- and predictable.

The goal is not to be clever.
The goal is to reliably manage a small graph of entities, anchors, entity-anchor associations, and links.

## Git-like similarities

The similarity to git should be mostly in ergonomics:
- one binary,
- subcommands,
- local workspace awareness,
- inspectable local state,
- shell-friendly commands.

`tep` is not a version-control system.

## Basic workflow

1. initialize a workspace
2. create entities
3. place anchor tags into files
4. scan files to discover anchors
5. connect entities to anchors
6. connect entities to other entities
7. query and traverse the graph

## Core command areas

### Workspace
- `tep init`
- `tep status`
- `tep doctor`

### Entities
- `tep entity create`
- `tep entity show`
- `tep entity list`
- `tep entity update`

### Anchors
- `tep anchor show`
- `tep anchor list`
- `tep scan`

### Entity-anchor associations
- `tep attach`
- `tep detach`

### Links
- `tep link add`
- `tep link list`
- `tep link remove`

### Retrieval / graph
- `tep resolve`
- `tep graph`
- `tep context get`

## Example flows

### Create entities
```bash
tep entity create "Student"
tep entity create "BasicUser"
```

### Place anchor tag manually in code
```java
//[#@#1#tep:123763636473]
public BasicUser fromStudent(Student student) { ... }
```

### Scan the workspace
```bash
tep scan
```

### Attach multiple entities to the same anchor
```bash
tep attach <student-entity-id> 123763636473
tep attach <basic-user-entity-id> 123763636473
```

### Link entities
```bash
tep link add <student-entity-id> converts-to <basic-user-entity-id> --priority 1
```

### Resolve an anchor or entity
```bash
tep anchor show 123763636473
tep resolve <student-entity-id>
```

Possible output:
- entity metadata
- connected anchors
- connected entities
- links sorted by priority
- current file paths and position metadata

### Traverse the graph
```bash
tep graph <student-entity-id> --depth 2 --limit 20
```

Possible traversal controls:
- `--depth`
- `--limit`
- `--max-priority`
- `--relation`
- `--json`

## Priority semantics

Priority is user-defined, but the main convention is:
- **1 is best / strongest / most important**
- larger numbers are less important

In practice, priority should mostly affect:
- sorting,
- traversal order,
- result limiting.

The tool should avoid over-interpreting priority beyond that.

## Location semantics

Anchors may expose file path, line, and offset metadata.

But because files change over time:
- location metadata is useful,
- location metadata is not fully trustworthy,
- the anchor ID inside the file tag is the durable identity.

## Retrieval philosophy

A retrieval command should ideally answer:
- what entities are attached to this anchor?
- where are those anchors currently found?
- what are these entities linked to?
- where are those related things anchored?

That is the core utility of the tool.

## Output modes

Default output should be human-readable.

Useful optional output modes:
- `--json`
- `--paths`
- `--compact`

## Non-goals for v1

- deep code intelligence
- AST-based language support
- editor plugin integration
- remote sync
- automatic relation inference
