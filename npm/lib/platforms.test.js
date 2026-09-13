"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { describe, it } = require("node:test");
const assert = require("node:assert/strict");
const { PLATFORMS, PLATFORM_MAP, optionalDependencyMap } = require("./platforms");
const { packageVersion } = require("./resolve");

describe("platforms", () => {
  it("covers macOS (Apple Silicon + Intel), Linux, and Windows", () => {
    assert.deepEqual(Object.keys(PLATFORMS).sort(), [
      "darwin-arm64",
      "darwin-x64",
      "linux-arm64",
      "linux-x64",
      "win32-x64",
    ]);
    assert.equal(PLATFORMS["darwin-arm64"].label, "macOS Apple Silicon");
    assert.equal(PLATFORMS["darwin-x64"].label, "macOS Intel");
    assert.equal(PLATFORMS["darwin-arm64"].rustTarget, "aarch64-apple-darwin");
    assert.equal(PLATFORMS["darwin-x64"].rustTarget, "x86_64-apple-darwin");
  });

  it("PLATFORM_MAP matches npm package names", () => {
    for (const [key, spec] of Object.entries(PLATFORMS)) {
      assert.equal(spec.key, key);
      assert.equal(PLATFORM_MAP[key], spec.npmName);
    }
  });

  it("optionalDependencies on the meta-package match this version", () => {
    const pkg = require("../package.json");
    assert.deepEqual(pkg.optionalDependencies, optionalDependencyMap(packageVersion()));
  });

  it("release.yml matrix builds every rust target and asset", () => {
    const yml = fs.readFileSync(
      path.join(__dirname, "..", "..", ".github", "workflows", "release.yml"),
      "utf8"
    );
    for (const spec of Object.values(PLATFORMS)) {
      assert.ok(yml.includes(`target: ${spec.rustTarget}`), spec.rustTarget);
      assert.ok(yml.includes(`asset: ${spec.asset}`), spec.asset);
      assert.ok(yml.includes(`bin: ${spec.binName}`), spec.binName);
    }
    assert.ok(yml.includes("macos-latest"), "Apple runners");
    assert.ok(yml.includes("windows-latest"), "Windows runner");
    assert.ok(yml.includes("ubuntu-24.04-arm"), "Linux ARM runner");
    assert.ok(yml.includes("container: ubuntu:22.04"), "Linux links glibc 2.35");
    assert.ok(yml.includes("x86_64-pc-windows-msvc"), "Windows MSVC");
    assert.ok(!yml.includes("windows-gnu"), "do not ship MinGW");
    assert.ok(yml.includes("assert-max-glibc.js"), "glibc gate");
  });
});
