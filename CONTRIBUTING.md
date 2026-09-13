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

## Release (all platforms)

Tag a version that matches `cli/Cargo.toml` and `npm/package.json`. CI (`.github/workflows/release.yml`) builds five binaries — **macOS Apple Silicon**, macOS Intel, Linux x64, Linux ARM64, Windows x64 — attaches them to the GitHub Release, then publishes `okfsync-<platform>` packages and the `okfsync` meta-package.

```bash
# bump version in cli/Cargo.toml, npm/package.json, and the root package.json
node npm/scripts/stamp-optional-deps.js   # optionalDependencies @ that version
git tag v0.1.5
git push origin v0.1.5
```

Needs repo secrets: `NPM_TOKEN`. Pull requests and `workflow_dispatch` compile all five binaries and upload them as Actions artifacts; they do not publish. Tag `v*` to attach them to a GitHub Release and publish npm.

Local Linux-only publish (legacy): `node npm/scripts/prepare-binary.js` then `cd npm && npm publish --access public`. `prepublishOnly` still refuses a mismatched vendor binary unless `OKFSYNC_RELEASE_PACK=1`.
