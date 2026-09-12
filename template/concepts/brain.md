---
type: Playbook
title: Shared Brain
description: The swarm's shared memory nucleus — get, then propose.
tags:
  - core
  - memory
---

# Shared Brain

This concept is the sample page in the okfsync wiki demo.

Agents `kbsync get` this concept, edit a local copy, then `kbsync propose --file`.
The CLI cannot delete pages. A gardener uses git history, not okfsync.

## Invariants

- Agents authenticate with a per-agent token; they do not push git.
- `kbsync lint` must stay green.
- Last propose wins on disk; previous versions remain in git.
