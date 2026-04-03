# TEP-2 Tag Syntax Spec

## Goal

Use one tag family with a shared prefix:

```txt
#!#tep:
```

The syntax should support:
- entity declaration
- entity metadata
- entity relations
- anchor declarations
- future anchor metadata

All graph state should be recoverable from files after `tep reset --yes`.

---

## Core forms

### Entity

```txt
#!#tep:(entity) #tepignore
#!#tep:(entity) #tepignore{ref="./src/file.rs", description="Main concept"}
```

Meaning:
- declares the entity
- optional metadata fills entity fields
- if the entity already exists, metadata is merged

### Relation

```txt
#!#tep:(entity1)->(entity2)
#!#tep:(entity1)->(entity2){description="depends on"}
```

Meaning:
- declares a directional relation from `entity1` to `entity2`
- optional metadata is attached to the relation

### Anchor

```txt
#!#tep:[anchor](entity1,entity2) #tepignore
#!#tep:[anchor](entity1,entity2){description="Entry point"} #tepignore
```

Meaning:
- declares a named anchor
- attaches the anchor to one or more entities
- optional metadata is attached to the anchor

---

## Naming rules

Entity names and anchor names use:

```txt
[a-z0-9._]
```

Rules:
- lowercase only
- dots and underscores allowed
- not purely numeric
- no dashes

Examples:
- `auth.flow`
- `student_processor`
- `v2_login`

---

## Metadata syntax

Metadata uses a compact key-value block:

```txt
{key="value", key2="value2"}
```

Rules:
- metadata block is optional
- keys are comma-separated
- whitespace around commas and `=` is allowed
- values are quoted strings
- duplicate keys inside the same tag are allowed
- if a key appears multiple times in one tag, the last value wins

Example:

```txt
#!#tep:(auth.flow){ref="./src/auth.rs", description="Login workflow"}
```

---

## Supported metadata fields

### Entity metadata

Supported fields:
- `ref`
- `description`

Example:

```txt
#!#tep:(auth.flow){ref="./src/auth.rs", description="Login workflow"}
```

### Relation metadata

Supported fields:
- `description`

Example:

```txt
#!#tep:(auth.flow)->(session){description="creates session"}
```

### Anchor metadata

Supported fields:
- `description`

Example:

```txt
#!#tep:[auth.entry](auth.flow,session){description="Login entry point"}
```

Unknown metadata fields:
- do not fail parsing
- do not block sync/reset
- mark workspace health as unhealthy/warn

---

## Merge and conflict rules

### Entity metadata across files

If the same entity is declared multiple times:
- metadata is merged when possible
- last value wins for the same field
- this should be reported by `tep health` and reset output as a warning when duplicates exist

Example:

```txt
#!#tep:(auth.flow){ref="./src/auth.rs"}
#!#tep:(auth.flow){description="Login workflow"}
```

Merged result:
- `ref = ./src/auth.rs`
- `description = Login workflow`

Conflict example:

```txt
#!#tep:(auth.flow){description="Login workflow"}
#!#tep:(auth.flow){description="Authentication pipeline"}
```

Resolved result:
- last wins

But:
- duplicate/conflicting declarations should be surfaced as health/reset warnings

### Duplicate fields inside one tag

Does not matter.

Example:

```txt
#!#tep:(auth.flow){description="A", description="B"}
```

Resolved result:
- `description = B`

No hard error required.

---

## Health behavior

`tep health` should warn on:
- duplicate entity metadata declarations across files
- conflicting values where last-wins resolution was applied
- unknown metadata fields
- duplicate relation declarations if detected
- duplicate anchor metadata declarations if detected

Warnings should not block normal operation.

`tep reset --yes` should also surface these warnings after rebuild.

---

## Reset behavior

After reset, the graph should be fully reconstructable from file tags alone:
- entities
- entity refs
- entity descriptions
- relations
- anchors
- anchor/entity attachments
- supported anchor metadata

Recommended reset flow:
1. scan entity declarations and entity metadata
2. scan relation tags
3. scan anchor tags and anchor metadata
4. report warnings

---

## Parsing notes

Parser should not rely on one large regex.

Recommended approach:
1. find candidate `#!#tep:` spans
2. parse subject form:
   - `(entity)`
   - `(entity1)->(entity2)`
   - `[anchor](entity1,entity2)`
3. parse optional metadata block `{...}`
4. validate known fields by tag type
5. keep unknown fields as warnings

---

## Examples

### Entity only

```txt
#!#tep:(workspace)
```

### Entity with metadata

```txt
#!#tep:(workspace){ref="./src/service/workspace_service.rs", description="Workspace lifecycle"}
```

### Relation

```txt
#!#tep:(workspace)->(anchor.sync){description="triggers full anchor sync"}
```

### Anchor

```txt
#!#tep:[workspace.reset](workspace,anchor.sync,entity.service)
```

### Anchor with metadata

```txt
#!#tep:[workspace.reset.meta](workspace,anchor.sync,entity.service){description="Reset entry point"}
```