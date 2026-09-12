"use strict";

/**
 * Fail `npm publish` if this package version has no matching vendor binary.
 * 0.1.3 shipped vendor/okfsync-linux-x64-0.1.2; the wrapper ignored it.
 */

const fs = require("node:fs");
const path = require("node:path");
const { packageVersion, vendorArtifactName, vendorBinaryPath } = require("../lib/resolve");

function main() {
  const version = packageVersion();
  const dest = vendorBinaryPath();
  if (!dest) {
    console.error(
      `prepublish: no vendor mapping for this platform (${process.platform}-${process.arch}).\n` +
        "Publish Linux x64 after `node npm/scripts/prepare-binary.js`."
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
        "  node npm/scripts/prepare-binary.js"
    );
    process.exit(1);
  }
  console.log(`prepublish: ${dest}`);
}

main();
