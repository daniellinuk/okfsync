"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { describe, it } = require("node:test");
const assert = require("node:assert/strict");
const { PLATFORMS } = require("../lib/platforms");
const { packageVersion } = require("../lib/resolve");
const { main } = require("./pack-platforms");

function writeFixtures(dir) {
  fs.mkdirSync(dir, { recursive: true });
  for (const spec of Object.values(PLATFORMS)) {
    fs.writeFileSync(path.join(dir, spec.asset), `fake-${spec.asset}\n`);
  }
}

describe("pack-platforms", () => {
  it("packs every platform package with bin/kbsync", () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "okfsync-pack-"));
    const binaries = path.join(tmp, "binaries");
    const out = path.join(tmp, "out");
    writeFixtures(binaries);

    const packed = main(["node", "pack-platforms.js", "--binaries", binaries, "--out", out]);
    assert.equal(packed.length, Object.keys(PLATFORMS).length);

    for (const spec of Object.values(PLATFORMS)) {
      const dir = path.join(out, spec.npmName);
      const pkg = JSON.parse(fs.readFileSync(path.join(dir, "package.json"), "utf8"));
      assert.equal(pkg.name, spec.npmName);
      assert.equal(pkg.version, packageVersion());
      assert.deepEqual(pkg.os, spec.os);
      assert.deepEqual(pkg.cpu, spec.cpu);
      assert.equal(pkg.bin, undefined);
      assert.ok(fs.existsSync(path.join(dir, "bin", spec.binName)));
      assert.ok(fs.existsSync(path.join(dir, "README.md")));
    }
  });

  it("fails when a platform binary is missing", () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "okfsync-pack-"));
    const binaries = path.join(tmp, "binaries");
    fs.mkdirSync(binaries, { recursive: true });
    fs.writeFileSync(path.join(binaries, "okfsync-linux-x64"), "only-linux\n");
    assert.throws(
      () => main(["node", "pack-platforms.js", "--binaries", binaries, "--out", path.join(tmp, "out")]),
      /missing binary/
    );
  });
});
