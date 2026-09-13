# okfsync

A concept wiki so your agents don't clobber the brain.

The package is **okfsync**. The command is **`kbsync`**. One owner runs the server; agents call it over HTTP with a token. Agents do not need this git repo.

`okfsync` is the installer. A tagged release publishes optional platform packages and GitHub Release assets for **macOS** (Apple Silicon `darwin-arm64`, Intel `darwin-x64`), **Linux** (x64 and arm64), and **Windows** (x64). npm/bun installs only the package that matches this machine.

`0.1.4` still ships a **Linux x64** `kbsync` in the tarball (`vendor/okfsync-linux-x64-<version>`), so that install works even when the package manager skips `postinstall`. Other platforms on 0.1.4: [build from source](https://github.com/daniellinuk/okfsync).

## Install

```bash
bun add -g okfsync
# or: npm i -g okfsync
# or: pnpm add -g okfsync
kbsync --help
```

## Owner

`--root` must already exist.

```bash
mkdir -p ./my-kb
kbsync init --root ./my-kb
kbsync token create --agent grok --root ./my-kb
kbsync serve --root ./my-kb
```

The token is printed once. Give each agent its own env or a chmod 600 token file. Keep `serve` running (default `127.0.0.1:7432`).

## Agents

```bash
export KBSYNC_URL=http://127.0.0.1:7432
export KBSYNC_TOKEN=kbs_…
# or: export KBSYNC_TOKEN_FILE=/path/to/grok.token

kbsync whoami
kbsync doctor
kbsync list
kbsync search routing
kbsync search shared brain
kbsync get brain
kbsync get ops/foo
kbsync propose brain --file ./brain.md
kbsync propose ops/foo --file ./foo.md
kbsync lint
```

`propose` requires `--file <path>` or `--file -`. `kbsync` cannot delete pages. Full CLI surface: [github.com/daniellinuk/okfsync](https://github.com/daniellinuk/okfsync).
