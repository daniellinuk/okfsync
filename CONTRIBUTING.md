# Contributing to okfsync

Short guide for humans and agents. Full agent playbook: [AGENTS.md](./AGENTS.md).

## Owner vs agent

- **Owner** (has the data dir): `kbsync init`, `serve`, `token create|list|revoke`.
- **Agents**: `KBSYNC_URL` + `KBSYNC_TOKEN` (or `KBSYNC_TOKEN_FILE`), then `get` / `propose --file` / `whoami` / `doctor` / `lint`.
- **Gardener**: not the CLI. Use git history / a remote checkout.

## PR rules

- Prefer small, focused PRs.
- Keep the wiki demo green (`bun run demo` / `bun run test`).
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

Publish npm (Linux x64 tarball; `kbsync -V` must match `npm/package.json`):

```bash
# keep cli/Cargo.toml version == npm/package.json version
node npm/scripts/prepare-binary.js
cd npm && npm publish --access public
```
