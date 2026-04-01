# Workspace and Ignore Spec

This document describes the intended local workspace behavior for `tep`, including `.tepignore`.

## Workspace shape

A `tep` workspace is expected to be a local project area where:
- files may contain anchor tags
- local graph state is stored under `.tep/`
- scanning and retrieval operate relative to the workspace

## Local data directory

Expected workspace-managed directory:
- `.tep/`

Possible contents later:
- database file
- state files
- migration metadata
- diagnostic artifacts

## Ignore file

Planned feature:
- `.tepignore`

Purpose:
- control which files or directories should be ignored during `tep` traversal and scanning
- reduce noise
- avoid indexing generated, irrelevant, or sensitive files

## Decision: syntax style

`.tepignore` should use a syntax **similar to `.gitignore`**.

That means the intended direction is:
- comments
- blank lines
- directory-oriented patterns
- path-oriented patterns
- familiar matching behavior for users

Important:
- `tep` should **not** honor `.gitignore`
- `.tepignore` is its own rule source
- built-in excludes may still exist separately, but `.gitignore` should not be treated as an input source for `tep`

## Decision: architecture

Ignore handling should be implemented as a **separate reusable filter component**, not embedded inside `anchor_service`.

Reason:
- path filtering will likely be useful across multiple modules later
- it should be reusable for anchor traversal, retrieval, diagnostics, imports, and any future file-walking features
- this keeps service modules focused on application workflows instead of pattern semantics

Planned direction:
- a dedicated filter module
- reusable path filtering API
- anchor service consumes that filter instead of owning ignore parsing itself

## Intended behavior of `.tepignore`

The file should define traversal exclusions for the current workspace.

Typical targets:
- build output
- dependencies
- caches
- generated files
- large irrelevant folders
- local scratch areas such as `playground/`

Examples:
```txt
target/
node_modules/
dist/
coverage/
.venv/
playground/
```

## Current implementation note

Basic `.tepignore` support exists today, but it should be treated as transitional.
The long-term direction is a cleaner reusable filter implementation with Git-like syntax semantics.

## Relationship to traversal behavior

Traversal-heavy commands should apply `.tepignore` before processing file contents.
This improves performance and reduces accidental noise.

## Current non-goal

Even though `.tepignore` should feel similar to `.gitignore`, `tep` should not automatically inherit or merge `.gitignore` behavior.
That separation is intentional.
