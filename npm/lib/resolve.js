"use strict";

const fs = require("node:fs");
const path = require("node:path");
const os = require("node:os");
const { PLATFORM_MAP } = require("./platforms");

function platformKey() {
  const plat = process.platform;
  let arch = process.arch;
  if (arch === "ia32") arch = "x64";
  return `${plat}-${arch}`;
}

function packageVersion() {
  return require("../package.json").version;
}

/** vendor/okfsync-linux-x64-<version> — version in the name so upgrades cannot keep a stale kbsync. */
function vendorArtifactName() {
  const pkg = PLATFORM_MAP[platformKey()];
  if (!pkg) return null;
  const version = packageVersion();
  return process.platform === "win32" ? `${pkg}-${version}.exe` : `${pkg}-${version}`;
}

function vendorBinaryPath() {
  const name = vendorArtifactName();
  if (!name) return null;
  return path.join(__dirname, "..", "vendor", name);
}

function candidatePaths() {
  const out = [];
  if (process.env.KBSYNC_BIN) {
    out.push(process.env.KBSYNC_BIN);
  }

  // Optional platform package (published alongside okfsync on npm)
  const key = platformKey();
  const pkg = PLATFORM_MAP[key];
  if (pkg) {
    try {
      const pkgRoot = path.dirname(require.resolve(`${pkg}/package.json`));
      const name = process.platform === "win32" ? "kbsync.exe" : "kbsync";
      out.push(path.join(pkgRoot, "bin", name));
    } catch {
      // not installed
    }
  }

  const vendor = vendorBinaryPath();
  if (vendor) out.push(vendor);

  // Monorepo local release / debug builds
  const repoRoot = path.resolve(__dirname, "..", "..");
  out.push(path.join(repoRoot, "cli", "target", "release", "kbsync"));
  out.push(path.join(repoRoot, "cli", "target", "debug", "kbsync"));
  if (process.platform === "win32") {
    out.push(path.join(repoRoot, "cli", "target", "release", "kbsync.exe"));
    out.push(path.join(repoRoot, "cli", "target", "debug", "kbsync.exe"));
  }

  return out;
}

function resolveBinary() {
  for (const p of candidatePaths()) {
    try {
      if (p && fs.existsSync(p) && fs.statSync(p).isFile()) {
        return p;
      }
    } catch {
      // continue
    }
  }
  return null;
}

function installHint() {
  return {
    platform: platformKey(),
    os: os.platform(),
    arch: os.arch(),
    package: PLATFORM_MAP[platformKey()] || null,
  };
}

module.exports = {
  PLATFORM_MAP,
  platformKey,
  packageVersion,
  vendorArtifactName,
  vendorBinaryPath,
  candidatePaths,
  resolveBinary,
  installHint,
};
