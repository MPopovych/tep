# Open Questions

## Product questions
- What is the narrowest and most immediately useful v1 workflow?
- Is the main value navigation, traceability, or AI context assembly?
- Should entity labels be required, or can IDs stand alone at first?

## Retrieval questions
- Should `tep entity context` become the main agent-facing retrieval command?
- What should the default snippet size be?
- Should `ref` always be shown first when present?
- Should the first version include only direct anchors, or also linked entities later?
- What should the JSON shape for agent-facing retrieval look like?
- How should token budgets affect traversal and sorting later?

## Scanning questions
- Which files or directories should be ignored automatically?
- How aggressively should stale anchors be pruned?
- What should happen if the same anchor ID appears in multiple files unexpectedly?

## AI questions
- What is the smallest useful context bundle for an agent?
- Should retrieval ranking be file-first, anchor-first, or link-aware?
- Should relation type influence retrieval ranking later?

## Strategic questions
- Is the real product the pointer graph itself, or the retrieval workflows built on top?
- How much should the core stay minimal before optional tooling layers are added?
- What is the smallest POC that proves this model is genuinely useful?
