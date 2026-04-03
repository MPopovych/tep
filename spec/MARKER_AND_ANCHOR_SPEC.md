# Marker and Anchor Spec

This document describes the current anchor tag syntax and behavior.

## Anchor tag format

The canonical anchor tag format is:

```txt
#!#tep:[anchor_name](entity1,entity2)
```

Rules:
- `#!#tep:[` — fixed opening pattern
- `anchor_name` — required, human-readable name (charset: `[a-z0-9._]`, not purely numeric)
- `(entity1,entity2)` — required entity reference list (at least one entity)
- anchors without entity refs are ignored

Examples:
```txt
#!#tep:[student_processor](student)
#!#tep:[auth_flow](auth,session)
```

## Anchor name rules

- lowercase letters, digits, dots, and underscores only: `[a-z0-9._]`
- mixed-case input is normalized to lowercase on parse
- purely numeric names are rejected
- dashes are not allowed

Valid: `student_processor`, `auth.flow`, `v2_login`
Invalid: `student-processor`, `123`, `AuthFlow`

## Ignored / rejected tags

The following are silently ignored:

```txt
#!#tep:[]                     ← no name
#!#tep:[my_anchor]            ← no entity refs
#!#tep:[my_anchor]()          ← empty entity refs
#!#tep:[123](student)         ← purely numeric name
#!#tep:[bad-name](student)    ← invalid charset
[#!#1#tep:name](student)      ← old version-prefixed format, not recognized
```

## Entity reference list

The `(...)` suffix is a list of entity names, comma-separated.

Behavior:
- each name is resolved by name in the DB; if missing, it is auto-created
- the anchor is linked to all listed entities after `anchor auto` runs
- the list replaces previous relations for that anchor on each sync

## Ignore controls

### Line-local: `#tepignore`

Anchors on a line containing `#tepignore` are ignored.

```txt
#!#tep:[example](student) #tepignore
```

Use for docs, examples, test strings that show anchor syntax.

### File-tail: `#tepignoreafter`

Everything after the first occurrence of `#tepignoreafter` is ignored.

```txt
// #tepignoreafter
#[cfg(test)]
mod tests { ... }
```

Use for test modules and fixture tails.

**Rule of thumb:** `#tepignore` for a few noisy lines, `#tepignoreafter` for entire test/fixture tails.

## Under the hood

Anchors still have numeric `anchor_id` in the DB — they are shown in `anchor list` and `anchor show` output. The name is the tag identity; the numeric ID is internal.

## Location metadata

`line`, `shift`, and `offset` are refreshable metadata, not identity. They are updated on each `anchor auto` run.

## Relationship to `tep anchor auto`

`tep anchor auto <pathspec...>` scans files, registers new named anchors, refreshes location metadata, and syncs entity relations. It does not rewrite files.
