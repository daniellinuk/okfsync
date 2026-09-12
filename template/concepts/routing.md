---
type: Playbook
title: Routing
description: How work is handed off between agents.
tags:
  - routing
---

# Routing

1. `kbsync get <concept>` — read current knowledge.
2. Edit a local markdown copy.
3. `kbsync propose <concept> --file …` — server writes + commits (create or update only).
4. Do not delete via kbsync. Gardening is out of band.
