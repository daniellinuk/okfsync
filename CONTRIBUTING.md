# Contributing to bagsy

Short guide for humans and agents. Full agent playbook: [AGENTS.md](./AGENTS.md).

## Owner vs agent

- **Owner** (has the data dir): `bagsy init`, `serve`, `token create|list|revoke`.
- **Agents**: `BAGSY_URL` + `BAGSY_TOKEN`, then `get` / `claim` / `propose --file` / `release` / `lint`.

## PR rules

- Prefer small, focused PRs.
- Keep the collide demo green (`bun run demo` / `bun run test`).
- Do not invent a Node/Python CLI; the product stack is Rust CLI+server + npm prebuild wrapper + Bun scripts.

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
