---
type: Playbook
title: Routing
description: How work is handed off between agents without stomping concepts.
tags:
  - routing
---

# Routing

1. `bagsy get <concept>` — read current knowledge.
2. `bagsy claim <concept>` — bagsy the concept; open a branch.
3. Edit the markdown concept.
4. Commit on your bagsy branch.
5. `bagsy propose` — push + open PR/MR (never main).
6. `bagsy release <concept>` after merge (or when abandoning).
