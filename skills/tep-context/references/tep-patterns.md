# tep patterns

## Goal

Use `tep` to identify the smallest relevant context bundle before reading many files, and to maintain graph coverage deliberately when needed.

---

## Retrieval patterns

### Entity context (preferred first step)
```bash
tep entity context <name-or-id>
tep entity context <name-or-id> --link-depth 2
tep entity context <name-or-id> --files-only
```

Typical output interpretation:
- `ref` = canonical definition file — read this first
- anchor list with snippets = grounded code/doc locations
- files list = shortlist of related files to scan
- linked entities = related concepts to follow if needed

### Entity show (compact graph view)
```bash
tep entity show <name-or-id>
```

Use when you only need the entity's metadata and its graph connections without reading snippets.

### Anchor show
```bash
tep anchor show <name>
```

Use when a specific anchor name is known and you want to see its location and attached entities.

### List-based discovery
```bash
tep entity list       # browse all known entities
tep anchor list       # browse all known anchors
```

Use when the right entity name is not obvious.

---

## Retrieval strategy

### Start narrow
If the user gives a likely entity name:
1. run `tep entity context <entity>`
2. read `ref` first
3. read only the returned files that look relevant

### Fall back carefully
If the entity is missing or coverage is thin:
1. try a nearby entity name or a shorter prefix
2. try `tep entity list` to scan for the right name
3. then fall back to normal repo exploration

### Signals to trust

**Strong signals:**
- entity `ref` pointing to a well-named file
- multiple anchors across docs and code for the same entity
- entity links forming a coherent subgraph

**Weak signals:**
- single-anchor entities with no `ref`
- stale-seeming snippets that don't match what you find in the file
- very sparse graph with few entities

---

## Tag syntax

### Anchor tag (placed in source/docs)

```
#!#tep:[anchor_name](entity1,entity2) #tepignore
```

- `anchor_name`: unique across workspace, lowercase `[a-z0-9._]`, not purely numeric
- At least one entity ref required — tag is ignored if refs are missing
- `anchor auto` registers the anchor and syncs entity relations without rewriting the file

### Entity declaration tag (placed at canonical definition point)

```
#!#tep:(entity_name) #tepignore
```

- Marks where an entity is primarily defined
- `entity auto` ensures entity exists and fills `ref` with the declaring file path

### Ignore controls

```
example #!#tep:[foo](bar)     ← this line is ignored entirely #tepignore
```

```
//after
// Everything below this line is ignored by tep parsers
#[cfg(test)]
mod tests { ... }
```

---

## Maintenance commands

### Declare entities from files
```bash
tep auto <pathspec...>
```

Scans for `#!#tep:(name)` tags. Creates entities and fills `ref` when missing.
Does **not** create anchors.

### Sync anchors from files
```bash
tep anchor auto <pathspec...>
```

Scans for `#!#tep:[name](entities)` tags. Creates or updates anchor records. Syncs entity relations.

### Reset and re-index
```bash
tep reset --yes
```

Deletes the DB, recreates schema, runs `entity auto .` then `anchor auto .` on the whole workspace.
Leaves `.tepignore` untouched.

### Health check
```bash
tep health
tep health ./docs
```

Reports: moved anchors, missing anchors, duplicate names, unknown names, entities without anchors, anchors without entities.

---

## How to add tep coverage to a new project

### Step 1 — Init
```bash
tep init
```

Edit `.tepignore` to exclude test fixtures, build dirs, generated files:
```
.tep/
.git/
target/
tests/
node_modules/
```

### Step 2 — Declare entities at their canonical locations

Add `#!#tep:(entity_name)` tags where each concept is primarily defined:

```rust
// #!#tep:(payment_flow) #tepignore
pub fn process_payment(order: &Order) -> Result<Receipt> { ... }
```

```markdown
#!#tep:(user) #tepignore
# User

A registered account holder.
```

Then run:
```bash
tep auto ./src
tep auto ./docs
```

### Step 3 — Place named anchor tags at important locations

Add `#!#tep:[name](entities)` tags at meaningful entry points, key algorithms, schema definitions, important doc sections:

```rust
// #!#tep:[payment.validation](payment_flow,order) #tepignore
fn validate_payment_method(method: &PaymentMethod) -> bool { ... }
```

```markdown
#!#tep:[user.permissions](user,permissions) #tepignore
## Permission model

Users have a role assigned at signup...
```

Then run:
```bash
tep anchor auto ./src
tep anchor auto ./docs
```

### Step 4 — Link related entities (optional)
```bash
tep entity link payment_flow order "payment flow processes order"
tep entity link user permissions "user has permissions"
```

### Step 5 — Verify
```bash
tep health
tep entity context payment_flow
```

---

## Tagging guidelines

### Placement
- Entity declarations: at the file or section that canonically defines the concept
- Anchors: at entry points, key function bodies, schema definitions, section headers worth revisiting
- One anchor per meaningful unit — not every line

### Naming
- Use dot-notation for hierarchy: `auth.token_generation`, `payment.refund_flow`
- Use underscore for compound words: `payment_processor`, `user_permissions`
- Keep names short but unique in context

### Entity refs in anchors
- At least one, must be a valid entity name
- Multiple refs when one location genuinely covers several concepts
- Don't artificially inflate refs

### Coverage density
| Location | Guideline |
|---|---|
| Core logic | Anchor key functions and service entry points |
| Docs | Anchor section headers and key paragraphs |
| Config/schema | Anchor the top-level definition |
| Tests | Skip unless the behavior being tested is worth tracking |
| Generated files | Skip |

### Common mistakes to avoid
- Duplicate anchor names across files (causes `anchor auto` to fail)
- Missing entity refs (tag silently ignored)
- Anchoring inside test fixtures without `#tepignore`
- Very generic names like `misc`, `util`, `helper`
- Placing entity declarations everywhere instead of just at the canonical definition

---

## Workspace behavior reminder

- `tep` resolves the nearest ancestor workspace from cwd
- Run it from inside the project tree
- Only `.tepignore` affects scanning — not `.gitignore`
- `tep reset --yes` is the clean-slate option: wipes DB, re-indexes everything

---

## When this skill is most useful

- Repo triage in a `tep`-annotated project
- Doc-first implementation work
- Context assembly for coding tasks
- Understanding architecture/doc relationships with minimal file scanning
- Seeding or extending graph coverage in a new or existing codebase
