---
type: Playbook
title: Shared Brain
description: The swarm's shared memory nucleus — claim before you write.
tags:
  - core
  - memory
---

# Shared Brain

This concept is the contested piece of shared memory in the bagsy collide demo.

Agents that need to update swarm priors **must** `bagsy claim` this concept first,
edit on their `bagsy/<agent>/…` branch, then `bagsy propose`. Never push `main`.

## Invariants

- One writer at a time (enforced by `bagsy serve` locks).
- Agents authenticate with a per-agent token; they do not push git.
- `bagsy lint` must stay green.
