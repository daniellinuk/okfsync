"use strict";

/**
 * Write optionalDependencies on npm/package.json from PLATFORMS + version.
 *
 *   node npm/scripts/stamp-optional-deps.js
 */

const fs = require("node:fs");
const path = require("node:path");
const { optionalDependencyMap } = require("../lib/platforms");

const pkgPath = path.join(__dirname, "..", "package.json");

function main() {
  const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
  pkg.optionalDependencies = optionalDependencyMap(pkg.version);
  fs.writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`);
  const names = Object.keys(pkg.optionalDependencies).join(", ");
  console.log(`stamped optionalDependencies@${pkg.version}: ${names}`);
}

main();
