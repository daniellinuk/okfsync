# okfsync (npm)

Prebuilt binary wrapper for the **okfsync** CLI (`kbsync`) (includes `kbsync serve`).

```bash
bun add -g okfsync
# or: npm i -g okfsync
# or: pnpm add -g okfsync
kbsync --help
```

After install the command is **`kbsync`**, not `okfsync`. `0.1.0` ships a **Linux x64** binary.

Agents do not need the GitHub repo. They need `KBSYNC_URL` and `KBSYNC_TOKEN` from the KB owner:

```bash
export KBSYNC_URL=http://127.0.0.1:7432
export KBSYNC_TOKEN=kbs_…
kbsync list
kbsync get brain
kbsync propose brain --file ./brain.md
```

Owner (one machine with the data dir):

```bash
mkdir -p ./my-kb
kbsync init --root ./my-kb
kbsync token create --agent agent-a --root ./my-kb
kbsync serve --root ./my-kb
```

Other platforms: build from the [monorepo](https://github.com/daniellinuk/okfsync) (needs rustc ≥ 1.88):

```bash
cargo build --release --manifest-path cli/Cargo.toml
export PATH="$PWD/cli/target/release:$PATH"
```

This is not the PyPI package `bagsy` (ROS bags). Unrelated to npm `@bagsy/cli`.

The package resolves a native binary via:

1. `KBSYNC_BIN`
2. optional platform package (`okfsync-linux-x64`, …)
3. `vendor/kbsync` (bundled in this package, or downloaded from GitHub Releases on postinstall)
4. monorepo `cli/target/{release,debug}/kbsync` (dev)

See the monorepo README for the full CLI surface.
