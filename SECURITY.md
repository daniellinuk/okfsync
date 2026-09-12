# Security Policy

## Supported versions

This project is early (0.x). Security fixes land on the default branch (`main`) via PR.

## Reporting a vulnerability

Please report security issues privately to the repository maintainers (Origin/GitHub security advisory or a private message to the owner). Do not open a public issue for undisclosed vulnerabilities.

## Scope notes for agents

okfsync is a CLI plus a local HTTP server around an OKF git working tree. Prefer reporting:

- Token hash storage / token leak in logs
- Path traversal or writes outside `concepts/` on propose
- Any way for the CLI/API to delete knowledge
- Unsafe git operations on propose/push
- Supply-chain problems in the npm binary download path (`npm/lib/install.js`)

Bearer tokens are capabilities. The owner should treat `kbsync token create` output like a password. `.okfsync/tokens.toml` stores hashes only.

Agent working rules: [AGENTS.md](./AGENTS.md).
