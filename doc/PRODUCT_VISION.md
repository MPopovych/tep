# Product Vision

## What tep is

`tep` is a local-first CLI for managing text entity pointers.

Its job is to connect:
- logical entities,
- anchor points in files,
- and relationships between entities.

The result is a lightweight graph that helps humans and agents retrieve relevant context from real project material.

## Product stance

`tep` should stay:
- small,
- explicit,
- composable,
- domain-agnostic,
- useful without a server,
- and useful without an editor plugin.

## What tep is trying to solve

Many projects have context spread across:
- code,
- docs,
- notes,
- drafts,
- plans,
- and design discussions.

That context is usually fragmented and hard to traverse systematically.

`tep` aims to make those relationships explicit so retrieval becomes easier.

## Primary product value

The main value of `tep` is not just storing relationships.
It is making connected context retrievable.

In practical terms, `tep` should help answer:
- what is this thing?
- where is it represented?
- what other things is it related to?
- what files matter if I want context on this entity?

## Current product philosophy

A good early version should prioritize:
- explicit data over inference
- simple commands over magic
- durable local state over network dependence
- useful traversal over broad ambition

## Long-term direction

If the core model proves useful, future layers may include:
- better graph queries
- richer diagnostics
- editor integrations
- visualization
- AI-oriented context assembly helpers

But those should remain layers on top of a simple CLI core.
