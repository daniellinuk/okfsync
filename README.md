# okfsync

**A concept wiki so your agents don't clobber the brain.**

A small **wiki** for multi-agent knowledge. One `kbsync serve` process holds an [OKF](https://okf.md/spec/) tree. Git on that machine is history (optional remote mirror). Agents never need the git repo.

The GitHub repo and npm package are **okfsync**. The terminal command is **kbsync**.

The CLI is for **retrieve and propose**. It cannot delete. Gardening (merge dupes, drop junk) is a separate role, out of band.

## Roles (do not confuse them)

| Who | Runs | Needs |
|-----|------|--------|
| **KB owner** | `kbsync init`, `kbsync serve`, `kbsync token` | The data directory |
| **Agent** | `list` `search` `get` `propose` `lint` | `KBSYNC_URL` + `KBSYNC_TOKEN` |
| **Gardener** | not the CLI | Git history / remote checkout — can delete or rewrite there |

`/template` is a **demo seed**, not a production KB.

```
/cli       Rust CLI + server (`kbsync serve`)
/npm       Prebuilt-binary wrapper (bun / npm / pnpm)
/template  Minimal OKF concepts + wiki demo
```

## For review agents (read first)

- **Canonical agent playbook:** [AGENTS.md](./AGENTS.md) (also linked as `CLAUDE.md`).
- **Product:** wiki, not a PR factory. `propose` creates or overwrites one `concepts/**/*.md` file and commits. Last write wins; git still has the previous commit.
- **Hard rule:** the CLI cannot delete knowledge (no `delete` command; HTTP DELETE is rejected).
- **Hard rule:** agents do not clone or push the KB. They talk HTTP.
- **Commands:** `init` / `serve` / `token` (owner); `list` `search` `get` `propose --file` `lint` (agents).

## Mental model

| Piece | What it is |
|-------|------------|
| `concepts/*.md` | OKF concepts on the **server disk** |
| git in the data dir | History; optional `serve --push` to a remote |
| `.okfsync/tokens.toml` | Hashed per-agent tokens (owner-only) |
| `kbsync serve --bind …` | HTTP API (`/health`, `/v1/concepts`, `/v1/pages`, `/v1/proposals`, `/v1/lint`) |

**Flow:** owner `init` + `token create` + `serve` → agent `get` → edit a local copy → `propose --file`.

## CLI surface

**Owner (on the machine with the data dir)**

| Command | When to use |
|---------|-------------|
| `kbsync init` | Create `concepts/`, `.okfsync/`, git if needed |
| `kbsync serve` | Listen (default `127.0.0.1:7432`) |
| `kbsync serve --bind 0.0.0.0:7432` | Reachable from other machines |
| `kbsync token create --agent <id>` | Mint one token (printed once) |
| `kbsync token list` / `revoke` / `create --rotate` | Who may call the API |

**Agents**

| Command | When to use |
|---------|-------------|
| `kbsync list` | Paths + titles (no bodies) |
| `kbsync search <query>` | Filter list by path/title/tags/body |
| `kbsync get <concept>` | Read raw markdown |
| `kbsync propose <concept> --file <md>` | Create or update one page (never deletes) |
| `kbsync lint` | OKF frontmatter |

`--url` / `KBSYNC_URL` and `--token` / `KBSYNC_TOKEN` select the server on agent commands only. Without them, `list`/`search`/`get`/`lint`/`propose` operate on `--root` (owner local mode).

`serve --push` / `--push-interval` optionally `git push` to `origin`. Hosting is the operator's choice.

## How an agent should work

```bash
export KBSYNC_URL=http://127.0.0.1:7432
export KBSYNC_TOKEN=kbs_…          # from the owner

kbsync list
kbsync search routing
kbsync get brain
# stdout is the raw markdown (round-trips into propose)
kbsync propose brain --file ./brain.md
kbsync lint
```

## Install

### From this monorepo (works today)

Needs a recent stable Rust. The current lockfile requires **rustc ≥ 1.88** (`rustup update stable`).

```bash
cargo build --release --manifest-path cli/Cargo.toml
export PATH="$PWD/cli/target/release:$PATH"
kbsync --help
```

### Via bun / npm / pnpm

Not published yet. `bun add -g okfsync` / `npm i -g okfsync` will not work until the `okfsync` package is on npmjs **and** GitHub Release binaries exist for this version. After install, the command is `kbsync`. Until then, use the source build above.

This is not the PyPI package `bagsy` (a ROS bag CLI). Do not `pip install bagsy`.

## Owner: local server

`--root` must already exist:

```bash
mkdir -p ./my-kb
kbsync init --root ./my-kb
kbsync token create --agent agent-a --root ./my-kb
kbsync serve --root ./my-kb
```

## Wiki demo

Two agents propose the same page. Latest body wins; git log still has both commits. `claim` is not a command.

```bash
cargo build --release --manifest-path cli/Cargo.toml
export PATH="$PWD/cli/target/release:$PATH"
bun install
bun run demo
```

## What NOT to do

- Give agents git write to the KB instead of a token (unless they are the gardener).
- Delete knowledge through kbsync — use git/history as gardener.
- Bind `0.0.0.0` with no tokens.
- Treat `/template` as your production knowledge base.

## Develop

```bash
bun install
bun run test:cli
bun run test:template
bun run lint
bun run test
```

## Design notes (MVP)

- OKF concepts are markdown + YAML frontmatter (`type` required).
- One process owns the working tree and git commits.
- Tokens are random secrets; only SHA-256 hashes are stored.
- No claim/release. Noise and contradictions are a gardener problem.
- Propose cannot write outside `concepts/` and cannot delete.
