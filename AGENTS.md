# Agent instructions — bagsy

Canonical instructions for coding agents working in this repo.

## What bagsy is

**Bagsy a concept so your agents don't clobber the brain.**

A wiki CLI: agents call `bagsy serve` with per-agent bearer tokens. The server holds OKF markdown; git on that disk is history. Agents do not need the git repo.

The CLI **cannot delete** knowledge. Gardening (dedupe, drop junk) is a separate role and does not use this CLI.

Typical **owner** flow:

1. `bagsy init --root <data-dir>`
2. `bagsy token create --agent <id>`
3. `bagsy serve --root <data-dir> [--bind 127.0.0.1:7432]`

Typical **worker** flow:

1. `BAGSY_URL` + `BAGSY_TOKEN`
2. `bagsy get` → edit a local file → `bagsy propose --file`
3. `bagsy lint`

There is no `claim` or `release`.

## Layout

| Path | Role |
|------|------|
| `/cli` | Rust CLI + `bagsy serve` |
| `/npm` | Thin JS wrapper + prebuilt binary resolution |
| `/template` | Minimal OKF concepts + wiki demo |
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

Keep the wiki demo green (`bun run demo` / `bun run test:template`).

## Command surface

Owner: `init`, `serve`, `token create|list|revoke`.

Agents: `get`, `propose --file`, `lint` with `BAGSY_URL` + `BAGSY_TOKEN`.

Token identity wins over `--agent` in server mode.

## Safety

- Propose only creates/updates `concepts/**/*.md`.
- No delete command; HTTP DELETE is rejected.
- Tokens: `.bagsy/tokens.toml` stores hashes only.

## Working rules for agents in this repo

- **Do not push `main`/`master` of this tooling repo.** Use a feature branch + PR.
- Prefer **small PRs**.
- Keep **demo / CI green**.
- Symlink note: `CLAUDE.md` → `AGENTS.md`. Edit `AGENTS.md` only.

## Where else to look

- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [SECURITY.md](./SECURITY.md)
- [README.md](./README.md)
