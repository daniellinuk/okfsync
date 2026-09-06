---
type: Playbook
title: Routing
description: How work is handed off between agents without stomping concepts.
tags:
  - routing
---

# Routing

1. `bagsy get <concept>` — read current knowledge.
2. `bagsy claim <concept>` — bagsy the concept (token identity).
3. Edit a local markdown copy.
4. `bagsy propose <concept> --file …` — server writes + commits.
5. `bagsy release <concept>` after propose (or when abandoning).
