# tep spec index

This directory contains more formal and implementation-oriented specs for `tep`.

Unlike `doc/`, which is meant for broader internal project thinking, `spec/` is where behavior and structure can be written down more precisely.

## What belongs here

Examples:
- marker grammar
- scan behavior
- anchor materialization behavior
- retrieval behavior
- storage/schema decisions
- workspace file conventions
- ignore behavior
- DB schema migration expectations

## Current specs

- [V1 Overview](./V1_OVERVIEW.md)
- [Entity Commands Spec](./ENTITY_COMMANDS_SPEC.md)
- [Entity Context Spec](./ENTITY_CONTEXT_SPEC.md)
- [Entity Links Spec](./ENTITY_LINKS_SPEC.md)
- [Anchor Flow Spec](./ANCHOR_FLOW_SPEC.md)
- [Marker and Anchor Spec](./MARKER_AND_ANCHOR_SPEC.md)
- [Workspace and Ignore Spec](./WORKSPACE_AND_IGNORE_SPEC.md)

## Working rule

Specs should stay practical.

They should be precise enough to implement against, but not inflated into unnecessary protocol ceremony too early.
