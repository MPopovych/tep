---
name: tep-context
description: Use the local `tep` CLI as a context-routing and graph-maintenance layer only when working in a repository or document set that already has a `tep` workspace or when the user explicitly asks to add or maintain `tep` coverage. Trigger for tasks that involve `tep entity context`, `tep entity show`, `tep anchor show`, `tep auto`, `tep anchor auto`, doc seeding with `tep`, or handling `#tepignore` example lines. Do not use for generic repo exploration in projects that are not using `tep`.
---

Use `tep` to reduce blind repo reading and to keep the graph useful over time. Prefer the smallest grounded retrieval pass first, and update graph coverage intentionally when the task calls for it.

## Workflow

1. Confirm a `tep` workspace exists for the repo tree.
2. Decide whether the task is mainly:
   - retrieval
   - maintenance (tagging + syncing)
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
- related anchors with snippets
- related file shortlist

Use `entity show` when you only need the graph shape quickly.

### 2. Anchor-centered retrieval
Use when you already have an anchor name or id.

```bash
tep anchor show <name>
tep anchor list
```

### 3. Entity list
```bash
tep entity list
```

Use to browse available entity names when the right entity is not obvious.

## Maintenance commands

Use these only when intentionally updating graph coverage.

```bash
tep auto <pathspec...>   # scan for entity declarations
tep anchor auto <pathspec...>   # scan for anchor tags and sync relations
tep reset --yes                 # nuke DB and re-index workspace from scratch
tep health [path]               # audit anchor state
```

## Tag syntax

### Anchor tags — placed in source files and docs

```
#!#tep:[anchor_name](entity1,entity2)
```

Rules:
- `anchor_name`: lowercase, `[a-z0-9._]`, not purely numeric, must be unique across the workspace
- `(entity1,entity2)`: required — at least one entity ref, comma-separated names
- The name is the durable identity; numeric IDs are internal
- `anchor auto` does **not** rewrite the file — the tag is already in final form

### Entity declaration tags — placed where an entity is canonically defined

```
#!#tep:(entity_name)
```

Rules:
- `entity_name`: same charset as anchor names
- `entity auto` ensures the entity exists in the DB and fills `ref` with the declaring file path if missing
- No anchors are created by `entity auto` — it only registers entities

### Ignoring lines

```
some text #!#tep:[example](student) #tepignore
```

Any line containing `#tepignore` is skipped entirely by both parsers.

For large fixture or test tails, use:

```
// #tepignoreafter
```

Everything after the first occurrence is ignored for the rest of the file.

## How to add tep coverage to a project

### 1. Init the workspace (once)

```bash
tep init
```

Creates `.tep/`, `.tep/tep.db`, `.tepignore`.

### 2. Declare key entities in their canonical files

Place entity declaration tags where a concept is primarily defined — top of a file, a key function, a doc section header:

```rust
// #!#tep:(auth_flow)
pub fn authenticate(user: &User) -> Result<Token> { ... }
```

```markdown
#!#tep:(student)
# Student

A learner enrolled in the system.
```

Then run:

```bash
tep auto ./src
tep auto ./docs
```

This creates each entity and records the file as its `ref`.

### 3. Tag important locations with named anchors

Place anchor tags at locations that are worth pointing to — entry points, schema definitions, key algorithm sections, important doc paragraphs:

```rust
// #!#tep:[auth.token_generation](auth_flow,token)
fn generate_token(claims: &Claims) -> String { ... }
```

```markdown
#!#tep:[student.enrollment_rules](student,enrollment)
## Enrollment rules

Students may enroll in up to 6 subjects per semester.
```

Then run:

```bash
tep anchor auto ./src
tep anchor auto ./docs
```

### 4. Connect entities with links (optional but powerful)

```bash
tep entity link student enrollment "student participates in enrollment"
tep entity link auth_flow token "auth flow produces token"
```

### 5. Query context

```bash
tep entity context auth_flow
tep entity context student --link-depth 2
```

## Tagging guidelines for agents

**Placement:**
- One anchor per meaningful unit (function, section, important paragraph)
- Entity declarations at canonical definition points — not everywhere the entity is mentioned
- Prefer anchors at stable section boundaries, not volatile line ranges

**Naming:**
- Use dot-notation for hierarchy: `auth.token_generation`, `student.permissions`
- Use underscore for compound words: `student_processor`, `enrollment_rules`
- Keep names short but unambiguous within the project

**Entity refs:**
- Each anchor must reference at least one entity
- A single anchor can reference multiple entities when one location is genuinely relevant to several concepts
- Don't inflate entity refs just to "cover" a concept — prefer separate anchors

**Coverage density:**
- Core logic: anchor every key function or method
- Docs: anchor section headers and key paragraphs
- Config/schema files: anchor the top-level definition
- Test files: usually skip unless testing behavior worth tracking

**Avoid:**
- Anchoring every single line
- Using generic names like `misc`, `stuff`, `helper`
- Placing anchors in test fixtures without `#tepignore`
- Duplicate anchor names across files

## Practical rules

- Treat `ref` as the primary reading suggestion when present.
- Treat anchor names as durable identity — do not rename without care.
- Treat `line`, `shift`, and `offset` as metadata only — they drift with edits.
- Respect `#tepignore` when editing docs that show example tags.
- Do not assume `.gitignore` affects `tep`; only `.tepignore` does.
- Prefer reading the smallest set of files surfaced by `tep` before broad repo scans.

## Full command reference

```bash
tep init                                          # create workspace
tep reset [--yes]                                 # wipe DB and re-index
tep auto <pathspec...>                               # run entity + anchor sync from tags
tep health [path]                                 # audit workspace

tep auto <pathspec...>                               # run entity + anchor sync from tags
tep entity show <name-or-id>
tep entity context <name-or-id> [--files-only] [--link-depth <n>]
tep entity list

tep anchor auto <pathspec...>                     # scan for #!#tep:[name](entities) tags
tep anchor show <name>
tep anchor list

tep e ...   # shorthand for entity
tep a ...   # shorthand for anchor
```

## Reference

Read `references/tep-patterns.md` for concrete retrieval patterns, interpretation guidance, and maintenance reminders.
