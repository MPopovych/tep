# Anchor Flow Spec

This document captures the current anchor workflow for `tep`.

## Core idea

Users place named anchor tags in files, then run `tep anchor auto` to register them in the workspace DB and sync entity relations.

The tool never rewrites anchor tags — names are placed by the user and stay as-is.

## Canonical tag format

```txt
#!#tep:[anchor_name](entity1,entity2) #tepignore
```

Examples:
```txt
#!#tep:[student_processor](student) #tepignore
#!#tep:[auth_flow](auth,session) #tepignore
```

Rules:
- name must be non-empty, not purely numeric, charset `[a-z0-9._]`
- at least one entity ref is required — anchors without entity refs are ignored
- multiple entity refs are comma-separated

## Command

```bash
tep anchor auto <pathspec...>
```

Examples:
```bash
tep anchor auto ./file.md
tep anchor auto .
tep anchor auto ./docs ./src
```

Shorthand: `tep a auto <pathspec...>`

## Workspace requirement

`anchor auto` requires a `tep` workspace. Run `tep init` first.

Commands resolve the nearest ancestor workspace from cwd, so nested directories inside a workspace work fine.

## Behavior of `tep anchor auto`

For each targeted file, the command:

1. Scans for valid anchor tags (`#!#tep:[name](entities)`)
2. For each anchor:
   - If the name exists in DB: update location metadata, sync entity relations
   - If the name is new: create an anchor record, sync entity relations
3. Detects anchors previously registered to this file that are no longer present
4. Deletes stale anchor records for dropped anchors

Files are **never rewritten** — the tag format is the final format.

## Entity ref behavior

When entity refs are present:
- each name is resolved by name; auto-created if it doesn't exist
- anchor-entity relations are **replaced** on each sync (not appended)

## Anchor show

```bash
tep anchor show <name>
tep a show <name>
```

Prints the anchor in compact form plus related entities.

Output format:
```txt
<anchor_id> (<name>)
<file> (<line>:<shift>) [<offset>]
1 (student)
```

## Anchor list

```bash
tep anchor list
tep a list
```

Lists all anchors in the workspace.

## Path selection

Supported inputs:
- direct file path
- directory path
- `.` (current directory)

Respects `.tepignore`. Does not use `.gitignore`.

## Storage model

### Anchors table
- anchor identity (numeric id, internal)
- anchor name (unique, the tag identity)
- current file path
- location metadata (`line`, `shift`, `offset`)
- timestamps

### Anchor-entity relation table
- many-to-many associations between anchors and entities

## Notes on metadata

`line`, `shift`, and `offset` are refreshable. They are useful for display and tooling, not durable identity. The durable identity is the anchor name.
