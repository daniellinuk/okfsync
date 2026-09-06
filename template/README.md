# Bagsy template

Minimal OKF knowledge bundle for the bagsy wiki demo (two agents propose via `bagsy serve`).

```bash
# from monorepo root (after building the CLI)
bun run demo
bun run lint
```

Concepts live in `concepts/`. The demo copies this tree, starts `bagsy serve`, mints two tokens, and proposes.
