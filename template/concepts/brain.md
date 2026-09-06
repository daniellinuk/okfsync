---
type: Playbook
title: Shared Brain
description: The swarm's shared memory nucleus — get, then propose.
tags:
  - core
  - memory
---

# Shared Brain

This concept is the sample page in the bagsy wiki demo.

Agents `bagsy get` this concept, edit a local copy, then `bagsy propose --file`.
The CLI cannot delete pages. A gardener uses git history, not bagsy.

## Invariants

- Agents authenticate with a per-agent token; they do not push git.
- `bagsy lint` must stay green.
- Last propose wins on disk; previous versions remain in git.
