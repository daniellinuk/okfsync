# bagsy (npm)

Prebuilt binary wrapper for the **bagsy** CLI (includes `bagsy serve`).

`bun add -g bagsy` / `npm i -g bagsy` / `pnpm add -g bagsy` only work after this package is published **and** GitHub Release assets exist for this version. Until then, build from the monorepo:

```bash
cargo build --release --manifest-path ../cli/Cargo.toml
export PATH="$PWD/../cli/target/release:$PATH"
```

This is not the PyPI package `bagsy` (ROS bags). Unrelated to npm `@bagsy/cli`.

The package resolves a native binary via:

1. `BAGSY_BIN`
2. optional platform package (`bagsy-linux-x64`, …)
3. downloaded `vendor/bagsy` (from GitHub Releases on postinstall)
4. monorepo `cli/target/{release,debug}/bagsy` (dev)

Agents need `BAGSY_URL` and `BAGSY_TOKEN` from the KB owner. See the monorepo README.
