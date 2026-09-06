# bagsy

**Bagsy a concept so your agents don't clobber the brain.**

OSS CLI for agent-swarm shared memory on [git](https://git-scm.com)/[OKF](https://okf.md/spec/) — collision hygiene.

Coding agents: start at [AGENTS.md](./AGENTS.md) (also linked as `CLAUDE.md`).

Workers `bagsy claim` a concept, write on a branch, open a PR/MR, and `bagsy lint`. **Never push `main`.**

```
/cli       Rust implementation of the bagsy CLI
/template  Minimal OKF concepts, locks, collide-then-recover demo + CI lint
/npm       Prebuilt-binary wrapper (bun / npm / pnpm) — no PyPI
```

## Install

### From this monorepo (dev)

```bash
# Rust CLI
cargo build --release --manifest-path cli/Cargo.toml
export PATH="$PWD/cli/target/release:$PATH"
bagsy --help

# Or via the npm wrapper (resolves the local release binary)
cd npm && npm install && cd ..
node npm/bin/bagsy.js --help
```

### Via bun / npm / pnpm (prebuilt binary)

```bash
bun add -g bagsy
# npm i -g bagsy
# pnpm add -g bagsy

bagsy --help
```

The npm package ships a small Node wrapper. On install it looks for a platform binary (`BAGSY_BIN`, optional `bagsy-<os>-<arch>` package, GitHub Release asset, or a monorepo `cli/target/*/bagsy` build).

## Commands

| Command | What it does |
|---------|----------------|
| `bagsy get <concept>` | Read an OKF concept |
| `bagsy claim <concept>` | Bagsy the concept — write a lock + open `bagsy/<agent>/…` branch |
| `bagsy release <concept>` | Drop the lock |
| `bagsy propose` | Refuse protected branches; push your bagsy branch; print PR/MR hints |
| `bagsy lint` | OKF frontmatter + lock hygiene |

Agent identity: `--agent`, or `BAGSY_AGENT`, or `$USER`.

## 5-minute two-agent collide demo

Two agents fight over `concepts/brain.md`. The second claim fails; release + reclaim recovers. Lint stays green.

```bash
# 0) build once
cargo build --release --manifest-path cli/Cargo.toml
export PATH="$PWD/cli/target/release:$PATH"

# 1) one-shot demo (Bun)
bun install
bun run demo
```

Or walk it by hand inside `/template`:

```bash
cd template
git init -b main && git add . && git commit -m "seed"

# Agent A bagsies the brain
BAGSY_AGENT=agent-a bagsy claim brain
# → lock written, on branch bagsy/agent-a/concepts-brain

# Agent B collides
BAGSY_AGENT=agent-b bagsy claim brain --no-branch
# → error: already bagsied by agent-a

# Recover
BAGSY_AGENT=agent-a bagsy release brain
BAGSY_AGENT=agent-b bagsy claim brain --no-branch
# → bagsied

bagsy lint
BAGSY_AGENT=agent-b bagsy release brain
```

`bagsy propose` will refuse if you are on `main`/`master` — workers open a PR/MR from their bagsy branch instead.

## Layout

```
cli/                 # Rust crate (bagsy binary)
npm/                 # bagsy npm package (binary wrapper)
template/
  concepts/          # OKF markdown concepts
  .bagsy/locks/      # claim locks (*.lock)
  scripts/           # collide demo + lint
.github/workflows/   # CI: cargo test, template lint, demo
```

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
- Locks are TOML under `.bagsy/locks/`.
- Claims create `bagsy/<agent>/<concept-slug>` branches.
- `propose` never pushes `main`/`master`.
- No framework, no database, no PyPI — just git + files + a small CLI.
