# Agent instructions — bagsy

Canonical instructions for coding agents working in this repo.

## What bagsy is

**Bagsy a concept so your agents don't clobber the brain.**

OSS CLI: agents call a `bagsy serve` HTTP API with per-agent bearer tokens. The server holds the OKF knowledge base (markdown + git + locks). Agents do not need the git repo.

Typical **owner** flow:

1. `bagsy init --root <data-dir>`
2. `bagsy token create --agent <id>` (repeat; `revoke` / `--rotate` as needed)
3. `bagsy serve --root <data-dir> [--bind 127.0.0.1:7432]`

Typical **worker** flow:

1. `BAGSY_URL` + `BAGSY_TOKEN`
2. `bagsy get` / `bagsy claim` / edit a local file / `bagsy propose --file` / `bagsy release`
3. `bagsy lint`

## Layout

| Path | Role |
|------|------|
| `/cli` | Rust CLI + `bagsy serve` |
| `/npm` | Thin JS wrapper + prebuilt binary resolution |
| `/template` | Minimal OKF concepts + collide-then-recover demo |
| root | Bun monorepo (`package.json` workspaces: `npm`, `template`) |

## Stack lock (do not invent alternatives)

- **CLI + server:** Rust (`cli/`)
- **Distribute:** npm package with prebuilds / binary resolution (`npm/`)
- **Scripts / monorepo:** Bun
- **No PyPI.** **No Node reimplementation** of bagsy.

## Build / test

```bash
cargo test --manifest-path cli/Cargo.toml
cargo build --release --manifest-path cli/Cargo.toml
cargo clippy --manifest-path cli/Cargo.toml --locked --all-targets -- -D warnings

bun install
bun run test
bun run demo
bun run lint
```

Keep the collide demo green (`bun run demo` / `bun run test:template`).

## Command surface

Owner: `init`, `serve`, `token create|list|revoke`.

Agents: `get`, `claim`, `propose --file`, `release`, `lint` with `BAGSY_URL` + `BAGSY_TOKEN`.

Token identity wins over `--agent` whenever the CLI is in server mode.

## Locks & tokens

- Locks: `.bagsy/locks/*.lock` on the **server** data dir. Claim is exclusive; same-agent re-claim is ok.
- Tokens: `.bagsy/tokens.toml` stores hashes only. Owner mints/revokes on the data dir (not over HTTP in v0).
- `serve --push` / `--push-interval` optionally `git push` to `origin`.

## Working rules for agents in this repo

- **Do not push `main`/`master` of this tooling repo.** Use a feature branch + PR.
- Prefer **small PRs** scoped to one concern.
- Keep **collide-demo / CI green**.
- Match existing style; do not invent a second CLI stack.
- Symlink note: `CLAUDE.md` → `AGENTS.md`. Edit `AGENTS.md` only.

## Where else to look

- Human contributor short form: [CONTRIBUTING.md](./CONTRIBUTING.md)
- Security contact: [SECURITY.md](./SECURITY.md)
- Product overview: [README.md](./README.md)
