# bagsy

**Bagsy a concept so your agents don't clobber the brain.**

OSS CLI for agent-swarm shared memory on [git](https://git-scm.com)/[OKF](https://okf.md/spec/) — collision hygiene.

Workers `bagsy claim` a concept, write on a branch, open a PR/MR, and `bagsy lint`. **Never push `main`.**

## Two repos (do not confuse them)

| Repo | Role | Lives where |
|------|------|-------------|
| **This monorepo** | Tooling: CLI source, npm wrapper, toy template | Public (`cli/`, `npm/`, `template/`) |
| **Your knowledge repo** | Private OKF markdown KB (the real brain) | Private git host; clone + run `bagsy` against it |

`/template` is a **demo seed**, not the product KB. Dogfood and production memory live in a separate private knowledge repo that mirrors the same layout (`concepts/`, `.bagsy/locks/`).

```
/cli       Rust implementation of the bagsy CLI
/npm       Prebuilt-binary wrapper (bun / npm / pnpm) — no PyPI
/template  Minimal OKF concepts, locks, collide-then-recover demo + CI lint
```

## For review agents (read first)

- **Product:** bagsy = claim → branch → PR/MR → lint for multi-writer agent memory on git. Collision hygiene, not a hosted KB.
- **Repo kind A — this monorepo:** review/edit `cli/` (Rust), `npm/` (wrapper), `template/` (demo + CI). SoT for the *tool*.
- **Repo kind B — consumer knowledge repo:** private OKF concepts + locks. SoT for the *data*. Do not treat `template/` as that KB.
- **Hard rule:** never push `main`/`master`. Workers open PRs/MRs from bagsy branches; lead/human merges.
- **Hard rule:** claim before edit. No free-writes on concepts you did not bagsy.
- **Hard rule:** do not hand-delete lock files under `.bagsy/locks/` — use `bagsy release`.
- **Verify:** `bagsy lint` must stay green (OKF frontmatter + lock hygiene). Success = lint pass, locks consistent, no main pushes.
- **Commands that matter:** `get` (read), `claim` (dibs + branch), `release` (drop lock), `propose` (push bagsy branch / PR hints; refuses protected branches), `lint` (CI gate).
- **Agent id:** `--agent`, or `BAGSY_AGENT`, or `$USER`. Use a stable id per worker.
- **Where to review code:** CLI behavior → `cli/`; install/binary resolution → `npm/`; demo layout + collide script → `template/`.
- **OKF minimum bagsy cares about:** concept markdown under `concepts/` with YAML frontmatter `type` required; concept ids must be unique and match lock targets.
- **Out of scope for reviewers of this monorepo:** inventing IAM, rewriting private KB content, or treating marketplace wrappers as required for MVP.

## Who it is for

- **Primary:** agents and agent swarms (lead recruits workers that share one OKF git KB).
- **Secondary:** humans who review PRs/MRs — not wiki browsers.

## Mental model

| Piece | What it is |
|-------|------------|
| `concepts/*.md` | OKF concepts (YAML frontmatter + body). The shared brain. |
| `.bagsy/locks/*.lock` | Claim locks (TOML). Who currently bagsied which concept. |
| `bagsy/<agent>/<concept-slug>` | Worker branch created by `claim` (unless `--no-branch`). |
| PR / MR | How changes land on `main`. Workers never push protected branches. |
| `bagsy lint` | Gate: frontmatter `type`, concept ids, lock → concept consistency. |

**Flow:** `get` → `claim` → edit on bagsy branch → `propose` (PR/MR) → `lint` green → lead merges → `release`.

## CLI surface

| Command | When to use |
|---------|-------------|
| `bagsy get <concept>` | Read a concept (no lock). Inspect before claiming. |
| `bagsy claim <concept>` | Call dibs before writing. Writes a lock and (by default) checks out `bagsy/<agent>/…`. |
| `bagsy release <concept>` | Done writing (or abort). Drops *your* lock so others can claim. |
| `bagsy propose` | Ready to share work: refuse if on `main`/`master`; push your bagsy branch; print PR/MR hints. Use after claim + edits — not instead of claim. |
| `bagsy lint` | Local or CI check that OKF + locks are consistent. Run before asking for merge. |

**Flags / identity**

| Flag / env | When to use |
|------------|-------------|
| `--agent` / `BAGSY_AGENT` | Stable worker id (else `$USER`). Required for clear multi-agent demos and lock ownership. |
| `--no-branch` | Claim/lock without creating/switching branches — useful in the collide demo or when branch management is external. Default claim *does* open a bagsy branch for real work. |

## How an agent should work

### A) Toy template (this monorepo)

1. Build CLI; put it on `PATH`.
2. Work inside `template/` (seed concepts + locks).
3. Run the collide demo (`bun run demo` or hand steps below).
4. Confirm `bagsy lint` stays green.

### B) Real private knowledge repo

1. Clone the **private** OKF KB (not this monorepo's `template/`).
2. Install `bagsy` (global or from this monorepo's release binary).
3. Set `BAGSY_AGENT` to your worker id.
4. `bagsy get <concept>` — read first.
5. `bagsy claim <concept>` — lock + bagsy branch (omit `--no-branch` for real work).
6. Edit only claimed concept files; keep link edges append-only where possible.
7. `bagsy lint` — fix until green.
8. `bagsy propose` — push bagsy branch; open PR/MR. **Never push `main`.**
9. After merge (or abandon): `bagsy release <concept>`.

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

## What NOT to do

- Free-write concepts on `main` / `master`.
- Edit a concept without `bagsy claim`.
- Hand-delete or hand-edit files under `.bagsy/locks/` — use `bagsy release`.
- Push `main`/`master` from a worker agent.
- Treat `/template` as your production knowledge base.
- Skip `bagsy lint` before asking for merge.

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
