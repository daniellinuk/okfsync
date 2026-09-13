"use strict";

/**
 * Fail if a Linux ELF needs a newer glibc than Ubuntu 22.04 (2.35).
 *
 *   node npm/scripts/assert-max-glibc.js ./kbsync [2.35]
 */

const { execFileSync } = require("node:child_process");

function parseMax(text) {
  const re = /GLIBC_(\d+)\.(\d+)/g;
  let max = [0, 0];
  let m;
  while ((m = re.exec(text))) {
    const v = [Number(m[1]), Number(m[2])];
    if (v[0] > max[0] || (v[0] === max[0] && v[1] > max[1])) max = v;
  }
  return max;
}

function main(argv = process.argv) {
  const bin = argv[2];
  const want = argv[3] || "2.35";
  if (!bin) {
    console.error("usage: node npm/scripts/assert-max-glibc.js <elf> [major.minor]");
    process.exit(1);
  }
  const text = execFileSync("objdump", ["-T", bin], {
    encoding: "utf8",
    maxBuffer: 20 * 1024 * 1024,
  });
  const [maj, min] = parseMax(text);
  const [wmaj, wmin] = want.split(".").map(Number);
  const got = `${maj}.${min}`;
  if (maj === 0) {
    console.error(`${bin}: no GLIBC_* symbols`);
    process.exit(1);
  }
  console.log(`${bin}: max GLIBC_${got}`);
  if (maj > wmaj || (maj === wmaj && min > wmin)) {
    console.error(`needs GLIBC_${got} > ${want} (Ubuntu 22.04 is 2.35)`);
    process.exit(1);
  }
}

if (require.main === module) {
  try {
    main();
  } catch (err) {
    console.error(err.message || err);
    process.exit(1);
  }
}

module.exports = { parseMax };
