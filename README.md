# bagsy

**Bagsy a concept so your agents don't clobber the brain.**

OSS CLI for agent-swarm shared memory on [OKF](https://okf.md/spec/). One server holds the knowledge base. Agents never need the git repo — they call `bagsy` with a URL and a per-agent token.

## Two roles (do not confuse them)

| Who | Runs | Needs |
|-----|------|--------|
| **KB owner** | `bagsy init`, `bagsy serve`, `bagsy token` | The data directory (OKF files + `.bagsy/`) |
| **Agent** | `get` `claim` `propose` `release` `lint` | `BAGSY_URL` + `BAGSY_TOKEN` |

`/template` is a **demo seed**, not a production KB.

```
/cli       Rust CLI + server (`bagsy serve`)
/npm       Prebuilt-binary wrapper (bun / npm / pnpm)
/template  Minimal OKF concepts + collide-then-recover demo
```

## For review agents (read first)

- **Canonical agent playbook:** [AGENTS.md](./AGENTS.md) (also linked as `CLAUDE.md`).
- **Product:** one `bagsy serve` process is the source of truth. File locks live on that host. Agents authenticate with per-agent bearer tokens the owner creates and revokes.
- **Hard rule:** agents do not clone or push the KB. They talk HTTP.
- **Hard rule:** claim before `propose`. Identity comes from the token, not `--agent`, when `BAGSY_URL` is set.
- **Commands that matter:** `init` / `serve` / `token` (owner); `get` `claim` `propose --file` `release` `lint` (agents).

## Mental model

| Piece | What it is |
|-------|------------|
| `concepts/*.md` | OKF concepts (YAML frontmatter + body) on the **server disk** |
| `.bagsy/locks/*.lock` | Who currently bagsied which concept (server-side) |
| `.bagsy/tokens.toml` | Hashed per-agent tokens (owner-only; never the secret) |
| `bagsy serve --bind …` | HTTP API (`/health`, `/v1/…`) |
| `BAGSY_URL` + `BAGSY_TOKEN` | How an agent attaches |

**Flow:** owner `init` + `token create` + `serve` → agent `get` → `claim` → edit a local copy → `propose --file` → `release`.

## CLI surface

**Owner (on the machine with the data dir)**

| Command | When to use |
|---------|-------------|
| `bagsy init` | Create `concepts/`, `.bagsy/`, git if needed |
| `bagsy serve` | Listen (default `127.0.0.1:7432`) |
| `bagsy serve --bind 0.0.0.0:7432` | Reachable from other machines (put a proxy in front if you want OIDC/Tailscale) |
| `bagsy token create --agent <id>` | Mint one token for one agent (printed once) |
| `bagsy token list` | See ids / agents / active vs revoked |
| `bagsy token revoke --agent <id>` | Cut off that agent |
| `bagsy token revoke --id <id>` | Revoke one token |
| `bagsy token create --agent <id> --rotate` | Replace that agent's active token |

**Agents**

| Command | When to use |
|---------|-------------|
| `bagsy get <concept>` | Read (no lock) |
| `bagsy claim <concept>` | Dibs. Fails if another agent holds it |
| `bagsy propose <concept> --file <md>` | Write markdown; server commits on its clone |
| `bagsy release <concept>` | Drop *your* lock (`--force` to steal) |
| `bagsy lint` | OKF frontmatter + lock hygiene |

`--url` / `BAGSY_URL` and `--token` / `BAGSY_TOKEN` select the server. Without them, `get`/`lint`/`claim`/`release`/`propose` operate on `--root` / `BAGSY_ROOT` (owner local mode).

`--agent` / `BAGSY_AGENT` is only for local mode. In server mode the token **is** the agent.

`serve --push` pushes `origin` after each propose. `--push-interval <secs>` also pushes on a timer. Hosting (localhost, Tailscale, Docker) is the operator's choice — bagsy only needs a reachable bind address and a writable data dir.

## How an agent should work

```bash
export BAGSY_URL=http://127.0.0.1:7432
export BAGSY_TOKEN=bgy_…          # from the owner

bagsy get brain
bagsy claim brain
# edit a local markdown file
bagsy propose brain --file ./brain.md
bagsy lint
bagsy release brain
```

## Install

### From this monorepo (dev)

```bash
cargo build --release --manifest-path cli/Cargo.toml
export PATH="$PWD/cli/target/release:$PATH"
bagsy --help
```

### Via bun / npm / pnpm (prebuilt binary)

```bash
bun add -g bagsy
bagsy --help
```

The npm package is a wrapper around the native binary (`BAGSY_BIN`, GitHub Release, or a local `cli/target` build).

## Owner: local server (5 minutes)

```bash
bagsy init --root ./my-kb
bagsy token create --agent agent-a --root ./my-kb
bagsy token create --agent agent-b --root ./my-kb
bagsy serve --root ./my-kb
# other terminals: BAGSY_URL=http://127.0.0.1:7432 BAGSY_TOKEN=… bagsy claim brain
```

## 5-minute two-agent collide demo

Two agents fight over `concepts/brain.md`. The second claim fails; release + reclaim recovers.

```bash
cargo build --release --manifest-path cli/Cargo.toml
export PATH="$PWD/cli/target/release:$PATH"
bun install
bun run demo
```

## What NOT to do

- Give agents git access to the KB instead of a token.
- Share one token across agents if you want revoke-per-agent.
- Bind `0.0.0.0` with no tokens.
- Skip `bagsy claim` before `propose`.
- Treat `/template` as your production knowledge base.

## Develop

```bash
bun install
bun run test:cli        # cargo test
bun run test:template   # collide demo tests
bun run lint            # template lint via bagsy
bun run test            # all
```

## Design notes (MVP)

- OKF concepts are plain markdown + YAML frontmatter (`type` required).
- One process (`bagsy serve`) owns the working tree, locks, and git commits.
- Tokens are random secrets; only SHA-256 hashes are stored.
- No framework, no database — git + files + a small HTTP API.
