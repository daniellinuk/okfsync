# Agent instructions — bagsy

Canonical instructions for coding agents working in this repo.

## What bagsy is

**Bagsy a concept so your agents don't clobber the brain.**

OSS CLI for agent-swarm shared memory on [git](https://git-scm.com)/[OKF](https://okf.md/spec/) — collision hygiene.

Typical worker flow:

1. `bagsy claim <concept>` — soft-exclusive ownership via a lock + `bagsy/<agent>/…` branch
2. Edit on that branch (never on `main`/`master`)
3. `bagsy propose` — push the bagsy branch and print PR/MR hints (refuses protected branches)
4. `bagsy lint` — OKF frontmatter + lock hygiene

**Workers never push `main`.** Open a small PR from a bagsy branch.

## Layout

| Path | Role |
|------|------|
| `/cli` | Rust implementation of the bagsy CLI |
| `/npm` | Thin JS wrapper + prebuilt binary resolution (bun / npm / pnpm) |
| `/template` | Minimal OKF concepts, `.bagsy` locks, collide-then-recover demo + lint |
| root | Bun monorepo (`package.json` workspaces: `npm`, `template`) |

Bun is the monorepo package manager for scripts and workspaces. The product CLI is the Rust binary; `/npm` only wraps/distributes it.

## Stack lock (do not invent alternatives)

- **CLI:** Rust (`cli/`)
- **Distribute:** npm package with prebuilds / binary resolution (`npm/`)
- **Scripts / monorepo:** Bun
- **No PyPI.** **No Node CLI stub** that reimplements bagsy — keep `/npm` as a thin wrapper only.

## Build / test

```bash
# Rust CLI
cargo test --manifest-path cli/Cargo.toml
cargo build --release --manifest-path cli/Cargo.toml
cargo clippy --manifest-path cli/Cargo.toml --locked --all-targets -- -D warnings
cargo fmt --manifest-path cli/Cargo.toml   # optional; rustfmt.toml present, CI gates on clippy not fmt

# Monorepo (Bun)
bun install
bun run test          # cli + npm + template
bun run demo          # collide-then-recover demo
bun run lint          # template lint via bagsy
```

Keep the collide demo green (`bun run demo` / `bun run test:template`).

## Command surface

| Command | Purpose |
|---------|---------|
| `bagsy get <concept>` | Read an OKF concept |
| `bagsy claim <concept>` | Write lock + open `bagsy/<agent>/…` branch |
| `bagsy release <concept>` | Drop the lock |
| `bagsy propose` | Refuse protected branches; push bagsy branch; print PR/MR hints |
| `bagsy lint` | OKF frontmatter + lock hygiene |

Agent identity: `--agent`, or `BAGSY_AGENT`, or `$USER`. Optional root: `--root` / `BAGSY_ROOT`.

## Locks & soft concept ownership

- Locks live under `.bagsy/locks/` as TOML (`*.lock`), keyed by concept path (slashes → `__`).
- Config defaults (`template/.bagsy/config.toml`): `lock_dir = ".bagsy/locks"`, `default_branch = "main"`.
- Ownership is **soft**: a claim blocks concurrent claims; release (or `--force` on release) clears it. Prefer reclaim after release over force.
- Repo `.gitignore` ignores generated lock noise (`.bagsy/locks/*.lock`) while keeping `template/.bagsy/locks/.gitkeep`.

## Working rules for agents in this repo

- **Do not push `main`/`master`.** Use a feature branch + PR.
- Prefer **small PRs** scoped to one concern.
- Keep **collide-demo / CI green** before considering work done.
- Match existing style; do not churn working layout or invent a second CLI stack.
- For Rust: run `cargo fmt` / prefer clippy-clean changes. Format config: `cli/rustfmt.toml`.
- Symlink note: `CLAUDE.md` → `AGENTS.md` (one source of truth). Edit `AGENTS.md` only.

## Where else to look

- Human contributor short form: [CONTRIBUTING.md](./CONTRIBUTING.md)
- Security contact: [SECURITY.md](./SECURITY.md)
- Product overview & demo walkthrough: [README.md](./README.md)
