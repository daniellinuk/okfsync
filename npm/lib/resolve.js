"use strict";

const fs = require("node:fs");
const path = require("node:path");
const os = require("node:os");

const PLATFORM_MAP = {
  "darwin-arm64": "bagsy-darwin-arm64",
  "darwin-x64": "bagsy-darwin-x64",
  "linux-x64": "bagsy-linux-x64",
  "linux-arm64": "bagsy-linux-arm64",
  "win32-x64": "bagsy-windows-x64",
};

function platformKey() {
  const plat = process.platform;
  let arch = process.arch;
  if (arch === "ia32") arch = "x64";
  return `${plat}-${arch}`;
}

function candidatePaths() {
  const out = [];
  if (process.env.BAGSY_BIN) {
    out.push(process.env.BAGSY_BIN);
  }

  // Optional platform package (published alongside bagsy on npm)
  const key = platformKey();
  const pkg = PLATFORM_MAP[key];
  if (pkg) {
    try {
      const pkgRoot = path.dirname(require.resolve(`${pkg}/package.json`));
      const name = process.platform === "win32" ? "bagsy.exe" : "bagsy";
      out.push(path.join(pkgRoot, "bin", name));
    } catch {
      // not installed
    }
  }

  // Vendor dir populated by postinstall download
  const vendor = path.join(__dirname, "..", "vendor", process.platform === "win32" ? "bagsy.exe" : "bagsy");
  out.push(vendor);

  // Monorepo local release / debug builds
  const repoRoot = path.resolve(__dirname, "..", "..");
  out.push(path.join(repoRoot, "cli", "target", "release", "bagsy"));
  out.push(path.join(repoRoot, "cli", "target", "debug", "bagsy"));
  if (process.platform === "win32") {
    out.push(path.join(repoRoot, "cli", "target", "release", "bagsy.exe"));
    out.push(path.join(repoRoot, "cli", "target", "debug", "bagsy.exe"));
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
  candidatePaths,
  resolveBinary,
  installHint,
};
