# Contributing to bagsy

Short guide for humans and agents. Full agent playbook: [AGENTS.md](./AGENTS.md).

## Claim flow (when editing OKF concepts)

1. Build or install bagsy so it is on `PATH`.
2. `bagsy claim <concept>` (sets agent via `--agent` / `BAGSY_AGENT` / `$USER`).
3. Work on the created `bagsy/<agent>/…` branch — **never push `main`/`master`**.
4. `bagsy lint`, then `bagsy propose` (or open a PR yourself).
5. `bagsy release <concept>` when done.

## PR rules

- Prefer small, focused PRs.
- Keep the collide demo green (`bun run demo` / `bun run test`).
- Do not invent a Node/Python CLI; the product stack is Rust CLI + npm prebuild wrapper + Bun scripts.

## Run demo / tests

```bash
bun install
bun run test
bun run demo
```

Rust-only:

```bash
cargo test --manifest-path cli/Cargo.toml
cargo build --release --manifest-path cli/Cargo.toml
cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings
```
