# V1 Overview

This document summarizes the baseline `tep` feature set before TEP-2 cleanup work.

## Core features

- Local workspace initialization with `tep init`
- SQLite-backed graph storage in `.tep/tep.db`
- Entity storage with names, refs, and descriptions
- Anchor storage with file location metadata
- Anchor-to-entity attachments
- Directional entity links
- Entity inspection via `show`, `list`, and `context`
- Anchor inspection via `show` and `list`
- Health checks for graph drift and orphaned records
- JSON output support
- `.tepignore` support for scan exclusion

## Baseline syntax

- Entity declaration tags
- Anchor tags

## Baseline workflow

- initialize workspace
- scan files
- inspect entities and anchors
- reset and rebuild the graph

## Follow-up milestones

- TEP-1: DTO / JSON output layer
- TEP-2: richer syntax and auto-driven reconstruction
- TEP-3: plugin-facing interface improvements
