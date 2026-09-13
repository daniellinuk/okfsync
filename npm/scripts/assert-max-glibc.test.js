"use strict";

const { describe, it } = require("node:test");
const assert = require("node:assert/strict");
const { parseMax } = require("./assert-max-glibc");

describe("assert-max-glibc", () => {
  it("picks the highest GLIBC_* symbol", () => {
    assert.deepEqual(parseMax("GLIBC_2.17\nGLIBC_2.35\nGLIBC_2.28"), [2, 35]);
    assert.deepEqual(parseMax("GLIBC_2.39"), [2, 39]);
    assert.deepEqual(parseMax("no symbols"), [0, 0]);
  });
});
