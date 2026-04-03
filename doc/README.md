# tep internal docs

This directory is the self-managed documentation area for `tep`.

The idea is similar in spirit to a tool documenting and evolving itself.
Over time, this doc set should become one of the first real bodies of material managed with `tep`.

## What belongs here

This is **not** the formal spec directory.

For now, this directory is for practical internal project documentation such as:
- feature planning
- entity modeling
- core module design
- architecture notes
- roadmap-oriented thinking
- implementation backlog and project notes
- workflow guidance and project-local skills

## Suggested reading order

1. [Product Vision](./PRODUCT_VISION.md)
2. [Features](./FEATURES.md)
3. [Entity Catalog](./ENTITY_CATALOG.md)
4. [Core Modules](./CORE_MODULES.md)
5. [Architecture Notes](./ARCHITECTURE_NOTES.md)
6. [Implementation Backlog](./IMPLEMENTATION_BACKLOG.md)
7. [tep Context Skill](../skills/tep-context/SKILL.md)

## Relationship to root docs

Use the root-level docs for:
- public/current project overview (`README.md`)
- current command and behavior summaries (`CLI_DESIGN.md`, `DATA_MODEL.md`)
- roadmap and open questions
- formal behavior commitments in `spec/`

Use `doc/` for:
- evolving internal notes
- architecture thinking
- module-level planning
- backlog material that may change more freely
- project-local workflow guidance/skills

## Working style

These docs should stay:
- practical
- lightweight
- easy to revise
- implementation-aware without becoming overdesigned

The goal is to support thinking and iteration, not to prematurely freeze the system.
