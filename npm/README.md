# okfsync (npm)

Prebuilt binary wrapper for the **okfsync** CLI (`kbsync`) (includes `kbsync serve`).

`bun add -g okfsync` / `npm i -g okfsync` / `pnpm add -g okfsync` only work after this package is published **and** GitHub Release assets exist for this version. After install, the command is `kbsync`. Until then, build from the monorepo:

```bash
cargo build --release --manifest-path ../cli/Cargo.toml
export PATH="$PWD/../cli/target/release:$PATH"
```

This is not the PyPI package `bagsy` (ROS bags). Unrelated to npm `@bagsy/cli`.

The package resolves a native binary via:

1. `KBSYNC_BIN`
2. optional platform package (`okfsync-linux-x64`, …)
3. downloaded `vendor/kbsync` (from GitHub Releases on postinstall)
4. monorepo `cli/target/{release,debug}/kbsync` (dev)

Agents need `KBSYNC_URL` and `KBSYNC_TOKEN` from the KB owner. See the monorepo README.
