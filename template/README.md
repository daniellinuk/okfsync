# Okfsync template

Minimal OKF knowledge bundle for the okfsync wiki demo (two agents propose via `kbsync serve`).

```bash
# from monorepo root (after building the CLI)
bun run demo
bun run lint
```

Concepts live in `concepts/`. The demo copies this tree, starts `kbsync serve`, mints two tokens, and proposes.
