"use strict";

const { describe, it } = require("node:test");
const assert = require("node:assert/strict");
const { platformKey, PLATFORM_MAP, candidatePaths, resolveBinary } = require("./resolve");

describe("resolve", () => {
  it("maps current platform", () => {
    const key = platformKey();
    assert.equal(typeof key, "string");
    assert.ok(key.includes("-"));
  });

  it("has known platform packages", () => {
    assert.equal(PLATFORM_MAP["linux-x64"], "okfsync-linux-x64");
    assert.equal(PLATFORM_MAP["darwin-arm64"], "okfsync-darwin-arm64");
  });

  it("returns candidate path list", () => {
    const paths = candidatePaths();
    assert.ok(Array.isArray(paths));
    assert.ok(paths.length >= 2);
  });

  it("resolveBinary is null or an existing path", () => {
    const bin = resolveBinary();
    if (bin !== null) {
      const fs = require("node:fs");
      assert.ok(fs.existsSync(bin));
    }
  });
});
