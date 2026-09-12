# bagsy

**Bagsy a concept so your agents don't clobber the brain.**

A small **wiki** for multi-agent knowledge. One `bagsy serve` process holds an [OKF](https://okf.md/spec/) tree. Git on that machine is history (optional remote mirror). Agents never need the git repo.

The CLI is for **retrieve and propose**. It cannot delete. Gardening (merge dupes, drop junk) is a separate role, out of band.

## Roles (do not confuse them)

| Who | Runs | Needs |
|-----|------|--------|
| **KB owner** | `bagsy init`, `bagsy serve`, `bagsy token` | The data directory |
| **Agent** | `list` `search` `get` `propose` `lint` | `BAGSY_URL` + `BAGSY_TOKEN` |
| **Gardener** | not the CLI | Git history / remote checkout — can delete or rewrite there |

`/template` is a **demo seed**, not a production KB.

```
/cli       Rust CLI + server (`bagsy serve`)
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
| `.bagsy/tokens.toml` | Hashed per-agent tokens (owner-only) |
| `bagsy serve --bind …` | HTTP API (`/health`, `/v1/concepts`, `/v1/pages`, `/v1/proposals`, `/v1/lint`) |

**Flow:** owner `init` + `token create` + `serve` → agent `get` → edit a local copy → `propose --file`.

## CLI surface

**Owner (on the machine with the data dir)**

| Command | When to use |
|---------|-------------|
| `bagsy init` | Create `concepts/`, `.bagsy/`, git if needed |
| `bagsy serve` | Listen (default `127.0.0.1:7432`) |
| `bagsy serve --bind 0.0.0.0:7432` | Reachable from other machines |
| `bagsy token create --agent <id>` | Mint one token (printed once) |
| `bagsy token list` / `revoke` / `create --rotate` | Who may call the API |

**Agents**

| Command | When to use |
|---------|-------------|
| `bagsy list` | Paths + titles (no bodies) |
| `bagsy search <query>` | Filter list by path/title/tags/body |
| `bagsy get <concept>` | Read raw markdown |
| `bagsy propose <concept> --file <md>` | Create or update one page (never deletes) |
| `bagsy lint` | OKF frontmatter |

`--url` / `BAGSY_URL` and `--token` / `BAGSY_TOKEN` select the server on agent commands only. Without them, `list`/`search`/`get`/`lint`/`propose` operate on `--root` (owner local mode).

`serve --push` / `--push-interval` optionally `git push` to `origin`. Hosting is the operator's choice.

## How an agent should work

```bash
export BAGSY_URL=http://127.0.0.1:7432
export BAGSY_TOKEN=bgy_…          # from the owner

bagsy list
bagsy search routing
bagsy get brain
# stdout is the raw markdown (round-trips into propose)
bagsy propose brain --file ./brain.md
bagsy lint
```

## Install

### From this monorepo (works today)

Needs a recent stable Rust. The current lockfile requires **rustc ≥ 1.88** (`rustup update stable`).

```bash
cargo build --release --manifest-path cli/Cargo.toml
export PATH="$PWD/cli/target/release:$PATH"
bagsy --help
```

### Via bun / npm / pnpm

Not published yet. `bun add -g bagsy` / `npm i -g bagsy` will not work until the `bagsy` package is on npmjs **and** GitHub Release binaries exist for this version. Until then, use the source build above.

This is not the PyPI package `bagsy` (a ROS bag CLI). Do not `pip install bagsy`.

## Owner: local server

`--root` must already exist:

```bash
mkdir -p ./my-kb
bagsy init --root ./my-kb
bagsy token create --agent agent-a --root ./my-kb
bagsy serve --root ./my-kb
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
- Delete knowledge through bagsy — use git/history as gardener.
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
