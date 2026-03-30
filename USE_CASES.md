# Use Cases

## 1. Product and engineering traceability

You have:
- a product spec
- backend code
- frontend code
- tests
- architecture notes

You create entities for meaningful things such as:
- endpoints
- features
- DTOs
- services
- UI flows

You place anchor tags into relevant files, attach entities to anchors, and connect entities with links.

Potential value:
- easier onboarding
- better implementation traceability
- faster debugging
- cleaner AI context retrieval

## 2. Writing and worldbuilding

You have:
- a manuscript
- character notes
- location notes
- plot notes

You create entities for:
- characters
- places
- scenes
- plot threads

You anchor them in the manuscript and link them to supporting notes.

Potential value:
- consistency checking
- easier editing
- faster navigation of worldbuilding context

## 3. Research and note systems

You have:
- research notes
- source excerpts
- article drafts
- claims and references

You create entities for:
- concepts
- claims
- sources
- themes

Potential value:
- clearer traceability
- better synthesis
- easier audit of claims and evidence

## 4. AI context aggregation

You have:
- tasks
- docs
- source files
- implementation notes
- prior decisions

An agent can:
- resolve an entity
- read the primary `ref`
- inspect related anchors
- extract local snippets around those anchors
- collect a small file shortlist
- assemble a more grounded context set

Potential value:
- better prompt composition
- less irrelevant context
- more repeatable context retrieval
- better grounded outputs

## 5. Cross-repo or cross-folder linking

You have material split across multiple projects or folders.

A shared entity graph can connect them even when the files live in different places.

Potential value:
- fewer silos
- easier navigation across boundaries
- stronger project-level understanding

## 6. Shared code locations

One anchor can connect to several entities.

This is useful when one place in code or docs is genuinely relevant to multiple concepts.

Example:
- a conversion method may connect to `student`, `basic_user`, and `user-conversion`
- without requiring several stacked anchor tags at the same location

This reduces visual clutter while keeping the graph expressive.
