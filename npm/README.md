# bagsy (npm)

Prebuilt binary wrapper for the **bagsy** CLI (includes `bagsy serve`).

```bash
bun add -g bagsy
# or: npm i -g bagsy
# or: pnpm add -g bagsy
```

The package resolves a native binary via:

1. `BAGSY_BIN`
2. optional platform package (`bagsy-linux-x64`, …)
3. downloaded `vendor/bagsy` (from GitHub Releases on postinstall)
4. monorepo `cli/target/{release,debug}/bagsy` (dev)

Agents need `BAGSY_URL` and `BAGSY_TOKEN` from the KB owner. See the monorepo README.
