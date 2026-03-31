# tep

> ⚠️ **Fully agent-coded.** This project was built entirely by an AI coding agent (Makki) with human direction. The code, tests, docs, and this README were written by the agent.

---

## What is tep?

`tep` is a local-first CLI for **text entity pointers** — a lightweight way to connect concepts to locations in your codebase and docs.

It maintains a local SQLite graph of **entities** (things you care about), **anchors** (tagged locations in files), and **links** (relationships between entities).

---

## Why tep?

Codebases grow. Concepts scatter across files. Documentation drifts from implementation.

**tep solves this by giving concepts a durable identity.**

Instead of grepping for "student" and hoping you find the right places, you:

1. **Tag locations** — drop anchor markers in code, docs, configs
2. **Name concepts** — create entities like `student`, `auth_flow`, `pricing_model`
3. **Connect them** — attach entities to anchors, link entities to each other
4. **Query the graph** — ask "where does `student` appear?" or "what's related to `auth_flow`?"

**Use cases:**

- **Onboarding** — new devs can trace concepts across the codebase
- **Refactoring** — see everywhere a concept touches before changing it
- **Documentation** — keep docs linked to the actual code locations
- **AI context** — feed precise, relevant code slices to LLMs instead of dumping entire files
- **Architecture** — map how logical concepts relate to each other

The graph lives in your repo (`.tep/`), works offline, and stays under your control.

---

## Current capabilities

- Initialize and auto-migrate local workspaces
- Create, edit, list, and query entities with names, descriptions, and refs
- Enforce entity name normalization (`student`, not `Student` — lowercase, `[a-z0-9._]`)
- Materialize anchor tags in files — numeric (`[#!#1#tep:42]`) or named (`[#!#1#tep:student_processor]`)
- Named anchors: assign human-readable names to anchors, look them up by name, edit them via CLI
- Auto-declare entities from declaration markers (`(#!#tep:student)`)
- Attach/detach entities to anchors by entity name or id, anchor name or id
- Directional entity-to-entity links with free-text relations
- Assemble retrieval-oriented context bundles for entities (`entity context`)
- Bounded link traversal (`--link-depth`)
- Audit anchor health and auto-repair metadata
- List all anchors in the workspace (`anchor list`)
- Respect `.tep_ignore` for test fixtures and examples

---

## Workspace model

