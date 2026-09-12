"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const npmRoot = path.join(__dirname, "..");
const repoRoot = path.join(npmRoot, "..");
const version = require("../package.json").version;

function cargoVersion(toml) {
  const m = toml.match(/^version = "([^"]+)"/m);
  return m ? m[1] : null;
}

function main() {
  const tomlPath = path.join(repoRoot, "cli", "Cargo.toml");
  const toml = fs.readFileSync(tomlPath, "utf8");
  const rustVersion = cargoVersion(toml);
  if (rustVersion !== version) {
    console.error(
      `version mismatch: npm ${version} vs cli/Cargo.toml ${rustVersion}\n` +
        "Bump both, then re-run this script so kbsync -V matches the package."
    );
    process.exit(1);
  }

  const env = { ...process.env, CARGO_TARGET_DIR: path.join(repoRoot, "cli", "target") };
  execFileSync("cargo", ["build", "--release", "--manifest-path", "cli/Cargo.toml"], {
    cwd: repoRoot,
    stdio: "inherit",
    env,
  });

  const src = path.join(repoRoot, "cli", "target", "release", "kbsync");
  if (!fs.existsSync(src)) {
    console.error(`missing ${src}`);
    process.exit(1);
  }

  const vendorDir = path.join(npmRoot, "vendor");
  fs.mkdirSync(vendorDir, { recursive: true });
  const dest = path.join(vendorDir, `okfsync-linux-x64-${version}`);
  fs.copyFileSync(src, dest);
  fs.chmodSync(dest, 0o755);
  console.log(`wrote ${dest}`);
}

main();
