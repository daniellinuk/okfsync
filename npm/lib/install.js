"use strict";

/**
 * postinstall — for published releases, download the matching prebuilt binary
 * into npm/vendor/. In the monorepo (or when BAGSY_SKIP_DOWNLOAD=1), no-op if
 * a local cargo build already satisfies resolveBinary().
 */

const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");
const { resolveBinary, platformKey, PLATFORM_MAP } = require("./resolve");

const RELEASE_BASE =
  process.env.BAGSY_RELEASE_BASE ||
  "https://github.com/bagsy-dev/bagsy/releases/download";

function log(msg) {
  if (process.env.BAGSY_INSTALL_SILENT === "1") return;
  console.log(`[bagsy] ${msg}`);
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https
      .get(url, (res) => {
        if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          file.close();
          fs.unlink(dest, () => {});
          return download(res.headers.location, dest).then(resolve, reject);
        }
        if (res.statusCode !== 200) {
          file.close();
          fs.unlink(dest, () => {});
          return reject(new Error(`download failed ${res.statusCode} for ${url}`));
        }
        res.pipe(file);
        file.on("finish", () => file.close(() => resolve()));
      })
      .on("error", (err) => {
        file.close();
        fs.unlink(dest, () => {});
        reject(err);
      });
  });
}

async function main() {
  if (process.env.BAGSY_SKIP_DOWNLOAD === "1") {
    log("BAGSY_SKIP_DOWNLOAD=1 — skipping binary download");
    return;
  }

  const existing = resolveBinary();
  if (existing) {
    log(`using existing binary at ${existing}`);
    return;
  }

  const key = platformKey();
  const pkg = PLATFORM_MAP[key];
  if (!pkg) {
    log(`no prebuilt binary mapping for ${key}; build from /cli with cargo`);
    return;
  }

  const version = require("../package.json").version;
  const asset =
    process.platform === "win32"
      ? `${pkg}.exe`
      : `${pkg}`;
  const url = `${RELEASE_BASE}/v${version}/${asset}`;
  const vendorDir = path.join(__dirname, "..", "vendor");
  fs.mkdirSync(vendorDir, { recursive: true });
  const dest = path.join(
    vendorDir,
    process.platform === "win32" ? "bagsy.exe" : "bagsy"
  );

  log(`downloading ${url}`);
  try {
    await download(url, dest);
    fs.chmodSync(dest, 0o755);
    log(`installed to ${dest}`);
  } catch (err) {
    log(`download skipped/failed: ${err.message}`);
    log("build the Rust CLI in /cli or set BAGSY_BIN");
  }
}

if (require.main === module) {
  main().catch((err) => {
    console.error(`[bagsy] install warning: ${err.message}`);
    // Never fail install — wrapper can still use monorepo binary
    process.exit(0);
  });
}

module.exports = { main, download };
