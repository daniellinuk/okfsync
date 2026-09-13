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

Do **not** commit `kbsync` binaries or cross-compile with MinGW/zig and `npm publish` those files. Linux must link **glibc ≤ 2.35** (Ubuntu 22.04). Windows must be **MSVC**. macOS must be a real Apple runner.

Tag a version that matches `cli/Cargo.toml` and `npm/package.json`. Push the tag to **GitHub** so `.github/workflows/release.yml` runs (this workspace’s `origin` remote is Cursor; GitHub is `github`).

```bash
# bump version in cli/Cargo.toml, npm/package.json, and the root package.json
node npm/scripts/stamp-optional-deps.js
git tag v0.1.5
git push github v0.1.5
# if origin is already github.com/daniellinuk/okfsync: git push origin v0.1.5
```

Needs repo secret `NPM_TOKEN`. The workflow:

1. Linux x64 + ARM64 inside `ubuntu:22.04` (glibc 2.35), then fails the job if `objdump` shows a newer GLIBC.
2. macOS Apple Silicon + Intel on `macos-latest`.
3. Windows x64 on `windows-latest` (`x86_64-pc-windows-msvc`).
4. Attaches all five to the GitHub Release.
5. Publishes `okfsync-darwin-arm64` … then the `okfsync` meta-package (`OKFSYNC_RELEASE_PACK=1`).

Pull requests that touch `cli/`, `npm/`, or this workflow compile the five binaries as artifacts; they do not publish.

Local Linux-only experiment: `node npm/scripts/prepare-binary.js` (that host’s glibc, not for npm). `prepublishOnly` refuses a mismatched vendor binary unless `OKFSYNC_RELEASE_PACK=1`.
