# Open Questions

## Product questions
- What is the narrowest and most immediately useful v1 workflow?
- Is the main value navigation, traceability, or AI context assembly?
- Should entity labels be required, or can IDs stand alone at first?

## Data model questions
- Should `kind` be freeform or constrained?
- Should relation types be fully user-defined in v1?
- Should an entity be allowed to exist with zero anchors long-term?
- Should entity-anchor associations stay minimal in v1, or carry extra metadata?

## Retrieval questions
- What should `tep resolve` return by default?
- How should traversal stop conditions work?
- Should priority behave only as sorting, or also as a hard traversal filter?
- How should cycles be surfaced in graph output?

## Scanning questions
- Should scan cover the entire workspace by default?
- Which files or directories should be ignored automatically?
- How aggressively should stale anchors be pruned?
- What should happen if the same anchor ID appears in multiple files unexpectedly?

## AI questions
- What should a great agent-facing JSON output look like?
- How should token budgets affect traversal and sorting?
- Should relation type influence retrieval ranking?

## Strategic questions
- Is the real product the pointer graph itself, or the retrieval workflows built on top?
- How much should the core stay minimal before optional tooling layers are added?
- What is the smallest POC that proves this model is genuinely useful?
