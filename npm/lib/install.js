"use strict";

/**
 * postinstall — download a versioned prebuilt if this package did not
 * already ship `vendor/okfsync-<platform>-<version>`. A leftover
 * `vendor/kbsync` from 0.1.0/0.1.1 is ignored.
 * KBSYNC_SKIP_DOWNLOAD=1 skips the network (CI / monorepo).
 */

const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");
const {
  resolveBinary,
  platformKey,
  PLATFORM_MAP,
  packageVersion,
  vendorBinaryPath,
} = require("./resolve");

const RELEASE_BASE =
  process.env.KBSYNC_RELEASE_BASE ||
  "https://github.com/daniellinuk/okfsync/releases/download";

function log(msg) {
  if (process.env.KBSYNC_INSTALL_SILENT === "1") return;
  console.log(`[okfsync] ${msg}`);
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
  if (process.env.KBSYNC_SKIP_DOWNLOAD === "1") {
    log("KBSYNC_SKIP_DOWNLOAD=1 — skipping binary download");
    return;
  }

  const dest = vendorBinaryPath();
  if (dest && fs.existsSync(dest) && fs.statSync(dest).isFile()) {
    log(`using ${dest}`);
    return;
  }

  // Unversioned vendor/kbsync is not a candidate, so a leftover 0.1.x
  // binary does not skip this install. A monorepo cargo build still does.
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
  if (!dest) {
    log(`no vendor path for ${key}`);
    return;
  }

  const version = packageVersion();
  const asset = process.platform === "win32" ? `${pkg}.exe` : `${pkg}`;
  const url = `${RELEASE_BASE}/v${version}/${asset}`;
  fs.mkdirSync(path.dirname(dest), { recursive: true });

  log(`downloading ${url}`);
  try {
    await download(url, dest);
    fs.chmodSync(dest, 0o755);
    log(`installed to ${dest}`);
  } catch (err) {
    log(`download skipped/failed: ${err.message}`);
    log("build the Rust CLI in /cli or set KBSYNC_BIN");
  }
}

if (require.main === module) {
  main().catch((err) => {
    console.error(`[okfsync] install warning: ${err.message}`);
    // Never fail install — wrapper can still use monorepo binary
    process.exit(0);
  });
}

module.exports = { main, download };
