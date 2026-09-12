---
type: Reference
title: OKF Conventions
description: Minimal Open Knowledge Format conventions used by this template.
tags:
  - okf
---

# OKF Conventions

Concepts live under `concepts/**/*.md` as markdown with YAML frontmatter.

The only required frontmatter field is `type`.

Optional: `title`, `description`, `tags`, `resource`, `timestamp`.

okfsync will not delete concepts via the CLI. Git on the server is history.

