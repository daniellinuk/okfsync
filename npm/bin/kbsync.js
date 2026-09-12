#!/usr/bin/env node
"use strict";

/**
 * kbsync npm wrapper — exec the platform binary (or monorepo build fallback).
 * Works with bun/npm/pnpm install.
 */

const { spawnSync } = require("node:child_process");
const { resolveBinary } = require("../lib/resolve");

const bin = resolveBinary();
if (!bin) {
  console.error(
    [
      "kbsync: could not find a kbsync binary for this package version.",
      "Tried KBSYNC_BIN, platform package, versioned vendor/, and monorepo cargo build.",
      "Fix: bun add -g okfsync (or npm i -g okfsync), or from the monorepo:",
      "  cargo build --release --manifest-path cli/Cargo.toml",
      "or set KBSYNC_BIN to the kbsync executable path.",
    ].join("\n")
  );
  process.exit(1);
}

const result = spawnSync(bin, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

if (result.error) {
  console.error(`kbsync: failed to spawn ${bin}: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status === null ? 1 : result.status);
