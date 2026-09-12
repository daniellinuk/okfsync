#!/usr/bin/env node
"use strict";

/**
 * kbsync npm wrapper — exec the platform binary (or monorepo build fallback).
 * Works with bun/npm/pnpm install.
 */

const { spawnSync } = require("node:child_process");
const { resolveBinary, packageVersion, vendorArtifactName, vendorBinaryPath } = require("../lib/resolve");
const fs = require("node:fs");
const path = require("node:path");

const bin = resolveBinary();
if (!bin) {
  const expected = vendorArtifactName() || vendorBinaryPath();
  const vendorDir = path.join(__dirname, "..", "vendor");
  let found = "(no vendor/)";
  try {
    found = fs.readdirSync(vendorDir).join(", ") || "(empty)";
  } catch {
    // ignore
  }
  console.error(
    [
      `kbsync: could not find a kbsync binary for okfsync ${packageVersion()}.`,
      expected ? `Expected vendor/${expected}` : "No prebuilt mapping for this platform.",
      `vendor/ has: ${found}`,
      "Tried KBSYNC_BIN, platform package, versioned vendor/, and monorepo cargo build.",
      "Fix: bun add -g okfsync (or npm i -g okfsync) after a release that includes the binary,",
      "  or from the monorepo:",
      "  cargo build --release --manifest-path cli/Cargo.toml",
      "  node npm/scripts/prepare-binary.js",
      "or set KBSYNC_BIN to the kbsync executable path.",
    ].join("\n")
  );
  process.exit(1);
}

const result = spawnSync(bin, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
  argv0: "kbsync",
});

if (result.error) {
  console.error(`kbsync: failed to spawn ${bin}: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status === null ? 1 : result.status);
