#!/usr/bin/env node
"use strict";

/**
 * bagsy npm wrapper — exec the platform binary (or monorepo build fallback).
 * Works with bun/npm/pnpm install.
 */

const { spawnSync } = require("node:child_process");
const { resolveBinary } = require("../lib/resolve");

const bin = resolveBinary();
if (!bin) {
  console.error(
    [
      "bagsy: could not find a bagsy binary.",
      "Tried platform package, BAGSY_BIN, and monorepo release build.",
      "Fix: from the monorepo run `cargo build --release -p bagsy` (or `bun run build:cli`),",
      "or set BAGSY_BIN to the bagsy executable path.",
    ].join("\n")
  );
  process.exit(1);
}

const result = spawnSync(bin, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

if (result.error) {
  console.error(`bagsy: failed to spawn ${bin}: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status === null ? 1 : result.status);
