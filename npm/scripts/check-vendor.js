"use strict";

/**
 * Fail `npm publish` if optionalDependencies are stale, or (unless
 * OKFSYNC_RELEASE_PACK=1) if this machine has no matching vendor binary.
 *
 * Release CI publishes platform packages first, then the thin meta-package.
 * Local `cd npm && npm publish` still requires prepare-binary for this OS.
 */

const fs = require("node:fs");
const path = require("node:path");
const { optionalDependencyMap } = require("../lib/platforms");
const { packageVersion, vendorArtifactName, vendorBinaryPath } = require("../lib/resolve");

function assertOptionalDependencies(version) {
  const pkg = require("../package.json");
  const expected = optionalDependencyMap(version);
  const actual = pkg.optionalDependencies || {};
  const missing = [];
  for (const [name, ver] of Object.entries(expected)) {
    if (actual[name] !== ver) {
      missing.push(`${name}@${ver} (got ${actual[name] || "missing"})`);
    }
  }
  const extra = Object.keys(actual).filter((name) => !expected[name]);
  if (missing.length || extra.length) {
    console.error(
      `prepublish: optionalDependencies must list every platform at ${version}.\n` +
        (missing.length ? `  mismatch: ${missing.join(", ")}\n` : "") +
        (extra.length ? `  unexpected: ${extra.join(", ")}\n` : "") +
        "  node npm/scripts/stamp-optional-deps.js"
    );
    process.exit(1);
  }
}

function main() {
  const version = packageVersion();
  assertOptionalDependencies(version);

  if (process.env.OKFSYNC_RELEASE_PACK === "1") {
    console.log(`prepublish: ${version} optionalDependencies ok (OKFSYNC_RELEASE_PACK=1)`);
    return;
  }

  const dest = vendorBinaryPath();
  if (!dest) {
    console.error(
      `prepublish: no vendor mapping for this platform (${process.platform}-${process.arch}).\n` +
        "Use the release workflow, or set OKFSYNC_RELEASE_PACK=1 after publishing platform packages."
    );
    process.exit(1);
  }
  if (!fs.existsSync(dest) || !fs.statSync(dest).isFile()) {
    const vendorDir = path.join(__dirname, "..", "vendor");
    let found = "(missing vendor/)";
    try {
      found = fs.readdirSync(vendorDir).join(", ") || "(empty)";
    } catch {
      // ignore
    }
    console.error(
      `prepublish: missing ${vendorArtifactName()} for okfsync ${version}.\n` +
        `vendor/ has: ${found}\n` +
        "  node npm/scripts/prepare-binary.js\n" +
        "or tag a release and let .github/workflows/release.yml publish."
    );
    process.exit(1);
  }
  console.log(`prepublish: ${dest}`);
}

if (require.main === module) {
  main();
}

module.exports = { main, assertOptionalDependencies };