[#!#1#tep:19](workspace,workspace.discovery)

A `tep` workspace is created with:

```bash
tep init
```

That creates:
- `.tep/`
- `.tep/tep.db`
- `.tep_ignore`

For commands that require the database, `tep` resolves the **nearest ancestor workspace** from the current working directory.

That means:
- inside a workspace tree, commands work from nested directories
- outside any workspace, DB-requiring commands fail and tell you to run `tep init`
- the location of the `tep` binary does not define the workspace
- the current terminal cwd does

## Schema and migration

`tep` stores its graph in SQLite and tracks schema version with:

```sql
PRAGMA user_version
```

Current behavior:
- `tep init` creates or upgrades the workspace DB
- normal DB-opening commands also auto-migrate older workspaces
- older DBs are upgraded in place when new columns/tables are required

## Core syntax

### Anchor tags

[#!#1#tep:20](anchor,anchor.tag)

Anchor tags use square brackets.

Incomplete (anonymous):
```txt
[#!#tep:](student) #tepignore
```

Materialized (numeric id):
```txt
[#!#1#tep:42](student) #tepignore
```

Materialized (named):
```txt
[#!#1#tep:student_processor](student) #tepignore
```

Meaning:
- square brackets identify an anchor tag
- the value after `tep:` is either a numeric anchor ID or a human-readable name
- numeric IDs are assigned automatically by `anchor auto`
- names are either set in the file directly or assigned via `anchor edit <id> --name <name>`
- the optional `( ... )` suffix is an entity reference instruction list

### Entity declaration tags

[#!#1#tep:21](entity,entity.declaration)

Entity declaration tags use parentheses.

Incomplete declaration:
```txt
(#!#tep:student) #tepignore
```

Materialized declaration:
```txt
(#!#1#tep:student) #tepignore
```

Meaning:
- parentheses identify an entity declaration marker
- `student` is the entity name
- `tep entity auto` ensures the entity exists
- if the entity has no `ref`, the declaring file path is stored in `ref`
- a backing anchor relation is created for that declaration location

## Ignore controls

### `#tepignore`
Ignores only the current line.

Use it for:
- one-off example lines
- regex/test strings that look like markers
- isolated fake literals in source or docs

Example:
```rust
let example = "[#!#tep:](student)"; // #tepignore
```

### `#tepignoreafter`
Ignores the rest of the file after the first occurrence.

Use it for:
- test modules
- large fixture tails
- intentionally broken example sections

Example:
```rust
// real implementation above

// #tepignoreafter
#[cfg(test)]
mod tests {
    ...
}
```

Practical rule:
- prefer `#tepignore` when a few lines are noisy
- use `#tepignoreafter` when an entire tail section is fixture/test territory

## Current command surface

### Workspace
```bash
tep init
tep version
tep -V
tep --version
tep health [path]
```

### Entities
```bash
tep entity create <name> [--ref <value>] [--description <value>]
tep entity ensure <name> [--ref <value>]
tep entity auto <pathspec...>
tep entity show <name-or-id>
tep entity context <name-or-id> [--files-only] [--link-depth <n>]
tep entity edit <name-or-id> [--name <value>] [--ref <value>] [--description <value>]
tep entity link <from> <to> --relation <text>
tep entity unlink <from> <to>
tep entity list
```

Shorthand:
```bash
tep e ...
```

### Anchors
```bash
tep anchor auto <pathspec...>
tep anchor show <anchor-id-or-name>
tep anchor edit <anchor-id> --name <name>
tep anchor list
```

Shorthand:
```bash
tep a ...
```

### Manual relations
```bash
tep attach <entity-id-or-name> <anchor-id-or-name>
tep detach <entity-id-or-name> <anchor-id-or-name>
```

## Health and root repair

`tep health` performs a read-only anchor health audit.

Example:
```bash
tep health
tep health ./docs
```

`tep anchor auto .` is the repair path for a workspace root.

In practice, repo-wide health and auto-fix should usually be paired with a sensible `.tep_ignore` so example fixtures and intentionally broken samples do not pollute canonical workspace health.

## Output style

Default output is intentionally compact.

Entity:
```txt
<id> (<name>)
```

Anchor:
```txt
<anchor_id>
<file> (<line>:<shift>) [<offset>]
```

Location fields are metadata, not identity.
Anchor identity is the anchor ID.

`entity context` is more retrieval-oriented and may include:
- primary `ref`
- description
- anchor snippets
- deduplicated files
- linked entities with explicit edge notation like:
  ```txt
  edge: (1->2)[1] student has subjects
  ```

## Notes

- entity IDs are integer-only
- entity names are unique, normalized to lowercase, charset `[a-z0-9._]`
- entity names cannot be purely numeric
- entity metadata includes `ref` and `description`
- entity links are directional in storage
- anchor names are optional, unique, same charset as entity names (`[a-z0-9._]`)
- anchor names cannot be purely numeric
- `anchor show`, `attach`, and `detach` accept either id or name
- `anchor edit <id> --name <name>` assigns/renames and rewrites the file tag in-place
- `entity context` always includes linked entities by default
- `.tep_ignore` is respected; `.gitignore` is not
- `line`, `shift`, and `offset` are refreshable metadata only; anchor identity is the id or name in the tag
- `shift` and `offset` are byte-oriented in practice

## Repo self-check

The `tep` repo uses `tep` to track its own entities and anchors.
`tep health` in the repo root reports clean for all source and doc files.
The README contains example anchor tags that are intentionally excluded via `#tepignore`.

## Documentation map

### Product / overview
- [Concept](./CONCEPT.md)
- [Data Model](./DATA_MODEL.md)
- [CLI Design](./CLI_DESIGN.md)
- [Use Cases](./USE_CASES.md)
- [Roadmap](./ROADMAP.md)
- [Open Questions](./OPEN_QUESTIONS.md)
- [Development Notes](./DEV_NOTES.md)

### Specs
- [Spec Index](./spec/README.md)
- [Entity Commands Spec](./spec/ENTITY_COMMANDS_SPEC.md)
- [Entity Context Spec](./spec/ENTITY_CONTEXT_SPEC.md)
- [Entity Links Spec](./spec/ENTITY_LINKS_SPEC.md)

### Internal docs
- [Internal Doc Index](./doc/README.md)
