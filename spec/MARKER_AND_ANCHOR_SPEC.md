# Marker and Anchor Spec

This document describes the current planned marker and anchor behavior.

## Marker states

There are two planned states:
- incomplete anchor
- materialized anchor

## Incomplete anchor format

Planned examples:

```txt
[#!#tep:](student)
[#!#tep:](student,basic_user)
[#!#tep:]
```

Important:
- the optional `( ... )` suffix is an entity reference instruction list
- incomplete anchors do not yet contain version or anchor ID information
- the `:` represents the anchor ID slot
- in incomplete form, the slot is empty

## Materialized anchor format

Planned examples:

```txt
[#!#1#tep:123763636473](student)
[#!#1#tep:123763636474](student,basic_user)
```

Without entity reference instruction:

```txt
[#!#1#tep:123763636475]
```

## Marker meaning

Parts of a materialized anchor:
- `[#!#` — special opening pattern
- `1` — tep anchor format/version marker
- `tep:` — namespace and anchor ID slot prefix
- `123763636473` — anchor ID filling the slot
- optional `( ... )` — entity reference instruction list

## Ignore controls

### Line-local ignore
If a line contains:

```txt
#tepignore
```

anchors on that line are ignored.

Use this for:
- isolated example lines
- one-off fake marker literals
- regex/test/example strings that only affect a single line

### File-tail ignore
If a file contains:

```txt
#tepignoreafter
```

then everything after the first occurrence of that marker is ignored by anchor parsing and auto-indexing.

Use this for:
- fixture files
- intentionally broken examples
- unit-test data stored below a cutoff marker
- `#[cfg(test)]` modules in source files when the whole tail is non-canonical graph material

### Practical rule

Prefer:
- `#tepignore` for a few noisy lines
- `#tepignoreafter` for a large fixture/test tail

## Core rules

1. The materialized marker contains the **anchor ID**.
2. The marker does **not** contain an entity ID as its durable identity.
3. The entity reference instruction list is optional.
4. A single anchor may be associated with multiple entities.
5. A single entity may be associated with multiple anchors.
6. Repeated incomplete entity reference instructions in different physical positions still become different anchors.
7. Multiple entity references may be listed with comma separation inside `( ... )`.

## Parsing expectations

A scanner or anchor-sync routine should:
- detect incomplete anchors
- detect materialized anchors
- parse version when present
- parse anchor ID when present
- parse optional entity reference instruction list when present
- associate the discovered anchor with the current file path
- optionally record current location metadata (`line`, `shift`, `offset`)
- stop parsing the remainder of a file after `#tepignoreafter`

## Durability rule

`line`, `shift`, and `offset` are not identity.
They are only current or last-known metadata.

## Future evolution

Because the materialized marker includes a version field, the syntax can evolve later while preserving backward-compatible scanning behavior.

## Relationship to `tep anchor`

The planned `tep anchor <pathspec...>` command is responsible for turning incomplete anchors into materialized anchors and synchronizing file-local anchor state.
