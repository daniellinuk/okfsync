# Agent instructions — okfsync

Canonical instructions for coding agents working in this repo.

## What okfsync is

**A concept wiki so your agents don't clobber the brain.**

A wiki CLI: agents call `kbsync serve` with per-agent bearer tokens. The server holds OKF markdown; git on that disk is history. Agents do not need the git repo.

The GitHub repo and npm package are **okfsync**. The terminal command is **kbsync**.

The CLI **cannot delete** knowledge. Gardening (dedupe, drop junk) is a separate role and does not use this CLI.

Typical **owner** flow:

1. `kbsync init --root <data-dir>`
2. `kbsync token create --agent <id>`
3. `kbsync serve --root <data-dir> [--bind 127.0.0.1:7432]`

Typical **worker** flow:

1. `KBSYNC_URL` + `KBSYNC_TOKEN`
2. `kbsync list` / `kbsync search <query>` → `kbsync get brain > brain.md` → edit → `kbsync propose brain --file brain.md`
3. `kbsync lint`

`--url` / `--token` exist only on agent commands (`list` `search` `get` `propose` `lint`), not on `init`/`serve`/`token`.

`get --help` / `propose --help` include copy-pasteable examples. `--json` is machine-readable. `propose --file -` reads stdin. There is no `claim` or `release`.

## Layout

| Path | Role |
|------|------|
| `/cli` | Rust CLI + `kbsync serve` |
| `/npm` | Thin JS wrapper + prebuilt binary resolution |
| `/template` | Minimal OKF concepts + wiki demo |
| root | Bun monorepo (`package.json` workspaces: `npm`, `template`) |

## Stack lock (do not invent alternatives)

- **CLI + server:** Rust (`cli/`)
- **Distribute:** npm package with prebuilds / binary resolution (`npm/`)
- **Scripts / monorepo:** Bun
- **No PyPI.** **No Node reimplementation** of okfsync.

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

Agents: `list`, `search`, `get`, `propose --file`, `lint` with `KBSYNC_URL` + `KBSYNC_TOKEN`.

Token identity wins over `--agent` in server mode.

## Safety

- Propose only creates/updates `concepts/**/*.md`.
- No delete command; HTTP DELETE is rejected.
- Tokens: `.okfsync/tokens.toml` stores hashes only.

## Working rules for agents in this repo

- **Do not push `main`/`master` of this tooling repo.** Use a feature branch + PR.
- Prefer **small PRs**.
- Keep **demo / CI green**.
- Symlink note: `CLAUDE.md` → `AGENTS.md`. Edit `AGENTS.md` only.

## Where else to look

- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [SECURITY.md](./SECURITY.md)
- [README.md](./README.md)
