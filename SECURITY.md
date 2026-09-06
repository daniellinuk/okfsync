# Security Policy

## Supported versions

This project is early (0.x). Security fixes land on the default branch (`main`) via PR.

## Reporting a vulnerability

Please report security issues privately to the repository maintainers (Origin/GitHub security advisory or a private message to the owner). Do not open a public issue for undisclosed vulnerabilities.

## Scope notes for agents

bagsy is a local CLI around git + OKF files and lock files under `.bagsy/`. It is not a networked service. Prefer reporting issues that could cause unsafe git operations, lock bypasses that mislead multi-agent workflows, or supply-chain problems in the npm binary download path (`npm/lib/install.js`).

Agent working rules: [AGENTS.md](./AGENTS.md).
