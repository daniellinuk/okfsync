"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { describe, it } = require("node:test");
const assert = require("node:assert/strict");
const {
  platformKey,
  PLATFORM_MAP,
  candidatePaths,
  resolveBinary,
  packageVersion,
  vendorArtifactName,
  vendorBinaryPath,
} = require("./resolve");

describe("resolve", () => {
  it("maps current platform", () => {
    const key = platformKey();
    assert.equal(typeof key, "string");
    assert.ok(key.includes("-"));
  });

  it("has known platform packages including Apple", () => {
    assert.equal(PLATFORM_MAP["linux-x64"], "okfsync-linux-x64");
    assert.equal(PLATFORM_MAP["linux-arm64"], "okfsync-linux-arm64");
    assert.equal(PLATFORM_MAP["darwin-arm64"], "okfsync-darwin-arm64");
    assert.equal(PLATFORM_MAP["darwin-x64"], "okfsync-darwin-x64");
    assert.equal(PLATFORM_MAP["win32-x64"], "okfsync-windows-x64");
  });

  it("vendor artifact includes package version", () => {
    const name = vendorArtifactName();
    if (name) {
      assert.ok(name.includes(packageVersion()));
      assert.notEqual(name, "kbsync");
      assert.notEqual(name, "kbsync.exe");
    }
  });

  it("does not treat unversioned vendor/kbsync as this release", () => {
    const stale = path.join(__dirname, "..", "vendor", "kbsync");
    const paths = candidatePaths();
    assert.ok(!paths.includes(stale));
    const dest = vendorBinaryPath();
    if (dest) {
      assert.ok(paths.includes(dest));
    }
  });

  it("returns candidate path list", () => {
    const paths = candidatePaths();
    assert.ok(Array.isArray(paths));
    assert.ok(paths.length >= 2);
  });

  it("resolveBinary is null or an existing path", () => {
    const bin = resolveBinary();
    if (bin !== null) {
      assert.ok(fs.existsSync(bin));
    }
  });

  it("npm version matches cli/Cargo.toml", () => {
    const toml = fs.readFileSync(path.join(__dirname, "..", "..", "cli", "Cargo.toml"), "utf8");
    const m = toml.match(/^version = "([^"]+)"/m);
    assert.equal(m && m[1], packageVersion());
  });

  it("does not treat a previous versioned vendor artifact as this release", () => {
    const dest = vendorBinaryPath();
    if (!dest) return;
    const stale = dest.replace(packageVersion(), "0.0.0-stale");
    assert.notEqual(dest, stale);
    assert.ok(!candidatePaths().includes(stale));
  });
});
