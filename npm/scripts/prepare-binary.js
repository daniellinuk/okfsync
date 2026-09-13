"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const { platformKey, packageVersion, vendorBinaryPath } = require("../lib/resolve");

const npmRoot = path.join(__dirname, "..");
const repoRoot = path.join(npmRoot, "..");

function cargoVersion(toml) {
  const m = toml.match(/^version = "([^"]+)"/m);
  return m ? m[1] : null;
}

function cargoBinName() {
  return process.platform === "win32" ? "kbsync.exe" : "kbsync";
}

function main() {
  const version = packageVersion();
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

  const dest = vendorBinaryPath();
  if (!dest) {
    console.error(`no vendor mapping for ${platformKey()}`);
    process.exit(1);
  }

  const env = { ...process.env, CARGO_TARGET_DIR: path.join(repoRoot, "cli", "target") };
  execFileSync("cargo", ["build", "--release", "--manifest-path", "cli/Cargo.toml"], {
    cwd: repoRoot,
    stdio: "inherit",
    env,
  });

  const src = path.join(repoRoot, "cli", "target", "release", cargoBinName());
  if (!fs.existsSync(src)) {
    console.error(`missing ${src}`);
    process.exit(1);
  }

  const vendorDir = path.join(npmRoot, "vendor");
  fs.mkdirSync(vendorDir, { recursive: true });
  for (const name of fs.readdirSync(vendorDir)) {
    const stale = path.join(vendorDir, name);
    if (stale !== dest && fs.statSync(stale).isFile()) {
      fs.unlinkSync(stale);
      console.log(`removed stale ${stale}`);
    }
  }
  fs.copyFileSync(src, dest);
  if (process.platform !== "win32") {
    fs.chmodSync(dest, 0o755);
  }
  console.log(`wrote ${dest} (${platformKey()})`);
}

main();
