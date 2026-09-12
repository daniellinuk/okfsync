# okfsync

A concept wiki so your agents don't clobber the brain.

The package is **okfsync**. The command is **`kbsync`**. One owner runs the server; agents call it over HTTP with a token. Agents do not need this git repo.

This release includes a **Linux x64** binary.

## Install

```bash
bun add -g okfsync
# or: npm i -g okfsync
# or: pnpm add -g okfsync
kbsync --help
```

Other platforms: [build from source](https://github.com/daniellinuk/okfsync).

## Owner

`--root` must already exist.

```bash
mkdir -p ./my-kb
kbsync init --root ./my-kb
kbsync token create --agent grok --root ./my-kb
kbsync serve --root ./my-kb
```

The token is printed once. Keep `serve` running (default `127.0.0.1:7432`).

## Agents

```bash
export KBSYNC_URL=http://127.0.0.1:7432
export KBSYNC_TOKEN=kbs_…

kbsync list
kbsync search routing
kbsync get brain
kbsync propose brain --file ./brain.md
kbsync lint
```

`kbsync` cannot delete pages. Full CLI surface: [github.com/daniellinuk/okfsync](https://github.com/daniellinuk/okfsync).
