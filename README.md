# tep

`tep` is a local-first CLI for **text entity pointers**.

It is a simple, domain-agnostic tool for connecting logical entities to anchor points in text, linking entities together, and retrieving related context across files.

## Core idea

A lot of useful context is fragmented across:
- docs,
- backend code,
- frontend code,
- notes,
- manuscripts,
- and planning files.

`tep` gives you a minimal system to connect those pieces explicitly.

## Current vision

At a high level:
- `tep` stores **entities**,
- users place **anchor tags** and **entity declaration tags** in files,
- anchor tags contain their own **anchor ID**,
- entities can connect to multiple anchors,
- anchors can connect to multiple entities,
- entities can also link to other entities.

This makes it possible for a human or an agent to:
- read an anchor ID from a file,
- resolve which entities are attached to it,
- discover entities declared in files,
- follow entity links,
- collect related anchors,
- and assemble context from connected files.

## Example syntax

### Anchor tag
In Java, for example:

```java
// [#!#1#tep:123763636473]
```

Meaning:
- `#1#` is the tep version marker for future compatibility
- `123763636473` is the **anchor ID**
- square brackets identify an anchor tag

### Entity declaration tag
An entity declaration uses parentheses:

```txt
(#!#tep:Student)
```

After `tep entity auto <file>`, it becomes:

```txt
(#!#1#tep:Student)
```

Meaning:
- parentheses identify an entity declaration marker
- `Student` is the entity name
- `entity auto` ensures the entity exists
- if the entity has no `ref`, the declaring file path is stored in `ref`
- a backing anchor relation is created for that declaration location

## Design principles

- **simple first** — avoid unnecessary abstraction
- **domain-agnostic** — works for code, docs, writing, research, and more
- **local-first** — no server required
- **CLI-driven** — scriptable and easy to automate
- **explicit graph** — relationships are user-defined
- **future-compatible markers** — versioned syntax

## Tech direction

Current preferred implementation direction:
- **Rust** for the CLI
- **SQLite** for internal storage

Why:
- small portable binary,
- good performance and reliability,
- clean CLI ecosystem,
- straightforward local database integration.

## Documentation map

### Project context docs
- [Concept](./CONCEPT.md)
- [Data Model](./DATA_MODEL.md)
- [CLI Design](./CLI_DESIGN.md)
- [Use Cases](./USE_CASES.md)
- [Roadmap](./ROADMAP.md)
- [Open Questions](./OPEN_QUESTIONS.md)
- [Development Notes](./DEV_NOTES.md)

### Internal self-managed docs
- [Internal Docs Index](./doc/README.md)
  - product vision
  - feature tracking
  - entity catalog
  - core modules
  - architecture notes
  - implementation backlog

### Formal / evolving specs
- [Spec Index](./spec/README.md)
  - protocol and behavior drafts
  - implementation-oriented specs
  - workspace features such as `.tep_ignore`

## Important note on future tooling

Editor integrations such as a VS Code plugin should be treated as a separate future layer.
The core product is the CLI and its local graph model.

## Forward note

After this documentation stage, the next logical step is a small proof of concept.
That POC can later become one of the first real projects tracked using `tep` itself.
