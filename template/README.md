# Bagsy template

Minimal OKF knowledge bundle for the bagsy collide-then-recover demo (two agents, one `bagsy serve`).

```bash
# from monorepo root (after building the CLI)
bun run demo
bun run lint
```

Concepts live in `concepts/`. The demo copies this tree, starts `bagsy serve`, mints two tokens, and collides on `brain`.
